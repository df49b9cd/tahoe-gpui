//! HIG Search Field — controlled search display with suggestion dropdown.
//!
//! Distinct from [`SearchBar`](super::SearchBar) which is display-only. This
//! component renders a capsule-shaped input area with a search icon, text
//! display, clear button, optional cancel button, and an optional dropdown
//! showing recent searches and suggestions.
//!
//! # Modes
//!
//! The field supports two input modes:
//!
//! - **Host-managed value**: pass `.value(...)` with a `SharedString` and
//!   bind `.on_change(...)`. The component handles key input itself — useful
//!   for lightweight search filters where IME/paste/selection isn't needed.
//! - **Embedded [`TextField`]**: pass `.text_field(Entity<TextField>)`. The
//!   component renders the supplied `TextField` inside the capsule, giving
//!   the user cursor, selection, paste, IME composition, and undo — the
//!   full editing surface expected by the HIG `#searching` pattern.
//!
//! New callers targeting macOS should prefer the embedded [`TextField`]
//! mode; the host-managed mode stays available for integrations that
//! drive the search buffer themselves.
//!
//! # HIG additions covered
//!
//! - Cancel button revealed on focus ([`SearchField::on_cancel`]).
//! - Scope bar segmented control ([`SearchField::scopes`]).
//! - Recent searches section in the dropdown ([`SearchField::recent_searches`]).
//! - Inline token chips ([`SearchField::tokens`]).
//! - HIG-specified filled-circle clear glyph (`xmark.circle.fill`).

use crate::callback_types::OnToggle;
use gpui::prelude::*;
use gpui::{
    App, ElementId, Entity, FocusHandle, KeyDownEvent, MouseDownEvent, SharedString, Window, div,
    px,
};

use crate::callback_types::{
    OnSharedStringChange, OnSharedStringRefChange, OnUsizeChange, rc_wrap,
};

type OnSimple = Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>;
use crate::components::navigation_and_search::token_field::TokenItem;
use crate::components::selection_and_input::segmented_control::{SegmentItem, SegmentedControl};
use crate::components::selection_and_input::text_field::TextField;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::layout::DROPDOWN_MAX_HEIGHT;
use crate::foundations::materials::{SurfaceContext, glass_surface};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};

/// A controlled search field with suggestion dropdown per HIG.
///
/// Stateless `RenderOnce` — the parent owns the value, suggestions list, and
/// dropdown visibility, providing callbacks for all mutations. See the
/// module-level docs for the two supported input modes.
///
/// # Example (host-managed)
///
/// ```ignore
/// SearchField::new("search")
///     .value("rust")
///     .placeholder("Search docs...")
///     .suggestions(vec!["rust-lang".into(), "rust-analyzer".into()])
///     .show_suggestions(true)
///     .on_change(|new_val, _window, cx| { /* filter suggestions */ })
///     .on_select_suggestion(|item, _window, cx| { /* pick suggestion */ })
/// ```
///
/// # Example (embedded TextField)
///
/// ```ignore
/// let text_field = cx.new(TextField::new);
/// SearchField::new("search")
///     .text_field(text_field.clone())
///     .placeholder("Search...")
///     .on_cancel(|_window, cx| { /* clear + dismiss */ })
/// ```
#[derive(IntoElement)]
pub struct SearchField {
    id: ElementId,
    value: SharedString,
    placeholder: SharedString,
    suggestions: Vec<SharedString>,
    show_suggestions: bool,
    on_change: OnSharedStringChange,
    on_select_suggestion: OnSharedStringRefChange,
    on_toggle_suggestions: OnToggle,
    focus_handle: Option<FocusHandle>,
    focused: bool,
    highlighted_suggestion: Option<usize>,
    text_field: Option<Entity<TextField>>,
    on_cancel: OnSimple,
    recent_searches: Vec<SharedString>,
    on_clear_recents: OnSimple,
    scopes: Vec<SharedString>,
    active_scope: Option<usize>,
    on_scope_change: OnUsizeChange,
    tokens: Vec<TokenItem>,
    on_remove_token: OnSharedStringRefChange,
    /// Fired when the keyboard highlight moves between suggestions (Up /
    /// Down / Home / End). Parents should update their tracked
    /// `highlighted_suggestion` to the emitted value; when `None` is
    /// emitted the highlight has cleared (e.g. Home with no items).
    on_highlight: Option<Box<dyn Fn(Option<usize>, &mut Window, &mut App) + 'static>>,
}

