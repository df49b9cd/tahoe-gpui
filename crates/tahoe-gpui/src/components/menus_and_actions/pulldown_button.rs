//! HIG Pull-down Button -- button that reveals an action menu.
//!
//! A stateless `RenderOnce` component that renders a trigger button with a label,
//! optional icon, and chevron. When open, an absolute-positioned dropdown of
//! action items appears below. Unlike [`PopupButton`](super::PopupButton), this
//! does NOT represent selection -- each item fires its own action callback.
//!
//! Supports full keyboard navigation in the open dropdown (arrow keys,
//! Home/End, Enter, Escape) mirroring [`super::popup_button::PopupButton`].

use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    AnyElement, App, ElementId, FocusHandle, KeyDownEvent, MouseDownEvent, SharedString, Window,
    deferred, div, px,
};

use crate::callback_types::{OnMutCallback, OnToggle, rc_wrap};
use crate::foundations::OverlayLayer;
use crate::foundations::accessibility::{AccessibilityProps, AccessibleExt};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::keyboard_shortcuts::MenuShortcut;
use crate::foundations::layout::DROPDOWN_MAX_HEIGHT;
use crate::foundations::materials::{
    SurfaceContext, apply_standard_control_styling, glass_surface,
};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};

/// Callback invoked when keyboard highlight changes in a [`PulldownButton`] dropdown.
pub type OnHighlight = Option<Box<dyn Fn(Option<usize>, &mut Window, &mut App) + 'static>>;

/// Step to the next actionable index (wrapping).
fn nav_next(actionable: &[usize], current: usize) -> usize {
    actionable
        .iter()
        .find(|&&i| i > current)
        .copied()
        .unwrap_or(actionable[0])
}

/// Step to the previous actionable index (wrapping).
///
/// # Panics
///
/// Panics if `actionable` is empty. Callers must guard empty slices (the
/// keyboard closure in [`PulldownButton::render`] does this with an early
/// `return`).
fn nav_prev(actionable: &[usize], current: usize) -> usize {
    actionable
        .iter()
        .rev()
        .find(|&&i| i < current)
        .copied()
        .or_else(|| actionable.last().copied())
        .expect("nav_prev requires a non-empty actionable slice")
}

/// Visual style for a pull-down menu item.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum PulldownItemStyle {
    /// Standard item appearance.
    #[default]
    Default,
    /// Destructive/warning appearance (e.g. delete actions).
    Destructive,
    /// Greyed-out, non-interactive appearance.
    Disabled,
}

/// A single action item in a [`PulldownButton`] menu.
///
/// Each item carries its own `on_click` handler since pull-down menus represent
/// independent actions, not mutually-exclusive selection.
pub struct PulldownItem {
    /// Display label for this action.
    pub label: SharedString,
    /// Optional leading icon.
    pub icon: Option<IconName>,
    /// Visual style.
    pub style: PulldownItemStyle,
    /// Optional keyboard-shortcut glyph (e.g. `⌘S`, `⇧⌘K`) rendered
    /// right-aligned on the same row — matches the `ContextMenuItem`
    /// shortcut slot so pull-down action menus can surface keybindings
    /// too (HIG *Menus > Keyboard shortcuts*).
    pub shortcut: Option<MenuShortcut>,
    /// Click handler invoked when the item is activated.
    pub on_click: OnMutCallback,
}

