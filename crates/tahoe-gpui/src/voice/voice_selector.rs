//! Voice/TTS voice selector component.
//!
//! A dropdown selector for choosing a TTS voice, with search/filter,
//! keyboard navigation, voice metadata (gender, accent, age), grouping,
//! preview playback states, and selection callbacks.

use std::collections::HashMap;

use gpui::prelude::*;
use gpui::{
    App, ElementId, FocusHandle, KeyDownEvent, MouseDownEvent, Pixels, SharedString, Window, div,
    px,
};

use crate::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use crate::components::presentation::modal::Modal;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::{GlassSize, glass_surface};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use crate::ids::next_element_id;

// ---------------------------------------------------------------------------
// Voice metadata types
// ---------------------------------------------------------------------------

/// Gender metadata for a voice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoiceGender {
    Male,
    Female,
    NonBinary,
    Transgender,
    Androgyne,
    Intersex,
}

impl VoiceGender {
    /// Display label for this gender.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Male => "Male",
            Self::Female => "Female",
            Self::NonBinary => "Non-binary",
            Self::Transgender => "Transgender",
            Self::Androgyne => "Androgyne",
            Self::Intersex => "Intersex",
        }
    }

    /// Unicode symbol used as a compact gender indicator.
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Male => "\u{2642}",        // ♂
            Self::Female => "\u{2640}",      // ♀
            Self::NonBinary => "\u{26A7}",   // ⚧
            Self::Transgender => "\u{26A7}", // ⚧
            Self::Androgyne => "\u{26A5}",   // ⚥
            Self::Intersex => "\u{26A5}",    // ⚥
        }
    }
}

/// Accent/locale metadata for a voice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoiceAccent {
    American,
    British,
    Australian,
    Canadian,
    Irish,
    Scottish,
    Indian,
    SouthAfrican,
    NewZealand,
    Spanish,
    French,
    German,
    Italian,
    Portuguese,
    Brazilian,
    Mexican,
    Argentinian,
    Japanese,
    Chinese,
    Korean,
    Russian,
    Arabic,
    Dutch,
    Swedish,
    Norwegian,
    Danish,
    Finnish,
    Polish,
    Turkish,
    Greek,
}

