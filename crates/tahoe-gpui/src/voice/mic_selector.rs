//! Microphone device selector component.
//!
//! A dropdown selector for choosing a microphone device, with search/filter,
//! keyboard navigation, device label parsing, and permission/loading states.
//!
//! Corresponds to the `MicSelector` family in the AI SDK Elements web library.
//! Key parity points: value-based selection by device ID, `on_open_change`,
//! `on_value_change`, device label parsing with hardware ID, two-stage
//! permission model, and searchable device list.

use gpui::prelude::*;
use gpui::{
    App, ElementId, FocusHandle, KeyDownEvent, MouseDownEvent, Pixels, SharedString, Window, div,
    px,
};

use crate::callback_types::{OnStrChange, OnToggle};
use crate::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::layout::DROPDOWN_MAX_HEIGHT;
use crate::foundations::materials::{Elevation, Glass, Shape, glass_effect};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use crate::ids::next_element_id;

pub use super::AudioDevice;

/// macOS URL for opening **System Settings → Privacy & Security → Microphone**
/// directly. Hosts pass this to `NSWorkspace.shared.open(_:)` from the
/// `on_open_privacy_settings` callback.
pub const PRIVACY_MICROPHONE_SETTINGS_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone";

/// Microphone permission state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MicPermission {
    /// Permission not yet requested; device names may be generic.
    #[default]
    Unknown,
    /// Permission granted; real device names are available.
    Granted,
    /// Permission denied by the user or system.
    Denied,
}

/// Loading/error state for the mic selector.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MicSelectorState {
    #[default]
    Idle,
    Loading,
    Error,
}

/// Result of parsing a device label for hardware ID extraction.
struct ParsedDeviceLabel {
    display_name: String,
    hardware_id: Option<String>,
}

/// Parse a device name, extracting a trailing hardware ID like `(1a2b:3c4d)`.
///
/// Returns the display name and optional hardware ID string.
fn parse_device_label(name: &str) -> ParsedDeviceLabel {
    let trimmed = name.trim();
    if let Some(open) = trimmed.rfind('(') {
        let candidate = &trimmed[open..];
        if candidate.ends_with(')') && candidate.len() >= 11 {
            let inner = &candidate[1..candidate.len() - 1];
            if let Some((left, right)) = inner.split_once(':')
                && left.len() == 4
                && right.len() == 4
                && left.chars().all(|c| c.is_ascii_hexdigit())
                && right.chars().all(|c| c.is_ascii_hexdigit())
            {
                return ParsedDeviceLabel {
                    display_name: trimmed[..open].trim().to_string(),
                    hardware_id: Some(inner.to_string()),
                };
            }
        }
    }
    ParsedDeviceLabel {
        display_name: trimmed.to_string(),
        hardware_id: None,
    }
}

/// Returns a generic device name like "Microphone 1" for the given index.
fn generic_device_name(index: usize) -> SharedString {
    SharedString::from(format!("Microphone {}", index + 1))
}

/// Clear `selected` if no device in `devices` matches the current ID.
fn preserve_selection(selected: &mut Option<String>, devices: &[AudioDevice]) {
    if let Some(ref id) = *selected
        && !devices.iter().any(|d| d.id == *id)
    {
        *selected = None;
    }
}

/// Find the device matching `selected` in `devices`.
fn find_selected_device<'a>(
    selected: &Option<String>,
    devices: &'a [AudioDevice],
) -> Option<&'a AudioDevice> {
    selected
        .as_ref()
        .and_then(|id| devices.iter().find(|d| d.id == *id))
}

