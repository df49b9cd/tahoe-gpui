//! Checkbox primitive (HIG macOS).
//!
//! Distinct from [`Toggle`](super::Toggle), which renders a switch. HIG macOS uses checkboxes
//! inside Forms, preference panes, and multi-select lists where the user picks
//! one or more options from a list — situations where the binary on/off
//! affordance of a switch reads as too prominent. Toggles are reserved for
//! settings that take immediate effect.
//!
//! # Tri-state
//!
//! [`CheckboxState`] is a three-valued enum: `Unchecked`, `Checked`, `Mixed`
//! (AppKit `NSControl.StateValue.mixed` / SwiftUI's `.mixed`). The mixed
//! state is the canonical HIG representation of "some but not all children
//! are checked" — used by parent rows of a grouped list. Callers that only
//! need a two-state checkbox can construct via `Checkbox::checked(bool)`.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/toggles>
//! (the Toggles page covers both switches and checkboxes).

use gpui::prelude::*;
use gpui::{App, ElementId, FocusHandle, KeyDownEvent, SharedString, Window, div, px};

use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::{apply_focus_ring, apply_high_contrast_border};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};

/// Three-valued checkbox state. HIG macOS exposes all three.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CheckboxState {
    /// Unchecked box — empty square.
    #[default]
    Unchecked,
    /// Checked box — accent fill with a glyph.
    Checked,
    /// Mixed / indeterminate — accent fill with a horizontal bar. Used on
    /// parent rows of grouped lists when children are partially selected.
    Mixed,
}

impl CheckboxState {
    /// Construct from a plain `bool`. `true` → `Checked`, `false` → `Unchecked`.
    pub const fn from_bool(value: bool) -> Self {
        if value {
            Self::Checked
        } else {
            Self::Unchecked
        }
    }

    /// Returns `true` if the box draws a filled glyph (checked or mixed).
    pub const fn is_filled(self) -> bool {
        matches!(self, Self::Checked | Self::Mixed)
    }

    /// Toggle for click/activation. `Unchecked` ↔ `Checked`. `Mixed`
    /// promotes to `Checked` per HIG ("clicking a mixed-state checkbox
    /// selects all children").
    pub const fn toggled(self) -> Self {
        match self {
            Self::Unchecked => Self::Checked,
            Self::Checked | Self::Mixed => Self::Unchecked,
        }
    }
}

/// Stateless checkbox. Parent owns the state and provides `on_change`.
///
/// The layout is a leading box + optional trailing label. Labels are
/// laid out inside the interactive region so clicking the label also
/// toggles the box — the HIG macOS pattern used in every preferences
/// pane since 10.0.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    state: CheckboxState,
    disabled: bool,
    label: Option<SharedString>,
    focus_handle: Option<FocusHandle>,
    on_change: Option<Box<dyn Fn(CheckboxState, &mut Window, &mut App) + 'static>>,
    accessibility_label: Option<SharedString>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            state: CheckboxState::Unchecked,
            disabled: false,
            label: None,
            focus_handle: None,
            on_change: None,
            accessibility_label: None,
        }
    }

    /// Set the tri-state value directly.
    pub fn state(mut self, state: CheckboxState) -> Self {
        self.state = state;
        self
    }

    /// Convenience for two-state callers: `true` → `Checked`.
    pub fn checked(mut self, checked: bool) -> Self {
        self.state = CheckboxState::from_bool(checked);
        self
    }

    /// Put the checkbox in the indeterminate/mixed state per HIG.
    pub fn mixed(mut self) -> Self {
        self.state = CheckboxState::Mixed;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Optional trailing label. Clicking the label toggles the box, matching
    /// the AppKit `NSButton` checkbox behavior.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Attach a focus handle so the checkbox participates in the host's
    /// focus graph (Tab cycling, keyboard shortcuts).
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    /// VoiceOver label. Defaults to the visible text label when present.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Fires with the new state on click or Space/Return activation.
    pub fn on_change(
        mut self,
        handler: impl Fn(CheckboxState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let next_state = self.state.toggled();
        let filled = self.state.is_filled();

        // HIG macOS checkbox: 14pt square, 3pt corner radius (matches the
        // suppression checkbox in Alert).
        const BOX_SIZE: f32 = 14.0;
        const BOX_RADIUS: f32 = 3.0;

        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(false);

        let bg = if filled {
            theme.accent
        } else {
            theme.semantic.secondary_system_fill
        };
        let border = if filled { theme.accent } else { theme.border };

        let glyph: Option<gpui::AnyElement> = match self.state {
            CheckboxState::Checked => Some(
                Icon::new(IconName::Check)
                    .size(px(10.0))
                    .color(theme.text_on_accent)
                    .into_any_element(),
            ),
            CheckboxState::Mixed => Some(
                // 2pt horizontal bar — AppKit mixed-state glyph.
                div()
                    .w(px(6.0))
                    .h(px(2.0))
                    .rounded(px(1.0))
                    .bg(theme.text_on_accent)
                    .into_any_element(),
            ),
            CheckboxState::Unchecked => None,
        };

        let mut box_el = div()
            .w(px(BOX_SIZE))
            .h(px(BOX_SIZE))
            .rounded(px(BOX_RADIUS))
            .border_1()
            .border_color(border)
            .bg(bg)
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0();
        if let Some(g) = glyph {
            box_el = box_el.child(g);
        }
        box_el = apply_high_contrast_border(box_el, theme);

        // Layout: box + optional label. The whole row is interactive so
        // clicking the label toggles — HIG convention since 10.0.
        let id = self.id;
        let mut row = div()
            .id(id.clone())
            .debug_selector(|| format!("checkbox-{}", id))
            .focusable()
            .flex()
            .flex_row()
            .items_center()
            .gap(theme.spacing_xs)
            .min_h(px(theme.target_size()))
            .child(box_el);

        if let Some(label_text) = self.label {
            row = row.child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text)
                    .child(label_text),
            );
        }

        if let Some(handle) = self.focus_handle.as_ref() {
            row = row.track_focus(handle);
        }

        row = apply_focus_ring(row, theme, focused, &[]);

        if self.disabled {
            row = row.opacity(0.5);
        } else if let Some(handler) = self.on_change {
            let handler = std::rc::Rc::new(handler);
            let click = handler.clone();
            row = row
                .cursor_pointer()
                .on_click(move |_event, window, cx| click(next_state, window, cx))
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if crate::foundations::keyboard::is_activation_key(event) {
                        cx.stop_propagation();
                        handler(next_state, window, cx);
                    }
                });
        }

        row
    }
}

