//! Tag/chip input per HIG Token Fields.
//!
//! Manages a list of tokens with inline text entry. Each token renders as a
//! capsule chip with an optional close button. The inline text area accepts
//! keyboard input: any key in [`TokenField::commit_keys`] commits the
//! current text as a new token (default: `Enter` and `,`), and Backspace
//! when the input is empty removes the last token.
//!
//! # Text input modes
//!
//! - **Default**: the field uses its built-in key-event accumulator for
//!   simple tag entry.
//! - **Embedded [`TextField`]**: call [`TokenField::set_text_field`] with
//!   an `Entity<TextField>` to get full GPUI text-editing (cursor,
//!   selection, IME, paste, undo). The TokenField installs its own
//!   `on_change` on the TextField to monitor for commit-key characters.
//!
//! # SearchField composition
//!
//! HIG v2: "Tokens can also represent search terms in some situations; for
//! guidance, see Search fields." Tokens and [`SearchField`](super::SearchField)
//! compose — pass the same [`TokenItem`] list to both components and wire
//! the same `on_remove_token` callback to keep them in sync. See
//! [`SearchField::tokens`](super::SearchField::tokens).

use gpui::prelude::*;
use gpui::{
    App, ElementId, FocusHandle, Focusable, KeyDownEvent, MouseButton, MouseDownEvent,
    SharedString, Window, div, px,
};

use crate::callback_types::OnStrChange;
use crate::components::selection_and_input::text_field::TextField;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use crate::ids::next_element_id;
use crate::text_actions::{Backspace, Delete, SelectAll};

/// Default keys that commit the current input text as a new token. Matches
/// the HIG default ("text people enter turns into a token whenever they
/// type a comma" + Enter/Return per convention).
pub const DEFAULT_COMMIT_KEYS: &[&str] = &["enter", "return", ","];

/// A single token displayed inside a [`TokenField`].
///
/// Tokens are opaque label/id pairs. Set `removable` to `false` to hide the
/// close button (e.g. for required tags).
#[derive(Clone)]
pub struct TokenItem {
    pub label: SharedString,
    pub id: SharedString,
    pub removable: bool,
}

impl TokenItem {
    /// Creates a new removable token.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            removable: true,
        }
    }

    /// Creates a non-removable token.
    pub fn fixed(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            removable: false,
        }
    }
}

/// Action menu shown when the user right-clicks a token.
#[derive(Clone)]
pub struct TokenContextMenuItem {
    pub label: SharedString,
    pub destructive: bool,
}

impl TokenContextMenuItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            destructive: false,
        }
    }

    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }
}

type OnContextMenuItems =
    Option<Box<dyn Fn(&SharedString, &mut App) -> Vec<TokenContextMenuItem> + 'static>>;

type OnContextMenuAction =
    Option<Box<dyn Fn(&SharedString, &SharedString, &mut Window, &mut App) + 'static>>;

/// A tag/chip input field per HIG Token Fields.
///
/// Renders a flex-wrap row of capsule chips with an inline text entry area.
/// Keyboard shortcuts on the focused container:
/// - Any key in [`TokenField::commit_keys`] commits the current input
///   text as a new token via `on_add`. Default: `Enter`, `Return`, `,`.
/// - **Backspace**: when the input is empty, removes the last token via `on_remove`.
pub struct TokenField {
    element_id: ElementId,
    tokens: Vec<TokenItem>,
    input_text: String,
    focus_handle: FocusHandle,
    on_add: OnStrChange,
    on_remove: OnStrChange,
    commit_keys: Vec<String>,
    text_field: Option<gpui::Entity<TextField>>,
    suggestions: Vec<SharedString>,
    suggestion_delay_ms: u64,
    show_suggestions: bool,
    highlighted_suggestion: Option<usize>,
    on_context_menu_items: OnContextMenuItems,
    on_context_menu_action: OnContextMenuAction,
    context_open_for: Option<SharedString>,
    /// When true, every token is in the "selected" state drawn with the
    /// accent tint. Toggled on by Cmd+A when the embedded text field is
    /// empty and focused; cleared on Backspace/Delete (after removing
    /// tokens), any other keystroke, or an outside click.
    all_tokens_selected: bool,
}