impl SearchField {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: SharedString::default(),
            placeholder: SharedString::from("Search"),
            suggestions: Vec::new(),
            show_suggestions: false,
            on_change: None,
            on_select_suggestion: None,
            on_toggle_suggestions: None,
            focus_handle: None,
            focused: false,
            highlighted_suggestion: None,
            text_field: None,
            on_cancel: None,
            recent_searches: Vec::new(),
            on_clear_recents: None,
            scopes: Vec::new(),
            active_scope: None,
            on_scope_change: None,
            tokens: Vec::new(),
            on_remove_token: None,
            on_highlight: None,
        }
    }

    /// Set the current search value (host-managed mode).
    pub fn value(mut self, text: impl Into<SharedString>) -> Self {
        self.value = text.into();
        self
    }

    /// Set the placeholder text shown when value is empty (default "Search").
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set the list of suggestions to display in the dropdown.
    pub fn suggestions(mut self, suggestions: Vec<SharedString>) -> Self {
        self.suggestions = suggestions;
        self
    }

    /// Control whether the suggestion dropdown is visible.
    pub fn show_suggestions(mut self, show: bool) -> Self {
        self.show_suggestions = show;
        self
    }

    /// Set the callback fired when the search value changes.
    pub fn on_change(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Set the callback fired when a suggestion is selected from the dropdown.
    pub fn on_select_suggestion(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_suggestion = Some(Box::new(handler));
        self
    }

    /// Set the callback fired when the suggestion dropdown should be toggled
    /// (e.g. dismissed via Escape key).
    pub fn on_toggle_suggestions(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_suggestions = Some(Box::new(handler));
        self
    }

    /// Provide an explicit focus handle for keyboard-driven hosts and tests.
    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
        self
    }

    /// Marks this search field as keyboard-focused, showing a visible focus ring.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the keyboard-highlighted suggestion index.
    pub fn highlighted_suggestion(mut self, index: Option<usize>) -> Self {
        self.highlighted_suggestion = index;
        self
    }

    /// Embed a real [`TextField`] for input. Replaces the host-managed
    /// key-event accumulator with GPUI's full text-editing surface (cursor,
    /// selection, IME, paste, undo). The caller owns the [`TextField`]
    /// entity and controls its change callback via `field.set_on_change`.
    ///
    /// Clicking the built-in clear button wipes the embedded field via
    /// `TextField::set_text("", …)`, which fires the TextField's `on_change`
    /// with an empty string before the `SearchField`-level `on_change` runs.
    /// A host that wires both callbacks will therefore observe two
    /// empty-string events for a single clear click — register on only one
    /// side, or dedupe explicitly.
    pub fn text_field(mut self, field: Entity<TextField>) -> Self {
        self.text_field = Some(field);
        self
    }

    /// Install a Cancel button that appears when the field is focused.
    /// Firing the handler typically clears the current value and dismisses
    /// the search UI, matching `UISearchController`'s Cancel affordance.
    pub fn on_cancel(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Box::new(handler));
        self
    }

    /// Provide a list of recent searches rendered at the top of the
    /// dropdown when the field is focused and the value is empty. Pair
    /// with [`SearchField::on_clear_recents`] to wire the "Clear" button.
    pub fn recent_searches(mut self, items: Vec<SharedString>) -> Self {
        self.recent_searches = items;
        self
    }

    /// Callback fired when the user clicks "Clear" next to the Recent
    /// Searches header.
    pub fn on_clear_recents(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_clear_recents = Some(Box::new(handler));
        self
    }

    /// Supply a list of scope labels. Rendered as a segmented control
    /// beneath the search capsule, matching `UISearchController`'s scope
    /// bar. HIG v2: "A scope control acts like a segmented control for
    /// choosing a category for the search."
    pub fn scopes(mut self, scopes: Vec<SharedString>) -> Self {
        self.scopes = scopes;
        self
    }

    /// Zero-based index of the currently active scope.
    pub fn active_scope(mut self, index: Option<usize>) -> Self {
        self.active_scope = index;
        self
    }

    /// Callback fired when the user selects a different scope.
    pub fn on_scope_change(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_scope_change = Some(Box::new(handler));
        self
    }

    /// Supply inline token chips rendered inside the capsule before the
    /// text cursor. Useful for pinning active filters (e.g. mailbox tokens
    /// in Mail search) next to the live query. Callers wishing to let the
    /// user remove tokens must also supply [`SearchField::on_remove_token`].
    pub fn tokens(mut self, tokens: Vec<TokenItem>) -> Self {
        self.tokens = tokens;
        self
    }

    /// Callback fired when the user clicks a token's close button. The
    /// callback receives the token's `id`.
    pub fn on_remove_token(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_remove_token = Some(Box::new(handler));
        self
    }

    /// Callback fired when the keyboard highlight moves between
    /// suggestions in the dropdown (Up / Down / Home / End). The parent
    /// is expected to update its tracked `highlighted_suggestion` value
    /// in response so the next render reflects the new selection. HIG
    /// *Searching*: arrow keys must walk the suggestion list when the
    /// dropdown is open.
    pub fn on_highlight(
        mut self,
        handler: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_highlight = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SearchField {
    fn render(mut self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let has_value = !self.value.is_empty();
        let has_text_field = self.text_field.is_some();

        let text_color = if has_value {
            theme.label_color(SurfaceContext::GlassDim)
        } else {
            theme.placeholder_text()
        };

        let display_text: SharedString = if has_value {
            self.value.clone()
        } else {
            self.placeholder.clone()
        };

        let icon_size = theme.icon_size_inline;

        // Wrap callbacks in Rc upfront so they can be reused by the input surface and dropdown.
        let on_change_rc = rc_wrap(self.on_change);
        let on_select_rc = rc_wrap(self.on_select_suggestion);
        let on_toggle_rc = rc_wrap(self.on_toggle_suggestions);
        let on_remove_token_rc = rc_wrap(self.on_remove_token);
        let highlighted_suggestion = self.highlighted_suggestion;
        let suggestion_count = self.suggestions.len();
        let suggestion_values = self.suggestions.clone();

        // -- Input area: search icon + tokens + text/TextField + clear button --
        let mut input_row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(theme.spacing_sm)
            .min_h(px(theme.target_size()))
            .px(theme.spacing_md)
            .py(theme.spacing_sm);

        // Search icon (left)
        input_row = input_row.child(
            Icon::new(IconName::Search)
                .size(icon_size)
                .color(theme.text_muted),
        );

        // Inline token chips.
        for token in &self.tokens {
            let chip_id = ElementId::from(SharedString::from(format!("sf-token-{}", token.id)));
            let mut chip = div()
                .flex()
                .flex_row()
                .items_center()
                .gap(theme.spacing_xs)
                .px(theme.spacing_sm_md)
                .py(theme.spacing_xs)
                .rounded(theme.radius_full)
                .bg(theme.semantic.quaternary_system_fill)
                .child(
                    div()
                        .text_style(TextStyle::Caption1, theme)
                        .text_color(theme.text)
                        .child(token.label.clone()),
                );

            if token.removable
                && let Some(ref handler) = on_remove_token_rc
            {
                let handler = handler.clone();
                let tid = token.id.clone();
                chip = chip.child(
                    div()
                        .id(ElementId::from(SharedString::from(format!(
                            "sf-token-x-{}",
                            token.id
                        ))))
                        .debug_selector({
                            let id = token.id.clone();
                            move || format!("search-field-token-remove-{id}")
                        })
                        .cursor_pointer()
                        .on_click(move |_event, window, cx| {
                            handler(&tid, window, cx);
                        })
                        .child(
                            Icon::new(IconName::X)
                                .size(px(10.0))
                                .color(theme.text_muted),
                        ),
                );
            }

            let _ = chip_id;
            input_row = input_row.child(chip);
        }

        // Text display (or embedded TextField).
        if let Some(text_field) = self.text_field.clone() {
            input_row = input_row.child(div().flex_1().child(text_field));
        } else {
            input_row = input_row.child(
                div()
                    .flex_1()
                    .text_style(TextStyle::Body, theme)
                    .text_color(text_color)
                    .child(display_text),
            );
        }

        // Clear button (right) — always reserve space to prevent layout shift.
        // Visible only when the value is non-empty and on_change is provided.
        // Uses HIG-specified xmark.circle.fill (filled-circle X).
        {
            let clear_id = ElementId::from((self.id.clone(), "clear"));
            let show_clear = has_value && (on_change_rc.is_some() || has_text_field);
            let mut clear_btn = div()
                .id(clear_id)
                .debug_selector(|| "search-field-clear".into())
                .min_w(px(theme.target_size()))
                .min_h(px(theme.target_size()))
                .flex()
                .items_center()
                .justify_center();

            if show_clear {
                let text_field_for_clear = self.text_field.clone();
                let clear_handler = on_change_rc.clone();
                clear_btn = clear_btn
                    .cursor_pointer()
                    .on_click(move |_event, window, cx| {
                        if let Some(ref field) = text_field_for_clear {
                            field.update(cx, |tf, cx| {
                                tf.set_text("", window, cx);
                            });
                        }
                        if let Some(ref handler) = clear_handler {
                            handler(SharedString::default(), window, cx);
                        }
                    })
                    .child(
                        Icon::new(IconName::XmarkCircleFill)
                            .size(icon_size)
                            .color(theme.text_muted),
                    );
            } else {
                clear_btn = clear_btn.opacity(0.0).child(
                    Icon::new(IconName::XmarkCircleFill)
                        .size(icon_size)
                        .color(theme.text_muted),
                );
            }

            input_row = input_row.child(clear_btn);
        }

        // Glass-styled capsule container with focusable
        let suggestions_id = ElementId::from((self.id.clone(), "suggestions"));
        let mut input_surface = glass_surface(input_row, theme, GlassSize::Small)
            .rounded(theme.radius_full)
            .id(self.id.clone())
            .debug_selector(|| "search-field-input".into())
            .focusable();

        if let Some(ref handle) = self.focus_handle {
            input_surface = input_surface.track_focus(handle);
        }

        // Subtle focus ring: thin accent border instead of heavy box-shadow.
        if self.focused {
            input_surface = input_surface.border_2().border_color(theme.accent);
        } else {
            input_surface = input_surface.border_1().border_color(theme.border);
        }

        // Basic text input handling on the capsule itself (host-managed
        // mode). Skipped when `text_field` is set — GPUI's TextField
        // then owns the editing surface.
        if !has_text_field {
            let capsule_on_change = on_change_rc.clone();
            let capsule_on_select = on_select_rc.clone();
            let capsule_on_toggle = on_toggle_rc.clone();
            let capsule_suggestion_values = suggestion_values.clone();
            let capsule_value = self.value.clone();
            input_surface = input_surface.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();

                if let Some(ref handler) = capsule_on_change {
                    match key {
                        "backspace" if !capsule_value.is_empty() => {
                            let mut s = capsule_value.to_string();
                            s.pop();
                            handler(SharedString::from(s), window, cx);
                        }
                        k if k.len() == 1 && !event.keystroke.modifiers.platform => {
                            let mut s = capsule_value.to_string();
                            s.push_str(k);
                            handler(SharedString::from(s), window, cx);
                        }
                        _ => {}
                    }
                }

                match key {
                    "escape" => {
                        if let Some(ref handler) = capsule_on_toggle {
                            handler(false, window, cx);
                        }
                    }
                    "enter" => {
                        if let Some(idx) = highlighted_suggestion
                            && idx < suggestion_count
                            && let Some(ref handler) = capsule_on_select
                        {
                            handler(&capsule_suggestion_values[idx], window, cx);
                        }
                    }
                    _ => {}
                }
            });
        }

        // Optional Cancel button on focus, per HIG.
        let mut capsule_row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(theme.spacing_sm)
            .child(div().flex_1().child(input_surface));

        if self.focused
            && let Some(on_cancel) = self.on_cancel
        {
            let cancel_id = ElementId::from((self.id.clone(), "cancel"));
            let handler = std::rc::Rc::new(on_cancel);
            capsule_row = capsule_row.child(
                div()
                    .id(cancel_id)
                    .debug_selector(|| "search-field-cancel".into())
                    .cursor_pointer()
                    .px(theme.spacing_sm)
                    .py(theme.spacing_xs)
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.accent)
                    .on_click(move |_event, window, cx| {
                        handler(window, cx);
                    })
                    .child("Cancel"),
            );
        }

        // Optional scope bar beneath the capsule.
        let scope_bar = if !self.scopes.is_empty() {
            let scope_id = ElementId::from((self.id.clone(), "scopes"));
            let items: Vec<SegmentItem> =
                self.scopes.iter().cloned().map(SegmentItem::new).collect();
            let mut seg = SegmentedControl::new(scope_id)
                .items(items)
                .selected(self.active_scope.unwrap_or(0));
            if let Some(handler) = self.on_scope_change {
                seg = seg.on_change(handler);
            }
            Some(seg)
        } else {
            None
        };

        // -- Dropdown: recent searches + suggestions --
        let mut container = div()
            .flex()
            .flex_col()
            .gap(theme.spacing_sm)
            .relative()
            .debug_selector(|| "search-field-root".into())
            .child(capsule_row);

        if let Some(scope_bar) = scope_bar {
            container = container.child(scope_bar);
        }

        // Recents appear when focused and the query is empty; suggestions
        // appear when `show_suggestions` is toggled. Both may be combined —
        // the recents section precedes the suggestion list.
        let show_recents = self.focused && !has_value && !self.recent_searches.is_empty();
        let show_dropdown = show_recents || (self.show_suggestions && !self.suggestions.is_empty());

        if show_dropdown {
            let on_select = on_select_rc.clone();
            let on_toggle = on_toggle_rc.clone();
            let list_suggestion_values = suggestion_values.clone();

            let list_div = div()
                .absolute()
                .left_0()
                .top(theme.dropdown_top())
                .w_full()
                .flex()
                .flex_col()
                .overflow_hidden()
                .max_h(px(DROPDOWN_MAX_HEIGHT));

            let mut list = glass_surface(list_div, theme, GlassSize::Medium)
                .id(suggestions_id)
                .debug_selector(|| "search-field-suggestions".into())
                .focusable();

            // Keyboard nav: Up/Down/Enter/Home/End/Escape.
            let key_on_toggle = on_toggle.clone();
            let key_on_select = on_select.clone();
            let key_on_highlight = self.on_highlight.take().map(std::rc::Rc::new);
            list = list.on_key_down(move |event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "escape" => {
                        if let Some(ref handler) = key_on_toggle {
                            handler(false, window, cx);
                        }
                    }
                    "enter" => {
                        if let Some(idx) = highlighted_suggestion
                            && idx < suggestion_count
                            && let Some(ref handler) = key_on_select
                        {
                            handler(&list_suggestion_values[idx], window, cx);
                        }
                    }
                    "down" => {
                        if suggestion_count == 0 {
                            return;
                        }
                        cx.stop_propagation();
                        let next = match highlighted_suggestion {
                            Some(i) if i + 1 < suggestion_count => Some(i + 1),
                            Some(_) => Some(0), // wrap to first
                            None => Some(0),
                        };
                        if let Some(ref handler) = key_on_highlight {
                            handler(next, window, cx);
                        }
                    }
                    "up" => {
                        if suggestion_count == 0 {
                            return;
                        }
                        cx.stop_propagation();
                        let next = match highlighted_suggestion {
                            Some(0) | None => Some(suggestion_count - 1), // wrap to last
                            Some(i) => Some(i - 1),
                        };
                        if let Some(ref handler) = key_on_highlight {
                            handler(next, window, cx);
                        }
                    }
                    "home" => {
                        if suggestion_count == 0 {
                            return;
                        }
                        cx.stop_propagation();
                        if let Some(ref handler) = key_on_highlight {
                            handler(Some(0), window, cx);
                        }
                    }
                    "end" => {
                        if suggestion_count == 0 {
                            return;
                        }
                        cx.stop_propagation();
                        if let Some(ref handler) = key_on_highlight {
                            handler(Some(suggestion_count - 1), window, cx);
                        }
                    }
                    _ => {}
                }
            });

            // Close dropdown on click outside.
            let mouse_out_toggle = on_toggle.clone();
            if let Some(handler) = mouse_out_toggle {
                list = list.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
                    handler(false, window, cx);
                });
            }

            // Glass-aware hover background for highlighted item.
            let hover_bg = theme.hover_bg();

            // Recent searches section header + items.
            if show_recents {
                let header_id =
                    ElementId::from((crate::ids::next_element_id("sf-recents-header"), "hdr"));
                let mut header = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(theme.spacing_sm)
                    .px(theme.spacing_md)
                    .py(theme.spacing_xs)
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.secondary_label_color(SurfaceContext::GlassDim))
                    .child(div().flex_1().child("Recent Searches"));

                if let Some(clear_handler) = self.on_clear_recents {
                    let handler = std::rc::Rc::new(clear_handler);
                    header = header.child(
                        div()
                            .id(ElementId::from((header_id, "clear")))
                            .debug_selector(|| "search-field-recents-clear".into())
                            .cursor_pointer()
                            .text_color(theme.accent)
                            .on_click(move |_event, window, cx| {
                                handler(window, cx);
                            })
                            .child("Clear"),
                    );
                }

                list = list.child(header);

                for (idx, recent) in self.recent_searches.iter().enumerate() {
                    let on_select_for_recent = on_select.clone();
                    let item_value = recent.clone();
                    let row = div()
                        .id(ElementId::NamedInteger("sf-recent".into(), idx as u64))
                        .debug_selector(|| format!("search-field-recent-{idx}"))
                        .min_h(px(theme.target_size()))
                        .flex()
                        .items_center()
                        .gap(theme.spacing_sm)
                        .px(theme.spacing_md)
                        .cursor_pointer()
                        .hover(|style| style.bg(hover_bg))
                        .child(
                            Icon::new(IconName::Loader)
                                .size(px(12.0))
                                .color(theme.text_muted),
                        )
                        .child(
                            div()
                                .text_style(TextStyle::Body, theme)
                                .text_color(theme.text)
                                .child(recent.clone()),
                        )
                        .on_click(move |_event, window, cx| {
                            if let Some(handler) = &on_select_for_recent {
                                handler(&item_value, window, cx);
                            }
                        });
                    list = list.child(row);
                }

                if self.show_suggestions && !self.suggestions.is_empty() {
                    list = list.child(
                        div()
                            .h(px(1.0))
                            .bg(crate::foundations::color::with_alpha(theme.border, 0.4))
                            .mx(theme.spacing_md),
                    );
                }
            }

            for (idx, suggestion) in self.suggestions.iter().enumerate() {
                let on_select = on_select.clone();
                let item_value = suggestion.clone();
                let item_label = suggestion.clone();
                let is_highlighted = highlighted_suggestion == Some(idx);

                let mut row = div()
                    .id(ElementId::NamedInteger("search-sug".into(), idx as u64))
                    .debug_selector(|| format!("search-field-suggestion-{idx}"))
                    .min_h(px(theme.target_size()))
                    .flex()
                    .items_center()
                    .px(theme.spacing_md)
                    .cursor_pointer()
                    .hover(|style| style.bg(hover_bg));

                if is_highlighted {
                    row = row.bg(hover_bg);
                }

                row = row.child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .child(item_label),
                );

                row = row.on_click(move |_event, window, cx| {
                    if let Some(handler) = &on_select {
                        handler(&item_value, window, cx);
                    }
                });

                list = list.child(row);
            }

            container = container.child(list);
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use gpui::SharedString;

    use super::SearchField;
    use crate::components::navigation_and_search::token_field::TokenItem;

    #[test]
    fn search_field_defaults() {
        let field = SearchField::new("test");
        assert!(field.value.is_empty());
        assert_eq!(field.placeholder.as_ref(), "Search");
        assert!(field.suggestions.is_empty());
        assert!(!field.show_suggestions);
        assert!(field.on_change.is_none());
        assert!(field.on_select_suggestion.is_none());
        assert!(field.on_toggle_suggestions.is_none());
        assert!(!field.focused);
        assert!(field.highlighted_suggestion.is_none());
        assert!(field.text_field.is_none());
        assert!(field.on_cancel.is_none());
        assert!(field.recent_searches.is_empty());
        assert!(field.scopes.is_empty());
        assert!(field.active_scope.is_none());
        assert!(field.tokens.is_empty());
        assert!(field.on_remove_token.is_none());
    }

    #[test]
    fn search_field_value_builder() {
        let field = SearchField::new("test").value("hello");
        assert_eq!(field.value.as_ref(), "hello");
    }

    #[test]
    fn search_field_placeholder_builder() {
        let field = SearchField::new("test").placeholder("Find...");
        assert_eq!(field.placeholder.as_ref(), "Find...");
    }

    #[test]
    fn search_field_suggestions_builder() {
        let items = vec![SharedString::from("Alpha"), SharedString::from("Beta")];
        let field = SearchField::new("test").suggestions(items);
        assert_eq!(field.suggestions.len(), 2);
        assert_eq!(field.suggestions[0].as_ref(), "Alpha");
        assert_eq!(field.suggestions[1].as_ref(), "Beta");
    }

    #[test]
    fn search_field_show_suggestions_builder() {
        let field = SearchField::new("test").show_suggestions(true);
        assert!(field.show_suggestions);
    }

    #[test]
    fn search_field_callbacks_are_some() {
        let field = SearchField::new("test")
            .on_change(|_, _, _| {})
            .on_select_suggestion(|_, _, _| {});
        assert!(field.on_change.is_some());
        assert!(field.on_select_suggestion.is_some());
    }

    #[test]
    fn search_field_chained_builders() {
        let items = vec![SharedString::from("Rust")];
        let field = SearchField::new("test")
            .value("rs")
            .placeholder("Type to search")
            .suggestions(items)
            .show_suggestions(true);
        assert_eq!(field.value.as_ref(), "rs");
        assert_eq!(field.placeholder.as_ref(), "Type to search");
        assert_eq!(field.suggestions.len(), 1);
        assert!(field.show_suggestions);
    }

    #[test]
    fn search_field_focused_builder() {
        let field = SearchField::new("test").focused(true);
        assert!(field.focused);
    }

    #[test]
    fn search_field_highlighted_suggestion_builder() {
        let field = SearchField::new("test").highlighted_suggestion(Some(3));
        assert_eq!(field.highlighted_suggestion, Some(3));
    }

    #[test]
    fn search_field_highlighted_suggestion_none() {
        let field = SearchField::new("test").highlighted_suggestion(None);
        assert_eq!(field.highlighted_suggestion, None);
    }

    #[test]
    fn search_field_on_toggle_suggestions_is_some() {
        let field = SearchField::new("test").on_toggle_suggestions(|_, _, _| {});
        assert!(field.on_toggle_suggestions.is_some());
    }

    #[test]
    fn search_field_on_cancel_builder() {
        let field = SearchField::new("test").on_cancel(|_, _| {});
        assert!(field.on_cancel.is_some());
    }

    #[test]
    fn search_field_recent_searches_builder() {
        let field = SearchField::new("test").recent_searches(vec!["foo".into(), "bar".into()]);
        assert_eq!(field.recent_searches.len(), 2);
    }

    #[test]
    fn search_field_on_clear_recents_builder() {
        let field = SearchField::new("test").on_clear_recents(|_, _| {});
        assert!(field.on_clear_recents.is_some());
    }

    #[test]
    fn search_field_scopes_builder() {
        let field = SearchField::new("test")
            .scopes(vec!["All".into(), "Inbox".into()])
            .active_scope(Some(1))
            .on_scope_change(|_, _, _| {});
        assert_eq!(field.scopes.len(), 2);
        assert_eq!(field.active_scope, Some(1));
        assert!(field.on_scope_change.is_some());
    }

    #[test]
    fn search_field_tokens_builder() {
        let field = SearchField::new("test")
            .tokens(vec![TokenItem::new("rust", "Rust")])
            .on_remove_token(|_, _, _| {});
        assert_eq!(field.tokens.len(), 1);
        assert!(field.on_remove_token.is_some());
    }
}

