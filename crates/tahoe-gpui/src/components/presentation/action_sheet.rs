//! Action sheet component with Apple glass material.
//!
//! Presents a list of action choices plus a cancel button. Structure
//! (title + choices + Cancel) applies universally per HIG
//! `#action-sheets`; the *chrome* varies by platform:
//!
//! - **iOS / iPadOS / watchOS** — springs from the bottom of the screen
//!   as a drawer. Cancel renders in its own grouped surface. Modal: a
//!   translucent backdrop dims the parent UI.
//! - **macOS / visionOS** — HIG: macOS action sheets are **non-modal** —
//!   rendered as a centered confirmation dialog (no bottom anchoring,
//!   no drawer, no dim backdrop, no outside-click dismissal). The sheet
//!   coexists with the parent UI; the user dismisses with Cancel or
//!   Escape. Use [`ActionSheetPresentation::Centered`] to request this
//!   chrome explicitly.
//!
//! The `presentation` is auto-selected from the active
//! [`TahoeTheme::platform`] unless [`ActionSheet::presentation`] is
//! called.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/action-sheets>

use gpui::prelude::*;
use gpui::{App, ClickEvent, ElementId, FocusHandle, KeyDownEvent, SharedString, Window, div, px};
use std::rc::Rc;

use crate::callback_types::OnMutCallback;
use crate::foundations::accessibility::{FocusGroup, FocusGroupMode};
use crate::foundations::layout::Platform;
use crate::foundations::materials::{Elevation, Glass, Shape, SurfaceContext, glass_effect_lens};
use crate::foundations::theme::{ActiveTheme, TahoeTheme, TextStyle, TextStyledExt};

/// Where the action sheet attaches on screen. Choose
/// [`Self::BottomDrawer`] on iOS/iPadOS/watchOS and [`Self::Centered`]
/// on macOS/visionOS. The default is selected automatically from the
/// active [`TahoeTheme::platform`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ActionSheetPresentation {
    /// iOS drawer that slides up from the bottom of the screen.
    #[default]
    BottomDrawer,
    /// macOS centered confirmation dialog — no bottom-anchoring, no
    /// drag indicator, glass panel centered over the parent window.
    Centered,
}

impl ActionSheetPresentation {
    /// Default presentation for a given platform per HIG.
    pub fn for_platform(platform: Platform) -> Self {
        match platform {
            Platform::MacOS | Platform::VisionOS | Platform::TvOS => Self::Centered,
            Platform::IOS | Platform::WatchOS => Self::BottomDrawer,
        }
    }
}

/// Visual style for an action sheet item.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ActionSheetStyle {
    /// Standard item appearance.
    #[default]
    Default,
    /// Destructive/warning appearance (e.g. delete actions).
    Destructive,
}

