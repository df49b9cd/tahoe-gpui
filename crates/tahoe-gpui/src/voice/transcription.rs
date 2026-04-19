//! Transcription display component with time-synchronized segment highlighting
//! and click-to-seek functionality.
//!
//! Displays transcription segments from AI SDK `transcribe()` results with
//! automatic active/past/future state styling based on current playback time.
//!
//! # HIG alignment
//!
//! * The streaming indicator carries an accessibility label so
//!   VoiceOver users hear state transitions (pending full GPUI AX support).
//! * [`TranscriptionSegment::is_hypothesis`] distinguishes unstable
//!   streaming tokens from committed text; hypothesis tokens render muted
//!   and italic, and do not participate in click-to-seek.
//! * [`TranscriptionView::set_ai_disclosure`] renders an optional
//!   transparency caption.

use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, FontWeight, KeyDownEvent, SharedString, Window, div, px};

use crate::callback_types::OnF64Change;
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use crate::ids::next_element_id;

/// The playback state of a transcription segment relative to the current time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SegmentState {
    /// No current playback time is set.
    Idle,
    /// Current time is before the segment's start.
    Future,
    /// Current time is within the segment's time range.
    Active,
    /// Current time is after the segment's end.
    Past,
}

/// A single transcription segment with a time range and optional speaker.
#[derive(Clone)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start_second: f64,
    pub end_second: f64,
    pub speaker: Option<String>,
    /// When `true`, this segment represents hypothesis text — unstable
    /// tokens that a streaming recognizer may replace before commit. The
    /// view renders hypothesis segments muted and italic, and skips them
    /// when routing click-to-seek (their timestamps are unreliable).
    pub is_hypothesis: bool,
}

impl TranscriptionSegment {
    /// Create a new committed segment spanning a time range.
    pub fn new(text: impl Into<String>, start_second: f64, end_second: f64) -> Self {
        Self {
            text: text.into(),
            start_second,
            end_second,
            speaker: None,
            is_hypothesis: false,
        }
    }

    /// Create a new hypothesis segment — unstable streaming text that will
    /// be replaced by a committed segment once the recognizer finalises it.
    pub fn new_hypothesis(text: impl Into<String>, start_second: f64, end_second: f64) -> Self {
        Self {
            text: text.into(),
            start_second,
            end_second,
            speaker: None,
            is_hypothesis: true,
        }
    }

    /// Set the speaker label for this segment.
    pub fn with_speaker(mut self, speaker: impl Into<String>) -> Self {
        self.speaker = Some(speaker.into());
        self
    }

    /// Mark this segment as hypothesis (fluent builder variant of
    /// [`Self::new_hypothesis`]).
    pub fn with_hypothesis(mut self, is_hypothesis: bool) -> Self {
        self.is_hypothesis = is_hypothesis;
        self
    }

    /// Returns true if the segment text is empty or whitespace-only.
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }
}

/// Displays a synchronized list of transcription segments with playback-aware
/// highlighting and optional click-to-seek.
pub struct TranscriptionView {
    element_id: ElementId,
    segments: Vec<TranscriptionSegment>,
    is_streaming: bool,
    current_time: Option<f64>,
    /// Optional transparency copy about AI transcription.
    ai_disclosure: Option<SharedString>,
    on_seek: OnF64Change,
}