impl VoiceAccent {
    /// Display label for this accent.
    pub fn label(&self) -> &'static str {
        match self {
            Self::American => "American",
            Self::British => "British",
            Self::Australian => "Australian",
            Self::Canadian => "Canadian",
            Self::Irish => "Irish",
            Self::Scottish => "Scottish",
            Self::Indian => "Indian",
            Self::SouthAfrican => "South African",
            Self::NewZealand => "New Zealand",
            Self::Spanish => "Spanish",
            Self::French => "French",
            Self::German => "German",
            Self::Italian => "Italian",
            Self::Portuguese => "Portuguese",
            Self::Brazilian => "Brazilian",
            Self::Mexican => "Mexican",
            Self::Argentinian => "Argentinian",
            Self::Japanese => "Japanese",
            Self::Chinese => "Chinese",
            Self::Korean => "Korean",
            Self::Russian => "Russian",
            Self::Arabic => "Arabic",
            Self::Dutch => "Dutch",
            Self::Swedish => "Swedish",
            Self::Norwegian => "Norwegian",
            Self::Danish => "Danish",
            Self::Finnish => "Finnish",
            Self::Polish => "Polish",
            Self::Turkish => "Turkish",
            Self::Greek => "Greek",
        }
    }

    /// Flag emoji for this accent, if available.
    pub fn flag(&self) -> &'static str {
        match self {
            Self::American => "\u{1F1FA}\u{1F1F8}",   // 🇺🇸
            Self::British => "\u{1F1EC}\u{1F1E7}",    // 🇬🇧
            Self::Australian => "\u{1F1E6}\u{1F1FA}", // 🇦🇺
            Self::Canadian => "\u{1F1E8}\u{1F1E6}",   // 🇨🇦
            Self::Irish => "\u{1F1EE}\u{1F1EA}",      // 🇮🇪
            // Issue #148 F20: The stable Scotland ZWJ sequence
            // `🏴󠁧󠁢󠁳󠁣󠁴󠁿` is not yet reliably supported in macOS bundled fonts.
            // Ship the generic black flag `🏴` as a cross-platform
            // fallback; update once Apple adds the subdivision variant.
            Self::Scottish => "\u{1F3F4}",
            Self::Indian => "\u{1F1EE}\u{1F1F3}",       // 🇮🇳
            Self::SouthAfrican => "\u{1F1FF}\u{1F1E6}", // 🇿🇦
            Self::NewZealand => "\u{1F1F3}\u{1F1FF}",   // 🇳🇿
            Self::Spanish => "\u{1F1EA}\u{1F1F8}",      // 🇪🇸
            Self::French => "\u{1F1EB}\u{1F1F7}",       // 🇫🇷
            Self::German => "\u{1F1E9}\u{1F1EA}",       // 🇩🇪
            Self::Italian => "\u{1F1EE}\u{1F1F9}",      // 🇮🇹
            Self::Portuguese => "\u{1F1F5}\u{1F1F9}",   // 🇵🇹
            Self::Brazilian => "\u{1F1E7}\u{1F1F7}",    // 🇧🇷
            Self::Mexican => "\u{1F1F2}\u{1F1FD}",      // 🇲🇽
            Self::Argentinian => "\u{1F1E6}\u{1F1F7}",  // 🇦🇷
            Self::Japanese => "\u{1F1EF}\u{1F1F5}",     // 🇯🇵
            Self::Chinese => "\u{1F1E8}\u{1F1F3}",      // 🇨🇳
            Self::Korean => "\u{1F1F0}\u{1F1F7}",       // 🇰🇷
            Self::Russian => "\u{1F1F7}\u{1F1FA}",      // 🇷🇺
            Self::Arabic => "\u{1F1F8}\u{1F1E6}",       // 🇸🇦
            Self::Dutch => "\u{1F1F3}\u{1F1F1}",        // 🇳🇱
            Self::Swedish => "\u{1F1F8}\u{1F1EA}",      // 🇸🇪
            Self::Norwegian => "\u{1F1F3}\u{1F1F4}",    // 🇳🇴
            Self::Danish => "\u{1F1E9}\u{1F1F0}",       // 🇩🇰
            Self::Finnish => "\u{1F1EB}\u{1F1EE}",      // 🇫🇮
            Self::Polish => "\u{1F1F5}\u{1F1F1}",       // 🇵🇱
            Self::Turkish => "\u{1F1F9}\u{1F1F7}",      // 🇹🇷
            Self::Greek => "\u{1F1EC}\u{1F1F7}",        // 🇬🇷
        }
    }
}

/// Preview playback state for a voice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VoicePreviewState {
    #[default]
    Idle,
    Loading,
    Playing,
}

// ---------------------------------------------------------------------------
// VoiceOption
// ---------------------------------------------------------------------------

/// A TTS voice option with optional metadata.
#[derive(Clone)]
pub struct VoiceOption {
    pub id: String,
    pub name: SharedString,
    /// Locale-specific display name used in the trigger label, dropdown
    /// rows, and any future Siri vocabulary registration. Falls back to
    /// [`VoiceOption::name`] when `None`. Issue #148 F18.
    pub localized_name: Option<SharedString>,
    pub description: Option<SharedString>,
    pub gender: Option<VoiceGender>,
    pub accent: Option<VoiceAccent>,
    pub age: Option<SharedString>,
    pub group: Option<SharedString>,
    pub shortcut: Option<SharedString>,
    /// Additional terms Siri should accept for this voice
    /// (e.g. pronunciations, nicknames). Feeds
    /// `INVocabulary.setVocabularyStrings(...)` when the Siri intent lands.
    /// Issue #148 F17/F18.
    pub siri_vocabulary: Vec<SharedString>,
}