/// Click callback signature for an `ActionSheetItem`.
type OnItemClick = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Shared cancel callback (wrapped in `Rc` so both the Cancel row click
/// handler and the keyboard Escape handler can fire it).
type SharedCancel = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

/// A single action item in an action sheet.
pub struct ActionSheetItem {
    /// Display label for this action.
    pub label: SharedString,
    /// Visual style (default or destructive).
    pub style: ActionSheetStyle,
    /// Optional click handler.
    pub on_click: Option<OnItemClick>,
}

impl ActionSheetItem {
    /// Create a new default-styled item.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            style: ActionSheetStyle::Default,
            on_click: None,
        }
    }

    /// Set the visual style.
    pub fn style(mut self, style: ActionSheetStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the click handler.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

/// Apple's action sheet that springs from a source element.
///
/// Displays a vertical stack of action items with a cancel button at the bottom.
/// When a glass theme is active, uses `GlassSize::Large` for sheet-level surfaces.
///
/// Defaults to closed; call [`ActionSheet::open`] with `true` to present it,
/// matching the gate used by `Alert`, `Modal`, and `Sheet`.
#[derive(IntoElement)]
pub struct ActionSheet {
    id: ElementId,
    items: Vec<ActionSheetItem>,
    cancel_text: SharedString,
    on_cancel: OnMutCallback,
    focus_handle: Option<FocusHandle>,
    /// Child focus handles wrapped in a Trap-mode [`FocusGroup`]. Tab /
    /// Shift+Tab cycle through the registered handles with wrap-around per
    /// the WAI-ARIA dialog pattern; the group's `handle_key_down` helper
    /// consumes Tab so focus cannot escape the action-sheet surface.
    focus_group: FocusGroup,
    /// Focus handle to restore when the action sheet dismisses.
    restore_focus_to: Option<FocusHandle>,
    is_open: bool,
    presentation: Option<ActionSheetPresentation>,
}

impl ActionSheet {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            cancel_text: SharedString::from("Cancel"),
            on_cancel: None,
            focus_handle: None,
            focus_group: FocusGroup::trap(),
            restore_focus_to: None,
            is_open: false,
            presentation: None,
        }
    }

    /// Set the action items.
    pub fn items(mut self, items: Vec<ActionSheetItem>) -> Self {
        self.items = items;
        self
    }

    /// Override the default cancel button text.
    pub fn cancel_text(mut self, text: impl Into<SharedString>) -> Self {
        self.cancel_text = text.into();
        self
    }

    /// Set the cancel button click handler. When set, clicking Cancel invokes this callback.
    pub fn on_cancel(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Box::new(handler));
        self
    }

    /// Override the focus handle tracked on the action-sheet panel. When
    /// unset, [`RenderOnce::render`] auto-mints one via `cx.focus_handle()`
    /// so `focus_cycle`, the Tab trap, and Escape/Cmd-. dismissal work
    /// standalone.
    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
        self
    }

    /// Control visibility. When `false` the sheet renders nothing,
    /// matching the `.open()` gate on `Alert` / `Modal` / `Sheet` so
    /// callers don't have to unmount/remount to hide the component.
    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    /// Override the default platform-selected presentation chrome.
    /// Pass [`ActionSheetPresentation::Centered`] to force the macOS
    /// confirmation-dialog chrome on any platform, or
    /// [`ActionSheetPresentation::BottomDrawer`] to force the iOS
    /// drawer chrome.
    pub fn presentation(mut self, presentation: ActionSheetPresentation) -> Self {
        self.presentation = Some(presentation);
        self
    }

    /// Register the focus handles of interactive children in tab order.
    ///
    /// When the user presses Tab past the last handle, focus returns to
    /// the first; Shift+Tab past the first returns to the last. The
    /// action sheet's `on_key_down` handler consumes Tab so focus cannot
    /// escape the sheet surface even when no children are registered.
    ///
    /// Internally allocates a fresh Trap-mode [`FocusGroup`]. Calling this
    /// after [`ActionSheet::focus_group`] is last-write-wins.
    pub fn focus_cycle(mut self, handles: Vec<FocusHandle>) -> Self {
        self.focus_group = FocusGroup::trap();
        for handle in &handles {
            self.focus_group.register(handle);
        }
        self
    }

    /// Attach a caller-managed [`FocusGroup`] driving the action sheet's
    /// Tab trap. Overrides any handles previously supplied via
    /// [`ActionSheet::focus_cycle`].
    ///
    /// The group **must** be in [`FocusGroupMode::Trap`] mode. Open /
    /// Cycle groups make the action sheet's Tab handler swallow Tab
    /// without advancing focus, stranding keyboard users — so this is a
    /// runtime `assert!` rather than a `debug_assert!`.
    pub fn focus_group(mut self, group: FocusGroup) -> Self {
        assert_eq!(
            group.mode(),
            FocusGroupMode::Trap,
            "ActionSheet::focus_group requires a Trap-mode FocusGroup; \
             other modes cause Tab to be swallowed without advancing \
             focus, stranding keyboard users."
        );
        self.focus_group = group;
        self
    }

    /// Record the focus handle that should regain focus when the action
    /// sheet dismisses. Call with the currently-focused element *before*
    /// opening the sheet so the user returns to the same position on
    /// dismiss.
    pub fn restore_focus_to(mut self, handle: FocusHandle) -> Self {
        self.restore_focus_to = Some(handle);
        self
    }
}