/// A dropdown selector for choosing a microphone device.
#[allow(clippy::type_complexity)]
pub struct MicSelectorView {
    element_id: ElementId,
    devices: Vec<AudioDevice>,
    /// Device ID of the confirmed selection (persists across open/close).
    selected_device_id: Option<String>,
    /// Index within the filtered list for keyboard navigation.
    nav_index: usize,
    filter: String,
    is_open: bool,
    focus_handle: FocusHandle,
    state: MicSelectorState,
    permission: MicPermission,
    dropdown_width: Option<Pixels>,
    /// Optional transparency copy about AI audio processing.
    ai_disclosure: Option<SharedString>,
    on_select: Option<Box<dyn Fn(&AudioDevice, &mut Window, &mut App) + 'static>>,
    on_value_change: OnStrChange,
    on_open_change: OnToggle,
    /// Callback for the "Open Privacy Settings" button shown under
    /// [`MicPermission::Denied`]. Hosts open
    /// [`PRIVACY_MICROPHONE_SETTINGS_URL`] via `NSWorkspace`.
    on_open_privacy_settings: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl MicSelectorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            element_id: next_element_id("mic-selector"),
            devices: Vec::new(),
            selected_device_id: None,
            nav_index: 0,
            filter: String::new(),
            is_open: false,
            focus_handle: cx.focus_handle(),
            state: MicSelectorState::Idle,
            permission: MicPermission::Unknown,
            dropdown_width: None,
            ai_disclosure: None,
            on_select: None,
            on_value_change: None,
            on_open_change: None,
            on_open_privacy_settings: None,
        }
    }

    /// Set transparency copy displayed at the top of the dropdown. When
    /// set, users see a short disclosure about how the audio captured by
    /// the selected device will be used.
    pub fn set_ai_disclosure(
        &mut self,
        disclosure: Option<impl Into<SharedString>>,
        cx: &mut Context<Self>,
    ) {
        self.ai_disclosure = disclosure.map(Into::into);
        cx.notify();
    }

    /// Register a callback for the "Open Privacy Settings" action shown
    /// while [`MicPermission::Denied`]. Hosts open
    /// [`PRIVACY_MICROPHONE_SETTINGS_URL`] via `NSWorkspace.shared.open(_:)`.
    pub fn set_on_open_privacy_settings(
        &mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) {
        self.on_open_privacy_settings = Some(Box::new(handler));
    }

    pub fn set_devices(&mut self, devices: Vec<AudioDevice>, cx: &mut Context<Self>) {
        preserve_selection(&mut self.selected_device_id, &devices);
        self.devices = devices;
        self.nav_index = 0;
        self.filter.clear();
        self.state = MicSelectorState::Idle;
        cx.notify();
    }

    pub fn set_on_select(
        &mut self,
        handler: impl Fn(&AudioDevice, &mut Window, &mut App) + 'static,
    ) {
        self.on_select = Some(Box::new(handler));
    }

    pub fn set_state(&mut self, state: MicSelectorState, cx: &mut Context<Self>) {
        self.state = state;
        cx.notify();
    }

    pub fn set_permission(&mut self, permission: MicPermission, cx: &mut Context<Self>) {
        self.permission = permission;
        cx.notify();
    }

    pub fn set_dropdown_width(&mut self, width: Pixels) {
        self.dropdown_width = Some(width);
    }

    pub fn set_on_value_change(&mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) {
        self.on_value_change = Some(Box::new(handler));
    }

    pub fn set_on_open_change(&mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) {
        self.on_open_change = Some(Box::new(handler));
    }

    /// Set the selected device by ID (controlled mode).
    ///
    /// Unlike `select_device_by_id`, this does not close the dropdown or
    /// fire callbacks — it is meant for parent-driven state synchronization.
    pub fn set_value(&mut self, device_id: Option<&str>, cx: &mut Context<Self>) {
        self.selected_device_id = device_id.map(|s| s.to_string());
        cx.notify();
    }

    /// Select the first device if no device is currently selected.
    ///
    /// Useful after `set_devices` to ensure the trigger button shows a
    /// real device name instead of "No microphone".
    pub fn select_default_if_unset(&mut self, cx: &mut Context<Self>) {
        if self.selected_device_id.is_none()
            && let Some(device) = self.devices.first()
        {
            self.selected_device_id = Some(device.id.clone());
            cx.notify();
        }
    }

    /// Re-enumerate input devices and update the device list.
    ///
    /// Preserves the current selection if the selected device is still present.
    /// The web equivalent listens for `devicechange` events; in native code,
    /// call this when the dropdown opens or on window focus.
    ///
    /// Note: this performs synchronous cpal host enumeration and may block
    /// the UI thread. Consider calling from a background task in
    /// latency-sensitive contexts.
    pub fn refresh_devices(&mut self, cx: &mut Context<Self>) {
        match super::audio_capture::enumerate_input_devices() {
            Ok(devices) => {
                self.permission = MicPermission::Granted;
                self.set_devices(devices, cx);
            }
            Err(_) => {
                self.devices.clear();
                self.selected_device_id = None;
                self.state = MicSelectorState::Error;
                cx.notify();
            }
        }
    }

    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_open {
            return;
        }
        self.is_open = true;
        self.filter.clear();
        self.nav_index = 0;
        self.focus_handle.focus(window, cx);
        if let Some(ref cb) = self.on_open_change {
            cb(true, window, &mut *cx);
        }
        cx.notify();
    }

    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.is_open = false;
        if let Some(ref cb) = self.on_open_change {
            cb(false, window, &mut *cx);
        }
        cx.notify();
    }

    pub fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_open {
            self.close(window, cx);
        } else {
            self.open(window, cx);
        }
    }

    /// Returns the currently selected device, if any.
    pub fn selected(&self) -> Option<&AudioDevice> {
        find_selected_device(&self.selected_device_id, &self.devices)
    }

    /// Select a device by its ID programmatically. Returns `true` if found.
    ///
    /// Closes the dropdown (firing `on_open_change`) but does not fire
    /// `on_select` or `on_value_change` — those are reserved for user
    /// interaction via the dropdown UI.
    pub fn select_device_by_id(
        &mut self,
        id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.devices.iter().any(|d| d.id == id) {
            self.selected_device_id = Some(id.to_string());
            self.close(window, cx);
            true
        } else {
            false
        }
    }

    /// Select a device by index programmatically.
    ///
    /// Closes the dropdown (firing `on_open_change`) but does not fire
    /// `on_select` or `on_value_change` — those are reserved for user
    /// interaction via the dropdown UI.
    pub fn select_device(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(device) = self.devices.get(index) {
            self.selected_device_id = Some(device.id.clone());
            self.close(window, cx);
        }
    }

    fn filtered_devices(&self) -> Vec<(usize, &AudioDevice)> {
        if self.filter.is_empty() {
            self.devices.iter().enumerate().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.devices
                .iter()
                .enumerate()
                .filter(|(_, d)| {
                    d.name.to_lowercase().contains(&filter_lower)
                        || d.id.to_lowercase().contains(&filter_lower)
                })
                .collect()
        }
    }

    fn select_current(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let filtered = self.filtered_devices();
        if let Some(&(original_idx, _)) = filtered.get(self.nav_index) {
            let device = self.devices[original_idx].clone();
            self.selected_device_id = Some(device.id.clone());
            if let Some(ref handler) = self.on_select {
                handler(&device, window, &mut *cx);
            }
            if let Some(ref handler) = self.on_value_change {
                handler(&device.id, window, &mut *cx);
            }
            self.close(window, cx);
        }
    }

    /// Returns the display label for the given device and index, respecting permission state.
    fn device_display_name(&self, device: &AudioDevice, index: usize) -> SharedString {
        match self.permission {
            MicPermission::Granted => device.name.clone(),
            _ => generic_device_name(index),
        }
    }
}