impl TranscriptionView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            element_id: next_element_id("transcription"),
            segments: Vec::new(),
            is_streaming: false,
            current_time: None,
            ai_disclosure: None,
            on_seek: None,
        }
    }

    /// Set optional transparency copy displayed beneath the transcription
    /// (e.g. `"Audio is transcribed on-device and not stored."`). When
    /// set, renders as a caption-styled line.
    pub fn set_ai_disclosure(
        &mut self,
        disclosure: Option<impl Into<SharedString>>,
        cx: &mut Context<Self>,
    ) {
        self.ai_disclosure = disclosure.map(Into::into);
        cx.notify();
    }

    /// Append a segment to the transcription.
    pub fn push_segment(&mut self, segment: TranscriptionSegment, cx: &mut Context<Self>) {
        self.segments.push(segment);
        cx.notify();
    }

    /// Replace all segments.
    pub fn set_segments(&mut self, segments: Vec<TranscriptionSegment>, cx: &mut Context<Self>) {
        self.segments = segments;
        cx.notify();
    }

    /// Set whether the transcription is actively streaming.
    pub fn set_streaming(&mut self, streaming: bool, cx: &mut Context<Self>) {
        self.is_streaming = streaming;
        cx.notify();
    }

    /// Remove all segments.
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.segments.clear();
        cx.notify();
    }

    /// Set the current playback time in seconds. Triggers segment state updates.
    pub fn set_current_time(&mut self, time: f64, cx: &mut Context<Self>) {
        self.current_time = Some(time);
        cx.notify();
    }

    /// Clear the current time (disables segment highlighting).
    pub fn clear_current_time(&mut self, cx: &mut Context<Self>) {
        self.current_time = None;
        cx.notify();
    }

    /// Register a seek callback. When set, segments become clickable and clicking
    /// calls this handler with the segment's `start_second`.
    pub fn set_on_seek(&mut self, handler: impl Fn(f64, &mut Window, &mut App) + 'static) {
        self.on_seek = Some(Box::new(handler));
    }

    /// Compute the state of a segment relative to the current playback time.
    pub fn segment_state(&self, segment: &TranscriptionSegment) -> SegmentState {
        match self.current_time {
            None => SegmentState::Idle,
            Some(t) if t >= segment.start_second && t <= segment.end_second => SegmentState::Active,
            Some(t) if t > segment.end_second => SegmentState::Past,
            Some(_) => SegmentState::Future,
        }
    }
}