impl TokenField {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            element_id: next_element_id("token-field"),
            tokens: Vec::new(),
            input_text: String::new(),
            focus_handle: cx.focus_handle(),
            on_add: None,
            on_remove: None,
            commit_keys: DEFAULT_COMMIT_KEYS
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            text_field: None,
            suggestions: Vec::new(),
            suggestion_delay_ms: 200,
            show_suggestions: false,
            highlighted_suggestion: None,
            on_context_menu_items: None,
            on_context_menu_action: None,
            context_open_for: None,
            all_tokens_selected: false,
        }
    }

    /// Replace all tokens.
    pub fn set_tokens(&mut self, tokens: Vec<TokenItem>, cx: &mut Context<Self>) {
        self.tokens = tokens;
        cx.notify();
    }

    /// Set the callback fired when the user commits a new token. The
    /// callback receives the token label text.
    pub fn set_on_add(&mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) {
        self.on_add = Some(Box::new(handler));
    }

    /// Set the callback fired when a token is removed (X click or Backspace).
    /// The callback receives the token id.
    pub fn set_on_remove(&mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) {
        self.on_remove = Some(Box::new(handler));
    }

    /// Override the list of keys that commit the current input as a new
    /// token. Each entry is a GPUI key name (e.g. `"enter"`, `"return"`,
    /// `","`, `";"`). Default: [`DEFAULT_COMMIT_KEYS`].
    pub fn set_commit_keys(&mut self, keys: Vec<impl Into<String>>) {
        self.commit_keys = keys.into_iter().map(Into::into).collect();
    }

    /// Replace the inline text-input surface with a GPUI [`TextField`].
    /// Subscribes to the field's on_change so that typing commit-key
    /// characters commits the current content as a token.
    pub fn set_text_field(&mut self, field: gpui::Entity<TextField>, cx: &mut Context<Self>) {
        // Weak handle: a strong clone would form a cycle because the
        // TextField stores this closure in on_change and self.text_field
        // holds the TextField.
        let weak = cx.weak_entity();
        field.update(cx, |tf, _cx| {
            tf.set_on_change(move |text, window, cx| {
                let current = text.to_string();
                // Detect trailing commit-key character (e.g. comma).
                // Enter is handled separately via the TextField's submit
                // binding when wired by the host.
                weak.update(cx, |this, cx| {
                    if let Some(commit_char) = this.commit_keys.iter().find_map(|k| {
                        let len = k.chars().count();
                        if len == 1 && current.ends_with(k.as_str()) {
                            Some(k.clone())
                        } else {
                            None
                        }
                    }) {
                        let mut trimmed = current.clone();
                        for _ in commit_char.chars() {
                            trimmed.pop();
                        }
                        let trimmed_final = trimmed.trim().to_string();
                        if !trimmed_final.is_empty()
                            && let Some(on_add) = &this.on_add
                        {
                            on_add(&trimmed_final, window, cx);
                        }
                        if let Some(ref tf) = this.text_field {
                            tf.update(cx, |tf, cx| tf.set_text("", cx));
                        }
                        cx.notify();
                    }
                })
                .ok();
            });
        });
        self.text_field = Some(field);
        cx.notify();
    }

    /// Supply a list of candidate suggestions. Rendered in a dropdown
    /// below the chip row when non-empty and the field is focused.
    pub fn set_suggestions(&mut self, suggestions: Vec<SharedString>, cx: &mut Context<Self>) {
        self.suggestions = suggestions;
        self.show_suggestions = !self.suggestions.is_empty();
        cx.notify();
    }

    /// Debounce delay applied to suggestion updates, in milliseconds. HIG:
    /// "consider adjusting the delay to a comfortable level." Stored for
    /// callers that drive suggestion computation; the default 200 ms
    /// tracks the HIG-recommended comfortable range.
    pub fn set_suggestion_delay_ms(&mut self, delay: u64) {
        self.suggestion_delay_ms = delay;
    }

    /// Access the configured suggestion delay.
    pub fn suggestion_delay_ms(&self) -> u64 {
        self.suggestion_delay_ms
    }

    /// Install a builder callback that returns the context-menu entries
    /// for a given token id. Called each time the user right-clicks a
    /// token so the entries can depend on runtime state.
    pub fn set_on_context_menu_items(
        &mut self,
        handler: impl Fn(&SharedString, &mut App) -> Vec<TokenContextMenuItem> + 'static,
    ) {
        self.on_context_menu_items = Some(Box::new(handler));
    }

    /// Install the callback fired when the user selects an entry from a
    /// token's context menu.
    pub fn set_on_context_menu_action(
        &mut self,
        handler: impl Fn(&SharedString, &SharedString, &mut Window, &mut App) + 'static,
    ) {
        self.on_context_menu_action = Some(Box::new(handler));
    }

    /// Append a single token.
    pub fn add_token(
        &mut self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.tokens.push(TokenItem::new(id, label));
        cx.notify();
    }

    /// Remove the token with the given id, if present.
    pub fn remove_token(&mut self, id: &str, cx: &mut Context<Self>) {
        self.tokens.retain(|t| t.id.as_ref() != id);
        cx.notify();
    }

    /// Returns a slice of the current tokens.
    pub fn tokens(&self) -> &[TokenItem] {
        &self.tokens
    }

    /// Returns the current input text.
    pub fn input_text(&self) -> &str {
        &self.input_text
    }

    /// Returns the configured commit keys.
    pub fn commit_keys(&self) -> &[String] {
        &self.commit_keys
    }

    /// Takes the current `input_text`, fires `on_add`, and clears the input.
    fn commit_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.input_text.trim().to_string();
        if text.is_empty() {
            return;
        }
        if let Some(on_add) = &self.on_add {
            on_add(&text, window, cx);
        }
        self.input_text.clear();
        cx.notify();
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // When a real TextField is embedded, the TextField drives input —
        // skip the manual accumulator.
        if self.text_field.is_some() {
            return;
        }

        let key = event.keystroke.key.as_str();

        // Commit-key? Any key in `commit_keys` (case-insensitive).
        let is_commit_key = self.commit_keys.iter().any(|k| k.eq_ignore_ascii_case(key));
        if is_commit_key {
            self.commit_input(window, cx);
            return;
        }

        match key {
            "backspace" => {
                if self.input_text.is_empty() {
                    if let Some(last) = self.tokens.last()
                        && last.removable
                    {
                        let id = last.id.clone();
                        if let Some(on_remove) = &self.on_remove {
                            on_remove(id.as_ref(), window, cx);
                        }
                        cx.notify();
                    }
                } else {
                    self.input_text.pop();
                    cx.notify();
                }
            }
            _ => {
                if key.len() == 1 {
                    self.input_text.push_str(key);
                    cx.notify();
                } else if key == "space" {
                    self.input_text.push(' ');
                    cx.notify();
                }
            }
        }
    }

    fn open_context_menu_for(&mut self, token_id: SharedString, cx: &mut Context<Self>) {
        self.context_open_for = Some(token_id);
        cx.notify();
    }

    fn close_context_menu(&mut self, cx: &mut Context<Self>) {
        self.context_open_for = None;
        cx.notify();
    }

    /// Returns true when the embedded TextField is present, focused, and
    /// holds no text — the state in which Cmd+A should target the chip
    /// row rather than the inner text.
    fn text_field_is_empty_and_focused(&self, window: &Window, cx: &App) -> bool {
        let Some(ref tf) = self.text_field else {
            // No embedded field. Fall back to the manual accumulator.
            return self.input_text.is_empty() && self.focus_handle.is_focused(window);
        };
        let field = tf.read(cx);
        field.text().is_empty() && field.focus_handle(cx).is_focused(window)
    }

    /// Returns `true` if there is currently at least one removable token
    /// in the selected-all state.
    #[cfg(test)]
    pub fn all_tokens_selected(&self) -> bool {
        self.all_tokens_selected
    }

    /// Select every token. Does not affect the embedded TextField text.
    fn select_all_tokens(&mut self, cx: &mut Context<Self>) {
        if self.tokens.is_empty() {
            return;
        }
        self.all_tokens_selected = true;
        cx.notify();
    }

    /// Clear the token selection without removing anything.
    fn deselect_all_tokens(&mut self, cx: &mut Context<Self>) {
        if !self.all_tokens_selected {
            return;
        }
        self.all_tokens_selected = false;
        cx.notify();
    }

    /// Remove every currently-selected token by firing `on_remove` for
    /// each removable id, then collapsing the selection state.
    fn remove_selected_tokens(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.all_tokens_selected {
            return;
        }
        // Snapshot ids before firing callbacks so the host can mutate
        // `self.tokens` via `set_tokens` without us iterating a stale view.
        let ids: Vec<SharedString> = self
            .tokens
            .iter()
            .filter(|t| t.removable)
            .map(|t| t.id.clone())
            .collect();
        if let Some(on_remove) = &self.on_remove {
            for id in &ids {
                on_remove(id.as_ref(), window, cx);
            }
        }
        self.all_tokens_selected = false;
        cx.notify();
    }
}