impl RenderOnce for ActionSheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.is_open {
            return div().into_any_element();
        }

        // Initial-focus bootstrap per WAI-ARIA dialog pattern: land on the
        // first focus-group member when one is registered, otherwise on
        // the outer handle so Escape / Tab reach the key handler. Auto-mint
        // a handle when the caller didn't wire one so `focus_cycle`, the
        // Tab trap, and Escape all work without requiring `.focus_handle(...)`.
        // Mirrors Sheet/Modal's two-check logic.
        let focus_handle = self
            .focus_handle
            .clone()
            .unwrap_or_else(|| cx.focus_handle());
        let any_sheet_focus =
            self.focus_group.contains_focused(window) || focus_handle.contains_focused(window, cx);
        if !any_sheet_focus {
            if self.focus_group.is_empty() {
                focus_handle.focus(window, cx);
            } else {
                self.focus_group.focus_first(window, cx);
            }
        }

        let theme = cx.theme();
        // Action-sheet groups use a fixed 34pt radius to match the
        // Figma Tahoe UI Kit. Using Shape::RoundedRectangle keeps the
        // corner explicit at the call site now that GlassStyle no
        // longer exposes per-tier radii.
        let glass_radius = px(34.0);
        let presentation = self
            .presentation
            .unwrap_or_else(|| ActionSheetPresentation::for_platform(theme.platform));

        // Build action item rows.
        let mut item_rows = Vec::new();
        for (idx, item) in self.items.into_iter().enumerate() {
            let text_color = match item.style {
                ActionSheetStyle::Default => theme.label_color(SurfaceContext::GlassDim),
                ActionSheetStyle::Destructive => theme.error,
            };

            let hover_bg = theme.hover_bg();

            let mut row = div()
                .id(ElementId::NamedInteger("action-item".into(), idx as u64))
                .debug_selector(|| format!("action-sheet-item-{idx}"))
                .w_full()
                .flex()
                .items_center()
                .justify_center()
                .min_h(px(theme.target_size()))
                .py(theme.spacing_md)
                .px(theme.spacing_lg)
                .text_style(TextStyle::Body, theme)
                .text_color(text_color)
                .hover(|style| style.bg(hover_bg))
                .child(item.label.clone());

            if idx > 0 {
                row = row.border_t_1().border_color(theme.modal_separator_color());
            }

            if let Some(handler) = item.on_click {
                row = row
                    .cursor_pointer()
                    .on_click(move |event, window, cx| handler(event, window, cx));
            }

            item_rows.push(row);
        }

        // Cancel button at bottom with slightly different style.
        let cancel_hover_bg = theme.hover_bg();

        // Wrap on_cancel so every dismissal path (cancel click, Escape)
        // first restores focus to `restore_focus_to` before invoking the
        // inner callback. Shared across the cancel_row click and the
        // keyboard handler further below via Rc.
        let restore = self.restore_focus_to.clone();
        let on_cancel_rc: Option<SharedCancel> = self.on_cancel.map(move |inner| {
            Rc::new(move |window: &mut Window, cx: &mut App| {
                if let Some(handle) = restore.as_ref() {
                    handle.focus(window, cx);
                }
                inner(window, cx);
            }) as Rc<dyn Fn(&mut Window, &mut App) + 'static>
        });

        let cancel_row = {
            let mut cancel_row = div()
                .id(ElementId::NamedInteger("action-cancel".into(), 0u64))
                .debug_selector(|| "action-sheet-cancel".into())
                .w_full()
                .flex()
                .items_center()
                .justify_center()
                .py(theme.spacing_md)
                .px(theme.spacing_lg)
                .text_style(TextStyle::Body, theme)
                .text_color(theme.secondary_label_color(SurfaceContext::GlassDim))
                .hover(|style| style.bg(cancel_hover_bg));

            if let Some(handler) = on_cancel_rc.clone() {
                cancel_row = cancel_row
                    .cursor_pointer()
                    .on_click(move |_event, window, cx| handler(window, cx));
            }

            cancel_row.child(self.cancel_text.clone())
        };

        match presentation {
            ActionSheetPresentation::BottomDrawer => render_bottom_drawer(
                self.id,
                theme,
                glass_radius,
                item_rows,
                cancel_row,
                on_cancel_rc,
                focus_handle,
                self.focus_group,
            ),
            ActionSheetPresentation::Centered => render_centered(
                self.id,
                theme,
                glass_radius,
                item_rows,
                cancel_row,
                on_cancel_rc,
                focus_handle,
                self.focus_group,
            ),
        }
    }
}