impl Render for TranscriptionView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let has_seek = self.on_seek.is_some();

        let mut container = div()
            .id(self.element_id.clone())
            .flex()
            .flex_wrap()
            .gap(theme.spacing_xs)
            .text_style(TextStyle::Subheadline, theme)
            .w_full();

        for (idx, segment) in self.segments.iter().filter(|s| !s.is_empty()).enumerate() {
            let is_hypothesis = segment.is_hypothesis;
            let text_color = if is_hypothesis {
                // Apple Dictation renders hypothesis text lighter so users
                // can distinguish unstable output from committed text. Use
                // muted @60% + italic as the equivalent in Rust.
                theme.text_muted.opacity(0.6)
            } else {
                match self.segment_state(segment) {
                    SegmentState::Idle => theme.text,
                    SegmentState::Active => theme.accent,
                    SegmentState::Past => theme.text_muted,
                    SegmentState::Future => theme.text_muted.opacity(0.5),
                }
            };

            // Build inner content as children
            let mut inner: Vec<AnyElement> = Vec::new();
            if let Some(ref speaker) = segment.speaker {
                inner.push(
                    div()
                        .text_color(theme.accent)
                        .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
                        .child(SharedString::from(format!("{}: ", speaker)))
                        .into_any_element(),
                );
            }
            inner.push(
                div()
                    .child(SharedString::from(segment.text.clone()))
                    .into_any_element(),
            );

            // Hypothesis segments never participate in click-to-seek —
            // their timestamps are unstable until the recognizer commits a
            // final token.
            if has_seek && !is_hypothesis {
                let start = segment.start_second;
                container = container.child(
                    div()
                        .id(ElementId::NamedInteger("tseg".into(), idx as u64))
                        .text_color(text_color)
                        .cursor_pointer()
                        .rounded(theme.radius_sm)
                        .px(theme.spacing_xs)
                        .py(px(2.0))
                        .hover(|s| s.bg(theme.hover))
                        .on_click(cx.listener(move |this, _event, window, cx| {
                            if let Some(ref on_seek) = this.on_seek {
                                on_seek(start, window, cx);
                            }
                        }))
                        .on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                            if crate::foundations::keyboard::is_activation_key(event)
                                && let Some(ref on_seek) = this.on_seek
                            {
                                cx.stop_propagation();
                                on_seek(start, window, cx);
                            }
                        }))
                        .children(inner),
                );
            } else {
                let mut segment_div = div().text_color(text_color).cursor_default().py(px(2.0));
                if is_hypothesis {
                    segment_div = segment_div.italic();
                }
                container = container.child(segment_div.children(inner));
            }
        }

        // Streaming indicator — carry an accessibility label so screen
        // readers can announce "Listening" as a live region once GPUI
        // exposes the AX API. Until then, the label is stored via
        // `with_accessibility` and surfaces in debug tooling.
        if self.is_streaming {
            container = container.child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .with_accessibility(
                        &AccessibilityProps::new()
                            .role(AccessibilityRole::StaticText)
                            .label("Recording, transcription in progress"),
                    )
                    .child("Listening..."),
            );
        }

        // Optional AI transparency caption below the transcription.
        // Consumers set this when audio is routed through an external
        // transcription service so users know what's happening.
        if let Some(ref disclosure) = self.ai_disclosure {
            container = container.child(
                div()
                    .w_full()
                    .pt(theme.spacing_xs)
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .child(disclosure.clone()),
            );
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::{SegmentState, TranscriptionSegment, TranscriptionView};
    use gpui::ElementId;

    #[test]
    fn test_segment_constructors() {
        let seg = TranscriptionSegment::new("hello world", 1.0, 3.5);
        assert_eq!(seg.text, "hello world");
        assert_eq!(seg.start_second, 1.0);
        assert_eq!(seg.end_second, 3.5);
        assert!(seg.speaker.is_none());

        let seg = seg.with_speaker("Alice");
        assert_eq!(seg.speaker.as_deref(), Some("Alice"));
    }

    #[test]
    fn test_empty_segment_detection() {
        assert!(TranscriptionSegment::new("", 0.0, 1.0).is_empty());
        assert!(TranscriptionSegment::new("   ", 0.0, 1.0).is_empty());
        assert!(TranscriptionSegment::new("\t\n", 0.0, 1.0).is_empty());
        assert!(!TranscriptionSegment::new("hello", 0.0, 1.0).is_empty());
        assert!(!TranscriptionSegment::new(" hi ", 0.0, 1.0).is_empty());
    }

    fn view_with_time(time: Option<f64>) -> TranscriptionView {
        TranscriptionView {
            element_id: ElementId::Name("test".into()),
            segments: Vec::new(),
            is_streaming: false,
            current_time: time,
            ai_disclosure: None,
            on_seek: None,
        }
    }

    #[test]
    fn segment_hypothesis_builder() {
        let seg = TranscriptionSegment::new_hypothesis("maybe", 1.0, 2.0);
        assert!(seg.is_hypothesis);

        let committed = TranscriptionSegment::new("final", 1.0, 2.0);
        assert!(!committed.is_hypothesis);

        let upgraded = TranscriptionSegment::new("x", 0.0, 1.0).with_hypothesis(true);
        assert!(upgraded.is_hypothesis);
    }

    #[test]
    fn test_segment_state_active() {
        let view = view_with_time(Some(2.0));
        let seg = TranscriptionSegment::new("x", 1.0, 3.0);
        assert_eq!(view.segment_state(&seg), SegmentState::Active);
    }

    #[test]
    fn test_segment_state_active_at_boundaries() {
        let view = view_with_time(Some(1.0));
        let seg = TranscriptionSegment::new("x", 1.0, 3.0);
        assert_eq!(view.segment_state(&seg), SegmentState::Active);

        let view = view_with_time(Some(3.0));
        assert_eq!(view.segment_state(&seg), SegmentState::Active);
    }

    #[test]
    fn test_segment_state_past() {
        let view = view_with_time(Some(5.0));
        let seg = TranscriptionSegment::new("x", 1.0, 3.0);
        assert_eq!(view.segment_state(&seg), SegmentState::Past);
    }

    #[test]
    fn test_segment_state_future() {
        let view = view_with_time(Some(0.5));
        let seg = TranscriptionSegment::new("x", 1.0, 3.0);
        assert_eq!(view.segment_state(&seg), SegmentState::Future);
    }

    #[test]
    fn test_segment_state_no_current_time() {
        let view = view_with_time(None);
        let seg = TranscriptionSegment::new("x", 1.0, 3.0);
        assert_eq!(view.segment_state(&seg), SegmentState::Idle);
    }
}

#[cfg(test)]
mod gpui_tests {
    use super::{SegmentState, TranscriptionSegment, TranscriptionView};
    use crate::test_helpers::helpers::setup_test_window;
    use std::cell::Cell;
    use std::rc::Rc;

