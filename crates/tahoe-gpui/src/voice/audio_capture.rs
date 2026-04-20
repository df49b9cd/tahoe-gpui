//! Audio capture via cpal — microphone recording and WAV encoding.
//!
//! # Permission model (HIG Voice audit)
//!
//! Apple's Generative AI HIG requires disclosing how personal data (audio)
//! is used and asking permission explicitly, so the voice module distinguishes
//! three distinct error conditions that cpal collapses together:
//!
//! * [`AudioCaptureError::NotDetermined`] — `AVCaptureDevice` authorization
//!   status is `.notDetermined`: the user has never been prompted. The host
//!   app should call `AVCaptureDevice.requestAccess(for: .audio, …)` to
//!   trigger the macOS permission dialog, then retry.
//! * [`AudioCaptureError::PermissionDenied`] — authorization is `.denied` or
//!   `.restricted`. The app must direct users to **System Settings → Privacy
//!   & Security → Microphone** via the "Open Privacy Settings" affordance.
//! * [`AudioCaptureError::NoDevice`] — authorization is `.authorized` (or
//!   unknown to this crate) but no input device is physically present.
//!
//! cpal does not itself expose the AVFoundation authorization status. The
//! `MicPermission` the host app passes into [`MicSelectorView`] and
//! [`SpeechInputState`] is the source of truth; consumers call
//! [`AudioCapture::start_with_permission`] so this module can map the
//! observed `None` from cpal onto the correct error variant.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use cpal::SampleFormat;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// Captured audio data ready for transcription.
#[derive(Clone, Debug)]
pub struct CapturedAudio {
    /// WAV-encoded audio bytes (RIFF/PCM16).
    pub data: Vec<u8>,
    /// MIME type — always `"audio/wav"`.
    pub mime_type: &'static str,
    /// Duration of the recording in seconds.
    pub duration_secs: f32,
    /// Sample rate of the audio.
    pub sample_rate: u32,
}

/// Authorization hint supplied by the host application. Mirrors
/// `AVCaptureDevice.authorizationStatus(for: .audio)` on macOS.
///
/// The host app owns the AVFoundation permission lifecycle. This enum lets
/// the host signal the observed state so `audio_capture` can produce an
/// accurate [`AudioCaptureError`] variant when cpal returns `None` from
/// `default_input_device()`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PermissionHint {
    /// The host has not inspected the authorization status. cpal's `None`
    /// will be reported as [`AudioCaptureError::NotDetermined`] because
    /// that is the most common initial state on macOS.
    #[default]
    Unknown,
    /// `.notDetermined` — the system has never prompted the user.
    NotDetermined,
    /// `.denied` or `.restricted`.
    Denied,
    /// `.authorized`.
    Authorized,
}

/// Errors that can occur during audio capture.
#[derive(Debug, Clone)]
pub enum AudioCaptureError {
    /// Authorization is granted (or not relevant on the platform), but no
    /// physical input device is present.
    NoDevice,
    /// The user has never been prompted for microphone access
    /// (`AVCaptureDevice` status `.notDetermined`). The host should call
    /// `requestAccess(for: .audio, …)` and retry.
    NotDetermined,
    /// The user denied or restricted microphone access. The UI should link
    /// to **System Settings → Privacy & Security → Microphone**.
    PermissionDenied,
    /// Failed to build or run the audio stream.
    StreamError(String),
}

impl std::fmt::Display for AudioCaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioCaptureError::NoDevice => write!(f, "No input device found"),
            AudioCaptureError::NotDetermined => {
                write!(f, "Microphone permission has not been requested")
            }
            AudioCaptureError::PermissionDenied => write!(f, "Microphone permission denied"),
            AudioCaptureError::StreamError(msg) => write!(f, "Audio stream error: {msg}"),
        }
    }
}

impl std::error::Error for AudioCaptureError {}

/// Callback fired from the cpal thread when the underlying audio stream
/// errors mid-recording (e.g. device disconnected, sample-rate change).
///
/// Previously these errors went to `eprintln!` and silently produced empty
/// audio or stalled. The Generative AI HIG asks for graceful propagation so
/// the UI can report a recoverable state. Consumers typically wire this into
/// a `SpeechInputView::set_disabled(true, …)` + toast.
pub type StreamErrorCallback = Arc<dyn Fn(cpal::StreamError) + Send + Sync + 'static>;