impl Focusable for TokenField {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TokenField {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus_handle.is_focused(_window);

        let context_target = self.context_open_for.clone();
        let context_items: Vec<TokenContextMenuItem> = if let Some(target) = context_target.as_ref()
        {
            if let Some(handler) = self.on_context_menu_items.as_ref() {
                handler(target, cx)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let theme = cx.theme();

        // Build token chips
        let mut chips: Vec<gpui::AnyElement> = Vec::new();
        let selection_active = self.all_tokens_selected && !self.tokens.is_empty();
        for token in &self.tokens {
            let label = token.label.clone();
            let id = token.id.clone();
            let removable = token.removable;

            let chip_bg = if selection_active {
                // HIG: selected tokens pick up the accent tint fill used
                // for Finder row selection.
                theme.selected_bg
            } else {
                theme.semantic.quaternary_system_fill
            };

            let mut chip = div()
                .id(ElementId::from(SharedString::from(format!(
                    "token-chip-{}",
                    id
                ))))
                .debug_selector({
                    let id = id.clone();
                    let selected = selection_active;
                    move || {
                        if selected {
                            format!("token-chip-{id}-selected")
                        } else {
                            format!("token-chip-{id}")
                        }
                    }
                })
                .flex()
                .flex_row()
                .items_center()
                .gap(theme.spacing_xs)
                .px(theme.spacing_sm_md)
                .py(theme.spacing_xs)
                .rounded(theme.radius_full)
                .min_h(px(theme.target_size()))
                .bg(chip_bg)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .child(label),
                );

            // Selected-all marker: 1pt accent border per spec.
            if selection_active {
                chip = chip.border_1().border_color(theme.accent);
            }

            // Right-click → open context menu for this token.
            if self.on_context_menu_items.is_some() {
                let open_id = id.clone();
                chip = chip.on_mouse_down(
                    MouseButton::Right,
                    cx.listener(move |this, _event: &MouseDownEvent, _window, cx| {
                        this.open_context_menu_for(open_id.clone(), cx);
                    }),
                );
            }

            if removable {
                let remove_id = id.clone();
                let entity = cx.entity().clone();
                let icon_size = (TextStyle::Body.attrs().size * 0.75).ceil();
                chip = chip.child(
                    div()
                        .id(ElementId::from(SharedString::from(format!(
                            "token-x-{}",
                            id
                        ))))
                        .debug_selector({
                            let id = id.clone();
                            move || format!("token-remove-{id}")
                        })
                        .cursor_pointer()
                        .min_w(px(theme.target_size()))
                        .min_h(px(theme.target_size()))
                        .flex()
                        .items_center()
                        .justify_center()
                        .on_click(move |_event, window, cx| {
                            entity.update(cx, |this, cx| {
                                let rid = remove_id.clone();
                                if let Some(on_remove) = &this.on_remove {
                                    on_remove(rid.as_ref(), window, cx);
                                }
                                cx.notify();
                            });
                        })
                        .child(
                            Icon::new(IconName::X)
                                .size(icon_size)
                                .color(theme.text_muted),
                        ),
                );
            }

            chips.push(chip.into_any_element());
        }

        // Inline input: TextField when embedded, otherwise text + cursor.
        let input_display = if let Some(text_field) = self.text_field.clone() {
            div()
                .flex_1()
                .min_w(px(60.0))
                .child(text_field)
                .into_any_element()
        } else {
            let display_text = if self.input_text.is_empty() {
                SharedString::default()
            } else {
                SharedString::from(self.input_text.clone())
            };
            let cursor_color = theme.accent;
            div()
                .flex_1()
                .min_w(px(60.0))
                .min_h(px(theme.target_size()))
                .flex()
                .items_center()
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .child(display_text),
                )
                .child(
                    div()
                        .w(px(2.0))
                        .h(TextStyle::Body.attrs().size)
                        .bg(cursor_color),
                )
                .into_any_element()
        };