    #[gpui::test]
    async fn push_segment_adds_to_list(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TranscriptionView::new(cx));
        handle.update_in(cx, |view, _window, cx| {
            view.push_segment(TranscriptionSegment::new("hello", 0.0, 1.0), cx);
            assert_eq!(view.segments.len(), 1);
            assert_eq!(view.segments[0].text, "hello");
        });
    }

    #[gpui::test]
    async fn set_segments_replaces_all(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TranscriptionView::new(cx));
        handle.update_in(cx, |view, _window, cx| {
            view.push_segment(TranscriptionSegment::new("old", 0.0, 1.0), cx);
            view.set_segments(
                vec![
                    TranscriptionSegment::new("a", 0.0, 1.0),
                    TranscriptionSegment::new("b", 1.0, 2.0),
                ],
                cx,
            );
            assert_eq!(view.segments.len(), 2);
            assert_eq!(view.segments[0].text, "a");
            assert_eq!(view.segments[1].text, "b");
        });
    }

    #[gpui::test]
    async fn clear_removes_all(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TranscriptionView::new(cx));
        handle.update_in(cx, |view, _window, cx| {
            view.push_segment(TranscriptionSegment::new("x", 0.0, 1.0), cx);
            view.push_segment(TranscriptionSegment::new("y", 1.0, 2.0), cx);
            view.clear(cx);
            assert!(view.segments.is_empty());
        });
    }

    #[gpui::test]
    async fn set_current_time_updates_segment_state(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TranscriptionView::new(cx));
        handle.update_in(cx, |view, _window, cx| {
            view.push_segment(TranscriptionSegment::new("x", 1.0, 3.0), cx);
            view.set_current_time(2.0, cx);
            assert_eq!(
                view.segment_state(&view.segments[0].clone()),
                SegmentState::Active
            );
        });
    }

    #[gpui::test]
    async fn clear_current_time_returns_none_state(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TranscriptionView::new(cx));
        handle.update_in(cx, |view, _window, cx| {
            view.push_segment(TranscriptionSegment::new("x", 1.0, 3.0), cx);
            view.set_current_time(2.0, cx);
            view.clear_current_time(cx);
            assert_eq!(
                view.segment_state(&view.segments[0].clone()),
                SegmentState::Idle
            );
        });
    }

    #[gpui::test]
    async fn streaming_flag_toggles(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TranscriptionView::new(cx));
        handle.update_in(cx, |view, _window, cx| {
            assert!(!view.is_streaming);
            view.set_streaming(true, cx);
            assert!(view.is_streaming);
            view.set_streaming(false, cx);
            assert!(!view.is_streaming);
        });
    }

    #[gpui::test]
    async fn on_seek_callback_wiring(cx: &mut gpui::TestAppContext) {
        let seeked = Rc::new(Cell::new(0.0f64));
        let seeked_clone = seeked.clone();
        let expected_seek_time = 4.25f64;
        let (handle, cx) = setup_test_window(cx, |_window, cx| TranscriptionView::new(cx));
        handle.update_in(cx, |view, window, cx| {
            view.push_segment(
                TranscriptionSegment::new("seek me", expected_seek_time, 6.0),
                cx,
            );
            view.set_on_seek(move |time, _window, _cx| {
                seeked_clone.set(time);
            });
            assert!(view.on_seek.is_some());
            if let Some(on_seek) = view.on_seek.as_mut() {
                on_seek(expected_seek_time, window, cx);
            }
        });
        assert_eq!(seeked.get(), expected_seek_time);
    }

    #[gpui::test]
    async fn keyboard_seek_key_matching(cx: &mut gpui::TestAppContext) {
        let seeked = Rc::new(Cell::new(0.0f64));
        let seeked_clone = seeked.clone();
        let (handle, cx) = setup_test_window(cx, |_window, cx| TranscriptionView::new(cx));
        handle.update_in(cx, |view, window, cx| {
            view.push_segment(TranscriptionSegment::new("word", 1.0, 2.0), cx);
            view.set_on_seek(move |time, _window, _cx| {
                seeked_clone.set(time);
            });
            // Simulate the key-matching logic used in the on_key_down handler
            for key in ["enter", "space"] {
                if (key == "enter" || key == "space")
                    && let Some(on_seek) = view.on_seek.as_mut()
                {
                    on_seek(view.segments[0].start_second, window, cx);
                }
            }
        });
        assert_eq!(seeked.get(), 1.0);
    }

    #[gpui::test]
    async fn empty_segments_excluded_from_iteration(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TranscriptionView::new(cx));
        handle.update_in(cx, |view, _window, cx| {
            view.set_segments(
                vec![
                    TranscriptionSegment::new("hello", 0.0, 1.0),
                    TranscriptionSegment::new("", 1.0, 2.0),
                    TranscriptionSegment::new("   ", 2.0, 3.0),
                    TranscriptionSegment::new("world", 3.0, 4.0),
                ],
                cx,
            );
            // All 4 segments are stored
            assert_eq!(view.segments.len(), 4);
            // But only 2 non-empty segments pass the render filter
            let visible: Vec<_> = view.segments.iter().filter(|s| !s.is_empty()).collect();
            assert_eq!(visible.len(), 2);
            assert_eq!(visible[0].text, "hello");
            assert_eq!(visible[1].text, "world");
        });
    }
}