#[cfg(test)]
mod interaction_tests {
    use gpui::{Context, FocusHandle, IntoElement, Render, SharedString, TestAppContext};

    use super::SearchField;
    use crate::test_helpers::helpers::{
        InteractionExt, assert_element_absent, assert_element_exists, setup_test_window,
    };

    const SEARCH_CLEAR: &str = "search-field-clear";
    const SEARCH_SUGGESTIONS: &str = "search-field-suggestions";
    const SEARCH_SUGGESTION_1: &str = "search-field-suggestion-1";
    const SEARCH_CANCEL: &str = "search-field-cancel";

    struct SearchFieldHarness {
        focus_handle: FocusHandle,
        value: SharedString,
        suggestions: Vec<SharedString>,
        show_suggestions: bool,
        highlighted: Option<usize>,
        selected: Option<SharedString>,
        toggle_events: Vec<bool>,
        cancel_events: usize,
    }

    impl SearchFieldHarness {
        fn new(
            cx: &mut Context<Self>,
            value: impl Into<SharedString>,
            show_suggestions: bool,
            highlighted: Option<usize>,
        ) -> Self {
            Self {
                focus_handle: cx.focus_handle(),
                value: value.into(),
                suggestions: vec![SharedString::from("Rust"), SharedString::from("Ruby")],
                show_suggestions,
                highlighted,
                selected: None,
                toggle_events: Vec::new(),
                cancel_events: 0,
            }
        }
    }