/// iOS/watchOS bottom-drawer chrome — cancel group sits below the item
/// group with a small gap.
///
/// ActionSheet items render as non-focusable click targets, so Home/End
/// (which jump to first/last focus-group member) are intentionally not
/// handled — they would have no user-visible effect. Tab is still
/// swallowed by the Trap-mode [`FocusGroup`] per the WAI-ARIA dialog
/// pattern so focus cannot escape the sheet surface.
fn render_bottom_drawer(
    id: ElementId,
    theme: &TahoeTheme,
    glass_radius: gpui::Pixels,
    item_rows: Vec<gpui::Stateful<gpui::Div>>,
    cancel_row: gpui::Stateful<gpui::Div>,
    on_cancel_rc: Option<SharedCancel>,
    focus_handle: FocusHandle,
    focus_group: FocusGroup,
) -> gpui::AnyElement {
    // Cancel container — Elevated tier (Medium UI fill + ambient + rim).
    let cancel_group = glass_effect_lens(
        theme,
        Glass::Regular,
        Shape::RoundedRectangle(glass_radius),
        Elevation::Elevated,
        None,
    )
    .w_full()
    .overflow_hidden()
    .rounded(glass_radius)
    .child(cancel_row);

    // Early return when there are no action items.
    if item_rows.is_empty() {
        let cancel_only = div()
            .flex()
            .flex_col()
            .w_full()
            .id(id)
            .child(cancel_group)
            .track_focus(&focus_handle);

        let on_cancel = on_cancel_rc;
        return cancel_only
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                handle_key(event, window, cx, &focus_group, &on_cancel);
            })
            .into_any_element();
    }

    // Item group container (only reached when items exist) — Elevated.
    let item_group = glass_effect_lens(
        theme,
        Glass::Regular,
        Shape::RoundedRectangle(glass_radius),
        Elevation::Elevated,
        None,
    )
    .w_full()
    .overflow_hidden()
    .rounded(glass_radius)
    .children(item_rows);

    // Overall container stacks item group and cancel group with spacing.
    let container = div()
        .flex()
        .flex_col()
        .w_full()
        .gap(theme.spacing_sm)
        .id(id)
        .child(item_group)
        .child(cancel_group)
        .track_focus(&focus_handle);

    let on_cancel = on_cancel_rc;
    container
        .on_key_down(move |event: &KeyDownEvent, window, cx| {
            handle_key(event, window, cx, &focus_group, &on_cancel);
        })
        .into_any_element()
}

/// Shared Escape / Tab handler for both ActionSheet presentations.
///
/// Escape routes through `on_cancel` so keyboard dismissal mirrors the
/// Cancel button. Tab is dispatched through the Trap-mode [`FocusGroup`];
/// when the group is empty (items are not focusable today) Tab is
/// swallowed so focus cannot escape the sheet surface per the WAI-ARIA
/// dialog pattern.
fn handle_key(
    event: &KeyDownEvent,
    window: &mut Window,
    cx: &mut App,
    focus_group: &FocusGroup,
    on_cancel: &Option<SharedCancel>,
) {
    if crate::foundations::keyboard::is_escape_key(event) {
        if let Some(handler) = on_cancel {
            handler(window, cx);
        }
        return;
    }
    if event.keystroke.key.as_str() == "tab" && !focus_group.handle_key_down(event, window, cx) {
        cx.stop_propagation();
    }
}