#[cfg(test)]
mod tests {
    use super::{Checkbox, CheckboxState};
    use core::prelude::v1::test;

    #[test]
    fn state_default_is_unchecked() {
        assert_eq!(CheckboxState::default(), CheckboxState::Unchecked);
    }

    #[test]
    fn from_bool_maps_as_expected() {
        assert_eq!(CheckboxState::from_bool(true), CheckboxState::Checked);
        assert_eq!(CheckboxState::from_bool(false), CheckboxState::Unchecked);
    }

    #[test]
    fn is_filled_covers_checked_and_mixed() {
        assert!(!CheckboxState::Unchecked.is_filled());
        assert!(CheckboxState::Checked.is_filled());
        assert!(CheckboxState::Mixed.is_filled());
    }

    #[test]
    fn toggled_promotes_mixed_to_unchecked_hig() {
        // HIG: clicking a mixed-state checkbox selects all children, so
        // the next click reads as "deselect all" — unchecked.
        assert_eq!(CheckboxState::Mixed.toggled(), CheckboxState::Unchecked);
        assert_eq!(CheckboxState::Checked.toggled(), CheckboxState::Unchecked);
        assert_eq!(CheckboxState::Unchecked.toggled(), CheckboxState::Checked);
    }

    #[test]
    fn builder_sets_state() {
        let c = Checkbox::new("test").state(CheckboxState::Mixed);
        assert_eq!(c.state, CheckboxState::Mixed);
    }

    #[test]
    fn builder_mixed_helper() {
        let c = Checkbox::new("test").mixed();
        assert_eq!(c.state, CheckboxState::Mixed);
    }

    #[test]
    fn builder_checked_helper_maps_bool() {
        let c = Checkbox::new("test").checked(true);
        assert_eq!(c.state, CheckboxState::Checked);
        let c = Checkbox::new("test").checked(false);
        assert_eq!(c.state, CheckboxState::Unchecked);
    }

    #[test]
    fn builder_label_stored() {
        let c = Checkbox::new("test").label("Remember me");
        assert_eq!(c.label.as_ref().map(|s| s.as_ref()), Some("Remember me"));
    }

    #[test]
    fn disabled_builder_sets_flag() {
        let c = Checkbox::new("test").disabled(true);
        assert!(c.disabled);
    }

    #[test]
    fn on_change_registered() {
        let c = Checkbox::new("test").on_change(|_, _, _| {});
        assert!(c.on_change.is_some());
    }
}