        // Container: opaque surface (content-layer — per HIG, input fields don't use Liquid Glass)
        let mut container = div()
            .flex()
            .flex_col()
            .w_full()
            .id(self.element_id.clone())
            .debug_selector(|| "token-field-root".into())
            .track_focus(&self.focus_handle)
            // Capture-phase hooks intercept Cmd+A / Backspace / Delete
            // BEFORE the embedded TextField (focused descendant) receives
            // them via its own on_action bindings. This is the only way to
            // "borrow" those shortcuts for chip-row selection without
            // having to rebind them at the host's keybinding layer.
            .capture_action(cx.listener(
                |this: &mut Self, _action: &SelectAll, window: &mut Window, cx| {
                    if this.text_field_is_empty_and_focused(window, cx) && !this.tokens.is_empty() {
                        this.select_all_tokens(cx);
                        cx.stop_propagation();
                    }
                },
            ))
            .capture_action(cx.listener(
                |this: &mut Self, _action: &Backspace, window: &mut Window, cx| {
                    if this.all_tokens_selected {
                        this.remove_selected_tokens(window, cx);
                        cx.stop_propagation();
                    }
                },
            ))
            .capture_action(cx.listener(
                |this: &mut Self, _action: &Delete, window: &mut Window, cx| {
                    if this.all_tokens_selected {
                        this.remove_selected_tokens(window, cx);
                        cx.stop_propagation();
                    }
                },
            ))
            .on_key_down(cx.listener(Self::handle_key_down))
            // Bubble-phase: any key that wasn't handled above dismisses
            // the selected-all state before it reaches the TextField's
            // text-editing logic. Typing a character when all tokens are
            // selected feels like "replace the selection" — clearing the
            // highlight lets the TextField's normal insert path take over.
            .on_key_down(
                cx.listener(|this: &mut Self, event: &KeyDownEvent, _window, cx| {
                    if !this.all_tokens_selected {
                        return;
                    }
                    let key = event.keystroke.key.as_str();
                    let mods = &event.keystroke.modifiers;
                    // Bare modifier-only presses (cmd / shift / alt)
                    // shouldn't cancel the selection — the user is likely
                    // preparing a shortcut.
                    if matches!(key, "cmd" | "platform" | "shift" | "alt" | "control") {
                        return;
                    }
                    // Cmd+A / Backspace / Delete were handled in the
                    // capture phase; leave the selection as-is.
                    let is_cmd_a = mods.platform && key.eq_ignore_ascii_case("a");
                    if is_cmd_a || key == "backspace" || key == "delete" {
                        return;
                    }
                    this.deselect_all_tokens(cx);
                }),
            )
            .on_mouse_down_out(cx.listener(
                |this: &mut Self, _event: &MouseDownEvent, _window, cx| {
                    this.deselect_all_tokens(cx);
                },
            ));