/// macOS centered confirmation-dialog chrome. HIG `#action-sheets`:
/// macOS action sheets are **non-modal** — they do not dim the parent
/// window and do not dismiss on an outside click. The panel is centered
/// in a transparent full-viewport container; dismissal is via Cancel
/// button or Escape key only.
fn render_centered(
    id: ElementId,
    theme: &TahoeTheme,
    _glass_radius: gpui::Pixels,
    item_rows: Vec<gpui::Stateful<gpui::Div>>,
    cancel_row: gpui::Stateful<gpui::Div>,
    on_cancel_rc: Option<SharedCancel>,
    focus_handle: FocusHandle,
    focus_group: FocusGroup,
) -> gpui::AnyElement {
    // macOS centered action sheet is a 320pt panel — Elevated tier.
    let mut panel = glass_effect_lens(
        theme,
        Glass::Regular,
        Shape::RoundedRectangle(theme.radius_lg),
        Elevation::Elevated,
        None,
    )
    .w(px(320.0))
    .overflow_hidden()
    .id(ElementId::from((id.clone(), "panel")))
    .flex()
    .flex_col();

    // Separator between items (reuse existing per-row top border on rows > 0)
    for row in item_rows {
        panel = panel.child(row);
    }
    panel = panel.child(cancel_row).track_focus(&focus_handle);

    let on_cancel = on_cancel_rc;
    panel = panel.on_key_down(move |event: &KeyDownEvent, window, cx| {
        handle_key(event, window, cx, &focus_group, &on_cancel);
    });

    // Non-modal: transparent full-viewport container, no backdrop, no
    // outside-click dismissal. The sheet coexists with the parent UI.
    div()
        .id(id)
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .child(panel)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{ActionSheet, ActionSheetItem, ActionSheetPresentation, ActionSheetStyle};
    use crate::foundations::accessibility::{FocusGroup, FocusGroupMode};
    use crate::foundations::layout::Platform;
    use core::prelude::v1::test;

    #[test]
    fn action_sheet_new_defaults() {
        let sheet = ActionSheet::new("sheet");
        assert!(sheet.items.is_empty());
        assert_eq!(sheet.cancel_text.as_ref(), "Cancel");
        assert!(!sheet.is_open, "default is_open should be false");
        assert!(sheet.presentation.is_none());
    }

    #[test]
    fn action_sheet_builder_items() {
        let sheet = ActionSheet::new("sheet").items(vec![
            ActionSheetItem::new("Delete"),
            ActionSheetItem::new("Share"),
        ]);
        assert_eq!(sheet.items.len(), 2);
    }

    #[test]
    fn action_sheet_builder_cancel_text() {
        let sheet = ActionSheet::new("sheet").cancel_text("Abort");
        assert_eq!(sheet.cancel_text.as_ref(), "Abort");
    }

    #[test]
    fn action_sheet_style_default() {
        assert_eq!(ActionSheetStyle::default(), ActionSheetStyle::Default);
    }

    #[test]
    fn action_sheet_item_new() {
        let item = ActionSheetItem::new("Edit");
        assert_eq!(item.label.as_ref(), "Edit");
        assert_eq!(item.style, ActionSheetStyle::Default);
        assert!(item.on_click.is_none());
    }

    #[test]
    fn action_sheet_item_builder() {
        let item = ActionSheetItem::new("Delete").style(ActionSheetStyle::Destructive);
        assert_eq!(item.style, ActionSheetStyle::Destructive);
    }

    #[test]
    fn action_sheet_style_all_distinct() {
        let styles = [ActionSheetStyle::Default, ActionSheetStyle::Destructive];
        assert_ne!(styles[0], styles[1]);
    }

    #[test]
    fn action_sheet_on_cancel_builder() {
        let sheet = ActionSheet::new("sheet").on_cancel(|_, _| {});
        assert!(sheet.on_cancel.is_some());
    }

    #[test]
    fn action_sheet_focus_handle_builder() {
        // We can't create a FocusHandle outside of a GPUI context,
        // but we can verify the field exists and defaults to None.
        let sheet = ActionSheet::new("sheet");
        assert!(sheet.focus_handle.is_none());
    }

    #[test]
    fn action_sheet_open_builder() {
        let sheet = ActionSheet::new("sheet").open(false);
        assert!(!sheet.is_open);
    }

    #[test]
    fn action_sheet_presentation_builder() {
        let sheet = ActionSheet::new("sheet").presentation(ActionSheetPresentation::Centered);
        assert_eq!(sheet.presentation, Some(ActionSheetPresentation::Centered));
    }

    #[test]
    fn action_sheet_focus_cycle_defaults_to_empty() {
        let sheet = ActionSheet::new("sheet");
        assert!(sheet.focus_group.is_empty());
    }

    #[test]
    fn action_sheet_focus_group_defaults_to_trap_mode() {
        let sheet = ActionSheet::new("sheet");
        assert_eq!(sheet.focus_group.mode(), FocusGroupMode::Trap);
    }

    #[test]
    #[should_panic(expected = "ActionSheet::focus_group requires a Trap-mode FocusGroup")]
    fn action_sheet_focus_group_rejects_non_trap_mode() {
        let _ = ActionSheet::new("sheet").focus_group(FocusGroup::cycle());
    }

    #[test]
    fn action_sheet_restore_focus_default_is_none() {
        let sheet = ActionSheet::new("sheet");
        assert!(sheet.restore_focus_to.is_none());
    }

    #[test]
    fn presentation_for_platform_matches_hig() {
        assert_eq!(
            ActionSheetPresentation::for_platform(Platform::MacOS),
            ActionSheetPresentation::Centered
        );
        assert_eq!(
            ActionSheetPresentation::for_platform(Platform::VisionOS),
            ActionSheetPresentation::Centered
        );
        assert_eq!(
            ActionSheetPresentation::for_platform(Platform::IOS),
            ActionSheetPresentation::BottomDrawer
        );
        assert_eq!(
            ActionSheetPresentation::for_platform(Platform::WatchOS),
            ActionSheetPresentation::BottomDrawer
        );
    }
}