/// Shared state between the cpal callback thread and the UI thread.
struct SharedState {
    /// Accumulated mono f32 samples.
    samples: Mutex<Vec<f32>>,
    /// RMS level of the most recent callback chunk (0.0..1.0).
    current_level: Mutex<f32>,
    /// Signal from the UI thread to stop recording.
    stop: AtomicBool,
}

/// Hardware-level audio capture using cpal.
///
/// The cpal stream runs on its own internal thread; this struct is held on the
/// UI thread. Call [`AudioCapture::stop`] to finalize and get the recorded audio.
pub struct AudioCapture {
    shared: Arc<SharedState>,
    sample_rate: u32,
    _stream: cpal::Stream,
}

impl AudioCapture {
    /// Start recording from the default input device, assuming the host has
    /// not supplied an `AVCaptureDevice` authorization hint.
    ///
    /// When cpal returns no default device, the error is reported as
    /// [`AudioCaptureError::NotDetermined`] — the most common initial state
    /// on macOS where the host app has not yet called `requestAccess`. Hosts
    /// that already know the authorization status should call
    /// [`AudioCapture::start_with_permission`] instead so the error reflects
    /// the real condition (denied vs. no device).
    ///
    /// The stream captures mono audio at the device's preferred sample rate.
    pub fn start() -> Result<Self, AudioCaptureError> {
        Self::start_with_permission(PermissionHint::Unknown, None)
    }

    /// Start recording, using the host-supplied permission hint to map a
    /// missing default device onto the correct [`AudioCaptureError`] variant.
    ///
    /// `on_stream_error` receives `cpal::StreamError` notifications that
    /// occur after the stream has started (e.g. a USB microphone unplugged
    /// mid-recording). Pass `None` to silently discard them — but UIs should
    /// provide a callback so the user can see the failure.
    pub fn start_with_permission(
        permission: PermissionHint,
        on_stream_error: Option<StreamErrorCallback>,
    ) -> Result<Self, AudioCaptureError> {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or_else(|| {
            // cpal returns `None` for several distinct conditions. Disambiguate
            // via the host-supplied permission hint plus device enumeration.
            let has_any_device = host
                .input_devices()
                .ok()
                .map(|mut iter| iter.next().is_some())
                .unwrap_or(false);
            match permission {
                PermissionHint::Denied => AudioCaptureError::PermissionDenied,
                PermissionHint::NotDetermined => AudioCaptureError::NotDetermined,
                PermissionHint::Authorized => AudioCaptureError::NoDevice,
                PermissionHint::Unknown => {
                    if has_any_device {
                        // Devices exist but no default — treat as hardware issue.
                        AudioCaptureError::NoDevice
                    } else {
                        // Empty list almost certainly means permission is not
                        // yet granted on macOS (cpal cannot enumerate without it).
                        AudioCaptureError::NotDetermined
                    }
                }
            }
        })?;

        let config = device
            .default_input_config()
            .map_err(|e| AudioCaptureError::StreamError(e.to_string()))?;

        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;

        let shared = Arc::new(SharedState {
            samples: Mutex::new(Vec::new()),
            current_level: Mutex::new(0.0),
            stop: AtomicBool::new(false),
        });

        let shared_cb = Arc::clone(&shared);
        let make_err_fn = move || {
            let cb = on_stream_error.clone();
            move |err: cpal::StreamError| {
                if let Some(ref cb) = cb {
                    cb(err);
                }
            }
        };

        let stream = match config.sample_format() {
            SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    process_samples(&shared_cb, data, channels);
                },
                make_err_fn(),
                None,
            ),
            SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let floats: Vec<f32> =
                        data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                    process_samples(&shared_cb, &floats, channels);
                },
                make_err_fn(),
                None,
            ),
            SampleFormat::U16 => device.build_input_stream(
                &config.into(),
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let floats: Vec<f32> = data
                        .iter()
                        .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                        .collect();
                    process_samples(&shared_cb, &floats, channels);
                },
                make_err_fn(),
                None,
            ),
            format => {
                return Err(AudioCaptureError::StreamError(format!(
                    "Unsupported sample format: {format:?}"
                )));
            }
        }
        .map_err(|e| AudioCaptureError::StreamError(e.to_string()))?;

        stream
            .play()
            .map_err(|e| AudioCaptureError::StreamError(e.to_string()))?;

        Ok(Self {
            shared,
            sample_rate,
            _stream: stream,
        })
    }

    /// Returns the current audio level (0.0..1.0) based on recent RMS amplitude.
    pub fn current_level(&self) -> f32 {
        *self
            .shared
            .current_level
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Stop recording and return the captured audio as WAV.
    pub fn stop(self) -> CapturedAudio {
        self.shared.stop.store(true, Ordering::Relaxed);
        // Drop the stream by consuming self, which stops the cpal thread.
        drop(self._stream);

        let samples = self
            .shared
            .samples
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let duration_secs = samples.len() as f32 / self.sample_rate as f32;
        let wav_data = encode_wav(&samples, self.sample_rate);

        CapturedAudio {
            data: wav_data,
            mime_type: "audio/wav",
            duration_secs,
            sample_rate: self.sample_rate,
        }
    }
}