impl Render for MicSelectorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_open = self.is_open;

        let mut container = div()
            .relative()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                match key {
                    "escape" => {
                        if this.is_open {
                            this.close(window, cx);
                        }
                    }
                    "up" => {
                        if this.nav_index > 0 {
                            this.nav_index -= 1;
                            cx.notify();
                        }
                    }
                    "down" => {
                        let count = this.filtered_devices().len();
                        if this.nav_index + 1 < count {
                            this.nav_index += 1;
                            cx.notify();
                        }
                    }
                    "enter" => {
                        if this.is_open {
                            this.select_current(window, cx);
                        }
                    }
                    "backspace" => {
                        if this.is_open {
                            this.filter.pop();
                            this.nav_index = 0;
                            cx.notify();
                        }
                    }
                    _ => {
                        if this.is_open
                            && let Some(ch) = &event.keystroke.key_char
                            && !event.keystroke.modifiers.control
                            && !event.keystroke.modifiers.platform
                        {
                            this.filter.push_str(ch);
                            this.nav_index = 0;
                            cx.notify();
                        }
                    }
                }
            }));

        // Trigger button
        let current_label = self
            .selected()
            .map(|d| {
                let idx = self
                    .devices
                    .iter()
                    .position(|dev| dev.id == d.id)
                    .unwrap_or(0);
                self.device_display_name(d, idx)
            })
            .unwrap_or_else(|| "No microphone".into());

        container = container.child(
            Button::new(self.element_id.clone())
                .icon(Icon::new(IconName::Mic).size(theme.icon_size_inline))
                .label(current_label)
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::Small)
                .on_click(cx.listener(|this, _event, window, cx| {
                    this.toggle(window, cx);
                })),
        );

        // Dropdown
        if is_open {
            // On macOS 26 Tahoe the HIG calls out Liquid Glass for floating
            // panels like this dropdown. `glass_effect` degrades cleanly on
            // non-glass themes by falling back to the same translucent fill
            // + shadows the previous implementation used directly.
            let glass_panel = glass_effect(
                div()
                    .absolute()
                    .top_full()
                    .left_0()
                    .mt(theme.spacing_xs)
                    .max_h(px(DROPDOWN_MAX_HEIGHT))
                    .flex()
                    .flex_col()
                    .overflow_hidden(),
                theme,
                Glass::Regular,
                Shape::Default,
                Elevation::Elevated,
            );
            let mut dropdown =
                glass_panel
                    .id("mic-selector-dropdown")
                    .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, window, cx| {
                        this.close(window, cx);
                    }));

            dropdown = if let Some(w) = self.dropdown_width {
                dropdown.w(w)
            } else {
                dropdown.w_full().min_w(px(200.0))
            };

            // Optional AI disclosure at the top of the dropdown — gives
            // hosts a stable place to communicate how captured audio will
            // be processed.
            if let Some(ref disclosure) = self.ai_disclosure {
                dropdown = dropdown.child(
                    div()
                        .px(theme.spacing_md)
                        .py(theme.spacing_sm)
                        .border_b_1()
                        .border_color(theme.border)
                        .text_style(TextStyle::Caption1, theme)
                        .text_color(theme.text_muted)
                        .child(disclosure.clone()),
                );
            }

            // Search input
            dropdown = dropdown.child(
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_sm)
                    .px(theme.spacing_md)
                    .py(theme.spacing_sm)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        Icon::new(IconName::Search)
                            .size(theme.icon_size_inline)
                            .color(theme.text_muted),
                    )
                    .child(
                        div()
                            .text_style(TextStyle::Subheadline, theme)
                            .text_color(if self.filter.is_empty() {
                                theme.text_muted
                            } else {
                                theme.text
                            })
                            .child(if self.filter.is_empty() {
                                SharedString::from("Search devices...")
                            } else {
                                SharedString::from(self.filter.clone())
                            }),
                    ),
            );

            // Content area
            let mut list = div()
                .id("mic-selector-list")
                .flex()
                .flex_col()
                .overflow_y_scroll()
                .max_h(px(250.0));

            match self.state {
                MicSelectorState::Loading => {
                    list = list.child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap(theme.spacing_sm)
                            .px(theme.spacing_md)
                            .py(theme.spacing_lg)
                            .child(
                                Icon::new(IconName::Loader)
                                    .size(theme.icon_size_inline)
                                    .color(theme.text_muted),
                            )
                            .child(
                                div()
                                    .text_style(TextStyle::Subheadline, theme)
                                    .text_color(theme.text_muted)
                                    .child("Detecting microphones..."),
                            ),
                    );
                }
                MicSelectorState::Error => {
                    list = list.child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap(theme.spacing_sm)
                            .px(theme.spacing_md)
                            .py(theme.spacing_lg)
                            .child(
                                Icon::new(IconName::AlertTriangle)
                                    .size(theme.icon_size_inline)
                                    .color(theme.error),
                            )
                            .child(
                                div()
                                    .text_style(TextStyle::Subheadline, theme)
                                    .text_color(theme.error)
                                    .child("Failed to access microphones"),
                            ),
                    );
                }
                MicSelectorState::Idle if self.permission == MicPermission::Denied => {
                    // Pair the denied state with an explicit remediation
                    // path. The Privacy & Security deep-link is documented
                    // by Apple for exactly this pattern.
                    list =
                        list.child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(theme.spacing_sm)
                                .px(theme.spacing_md)
                                .py(theme.spacing_lg)
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(theme.spacing_sm)
                                        .child(
                                            Icon::new(IconName::AlertTriangle)
                                                .size(theme.icon_size_inline)
                                                .color(theme.warning),
                                        )
                                        .child(
                                            div()
                                                .text_style(TextStyle::Subheadline, theme)
                                                .text_color(theme.text)
                                                .child("Microphone access is denied"),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_style(TextStyle::Caption1, theme)
                                        .text_color(theme.text_muted)
                                        .child(
                                            "Grant access in System Settings to see \
                                         your available microphones.",
                                        ),
                                )
                                .child(
                                    Button::new(ElementId::from(SharedString::from(format!(
                                        "{}-open-settings",
                                        self.element_id
                                    ))))
                                    .label("Open Privacy Settings")
                                    .variant(ButtonVariant::Primary)
                                    .size(ButtonSize::Small)
                                    .accessibility_label(
                                        "Open System Settings, Privacy and Security, Microphone",
                                    )
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        if let Some(ref handler) = this.on_open_privacy_settings {
                                            handler(window, &mut *cx);
                                        }
                                    })),
                                ),
                        );
                }
                MicSelectorState::Idle if self.permission == MicPermission::Unknown => {
                    // When permission has not yet been granted, explain why
                    // mic access is needed before the system prompt or
                    // re-enumeration can populate the list.
                    list = list.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(theme.spacing_sm)
                            .px(theme.spacing_md)
                            .py(theme.spacing_lg)
                            .child(
                                div()
                                    .text_style(TextStyle::Subheadline, theme)
                                    .text_color(theme.text)
                                    .child("Microphone access is needed"),
                            )
                            .child(
                                div()
                                    .text_style(TextStyle::Caption1, theme)
                                    .text_color(theme.text_muted)
                                    .child(
                                        "This app needs microphone access to record \
                                         your voice and list connected devices.",
                                    ),
                            ),
                    );
                }
                MicSelectorState::Idle => {
                    let filtered = self.filtered_devices();
                    let nav_selected = self.nav_index;

                    if filtered.is_empty() && !self.devices.is_empty() {
                        list = list.child(
                            div()
                                .px(theme.spacing_md)
                                .py(theme.spacing_lg)
                                .text_style(TextStyle::Subheadline, theme)
                                .text_color(theme.text_muted)
                                .child("No devices match your search"),
                        );
                    } else if filtered.is_empty() {
                        list = list.child(
                            div()
                                .px(theme.spacing_md)
                                .py(theme.spacing_lg)
                                .text_style(TextStyle::Subheadline, theme)
                                .text_color(theme.text_muted)
                                .child("No microphones found"),
                        );
                    } else {
                        for (display_idx, &(original_idx, device)) in filtered.iter().enumerate() {
                            let is_nav = display_idx == nav_selected;
                            let display_idx_copy = display_idx;

                            let mut item = div()
                                .id(ElementId::NamedInteger(
                                    "mic-item".into(),
                                    original_idx as u64,
                                ))
                                .flex()
                                .items_center()
                                .gap(theme.spacing_sm)
                                .px(theme.spacing_md)
                                .py(theme.spacing_sm)
                                .cursor_pointer()
                                .bg(if is_nav { theme.hover } else { theme.surface })
                                .hover(|s| s.bg(theme.hover))
                                .on_click(cx.listener(move |this, _event, window, cx| {
                                    this.nav_index = display_idx_copy;
                                    this.select_current(window, cx);
                                }));

                            // Mic icon
                            item = item.child(
                                Icon::new(IconName::Mic)
                                    .size(px(12.0))
                                    .color(theme.text_muted),
                            );

                            // Device label (flex_1 to push check icon right)
                            let display_name = self.device_display_name(device, original_idx);

                            if self.permission == MicPermission::Granted {
                                let parsed = parse_device_label(&device.name);
                                let mut label_col = div().flex().flex_col().flex_1().child(
                                    div()
                                        .text_style(TextStyle::Subheadline, theme)
                                        .text_color(theme.text)
                                        .child(SharedString::from(parsed.display_name)),
                                );
                                if let Some(hw_id) = parsed.hardware_id {
                                    label_col = label_col.child(
                                        div()
                                            .text_style(TextStyle::Caption1, theme)
                                            .text_color(theme.text_muted)
                                            .child(SharedString::from(hw_id)),
                                    );
                                }
                                item = item.child(label_col);
                            } else {
                                item = item.child(
                                    div()
                                        .flex_1()
                                        .text_style(TextStyle::Subheadline, theme)
                                        .text_color(theme.text)
                                        .child(display_name),
                                );
                            }

                            // Check icon for confirmed selection
                            if self.selected_device_id.as_deref() == Some(device.id.as_str()) {
                                item = item.child(
                                    Icon::new(IconName::Check)
                                        .size(px(12.0))
                                        .color(theme.accent),
                                );
                            }

                            list = list.child(item);
                        }
                    }
                }
            }

            dropdown = dropdown.child(list);
            container = container.child(dropdown);
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use gpui::SharedString;

    use super::{
        AudioDevice, find_selected_device, generic_device_name, parse_device_label,
        preserve_selection,
    };

    fn make_device(id: &str, name: &str) -> AudioDevice {
        AudioDevice {
            id: id.to_string(),
            name: SharedString::from(name.to_string()),
        }
    }

    #[test]
    fn parse_device_label_with_hardware_id() {
        let result = parse_device_label("MacBook Pro Microphone (1a2b:3c4d)");
        assert_eq!(result.display_name, "MacBook Pro Microphone");
        assert_eq!(result.hardware_id.as_deref(), Some("1a2b:3c4d"));
    }

    #[test]
    fn parse_device_label_without_hardware_id() {
        let result = parse_device_label("Built-in Microphone");
        assert_eq!(result.display_name, "Built-in Microphone");
        assert_eq!(result.hardware_id, None);
    }

    #[test]
    fn parse_device_label_empty() {
        let result = parse_device_label("");
        assert_eq!(result.display_name, "");
        assert_eq!(result.hardware_id, None);
    }

    #[test]
    fn parse_device_label_non_hex_parens() {
        let result = parse_device_label("Mic (not a hw id)");
        assert_eq!(result.display_name, "Mic (not a hw id)");
        assert_eq!(result.hardware_id, None);
    }

    #[test]
    fn parse_device_label_multiple_parens() {
        let result = parse_device_label("Mic (USB) (abcd:ef01)");
        assert_eq!(result.display_name, "Mic (USB)");
        assert_eq!(result.hardware_id.as_deref(), Some("abcd:ef01"));
    }

    #[test]
    fn parse_device_label_uppercase_hex() {
        let result = parse_device_label("Mic (ABCD:EF01)");
        assert_eq!(result.display_name, "Mic");
        assert_eq!(result.hardware_id.as_deref(), Some("ABCD:EF01"));
    }

    #[test]
    fn generic_device_name_numbering() {
        assert_eq!(generic_device_name(0).as_ref(), "Microphone 1");
        assert_eq!(generic_device_name(1).as_ref(), "Microphone 2");
        assert_eq!(generic_device_name(9).as_ref(), "Microphone 10");
    }

    #[test]
    fn preserve_selection_keeps_id_when_present() {
        let devices = vec![
            make_device("c", "Mic C"),
            make_device("a", "Mic A"),
            make_device("b", "Mic B"),
        ];
        let mut selected = Some("b".to_string());
        preserve_selection(&mut selected, &devices);
        assert_eq!(selected.as_deref(), Some("b"));
    }

    #[test]
    fn preserve_selection_clears_id_when_removed() {
        let devices = vec![make_device("a", "Mic A"), make_device("c", "Mic C")];
        let mut selected = Some("b".to_string());
        preserve_selection(&mut selected, &devices);
        assert_eq!(selected, None);
    }

    #[test]
    fn find_selected_device_returns_match() {
        let devices = vec![make_device("a", "Mic A"), make_device("b", "Mic B")];
        let selected = Some("b".to_string());
        let found = find_selected_device(&selected, &devices);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "b");
    }

    #[test]
    fn find_selected_device_returns_none_for_missing_id() {
        let devices = vec![make_device("a", "Mic A")];
        let selected = Some("z".to_string());
        let found = find_selected_device(&selected, &devices);
        assert!(found.is_none());
    }
}