impl PulldownItem {
    /// Create a new default-styled item with the given label.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            style: PulldownItemStyle::Default,
            shortcut: None,
            on_click: None,
        }
    }

    /// Set the leading icon.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set the visual style.
    pub fn style(mut self, style: PulldownItemStyle) -> Self {
        self.style = style;
        self
    }

    /// Attach a keyboard-shortcut glyph displayed on the trailing edge of
    /// the row (e.g. `⌘S` for Save). The caller is still responsible for
    /// wiring the actual shortcut — this is a display-only affordance.
    pub fn shortcut(mut self, shortcut: MenuShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Set the click handler.
    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

/// HIG Pull-down Button -- button that reveals an action menu.
///
/// Renders a trigger button with a label, optional icon, and chevron. When open,
/// an absolute-positioned dropdown of action rows appears below. Each action
/// carries its own handler -- this is for commands, not selection.
#[derive(IntoElement)]
pub struct PulldownButton {
    id: ElementId,
    label: SharedString,
    icon: Option<AnyElement>,
    items: Vec<PulldownItem>,
    is_open: bool,
    disabled: bool,
    focused: bool,
    /// Optional focus handle; when set, the pull-down tracks GPUI's focus
    /// graph and lights the ring reactively. Takes precedence over
    /// [`PulldownButton::focused`].
    focus_handle: Option<FocusHandle>,
    compact: bool,
    borderless: bool,
    highlighted_index: Option<usize>,
    on_toggle: OnToggle,
    on_highlight: OnHighlight,
}

impl PulldownButton {
    /// Create a new pull-down button with the given id and label.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            items: Vec::new(),
            is_open: false,
            disabled: false,
            focused: false,
            focus_handle: None,
            compact: false,
            borderless: false,
            highlighted_index: None,
            on_toggle: None,
            on_highlight: None,
        }
    }

    /// Use compact 22pt dropdown rows — matches native `NSPullDownButton`
    /// mini-controls used in toolbars.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Render the trigger without the glass-surface border. HIG pull-down
    /// buttons in toolbar contexts are borderless — use this flag when the
    /// pull-down sits in a `Toolbar` element to avoid a redundant chrome
    /// rim.
    pub fn borderless(mut self, borderless: bool) -> Self {
        self.borderless = borderless;
        self
    }

    /// Currently keyboard-highlighted item index in the open dropdown.
    pub fn highlighted_index(mut self, index: Option<usize>) -> Self {
        self.highlighted_index = index;
        self
    }

    /// Handler called when keyboard navigation moves the highlight.
    pub fn on_highlight(
        mut self,
        handler: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_highlight = Some(Box::new(handler));
        self
    }

    /// Set an optional leading icon element for the trigger button.
    pub fn icon(mut self, element: impl IntoElement) -> Self {
        self.icon = Some(element.into_any_element());
        self
    }

    /// Append a single action item to the dropdown menu.
    pub fn item(mut self, item: PulldownItem) -> Self {
        self.items.push(item);
        self
    }

    /// Set the open/closed state of the dropdown.
    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    /// Set the disabled state. When disabled, the button is visually dimmed
    /// and click/keyboard handlers are suppressed.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the focused state for rendering a focus ring. Ignored when a
    /// [`focus_handle`](Self::focus_handle) is supplied — the handle's
    /// reactive state (`handle.is_focused(window)`) takes precedence.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Wire the pull-down into GPUI's focus graph. When set, the focus
    /// ring renders based on `handle.is_focused(window)` — takes
    /// precedence over [`PulldownButton::focused`].
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    /// Set the handler called when the dropdown opens or closes.
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for PulldownButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(self.focused);

        // Wrap on_toggle in Rc for sharing across closures.
        let on_toggle = rc_wrap(self.on_toggle);

        // ── Trigger button ──────────────────────────────────────────────────
        let toggle_for_trigger = on_toggle.clone();
        let trigger_key_toggle = on_toggle.clone();
        let is_open = self.is_open;

        let mut trigger_content = div().flex().items_center().gap(theme.spacing_sm);

        // Optional leading icon.
        if let Some(icon_el) = self.icon {
            trigger_content = trigger_content.child(icon_el);
        }

        trigger_content = trigger_content
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.label_color(SurfaceContext::GlassDim))
                    .child(self.label),
            )
            .child(
                Icon::new(IconName::ChevronDown)
                    .size(px(12.0))
                    .color(theme.secondary_label_color(SurfaceContext::GlassDim)),
            );

        let disabled = self.disabled;
        let borderless = self.borderless;

        let mut trigger = div()
            .id(self.id.clone())
            .min_h(px(theme.target_size()))
            .flex()
            .items_center()
            .px(theme.spacing_md)
            .focusable();

        // `track_focus` is unconditional on handle presence — a caller
        // who supplied a handle expects it wired even when `disabled` is
        // flipped transiently. Only interactivity (`cursor_pointer`,
        // `on_click`, `on_key_down`) is gated on `!disabled`.
        if let Some(handle) = self.focus_handle.as_ref() {
            trigger = trigger.track_focus(handle);
        }
        if !disabled {
            trigger = trigger.cursor_pointer();
        }

        // Glass-styled trigger surface (suppressed when `borderless`).
        if !borderless {
            trigger = apply_standard_control_styling(trigger, theme, GlassSize::Small, focused);
        }

        if disabled {
            trigger = trigger.opacity(0.5).cursor_default();
        } else {
            trigger = trigger.hover(|style| style.cursor_pointer());
        }

        trigger = trigger.child(trigger_content);

        if !disabled && let Some(handler) = toggle_for_trigger {
            trigger = trigger.on_click(move |_event, window, cx| {
                handler(!is_open, window, cx);
            });
        }

        // Trigger keyboard activation: Enter/Space opens the dropdown.
        if !disabled && let Some(handler) = trigger_key_toggle {
            trigger = trigger.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if crate::foundations::keyboard::is_activation_key(event) && !is_open {
                    cx.stop_propagation();
                    handler(true, window, cx);
                }
            });
        }

        // ── Container (trigger + optional dropdown) ─────────────────────────
        let mut container = div().relative().child(trigger);

        if self.is_open {
            // ── Dropdown action list ────────────────────────────────────────
            let compact = self.compact;
            let row_height = if compact {
                px(22.0)
            } else {
                px(theme.target_size())
            };
            // Transform items into a Vec<PreparedItem> that stores the
            // click handler as an `Rc<dyn Fn>` so the same handler can be
            // invoked from both the keyboard Enter path and the mouse
            // click path below.
            type PulldownHandlerRc = Rc<dyn Fn(&mut Window, &mut App)>;
            struct PreparedItem {
                label: SharedString,
                icon: Option<IconName>,
                style: PulldownItemStyle,
                shortcut: Option<MenuShortcut>,
                on_click: Option<PulldownHandlerRc>,
            }
            let items: Vec<PreparedItem> = self
                .items
                .into_iter()
                .map(|action| {
                    let on_click: Option<PulldownHandlerRc> =
                        action.on_click.map(|h| Rc::from(h) as PulldownHandlerRc);
                    PreparedItem {
                        label: action.label,
                        icon: action.icon,
                        style: action.style,
                        shortcut: action.shortcut,
                        on_click,
                    }
                })
                .collect();
            let actionable: Vec<usize> = items
                .iter()
                .enumerate()
                .filter_map(|(i, item)| {
                    if item.style == PulldownItemStyle::Disabled {
                        None
                    } else {
                        Some(i)
                    }
                })
                .collect();
            let action_handlers: Vec<Option<PulldownHandlerRc>> =
                items.iter().map(|item| item.on_click.clone()).collect();
            let labels_lower: Vec<String> =
                items.iter().map(|item| item.label.to_lowercase()).collect();
            let highlighted = self.highlighted_index;
            let on_highlight = self.on_highlight.map(Rc::new);

            let mut list = glass_surface(
                div()
                    .absolute()
                    .left_0()
                    .top(theme.dropdown_top())
                    .w_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .max_h(px(DROPDOWN_MAX_HEIGHT)),
                theme,
                GlassSize::Medium,
            )
            .id(ElementId::from((self.id.clone(), "dropdown")))
            .focusable();

            // Keyboard navigation in the open dropdown (mirrors PopupButton).
            let key_toggle = on_toggle.clone();
            let key_highlight = on_highlight.clone();
            let key_handlers = action_handlers.clone();
            let key_actionable = actionable.clone();
            let key_labels = labels_lower.clone();
            list = list.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if crate::foundations::keyboard::is_escape_key(event) {
                    if let Some(handler) = &key_toggle {
                        handler(false, window, cx);
                    }
                    return;
                }
                if key_actionable.is_empty() {
                    return;
                }
                let key = event.keystroke.key.as_str();
                match key {
                    "down" => {
                        cx.stop_propagation();
                        let next = match highlighted {
                            Some(current) => nav_next(&key_actionable, current),
                            None => key_actionable[0],
                        };
                        if let Some(handler) = &key_highlight {
                            handler(Some(next), window, cx);
                        }
                    }
                    "up" => {
                        cx.stop_propagation();
                        let prev = match highlighted {
                            Some(current) => nav_prev(&key_actionable, current),
                            None => *key_actionable
                                .last()
                                .expect("key_actionable non-empty — guarded above"),
                        };
                        if let Some(handler) = &key_highlight {
                            handler(Some(prev), window, cx);
                        }
                    }
                    "home" => {
                        cx.stop_propagation();
                        if let Some(handler) = &key_highlight {
                            handler(Some(key_actionable[0]), window, cx);
                        }
                    }
                    "end" => {
                        cx.stop_propagation();
                        if let Some(handler) = &key_highlight {
                            handler(
                                Some(
                                    *key_actionable
                                        .last()
                                        .expect("key_actionable non-empty — guarded above"),
                                ),
                                window,
                                cx,
                            );
                        }
                    }
                    "enter" => {
                        cx.stop_propagation();
                        if let Some(idx) = highlighted {
                            if let Some(Some(handler)) = key_handlers.get(idx) {
                                handler(window, cx);
                            }
                            if let Some(t) = &key_toggle {
                                t(false, window, cx);
                            }
                        }
                    }
                    _ => {
                        // Type-ahead: match first character against label.
                        let typed = event
                            .keystroke
                            .key_char
                            .as_deref()
                            .or(Some(key))
                            .filter(|s| s.chars().count() == 1);
                        if let Some(ch) = typed {
                            let ch_lower = ch.to_lowercase();
                            let start = highlighted
                                .map(|cur| {
                                    let next = cur + 1;
                                    if next < key_labels.len() { next } else { 0 }
                                })
                                .unwrap_or(0);
                            let total = key_labels.len();
                            let mut found = None;
                            for offset in 0..total {
                                let idx = (start + offset) % total;
                                if key_labels[idx].starts_with(&ch_lower)
                                    && key_actionable.contains(&idx)
                                {
                                    found = Some(idx);
                                    break;
                                }
                            }
                            if let Some(idx) = found {
                                cx.stop_propagation();
                                if let Some(handler) = &key_highlight {
                                    handler(Some(idx), window, cx);
                                }
                            }
                        }
                    }
                }
            });

            // Close dropdown on click outside.
            let mouse_out_toggle = on_toggle.clone();
            if let Some(handler) = mouse_out_toggle {
                list = list.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
                    handler(false, window, cx);
                });
            }

            for (idx, action) in items.into_iter().enumerate() {
                let is_disabled = action.style == PulldownItemStyle::Disabled;
                let is_highlighted = highlighted == Some(idx);

                let text_color = match action.style {
                    PulldownItemStyle::Default => theme.label_color(SurfaceContext::GlassDim),
                    PulldownItemStyle::Destructive => theme.error,
                    PulldownItemStyle::Disabled => {
                        theme.secondary_label_color(SurfaceContext::GlassDim)
                    }
                };

                let mut row = div()
                    .id(ElementId::NamedInteger("pulldown-item".into(), idx as u64))
                    .min_h(row_height)
                    .flex()
                    .items_center()
                    .px(theme.spacing_md)
                    .gap(theme.spacing_sm)
                    .text_style(TextStyle::Body, theme)
                    .text_color(text_color);

                if is_disabled {
                    row = row.opacity(0.5);
                } else {
                    row = row.cursor_pointer().hover(|style| style.bg(theme.hover));
                }

                if is_highlighted {
                    row = row.bg(theme.hover);
                }

                if let Some(icon_name) = action.icon {
                    row = row.child(Icon::new(icon_name).size(px(16.0)).color(text_color));
                }

                let action_label = action.label.clone();
                row = row.child(div().flex_1().child(action.label));

                // Trailing keyboard-shortcut glyph. Uses the secondary-label
                // color so it reads as annotation, matching ContextMenuItem.
                if let Some(shortcut) = action.shortcut {
                    let shortcut_color = theme.secondary_label_color(SurfaceContext::GlassDim);
                    row = row.child(
                        div()
                            .text_color(shortcut_color)
                            .child(SharedString::from(shortcut.render())),
                    );
                }

                if !is_disabled {
                    let on_toggle = on_toggle.clone();
                    let on_click = action.on_click;
                    row = row.on_click(move |_event, window, cx| {
                        if let Some(handler) = &on_click {
                            handler(window, cx);
                        }
                        if let Some(handler) = &on_toggle {
                            handler(false, window, cx);
                        }
                    });
                }

                let a11y = AccessibilityProps::menu_item(action_label);
                row = row.with_accessibility(&a11y);

                list = list.child(row);
                // Silence unused: `action_handlers` is captured by the
                // keyboard listener above; we do not re-use it in this
                // loop because the per-row on_click handler already has
                // access to its own `Rc<dyn Fn>` copy.
                let _ = &action_handlers;
            }

            container = container.child(deferred(list).with_priority(OverlayLayer::DROPDOWN));
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use gpui::SharedString;

    use crate::components::menus_and_actions::pulldown_button::{
        PulldownButton, PulldownItem, PulldownItemStyle,
    };

    #[test]
    fn pulldown_button_defaults() {
        let pb = PulldownButton::new("test", "Actions");
        assert_eq!(pb.label.as_ref(), "Actions");
        assert!(pb.icon.is_none());
        assert!(pb.items.is_empty());
        assert!(!pb.is_open);
        assert!(!pb.disabled);
        assert!(!pb.focused);
        assert!(!pb.compact);
        assert!(!pb.borderless);
        assert!(pb.highlighted_index.is_none());
        assert!(pb.on_toggle.is_none());
        assert!(pb.on_highlight.is_none());
    }

    #[test]
    fn pulldown_button_compact_borderless_builders() {
        let pb = PulldownButton::new("test", "Menu")
            .compact(true)
            .borderless(true)
            .highlighted_index(Some(2));
        assert!(pb.compact);
        assert!(pb.borderless);
        assert_eq!(pb.highlighted_index, Some(2));
    }

    #[test]
    fn pulldown_button_focus_handle_none_by_default() {
        let pb = PulldownButton::new("test", "Menu");
        assert!(pb.focus_handle.is_none());
    }

    #[gpui::test]
    async fn pulldown_button_focus_handle_builder_stores_handle(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let handle = cx.focus_handle();
            let pb = PulldownButton::new("test", "Menu").focus_handle(&handle);
            assert!(
                pb.focus_handle.is_some(),
                "focus_handle(..) must round-trip into the field"
            );
        });
    }

    #[test]
    fn pulldown_nav_next_wraps() {
        let actionable = vec![0_usize, 2, 4];
        assert_eq!(super::nav_next(&actionable, 0), 2);
        assert_eq!(super::nav_next(&actionable, 4), 0);
    }

    #[test]
    fn pulldown_nav_prev_wraps() {
        let actionable = vec![0_usize, 2, 4];
        assert_eq!(super::nav_prev(&actionable, 4), 2);
        assert_eq!(super::nav_prev(&actionable, 0), 4);
    }

    #[test]
    fn pulldown_button_item_builder() {
        let pb = PulldownButton::new("test", "Edit")
            .item(PulldownItem::new("Cut"))
            .item(PulldownItem::new("Copy"));
        assert_eq!(pb.items.len(), 2);
        assert_eq!(pb.items[0].label.as_ref(), "Cut");
        assert_eq!(pb.items[1].label.as_ref(), "Copy");
    }

    #[test]
    fn pulldown_button_open_builder() {
        let pb = PulldownButton::new("test", "Menu").open(true);
        assert!(pb.is_open);
    }

    #[test]
    fn pulldown_button_on_toggle_is_some() {
        let pb = PulldownButton::new("test", "Menu").on_toggle(|_, _, _| {});
        assert!(pb.on_toggle.is_some());
    }

    #[test]
    fn pulldown_item_defaults() {
        let item = PulldownItem::new("Delete");
        assert_eq!(item.label.as_ref(), "Delete");
        assert!(item.icon.is_none());
        assert_eq!(item.style, PulldownItemStyle::Default);
        assert!(item.on_click.is_none());
    }

    #[test]
    fn pulldown_item_builder_all_fields() {
        use crate::foundations::icons::IconName;
        let item = PulldownItem::new("Remove")
            .icon(IconName::Trash)
            .style(PulldownItemStyle::Destructive)
            .on_click(|_, _| {});
        assert_eq!(item.label.as_ref(), "Remove");
        assert_eq!(item.icon, Some(IconName::Trash));
        assert_eq!(item.style, PulldownItemStyle::Destructive);
        assert!(item.on_click.is_some());
    }

    #[test]
    fn pulldown_item_style_default_is_default() {
        assert_eq!(PulldownItemStyle::default(), PulldownItemStyle::Default);
    }

    #[test]
    fn pulldown_item_all_styles_distinct() {
        let styles = [
            PulldownItemStyle::Default,
            PulldownItemStyle::Destructive,
            PulldownItemStyle::Disabled,
        ];
        for i in 0..styles.len() {
            for j in (i + 1)..styles.len() {
                assert_ne!(styles[i], styles[j]);
            }
        }
    }

    #[test]
    fn pulldown_item_accepts_shared_string() {
        let label = SharedString::from("Shared");
        let item = PulldownItem::new(label);
        assert_eq!(item.label.as_ref(), "Shared");
    }
}