/// Enumerate all available audio input devices.
///
/// Returns a list of [`super::AudioDevice`] structs. On macOS, if microphone
/// permission has not been granted, `input_devices()` returns an empty
/// iterator — cpal provides no generic device names in that case, so the
/// caller sees zero devices rather than placeholders.
///
/// Note: cpal does not provide stable device IDs across sessions; device
/// names are used as identifiers.
///
/// This is the Rust equivalent of `useAudioDevices()` from the AI SDK
/// Elements web library.
pub fn enumerate_input_devices() -> Result<Vec<super::AudioDevice>, AudioCaptureError> {
    let host = cpal::default_host();
    let input_devices = host
        .input_devices()
        .map_err(|e| AudioCaptureError::StreamError(e.to_string()))?;

    let mut seen_names: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let devices: Vec<super::AudioDevice> = input_devices
        .map(|device| {
            let name = device
                .description()
                .map(|d| d.name().to_string())
                .unwrap_or_default();
            let count = seen_names.entry(name.clone()).or_insert(0);
            *count += 1;
            let id = if *count > 1 {
                format!("{} ({})", name, count)
            } else {
                name.clone()
            };
            super::AudioDevice {
                id,
                name: name.into(),
            }
        })
        .collect();

    Ok(devices)
}

/// Returns the name of the default input device, if available.
pub fn default_input_device_name() -> Option<String> {
    let host = cpal::default_host();
    host.default_input_device()
        .and_then(|d| d.description().ok())
        .map(|desc| desc.name().to_string())
}