#[cfg(test)]
mod interaction_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{
        Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render,
        TestAppContext, div,
    };

    use super::{ActionSheet, ActionSheetItem, ActionSheetStyle};
    use crate::test_helpers::helpers::{InteractionExt, setup_test_window};

    const ACTION_DELETE: &str = "action-sheet-item-0";
    const ACTION_SHARE: &str = "action-sheet-item-1";
    const ACTION_CANCEL: &str = "action-sheet-cancel";

    struct ActionSheetHarness {
        focus_handle: FocusHandle,
        actions: Rc<RefCell<Vec<String>>>,
        cancel_count: Rc<RefCell<usize>>,
    }

    impl ActionSheetHarness {
        fn new(
            cx: &mut Context<Self>,
            actions: Rc<RefCell<Vec<String>>>,
            cancel_count: Rc<RefCell<usize>>,
        ) -> Self {
            Self {
                focus_handle: cx.focus_handle(),
                actions,
                cancel_count,
            }
        }
    }

    impl Render for ActionSheetHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            ActionSheet::new("sheet")
                .items(vec![
                    ActionSheetItem::new("Delete")
                        .style(ActionSheetStyle::Destructive)
                        .on_click({
                            let actions = self.actions.clone();
                            move |_, _, _| actions.borrow_mut().push("Delete".into())
                        }),
                    ActionSheetItem::new("Share").on_click({
                        let actions = self.actions.clone();
                        move |_, _, _| actions.borrow_mut().push("Share".into())
                    }),
                ])
                .on_cancel({
                    let cancel_count = self.cancel_count.clone();
                    move |_, _| *cancel_count.borrow_mut() += 1
                })
                .focus_handle(self.focus_handle.clone())
                .open(true)
        }
    }

    fn focus_sheet(host: &gpui::Entity<ActionSheetHarness>, cx: &mut gpui::VisualTestContext) {
        host.update_in(cx, |host, window, cx| {
            host.focus_handle.focus(window, cx);
        });
    }

    #[gpui::test]
    async fn clicking_items_invokes_default_and_destructive_actions(cx: &mut TestAppContext) {
        let actions = Rc::new(RefCell::new(Vec::new()));
        let cancel_count = Rc::new(RefCell::new(0));
        let (_host, cx) = setup_test_window(cx, |_window, cx| {
            ActionSheetHarness::new(cx, actions.clone(), cancel_count.clone())
        });

        cx.click_on(ACTION_DELETE);
        cx.click_on(ACTION_SHARE);

        assert_eq!(
            &*actions.borrow(),
            &["Delete".to_string(), "Share".to_string()]
        );
        assert_eq!(*cancel_count.borrow(), 0);
    }

    #[gpui::test]
    async fn cancel_button_and_escape_trigger_cancel(cx: &mut TestAppContext) {
        let actions = Rc::new(RefCell::new(Vec::new()));
        let cancel_count = Rc::new(RefCell::new(0));
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            ActionSheetHarness::new(cx, actions.clone(), cancel_count.clone())
        });

        cx.click_on(ACTION_CANCEL);
        focus_sheet(&host, cx);
        cx.press("escape");

        assert!(actions.borrow().is_empty());
        assert_eq!(*cancel_count.borrow(), 2);
    }

    // Harness verifying the empty-trap contract: ActionSheet items render
    // as non-focusable divs today, so the Trap FocusGroup is empty by
    // construction. Tab must still be swallowed to keep focus inside the
    // sheet per WAI-ARIA dialog pattern.
    struct EmptyTrapHarness {
        outer_focus: FocusHandle,
        outside: FocusHandle,
        presentation: super::ActionSheetPresentation,
    }

    impl EmptyTrapHarness {
        fn new(cx: &mut gpui::Context<Self>, presentation: super::ActionSheetPresentation) -> Self {
            Self {
                outer_focus: cx.focus_handle(),
                outside: cx.focus_handle(),
                presentation,
            }
        }
    }

    impl Render for EmptyTrapHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            div()
                .child(
                    div()
                        .id("outside")
                        .track_focus(&self.outside)
                        .child("Outside"),
                )
                .child(
                    ActionSheet::new("sheet")
                        .items(vec![
                            ActionSheetItem::new("Delete"),
                            ActionSheetItem::new("Share"),
                        ])
                        .presentation(self.presentation)
                        .focus_handle(self.outer_focus.clone())
                        .on_cancel(|_, _| {})
                        .open(true),
                )
        }
    }

    #[gpui::test]
    async fn tab_in_action_sheet_does_not_escape_bottom_drawer(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            EmptyTrapHarness::new(cx, super::ActionSheetPresentation::BottomDrawer)
        });
        host.update_in(cx, |host, window, cx| {
            host.outer_focus.focus(window, cx);
        });
        cx.press("tab");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.outer_focus.is_focused(window),
                "Tab with an empty FocusGroup must leave outer focus put"
            );
            assert!(
                !host.outside.is_focused(window),
                "Tab must not reach elements outside the action sheet"
            );
        });
    }

    #[gpui::test]
    async fn tab_in_action_sheet_does_not_escape_centered(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            EmptyTrapHarness::new(cx, super::ActionSheetPresentation::Centered)
        });
        host.update_in(cx, |host, window, cx| {
            host.outer_focus.focus(window, cx);
        });
        cx.press("tab");
        host.update_in(cx, |host, window, _cx| {
            assert!(host.outer_focus.is_focused(window));
            assert!(!host.outside.is_focused(window));
        });
    }

    // Harness that wires `restore_focus_to` so we can observe focus
    // returning to the previously-focused element after Escape dismisses
    // the sheet.
    struct RestoreFocusHarness {
        sheet_focus: FocusHandle,
        previous: FocusHandle,
        is_open: bool,
        cancelled: Rc<RefCell<bool>>,
        presentation: super::ActionSheetPresentation,
    }

    impl RestoreFocusHarness {
        fn new(
            cx: &mut gpui::Context<Self>,
            cancelled: Rc<RefCell<bool>>,
            presentation: super::ActionSheetPresentation,
        ) -> Self {
            Self {
                sheet_focus: cx.focus_handle(),
                previous: cx.focus_handle(),
                is_open: true,
                cancelled,
                presentation,
            }
        }
    }

    impl Render for RestoreFocusHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            cx: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            let entity = cx.entity().clone();
            let mut outer = div().child(
                div()
                    .id("prev")
                    .track_focus(&self.previous)
                    .child("Previous"),
            );
            if self.is_open {
                outer = outer.child(
                    ActionSheet::new("sheet")
                        .items(vec![ActionSheetItem::new("Delete")])
                        .presentation(self.presentation)
                        .focus_handle(self.sheet_focus.clone())
                        .restore_focus_to(self.previous.clone())
                        .on_cancel(move |_, cx| {
                            entity.update(cx, |this, cx| {
                                this.is_open = false;
                                *this.cancelled.borrow_mut() = true;
                                cx.notify();
                            });
                        })
                        .open(true),
                );
            }
            outer
        }
    }

    #[gpui::test]
    async fn escape_restores_focus_to_previous_element_bottom_drawer(cx: &mut TestAppContext) {
        let cancelled = Rc::new(RefCell::new(false));
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            RestoreFocusHarness::new(
                cx,
                cancelled.clone(),
                super::ActionSheetPresentation::BottomDrawer,
            )
        });

        host.update_in(cx, |host, window, cx| {
            host.sheet_focus.focus(window, cx);
        });
        cx.press("escape");

        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.previous.is_focused(window),
                "focus should be restored to `previous` after Escape"
            );
        });
        assert!(*cancelled.borrow(), "on_cancel should still fire");
    }

    #[gpui::test]
    async fn escape_restores_focus_to_previous_element_centered(cx: &mut TestAppContext) {
        let cancelled = Rc::new(RefCell::new(false));
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            RestoreFocusHarness::new(
                cx,
                cancelled.clone(),
                super::ActionSheetPresentation::Centered,
            )
        });

        host.update_in(cx, |host, window, cx| {
            host.sheet_focus.focus(window, cx);
        });
        cx.press("escape");

        host.update_in(cx, |host, window, _cx| {
            assert!(host.previous.is_focused(window));
        });
        assert!(*cancelled.borrow());
    }
}