        let mut field_row = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(theme.spacing_xs)
            .p(theme.spacing_sm)
            .bg(theme.surface)
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius_md);

        for chip in chips {
            field_row = field_row.child(chip);
        }
        field_row = field_row.child(input_display);

        container = container.child(field_row);
        let mut outer = container;

        // Suggestion dropdown, shown when focused + suggestions are present.
        if focused && self.show_suggestions && !self.suggestions.is_empty() {
            let hover_bg = theme.hover_bg();
            let mut list = div()
                .id(ElementId::from((self.element_id.clone(), "suggestions")))
                .debug_selector(|| "token-field-suggestions".into())
                .flex()
                .flex_col()
                .mt(theme.spacing_xs)
                .bg(theme.surface)
                .border_1()
                .border_color(theme.border)
                .rounded(theme.radius_md)
                .overflow_hidden();
            for (idx, suggestion) in self.suggestions.iter().enumerate() {
                let label = suggestion.clone();
                let is_highlighted = self.highlighted_suggestion == Some(idx);
                let row_id = ElementId::NamedInteger("token-sug".into(), idx as u64);
                let row = div()
                    .id(row_id)
                    .debug_selector(|| format!("token-field-suggestion-{idx}"))
                    .min_h(px(theme.target_size()))
                    .flex()
                    .items_center()
                    .px(theme.spacing_md)
                    .cursor_pointer()
                    .hover(|style| style.bg(hover_bg))
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text)
                    .child(label.clone())
                    .on_click(cx.listener(move |this, _event, window, cx| {
                        let added = label.clone();
                        if let Some(on_add) = &this.on_add {
                            on_add(added.as_ref(), window, cx);
                        }
                        if let Some(ref tf) = this.text_field {
                            tf.update(cx, |tf, cx| tf.set_text("", cx));
                        }
                        this.input_text.clear();
                        cx.notify();
                    }));
                let row = if is_highlighted {
                    row.bg(hover_bg)
                } else {
                    row
                };
                list = list.child(row);
            }
            outer = outer.child(list);
        }

        // Context menu for the currently opened token.
        if let Some(target_id) = context_target
            && !context_items.is_empty()
        {
            let mut menu = div()
                .id(ElementId::from((self.element_id.clone(), "ctx-menu")))
                .debug_selector(|| "token-field-context-menu".into())
                .absolute()
                .flex()
                .flex_col()
                .min_w(px(180.0))
                .bg(theme.surface)
                .border_1()
                .border_color(theme.border)
                .rounded(theme.radius_md)
                .overflow_hidden()
                .shadow(
                    theme
                        .glass
                        .shadows(crate::foundations::theme::GlassSize::Medium)
                        .to_vec(),
                )
                .on_mouse_down_out(cx.listener(|this, _event: &MouseDownEvent, _window, cx| {
                    this.close_context_menu(cx);
                }));

            for (idx, item) in context_items.into_iter().enumerate() {
                let label = item.label.clone();
                let destructive = item.destructive;
                let target_id = target_id.clone();
                let row_id = ElementId::NamedInteger("token-ctx".into(), idx as u64);
                let row = div()
                    .id(row_id)
                    .debug_selector(|| format!("token-field-context-{idx}"))
                    .px(theme.spacing_md)
                    .py(theme.spacing_sm)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover_bg()))
                    .text_style(TextStyle::Body, theme)
                    .text_color(if destructive { theme.error } else { theme.text })
                    .child(label.clone())
                    .on_click(cx.listener(move |this, _event, window, cx| {
                        if let Some(handler) = &this.on_context_menu_action {
                            handler(&target_id, &label, window, cx);
                        }
                        this.close_context_menu(cx);
                    }));
                menu = menu.child(row);
            }

            outer = outer.child(menu);
        }

        outer
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::{DEFAULT_COMMIT_KEYS, TokenContextMenuItem, TokenItem};

    // TokenField requires a FocusHandle which needs a GPUI Context,
    // so we test the data types and logic independently.

    #[test]
    fn token_item_new_is_removable() {
        let item = TokenItem::new("rust", "Rust");
        assert_eq!(item.label.as_ref(), "Rust");
        assert_eq!(item.id.as_ref(), "rust");
        assert!(item.removable);
    }

    #[test]
    fn token_item_fixed_is_not_removable() {
        let item = TokenItem::fixed("req", "Required");
        assert!(!item.removable);
    }

    #[test]
    fn token_vec_operations() {
        let mut tokens = vec![TokenItem::new("a", "A"), TokenItem::new("b", "B")];
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].label.as_ref(), "A");
        assert_eq!(tokens[1].id.as_ref(), "b");

        tokens.retain(|t| t.id.as_ref() != "a");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].id.as_ref(), "b");
    }

    #[test]
    fn input_text_operations() {
        let mut input = String::new();
        input.push_str("hello");
        assert_eq!(input, "hello");
        input.pop();
        assert_eq!(input, "hell");
    }

    #[test]
    fn commit_empty_input_is_noop() {
        let trimmed = "   ".trim();
        assert!(trimmed.is_empty());
    }

    #[test]
    fn commit_newline_only_is_empty() {
        let trimmed = "\n\n".trim();
        assert!(trimmed.is_empty());
    }

    #[test]
    fn commit_mixed_whitespace_is_empty() {
        let trimmed = "   \t\n  ".trim();
        assert!(trimmed.is_empty());
    }

    #[test]
    fn default_commit_keys_includes_enter_and_comma() {
        assert!(DEFAULT_COMMIT_KEYS.contains(&"enter"));
        assert!(DEFAULT_COMMIT_KEYS.contains(&"return"));
        assert!(DEFAULT_COMMIT_KEYS.contains(&","));
    }

    #[test]
    fn token_context_menu_item_destructive_builder() {
        let item = TokenContextMenuItem::new("Remove").destructive(true);
        assert!(item.destructive);
    }
}