/// Process incoming samples: mix to mono, compute RMS level, accumulate.
fn process_samples(shared: &SharedState, data: &[f32], channels: usize) {
    if shared.stop.load(Ordering::Relaxed) {
        return;
    }

    // Mix to mono by averaging channels.
    let mono: Vec<f32> = if channels == 1 {
        data.to_vec()
    } else {
        data.chunks(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    };

    // Compute RMS for this chunk.
    if !mono.is_empty() {
        let sum_sq: f32 = mono.iter().map(|s| s * s).sum();
        let rms = (sum_sq / mono.len() as f32).sqrt();
        // Scale RMS to 0..1 (typical speech RMS is 0.01..0.3).
        let level = (rms * 5.0).min(1.0);
        *shared
            .current_level
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = level;
    }

    shared
        .samples
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .extend_from_slice(&mono);
}

/// Encode f32 samples as a 16-bit PCM WAV file.
fn encode_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * u32::from(num_channels) * u32::from(bits_per_sample) / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let data_size = (samples.len() * 2) as u32;
    let file_size = 36 + data_size;

    let mut buf = Vec::with_capacity(44 + data_size as usize);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // fmt sub-chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // sub-chunk size
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    buf.extend_from_slice(&num_channels.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data sub-chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let pcm16 = (clamped * i16::MAX as f32) as i16;
        buf.extend_from_slice(&pcm16.to_le_bytes());
    }

    buf
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicBool;

    use super::{
        AudioCaptureError, CapturedAudio, PermissionHint, SharedState, encode_wav, process_samples,
    };

    #[test]
    fn permission_hint_default_is_unknown() {
        assert_eq!(PermissionHint::default(), PermissionHint::Unknown);
    }

    #[test]
    fn audio_capture_error_display() {
        assert_eq!(
            AudioCaptureError::NoDevice.to_string(),
            "No input device found"
        );
        assert_eq!(
            AudioCaptureError::NotDetermined.to_string(),
            "Microphone permission has not been requested"
        );
        assert_eq!(
            AudioCaptureError::PermissionDenied.to_string(),
            "Microphone permission denied"
        );
        assert_eq!(
            AudioCaptureError::StreamError("boom".into()).to_string(),
            "Audio stream error: boom"
        );
    }

    #[test]
    fn encode_wav_produces_valid_header() {
        let samples = vec![0.0f32; 100];
        let wav = encode_wav(&samples, 16000);

        // RIFF header
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");

        // Format: PCM, 1 channel, 16000 Hz, 16-bit
        let format = u16::from_le_bytes([wav[20], wav[21]]);
        assert_eq!(format, 1); // PCM

        let channels = u16::from_le_bytes([wav[22], wav[23]]);
        assert_eq!(channels, 1);

        let rate = u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]);
        assert_eq!(rate, 16000);

        let bits = u16::from_le_bytes([wav[34], wav[35]]);
        assert_eq!(bits, 16);

        // Data sub-chunk
        assert_eq!(&wav[36..40], b"data");
        let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
        assert_eq!(data_size, 200); // 100 samples * 2 bytes

        assert_eq!(wav.len(), 44 + 200);
    }

    #[test]
    fn encode_wav_clamps_samples() {
        let samples = vec![2.0, -2.0, 0.5];
        let wav = encode_wav(&samples, 44100);

        // First sample should be clamped to i16::MAX
        let s0 = i16::from_le_bytes([wav[44], wav[45]]);
        assert_eq!(s0, i16::MAX);

        // Second sample should be clamped to -i16::MAX (not i16::MIN to avoid asymmetry)
        let s1 = i16::from_le_bytes([wav[46], wav[47]]);
        assert_eq!(s1, i16::MIN + 1); // -1.0 * 32767 = -32767
    }

    #[test]
    fn encode_wav_empty_samples() {
        let wav = encode_wav(&[], 16000);
        assert_eq!(wav.len(), 44); // header only
        let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
        assert_eq!(data_size, 0);
    }

    #[test]
    fn process_samples_computes_mono_and_level() {
        let shared = SharedState {
            samples: Mutex::new(Vec::new()),
            current_level: Mutex::new(0.0),
            stop: AtomicBool::new(false),
        };

        // Stereo input: L=0.5, R=-0.5 -> mono average = 0.0
        process_samples(&shared, &[0.5, -0.5, 0.5, -0.5], 2);
        let samples = shared.samples.lock().unwrap();
        assert_eq!(samples.len(), 2);
        assert!((samples[0] - 0.0).abs() < 1e-6);

        // Level should be ~0 for this case
        let level = *shared.current_level.lock().unwrap();
        assert!(level < 0.01);
    }

    #[test]
    fn process_samples_mono_passthrough() {
        let shared = SharedState {
            samples: Mutex::new(Vec::new()),
            current_level: Mutex::new(0.0),
            stop: AtomicBool::new(false),
        };

        process_samples(&shared, &[0.3, -0.3, 0.3, -0.3], 1);
        let samples = shared.samples.lock().unwrap();
        assert_eq!(samples.len(), 4);
        assert!((samples[0] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn process_samples_respects_stop() {
        let shared = SharedState {
            samples: Mutex::new(Vec::new()),
            current_level: Mutex::new(0.0),
            stop: AtomicBool::new(true),
        };

        process_samples(&shared, &[0.5, 0.5], 1);
        assert!(shared.samples.lock().unwrap().is_empty());
    }

    #[test]
    fn process_samples_survives_poisoned_mutex() {
        use std::sync::Arc;
        use std::thread;

        let shared = Arc::new(SharedState {
            samples: Mutex::new(Vec::new()),
            current_level: Mutex::new(0.0),
            stop: AtomicBool::new(false),
        });

        // Simulate a cpal-thread panic while holding both guards — poisons
        // both mutexes, so any naive `.lock().expect(...)` from the UI
        // thread (`current_level`, `stop`) would abort the window.
        let poisoner = Arc::clone(&shared);
        let _ = thread::spawn(move || {
            let _s = poisoner.samples.lock().unwrap();
            let _l = poisoner.current_level.lock().unwrap();
            panic!("poison-the-mutexes");
        })
        .join();
        assert!(shared.samples.is_poisoned());
        assert!(shared.current_level.is_poisoned());

        // Callback must not panic despite the poison.
        process_samples(&shared, &[0.4, 0.4], 1);

        // Data still accumulated after poison recovery.
        let samples = shared.samples.lock().unwrap_or_else(|p| p.into_inner());
        assert_eq!(samples.len(), 2);
        let level = *shared
            .current_level
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        assert!(level > 0.0);
    }

    #[test]
    fn captured_audio_fields() {
        let audio = CapturedAudio {
            data: vec![0; 44],
            mime_type: "audio/wav",
            duration_secs: 1.5,
            sample_rate: 16000,
        };
        assert_eq!(audio.mime_type, "audio/wav");
        assert_eq!(audio.sample_rate, 16000);
    }
}