    impl Render for SearchFieldHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let entity = cx.entity().clone();
            SearchField::new("search")
                .focus_handle(self.focus_handle.clone())
                .value(self.value.clone())
                .suggestions(self.suggestions.clone())
                .show_suggestions(self.show_suggestions)
                .highlighted_suggestion(self.highlighted)
                .focused(true)
                .on_change(move |value, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.value = value.clone();
                        this.show_suggestions = !value.is_empty() && !this.suggestions.is_empty();
                        this.highlighted = this.show_suggestions.then_some(0);
                        cx.notify();
                    });
                })
                .on_select_suggestion({
                    let entity = cx.entity().clone();
                    move |value, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.value = value.clone();
                            this.selected = Some(value.clone());
                            this.show_suggestions = false;
                            this.highlighted = None;
                            cx.notify();
                        });
                    }
                })
                .on_toggle_suggestions({
                    let entity = cx.entity().clone();
                    move |show, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.show_suggestions = show;
                            if !show {
                                this.highlighted = None;
                            }
                            this.toggle_events.push(show);
                            cx.notify();
                        });
                    }
                })
                .on_cancel({
                    let entity = cx.entity().clone();
                    move |_window, cx| {
                        entity.update(cx, |this, cx| {
                            this.cancel_events += 1;
                            this.value = SharedString::default();
                            cx.notify();
                        });
                    }
                })
        }
    }

    fn focus_search_field(
        host: &gpui::Entity<SearchFieldHarness>,
        cx: &mut gpui::VisualTestContext,
    ) {
        host.update_in(cx, |host, window, cx| {
            host.focus_handle.focus(window, cx);
        });
    }

    #[gpui::test]
    async fn typing_updates_value_and_shows_suggestions(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            SearchFieldHarness::new(cx, "", false, None)
        });

        focus_search_field(&host, cx);
        cx.press("r");
        cx.press("u");

        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.value.as_ref(), "ru");
            assert!(host.show_suggestions);
            assert_eq!(host.highlighted, Some(0));
        });
        assert_element_exists(cx, SEARCH_SUGGESTIONS);
    }

    #[gpui::test]
    async fn clicking_suggestion_selects_and_hides_dropdown(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            SearchFieldHarness::new(cx, "r", true, Some(0))
        });

        cx.click_on(SEARCH_SUGGESTION_1);

        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.value.as_ref(), "Ruby");
            assert_eq!(
                host.selected.as_ref().map(SharedString::as_ref),
                Some("Ruby")
            );
            assert!(!host.show_suggestions);
        });
        assert_element_absent(cx, SEARCH_SUGGESTIONS);
    }

    #[gpui::test]
    async fn enter_selects_highlighted_suggestion(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            SearchFieldHarness::new(cx, "r", true, Some(1))
        });

        focus_search_field(&host, cx);
        cx.press("enter");

        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(
                host.selected.as_ref().map(SharedString::as_ref),
                Some("Ruby")
            );
            assert_eq!(host.value.as_ref(), "Ruby");
            assert!(!host.show_suggestions);
        });
    }

    #[gpui::test]
    async fn clear_button_clears_existing_value(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            SearchFieldHarness::new(cx, "Rust", false, None)
        });

        assert_element_exists(cx, SEARCH_CLEAR);
        cx.click_on(SEARCH_CLEAR);

        host.update_in(cx, |host, _window, _cx| {
            assert!(host.value.is_empty());
            assert!(!host.show_suggestions);
        });
    }

    #[gpui::test]
    async fn escape_hides_suggestions(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            SearchFieldHarness::new(cx, "r", true, Some(0))
        });

        focus_search_field(&host, cx);
        cx.press("escape");

        host.update_in(cx, |host, _window, _cx| {
            assert!(!host.show_suggestions);
            assert_eq!(host.toggle_events.last().copied(), Some(false));
        });
        assert_element_absent(cx, SEARCH_SUGGESTIONS);
    }

    #[gpui::test]
    async fn cancel_button_fires_on_cancel(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            SearchFieldHarness::new(cx, "Rust", false, None)
        });

        assert_element_exists(cx, SEARCH_CANCEL);
        cx.click_on(SEARCH_CANCEL);

        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.cancel_events, 1);
            assert!(host.value.is_empty());
        });
    }
}