#[cfg(test)]
mod interaction_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::TestAppContext;

    use super::{TokenContextMenuItem, TokenField, TokenItem};
    use crate::test_helpers::helpers::{InteractionExt, assert_element_exists, setup_test_window};

    const TOKEN_FIELD_ROOT: &str = "token-field-root";
    const TOKEN_REMOVE_RUST: &str = "token-remove-rust";

    fn focus_token_field(field: &gpui::Entity<TokenField>, cx: &mut gpui::VisualTestContext) {
        field.update_in(cx, |field, window, cx| {
            field.focus_handle.focus(window, cx);
        });
    }

    fn wire_callbacks(
        field: &gpui::Entity<TokenField>,
        cx: &mut gpui::VisualTestContext,
        added: Rc<RefCell<Vec<String>>>,
        removed: Rc<RefCell<Vec<String>>>,
    ) {
        field.update_in(cx, |field, _window, _cx| {
            field.set_on_add({
                let added = added.clone();
                move |text, _, _| {
                    added.borrow_mut().push(text.to_string());
                }
            });
            field.set_on_remove({
                let removed = removed.clone();
                move |id, _, _| {
                    removed.borrow_mut().push(id.to_string());
                }
            });
        });
    }

    #[gpui::test]
    async fn enter_commits_token_and_clears_input(cx: &mut TestAppContext) {
        let added = Rc::new(RefCell::new(Vec::new()));
        let removed = Rc::new(RefCell::new(Vec::new()));
        let (field, cx) = setup_test_window(cx, |_window, cx| TokenField::new(cx));

        assert_element_exists(cx, TOKEN_FIELD_ROOT);
        wire_callbacks(&field, cx, added.clone(), removed.clone());
        focus_token_field(&field, cx);

        cx.press("r");
        cx.press("u");
        cx.press("s");
        cx.press("t");
        cx.press("enter");

        assert_eq!(&*added.borrow(), &["rust".to_string()]);
        assert!(removed.borrow().is_empty());
        field.update_in(cx, |field, _window, cx| {
            assert_eq!(field.input_text(), "");
            assert!(field.tokens().is_empty());
            field.add_token("rust", "rust", cx);
            assert_eq!(field.tokens().len(), 1);
            assert_eq!(field.tokens()[0].id.as_ref(), "rust");
        });
    }

    #[gpui::test]
    async fn comma_commits_token_by_default(cx: &mut TestAppContext) {
        let added = Rc::new(RefCell::new(Vec::new()));
        let removed = Rc::new(RefCell::new(Vec::new()));
        let (field, cx) = setup_test_window(cx, |_window, cx| TokenField::new(cx));

        wire_callbacks(&field, cx, added.clone(), removed.clone());
        focus_token_field(&field, cx);

        cx.press("g");
        cx.press("o");
        cx.press(",");

        assert_eq!(&*added.borrow(), &["go".to_string()]);
    }

    #[gpui::test]
    async fn set_commit_keys_overrides_default(cx: &mut TestAppContext) {
        let added = Rc::new(RefCell::new(Vec::new()));
        let removed = Rc::new(RefCell::new(Vec::new()));
        let (field, cx) = setup_test_window(cx, |_window, cx| TokenField::new(cx));

        wire_callbacks(&field, cx, added.clone(), removed.clone());
        field.update_in(cx, |field, _window, _cx| {
            field.set_commit_keys(vec![";".to_string()]);
        });
        focus_token_field(&field, cx);

        // Enter should no longer commit since only ";" is in commit_keys now.
        cx.press("g");
        cx.press("o");
        cx.press("enter");
        assert!(added.borrow().is_empty());

        cx.press(";");
        assert_eq!(&*added.borrow(), &["go".to_string()]);
    }

    #[gpui::test]
    async fn clicking_remove_button_removes_token(cx: &mut TestAppContext) {
        let added = Rc::new(RefCell::new(Vec::new()));
        let removed = Rc::new(RefCell::new(Vec::new()));
        let (field, cx) = setup_test_window(cx, |_window, cx| TokenField::new(cx));

        wire_callbacks(&field, cx, added, removed.clone());
        field.update_in(cx, |field, _window, cx| {
            field.set_tokens(vec![TokenItem::new("rust", "Rust")], cx);
        });

        assert_element_exists(cx, TOKEN_REMOVE_RUST);
        cx.click_on(TOKEN_REMOVE_RUST);

        assert_eq!(&*removed.borrow(), &["rust".to_string()]);
        field.update_in(cx, |field, _window, cx| {
            field.remove_token("rust", cx);
            assert!(field.tokens().is_empty());
        });
    }

    #[gpui::test]
    async fn backspace_removes_last_removable_but_not_fixed_token(cx: &mut TestAppContext) {
        let added = Rc::new(RefCell::new(Vec::new()));
        let removed = Rc::new(RefCell::new(Vec::new()));
        let (field, cx) = setup_test_window(cx, |_window, cx| TokenField::new(cx));

        wire_callbacks(&field, cx, added, removed.clone());
        field.update_in(cx, |field, _window, cx| {
            field.set_tokens(
                vec![
                    TokenItem::fixed("req", "Required"),
                    TokenItem::new("rust", "Rust"),
                ],
                cx,
            );
        });
        focus_token_field(&field, cx);

        cx.press("backspace");
        field.update_in(cx, |field, _window, cx| {
            field.remove_token("rust", cx);
        });
        cx.press("backspace");

        assert_eq!(&*removed.borrow(), &["rust".to_string()]);
        field.update_in(cx, |field, _window, _cx| {
            assert_eq!(field.tokens().len(), 1);
            assert_eq!(field.tokens()[0].id.as_ref(), "req");
        });
    }

    #[gpui::test]
    async fn context_menu_items_render_and_fire_action(cx: &mut TestAppContext) {
        let actions = Rc::new(RefCell::new(Vec::<(String, String)>::new()));
        let (field, cx) = setup_test_window(cx, |_window, cx| TokenField::new(cx));

        field.update_in(cx, |field, _window, cx| {
            field.set_tokens(vec![TokenItem::new("rust", "Rust")], cx);
            field.set_on_context_menu_items(|_id, _cx| {
                vec![
                    TokenContextMenuItem::new("Edit"),
                    TokenContextMenuItem::new("Remove").destructive(true),
                ]
            });
            let actions = actions.clone();
            field.set_on_context_menu_action(move |id, action, _, _| {
                actions
                    .borrow_mut()
                    .push((id.to_string(), action.to_string()));
            });
            // Directly trigger the menu open state — simulates a right-
            // click on the chip.
            field.open_context_menu_for(gpui::SharedString::from("rust"), cx);
        });

        assert_element_exists(cx, "token-field-context-menu");
        cx.click_on("token-field-context-0");

        assert_eq!(
            &*actions.borrow(),
            &[("rust".to_string(), "Edit".to_string())]
        );
    }

    // ── Cmd+A select-all-tokens ────────────────────────────────────────

    use gpui::{AppContext, Focusable};

    use crate::components::selection_and_input::text_field::TextField;

    fn embed_text_field(
        token_field: &gpui::Entity<TokenField>,
        cx: &mut gpui::VisualTestContext,
    ) -> gpui::Entity<TextField> {
        let tf = token_field.update_in(cx, |_tok, _window, cx| cx.new(TextField::new));
        token_field.update_in(cx, |tok, _window, cx| {
            tok.set_text_field(tf.clone(), cx);
        });
        tf
    }

    fn focus_inner_text_field(field: &gpui::Entity<TextField>, cx: &mut gpui::VisualTestContext) {
        field.update_in(cx, |field, window, cx| {
            let handle = field.focus_handle(cx);
            handle.focus(window, cx);
        });
    }

    #[gpui::test]
    async fn cmd_a_on_empty_input_selects_all_tokens(cx: &mut TestAppContext) {
        let added = Rc::new(RefCell::new(Vec::new()));
        let removed = Rc::new(RefCell::new(Vec::new()));
        let (field, cx) = setup_test_window(cx, |_window, cx| TokenField::new(cx));

        wire_callbacks(&field, cx, added, removed);
        field.update_in(cx, |field, _window, cx| {
            field.set_tokens(
                vec![TokenItem::new("rust", "Rust"), TokenItem::new("go", "Go")],
                cx,
            );
        });
        let tf = embed_text_field(&field, cx);
        focus_inner_text_field(&tf, cx);

        // Input is empty → Cmd+A should put the chip row into the
        // "selected-all" state.
        cx.press("cmd-a");

        field.update_in(cx, |field, _window, _cx| {
            assert!(field.all_tokens_selected());
        });
        // The inner TextField should NOT have any text selected (nothing
        // was typed), and the outer should have intercepted the shortcut
        // — i.e. the TextField's content is still empty.
        tf.update_in(cx, |t, _window, _cx| {
            assert!(t.text().is_empty());
        });
    }

    #[gpui::test]
    async fn cmd_a_on_non_empty_input_defers_to_text_field(cx: &mut TestAppContext) {
        let added = Rc::new(RefCell::new(Vec::new()));
        let removed = Rc::new(RefCell::new(Vec::new()));
        let (field, cx) = setup_test_window(cx, |_window, cx| TokenField::new(cx));

        wire_callbacks(&field, cx, added, removed);
        field.update_in(cx, |field, _window, cx| {
            field.set_tokens(vec![TokenItem::new("rust", "Rust")], cx);
        });
        let tf = embed_text_field(&field, cx);
        focus_inner_text_field(&tf, cx);
        tf.update_in(cx, |t, _window, cx| t.set_text("hello", cx));

        cx.press("cmd-a");

        field.update_in(cx, |field, _window, _cx| {
            assert!(
                !field.all_tokens_selected(),
                "non-empty input must let TextField handle Cmd+A"
            );
        });
    }

    #[gpui::test]
    async fn backspace_with_selected_tokens_clears_them(cx: &mut TestAppContext) {
        let added = Rc::new(RefCell::new(Vec::new()));
        let removed = Rc::new(RefCell::new(Vec::new()));
        let (field, cx) = setup_test_window(cx, |_window, cx| TokenField::new(cx));

        wire_callbacks(&field, cx, added, removed.clone());
        field.update_in(cx, |field, _window, cx| {
            field.set_tokens(
                vec![
                    TokenItem::fixed("req", "Required"),
                    TokenItem::new("rust", "Rust"),
                    TokenItem::new("go", "Go"),
                ],
                cx,
            );
        });
        let tf = embed_text_field(&field, cx);
        focus_inner_text_field(&tf, cx);

        cx.press("cmd-a");
        field.update_in(cx, |field, _window, _cx| {
            assert!(field.all_tokens_selected());
        });

        cx.press("backspace");

        // Both removable tokens should have been targeted by on_remove.
        // The fixed token's id should not appear.
        let ids: Vec<String> = removed.borrow().clone();
        assert!(ids.iter().any(|id| id == "rust"));
        assert!(ids.iter().any(|id| id == "go"));
        assert!(!ids.iter().any(|id| id == "req"));
        field.update_in(cx, |field, _window, _cx| {
            assert!(!field.all_tokens_selected());
        });
    }

    // ── Reference-cycle regression ─────────────────────────────────────

    #[gpui::test]
    async fn set_text_field_does_not_leak_via_on_change_cycle(cx: &mut TestAppContext) {
        use gpui::{Context, IntoElement, Render, Window, div};

        // Neutral root view: Host does not retain the TokenField or
        // TextField, so the only strong refs are the ones we manage here.
        struct Host;
        impl Render for Host {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                div()
            }
        }

        let (host, cx) = setup_test_window(cx, |_window, _cx| Host);

        let (weak_field, weak_tf) = host.update_in(cx, |_host, _window, cx| {
            let field = cx.new(TokenField::new);
            let tf = cx.new(TextField::new);
            field.update(cx, |tok, cx| tok.set_text_field(tf.clone(), cx));
            (field.downgrade(), tf.downgrade())
        });

        cx.run_until_parked();

        assert!(
            weak_field.upgrade().is_none(),
            "TokenField leaked — strong-ref cycle with TextField regressed"
        );
        assert!(
            weak_tf.upgrade().is_none(),
            "TextField leaked — strong-ref cycle with TokenField regressed"
        );
    }
}