impl VoiceOption {
    pub fn new(id: impl Into<String>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            localized_name: None,
            description: None,
            gender: None,
            accent: None,
            age: None,
            group: None,
            shortcut: None,
            siri_vocabulary: Vec::new(),
        }
    }

    pub fn description(mut self, desc: impl Into<SharedString>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn gender(mut self, gender: VoiceGender) -> Self {
        self.gender = Some(gender);
        self
    }

    pub fn accent(mut self, accent: VoiceAccent) -> Self {
        self.accent = Some(accent);
        self
    }

    pub fn age(mut self, age: impl Into<SharedString>) -> Self {
        self.age = Some(age.into());
        self
    }

    pub fn group(mut self, group: impl Into<SharedString>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Set the locale-specific display name (issue #148 F18).
    pub fn localized_name(mut self, name: impl Into<SharedString>) -> Self {
        self.localized_name = Some(name.into());
        self
    }

    /// Replace the Siri vocabulary list (issue #148 F17/F18).
    pub fn siri_vocabulary<I, S>(mut self, terms: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.siri_vocabulary = terms.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the [`localized_name`](Self::localized_name) when set,
    /// falling back to [`name`](Self::name). Used as the visible label in
    /// the dropdown and trigger button.
    pub fn display_name(&self) -> &SharedString {
        self.localized_name.as_ref().unwrap_or(&self.name)
    }
}

// ---------------------------------------------------------------------------
// VoiceSelectorVariant
// ---------------------------------------------------------------------------

/// Display variant for the voice selector.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VoiceSelectorVariant {
    /// Inline dropdown positioned below the trigger button.
    #[default]
    Dropdown,
    /// Full-screen centered modal dialog with backdrop.
    Dialog,
}

// ---------------------------------------------------------------------------
// VoiceSelectorView
// ---------------------------------------------------------------------------

/// A dropdown selector for choosing a TTS voice with search, keyboard
/// navigation, metadata display, grouping, and preview support.
#[allow(clippy::type_complexity)]
pub struct VoiceSelectorView {
    element_id: ElementId,
    voices: Vec<VoiceOption>,
    /// ID of the confirmed selection (persists across open/close).
    selected_id: Option<String>,
    /// Index within the filtered list for keyboard navigation.
    nav_index: usize,
    filter: String,
    is_open: bool,
    focus_handle: FocusHandle,
    dropdown_width: Pixels,
    preview_states: HashMap<String, VoicePreviewState>,
    empty_message: Option<SharedString>,
    on_select: Option<Box<dyn Fn(&VoiceOption, &mut Window, &mut App) + 'static>>,
    on_preview: Option<Box<dyn Fn(&VoiceOption, &mut Window, &mut App) + 'static>>,
    variant: VoiceSelectorVariant,
}

impl VoiceSelectorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            element_id: next_element_id("voice-selector"),
            voices: Vec::new(),
            selected_id: None,
            nav_index: 0,
            filter: String::new(),
            is_open: false,
            focus_handle: cx.focus_handle(),
            dropdown_width: px(320.0),
            preview_states: HashMap::new(),
            empty_message: None,
            on_select: None,
            on_preview: None,
            variant: VoiceSelectorVariant::default(),
        }
    }

    pub fn set_voices(&mut self, voices: Vec<VoiceOption>, cx: &mut Context<Self>) {
        self.voices = voices;
        self.selected_id = None;
        self.nav_index = 0;
        self.filter.clear();
        self.preview_states.clear();
        cx.notify();
    }

    pub fn set_on_select(
        &mut self,
        handler: impl Fn(&VoiceOption, &mut Window, &mut App) + 'static,
    ) {
        self.on_select = Some(Box::new(handler));
    }

    pub fn set_on_preview(
        &mut self,
        handler: impl Fn(&VoiceOption, &mut Window, &mut App) + 'static,
    ) {
        self.on_preview = Some(Box::new(handler));
    }

    /// Set the preview state for a specific voice (driven by the consumer).
    pub fn set_preview_state(
        &mut self,
        voice_id: &str,
        state: VoicePreviewState,
        cx: &mut Context<Self>,
    ) {
        self.preview_states.insert(voice_id.to_string(), state);
        cx.notify();
    }

    pub fn set_dropdown_width(&mut self, width: Pixels) {
        self.dropdown_width = width;
    }

    pub fn set_empty_message(&mut self, msg: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.empty_message = Some(msg.into());
        cx.notify();
    }

    pub fn set_variant(&mut self, variant: VoiceSelectorVariant, cx: &mut Context<Self>) {
        self.variant = variant;
        cx.notify();
    }

    /// Returns whether the selector is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Set the open state of the selector.
    pub fn set_open(&mut self, open: bool, window: &mut Window, cx: &mut Context<Self>) {
        if open {
            self.open(window, cx);
        } else {
            self.close(cx);
        }
    }

    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.is_open = true;
        self.filter.clear();
        self.nav_index = 0;
        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.is_open = false;
        cx.notify();
    }

    pub fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_open {
            self.close(cx);
        } else {
            self.open(window, cx);
        }
    }

    /// Returns the currently selected voice, if any.
    pub fn selected(&self) -> Option<&VoiceOption> {
        let id = self.selected_id.as_ref()?;
        self.voices.iter().find(|v| v.id == *id)
    }

    /// Returns the currently selected voice ID, if any.
    pub fn selected_value(&self) -> Option<&str> {
        self.selected_id.as_deref()
    }

    /// Set the selected voice by ID. Pass `None` to clear the selection.
    /// Returns `true` if the voice was found (or selection was cleared).
    ///
    /// This is a silent setter — it does not fire `on_select`.
    pub fn set_value(&mut self, id: Option<&str>, cx: &mut Context<Self>) -> bool {
        match id {
            None => {
                self.selected_id = None;
                cx.notify();
                true
            }
            Some(id) => {
                if self.voices.iter().any(|v| v.id == id) {
                    self.selected_id = Some(id.to_string());
                    cx.notify();
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Select a voice by its ID and close the selector.
    /// Returns `true` if a matching voice was found.
    ///
    /// This is a silent setter — it does not fire `on_select`.
    pub fn select_voice_by_id(&mut self, id: &str, cx: &mut Context<Self>) -> bool {
        let found = self.set_value(Some(id), cx);
        if found {
            self.close(cx);
        }
        found
    }

    fn filtered_voices(&self) -> Vec<(usize, &VoiceOption)> {
        if self.filter.is_empty() {
            self.voices.iter().enumerate().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.voices
                .iter()
                .enumerate()
                .filter(|(_, v)| {
                    v.name.to_lowercase().contains(&filter_lower)
                        || v.id.to_lowercase().contains(&filter_lower)
                        || v.localized_name
                            .as_ref()
                            .is_some_and(|n| n.to_lowercase().contains(&filter_lower))
                        || v.description
                            .as_ref()
                            .is_some_and(|d| d.to_lowercase().contains(&filter_lower))
                        || v.group
                            .as_ref()
                            .is_some_and(|g| g.to_lowercase().contains(&filter_lower))
                        // Issue #148 F18: searching vocabulary terms lets
                        // users find a voice by any of the terms that Siri
                        // will accept for it.
                        || v.siri_vocabulary
                            .iter()
                            .any(|t| t.to_lowercase().contains(&filter_lower))
                })
                .collect()
        }
    }

    fn select_current(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let filtered = self.filtered_voices();
        if let Some(&(original_idx, _)) = filtered.get(self.nav_index) {
            let voice = self.voices[original_idx].clone();
            self.selected_id = Some(voice.id.clone());
            if let Some(ref handler) = self.on_select {
                handler(&voice, window, &mut *cx);
            }
        }
        self.close(cx);
    }
}

impl Render for VoiceSelectorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_open = self.is_open;

        let mut container = div()
            .relative()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                match key {
                    "escape" => this.close(cx),
                    "up" => {
                        if this.nav_index > 0 {
                            this.nav_index -= 1;
                            cx.notify();
                        }
                    }
                    "down" => {
                        let count = this.filtered_voices().len();
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

        // Trigger button — display the localized name when provided
        // (issue #148 F18) so Siri vocabulary and UI labels stay in sync.
        let current_label = self
            .selected()
            .map(|v| v.display_name().clone())
            .unwrap_or_else(|| "Select voice".into());

        container = container.child(
            Button::new(self.element_id.clone())
                .icon(Icon::new(IconName::Sparkle).size(theme.icon_size_inline))
                .label(current_label)
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::Sm)
                .on_click(cx.listener(|this, _event, window, cx| {
                    this.toggle(window, cx);
                })),
        );

        // Voice list panel (shared between Dropdown and Dialog variants)
        if is_open {
            let filtered = self.filtered_voices();
            let nav_selected = self.nav_index;

            // Search input
            let search_input = div()
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
                            SharedString::from("Search voices...")
                        } else {
                            SharedString::from(self.filter.clone())
                        }),
                );

            // Voice list
            let mut list = div()
                .id("voice-selector-list")
                .flex()
                .flex_col()
                .overflow_y_scroll()
                .max_h(px(350.0));

            if filtered.is_empty() && !self.voices.is_empty() {
                list = list.child(
                    div()
                        .px(theme.spacing_md)
                        .py(theme.spacing_lg)
                        .text_style(TextStyle::Subheadline, theme)
                        .text_color(theme.text_muted)
                        .child(
                            self.empty_message
                                .clone()
                                .unwrap_or_else(|| "No voices match your search".into()),
                        ),
                );
            } else if filtered.is_empty() {
                list = list.child(
                    div()
                        .px(theme.spacing_md)
                        .py(theme.spacing_lg)
                        .text_style(TextStyle::Subheadline, theme)
                        .text_color(theme.text_muted)
                        .child(
                            self.empty_message
                                .clone()
                                .unwrap_or_else(|| "No voices available".into()),
                        ),
                );
            } else {
                let mut current_group: Option<SharedString> = None;

                for (display_idx, &(original_idx, voice)) in filtered.iter().enumerate() {
                    // Group header
                    if let Some(ref group) = voice.group {
                        if current_group.as_ref() != Some(group) {
                            current_group = Some(group.clone());
                            list = list.child(
                                div()
                                    .px(theme.spacing_md)
                                    .pt(theme.spacing_sm)
                                    .pb(theme.spacing_xs)
                                    .text_style(TextStyle::Caption1, theme)
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.text_muted)
                                    .child(group.clone()),
                            );
                        }
                    } else if current_group.is_some() {
                        current_group = None;
                    }

                    let is_nav = display_idx == nav_selected;
                    let display_idx_copy = display_idx;
                    let voice_id = voice.id.clone();

                    let mut item = div()
                        .id(ElementId::NamedInteger(
                            "voice-item".into(),
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

                    // Voice info column (name + metadata + description)
                    let mut info_col = div().flex().flex_col().flex_1().gap(px(2.0));

                    // Name — prefer the localized variant (issue #148 F18).
                    info_col = info_col.child(
                        div()
                            .text_style(TextStyle::Subheadline, theme)
                            .text_color(theme.text)
                            .child(voice.display_name().clone()),
                    );

                    // Metadata row: gender + accent + age
                    let has_metadata =
                        voice.gender.is_some() || voice.accent.is_some() || voice.age.is_some();
                    if has_metadata {
                        let mut meta_row = div()
                            .flex()
                            .items_center()
                            .gap(theme.spacing_xs)
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.text_muted);

                        let mut parts_added = 0;

                        if let Some(gender) = voice.gender {
                            meta_row = meta_row.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(2.0))
                                    .child(gender.symbol())
                                    .child(gender.label()),
                            );
                            parts_added += 1;
                        }

                        if let Some(accent) = voice.accent {
                            if parts_added > 0 {
                                meta_row = meta_row.child("\u{00B7}"); // ·
                            }
                            meta_row = meta_row.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(2.0))
                                    .child(accent.flag())
                                    .child(accent.label()),
                            );
                            parts_added += 1;
                        }

                        if let Some(ref age) = voice.age {
                            if parts_added > 0 {
                                meta_row = meta_row.child("\u{00B7}"); // ·
                            }
                            meta_row = meta_row.child(age.clone());
                        }

                        info_col = info_col.child(meta_row);
                    }

                    // Description
                    if let Some(ref desc) = voice.description {
                        info_col = info_col.child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text_muted)
                                .child(desc.clone()),
                        );
                    }

                    item = item.child(info_col);

                    // Keyboard shortcut badge
                    if let Some(ref shortcut) = voice.shortcut {
                        item = item.child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text_muted)
                                .px(theme.spacing_xs)
                                .py(px(1.0))
                                .bg(theme.background)
                                .rounded(theme.radius_sm)
                                .border_1()
                                .border_color(theme.border)
                                .child(shortcut.clone()),
                        );
                    }

                    // Preview button
                    if self.on_preview.is_some() {
                        let preview_state = self
                            .preview_states
                            .get(&voice_id)
                            .copied()
                            .unwrap_or_default();

                        let preview_icon = match preview_state {
                            VoicePreviewState::Idle => IconName::Play,
                            VoicePreviewState::Loading => IconName::Loader,
                            VoicePreviewState::Playing => IconName::Pause,
                        };

                        item = item.child(
                            div()
                                .id(ElementId::NamedInteger(
                                    "voice-preview".into(),
                                    original_idx as u64,
                                ))
                                .cursor_pointer()
                                .rounded(theme.radius_md)
                                .p(px(4.0))
                                .hover(|s| s.bg(theme.hover))
                                .on_click(cx.listener(move |this, _event, window, cx| {
                                    if let Some(voice) = this.voices.get(original_idx).cloned()
                                        && let Some(ref handler) = this.on_preview
                                    {
                                        handler(&voice, window, &mut *cx);
                                    }
                                }))
                                .child(
                                    Icon::new(preview_icon)
                                        .size(px(12.0))
                                        .color(theme.text_muted),
                                ),
                        );
                    }

                    // Check icon for confirmed selection
                    if self.selected_id.as_ref() == Some(&voice.id) {
                        item = item.child(
                            Icon::new(IconName::Check)
                                .size(px(12.0))
                                .color(theme.accent),
                        );
                    }

                    list = list.child(item);
                }
            }

            // Build the panel content (search + list)
            let panel_content = div().flex().flex_col().child(search_input).child(list);

            match self.variant {
                VoiceSelectorVariant::Dropdown => {
                    // Issue #148 F19: Liquid Glass surface for the floating
                    // dropdown to match macOS 26 Tahoe HIG. Falls back to
                    // the translucent fill + shadows used by glass_surface
                    // on non-glass themes.
                    let glass_panel = glass_surface(
                        div()
                            .absolute()
                            .top_full()
                            .left_0()
                            .mt(theme.spacing_xs)
                            .w(self.dropdown_width)
                            .max_h(px(400.0))
                            .overflow_hidden(),
                        theme,
                        GlassSize::Medium,
                    );
                    let dropdown = glass_panel
                        .id("voice-selector-dropdown")
                        .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _window, cx| {
                            this.close(cx);
                        }))
                        .child(panel_content);
                    container = container.child(dropdown);
                }
                VoiceSelectorVariant::Dialog => {
                    let entity = cx.entity().downgrade();
                    container = container.child(
                        Modal::new("voice-selector-dialog", panel_content)
                            .open(is_open)
                            .width(px(500.0))
                            .scroll(false)
                            .focus_handle(self.focus_handle.clone())
                            .on_dismiss(move |_window, cx| {
                                if let Some(entity) = entity.upgrade() {
                                    entity.update(cx, |this, cx| this.close(cx));
                                }
                            }),
                    );
                }
            }
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::{VoiceAccent, VoiceGender, VoiceOption, VoicePreviewState, VoiceSelectorVariant};

    #[test]
    fn voice_gender_labels() {
        assert_eq!(VoiceGender::Male.label(), "Male");
        assert_eq!(VoiceGender::Female.label(), "Female");
        assert_eq!(VoiceGender::NonBinary.label(), "Non-binary");
        assert_eq!(VoiceGender::Transgender.label(), "Transgender");
        assert_eq!(VoiceGender::Androgyne.label(), "Androgyne");
        assert_eq!(VoiceGender::Intersex.label(), "Intersex");
    }

    #[test]
    fn voice_gender_symbols() {
        assert_eq!(VoiceGender::Male.symbol(), "\u{2642}");
        assert_eq!(VoiceGender::Female.symbol(), "\u{2640}");
        assert_eq!(VoiceGender::NonBinary.symbol(), "\u{26A7}");
        assert_eq!(VoiceGender::Transgender.symbol(), "\u{26A7}");
        assert_eq!(VoiceGender::Androgyne.symbol(), "\u{26A5}");
        assert_eq!(VoiceGender::Intersex.symbol(), "\u{26A5}");
    }

    #[test]
    fn voice_accent_labels_and_flags() {
        assert_eq!(VoiceAccent::American.label(), "American");
        assert_eq!(VoiceAccent::American.flag(), "\u{1F1FA}\u{1F1F8}");
        assert_eq!(VoiceAccent::British.label(), "British");
        assert_eq!(VoiceAccent::British.flag(), "\u{1F1EC}\u{1F1E7}");
        assert_eq!(VoiceAccent::Japanese.label(), "Japanese");
        assert_eq!(VoiceAccent::Japanese.flag(), "\u{1F1EF}\u{1F1F5}");
    }

    #[test]
    fn voice_option_builder() {
        let voice = VoiceOption::new("alloy", "Alloy")
            .description("A warm voice")
            .gender(VoiceGender::Female)
            .accent(VoiceAccent::American)
            .age("Young adult")
            .group("OpenAI");

        assert_eq!(voice.id, "alloy");
        assert_eq!(voice.name.as_ref(), "Alloy");
        assert_eq!(
            voice.description.as_ref().map(|s| s.as_ref()),
            Some("A warm voice")
        );
        assert_eq!(voice.gender, Some(VoiceGender::Female));
        assert_eq!(voice.accent, Some(VoiceAccent::American));
        assert_eq!(voice.age.as_ref().map(|s| s.as_ref()), Some("Young adult"));
        assert_eq!(voice.group.as_ref().map(|s| s.as_ref()), Some("OpenAI"));
    }

    #[test]
    fn voice_option_localized_name_and_vocabulary() {
        let voice = VoiceOption::new("echo", "Echo")
            .localized_name("エコー")
            .siri_vocabulary(["Echo voice", "Echo speaker"]);
        assert_eq!(voice.display_name().as_ref(), "エコー");
        assert_eq!(voice.siri_vocabulary.len(), 2);
        assert_eq!(voice.siri_vocabulary[0].as_ref(), "Echo voice");

        // Without a localized name, display_name falls back to name.
        let plain = VoiceOption::new("nova", "Nova");
        assert_eq!(plain.display_name().as_ref(), "Nova");
    }

    #[test]
    fn voice_option_defaults() {
        let voice = VoiceOption::new("test", "Test");
        assert!(voice.description.is_none());
        assert!(voice.gender.is_none());
        assert!(voice.accent.is_none());
        assert!(voice.age.is_none());
        assert!(voice.group.is_none());
        assert!(voice.shortcut.is_none());
    }

    #[test]
    fn voice_option_shortcut_builder() {
        let voice = VoiceOption::new("alloy", "Alloy").shortcut("Cmd+1");
        assert_eq!(voice.shortcut.as_ref().map(|s| s.as_ref()), Some("Cmd+1"));
    }

    #[test]
    fn voice_selector_variant_default() {
        assert_eq!(
            VoiceSelectorVariant::default(),
            VoiceSelectorVariant::Dropdown
        );
    }

    #[test]
    fn find_voice_by_id() {
        let voices = &[
            VoiceOption::new("alloy", "Alloy"),
            VoiceOption::new("echo", "Echo"),
            VoiceOption::new("nova", "Nova"),
        ];
        assert!(voices.iter().any(|v| v.id == "echo"));
        assert!(!voices.iter().any(|v| v.id == "missing"));
    }

    #[test]
    fn voice_preview_state_default() {
        assert_eq!(VoicePreviewState::default(), VoicePreviewState::Idle);
    }

    #[gpui::test]
    async fn set_value_and_selected_value(cx: &mut gpui::TestAppContext) {
        use super::VoiceSelectorView;
        use crate::test_helpers::helpers::setup_test_window;

        let (handle, cx) = setup_test_window(cx, |_window, cx| VoiceSelectorView::new(cx));
        handle.update_in(cx, |view, _window, cx| {
            view.set_voices(
                vec![
                    VoiceOption::new("alloy", "Alloy"),
                    VoiceOption::new("echo", "Echo"),
                ],
                cx,
            );

            // No selection initially.
            assert!(view.selected_value().is_none());

            // Valid ID.
            assert!(view.set_value(Some("echo"), cx));
            assert_eq!(view.selected_value(), Some("echo"));

            // Invalid ID leaves selection unchanged.
            assert!(!view.set_value(Some("missing"), cx));
            assert_eq!(view.selected_value(), Some("echo"));

            // Clear selection.
            assert!(view.set_value(None, cx));
            assert!(view.selected_value().is_none());
        });
    }

    #[gpui::test]
    async fn select_voice_by_id_closes_selector(cx: &mut gpui::TestAppContext) {
        use super::VoiceSelectorView;
        use crate::test_helpers::helpers::setup_test_window;

        let (handle, cx) = setup_test_window(cx, |_window, cx| VoiceSelectorView::new(cx));
        handle.update_in(cx, |view, window, cx| {
            view.set_voices(
                vec![
                    VoiceOption::new("alloy", "Alloy"),
                    VoiceOption::new("echo", "Echo"),
                ],
                cx,
            );
            view.open(window, cx);
            assert!(view.is_open());

            // Found: selects and closes.
            assert!(view.select_voice_by_id("echo", cx));
            assert_eq!(view.selected_value(), Some("echo"));
            assert!(!view.is_open());

            // Not found: returns false, selection unchanged.
            view.open(window, cx);
            assert!(!view.select_voice_by_id("missing", cx));
            assert_eq!(view.selected_value(), Some("echo"));
            assert!(view.is_open());
        });
    }

    #[gpui::test]
    async fn select_voice_by_index(cx: &mut gpui::TestAppContext) {
        use super::VoiceSelectorView;
        use crate::test_helpers::helpers::setup_test_window;

        let (handle, cx) = setup_test_window(cx, |_window, cx| VoiceSelectorView::new(cx));
        handle.update_in(cx, |view, window, cx| {
            view.set_voices(
                vec![
                    VoiceOption::new("alloy", "Alloy"),
                    VoiceOption::new("echo", "Echo"),
                ],
                cx,
            );
            view.open(window, cx);

            let found = view.select_voice_by_id("echo", cx);
            assert!(found);
            assert_eq!(view.selected_value(), Some("echo"));
            assert!(!view.is_open());

            // Unknown id is a no-op.
            let found = view.select_voice_by_id("missing", cx);
            assert!(!found);
            assert_eq!(view.selected_value(), Some("echo"));
        });
    }

    #[gpui::test]
    async fn filtered_voices_matches_fields(cx: &mut gpui::TestAppContext) {
        use super::VoiceSelectorView;
        use crate::test_helpers::helpers::setup_test_window;

        let (handle, cx) = setup_test_window(cx, |_window, cx| VoiceSelectorView::new(cx));
        handle.update_in(cx, |view, _window, cx| {
            view.set_voices(
                vec![
                    VoiceOption::new("alloy", "Alloy")
                        .description("Warm and confident")
                        .group("OpenAI"),
                    VoiceOption::new("echo", "Echo")
                        .description("Clear and precise")
                        .group("ElevenLabs"),
                ],
                cx,
            );

            // Match by name.
            view.filter = "alloy".into();
            assert_eq!(view.filtered_voices().len(), 1);

            // Match by ID.
            view.filter = "echo".into();
            assert_eq!(view.filtered_voices().len(), 1);

            // Match by description.
            view.filter = "warm".into();
            assert_eq!(view.filtered_voices().len(), 1);

            // Match by group.
            view.filter = "ElevenLabs".into();
            assert_eq!(view.filtered_voices().len(), 1);

            // No match.
            view.filter = "zzz".into();
            assert_eq!(view.filtered_voices().len(), 0);

            // Empty filter returns all.
            view.filter.clear();
            assert_eq!(view.filtered_voices().len(), 2);
        });
    }
}
