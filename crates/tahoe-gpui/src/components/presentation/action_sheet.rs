//! Action sheet component with Apple glass material.
//!
//! Presents a list of action choices plus a cancel button. Structure
//! (title + choices + Cancel) applies universally per HIG
//! `#action-sheets`; the *chrome* varies by platform:
//!
//! - **iOS / iPadOS / watchOS** — springs from the bottom of the screen
//!   as a drawer. Cancel renders in its own grouped surface.
//! - **macOS / visionOS** — HIG: "No additional considerations for
//!   macOS" — renders as a centered confirmation dialog (no bottom
//!   anchoring, no drawer). Use [`ActionSheetPresentation::Centered`]
//!   to request this chrome explicitly.
//!
//! The `presentation` is auto-selected from the active
//! [`TahoeTheme::platform`] unless [`ActionSheet::presentation`] is
//! called.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/action-sheets>

use gpui::prelude::*;
use gpui::{
    App, ClickEvent, ElementId, FocusHandle, KeyDownEvent, MouseDownEvent, SharedString, Window,
    div, px,
};
use std::rc::Rc;

use crate::callback_types::OnMutCallback;
use crate::foundations::layout::Platform;
use crate::foundations::materials::{SurfaceContext, backdrop_overlay, glass_surface};
use crate::foundations::theme::{ActiveTheme, GlassSize, TahoeTheme, TextStyle, TextStyledExt};

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
#[derive(IntoElement)]
pub struct ActionSheet {
    id: ElementId,
    items: Vec<ActionSheetItem>,
    cancel_text: SharedString,
    on_cancel: OnMutCallback,
    focus_handle: Option<FocusHandle>,
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
            // Default to open so pre-existing call sites that managed
            // visibility by unmount/remount behave unchanged. New code
            // should prefer the explicit `.open(bool)` gate.
            is_open: true,
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

    /// Set a focus handle for keyboard navigation (Escape to cancel).
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
}

impl RenderOnce for ActionSheet {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.is_open {
            return div().into_any_element();
        }

        let theme = cx.theme();
        let glass_radius = theme.glass.radius(GlassSize::Medium);
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

        // Wrap on_cancel in Rc so it can be shared between cancel_row click
        // and the keyboard handler further below.
        let on_cancel_rc: Option<SharedCancel> = self.on_cancel.map(Rc::from);

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
                self.focus_handle,
            ),
            ActionSheetPresentation::Centered => render_centered(
                self.id,
                theme,
                glass_radius,
                item_rows,
                cancel_row,
                on_cancel_rc,
                self.focus_handle,
            ),
        }
    }
}

/// iOS/watchOS bottom-drawer chrome — cancel group sits below the item
/// group with a small gap.
fn render_bottom_drawer(
    id: ElementId,
    theme: &TahoeTheme,
    glass_radius: gpui::Pixels,
    item_rows: Vec<gpui::Stateful<gpui::Div>>,
    cancel_row: gpui::Stateful<gpui::Div>,
    on_cancel_rc: Option<SharedCancel>,
    focus_handle: Option<FocusHandle>,
) -> gpui::AnyElement {
    // Cancel container.
    let cancel_group = glass_surface(
        div().w_full().overflow_hidden().rounded(glass_radius),
        theme,
        GlassSize::Large,
    )
    .child(cancel_row);

    // Early return when there are no action items.
    if item_rows.is_empty() {
        let mut cancel_only = div().flex().flex_col().w_full().id(id).child(cancel_group);

        let has_focus = focus_handle.is_some();
        if let Some(ref handle) = focus_handle {
            cancel_only = cancel_only.track_focus(handle);
        }

        if has_focus {
            let on_cancel = on_cancel_rc;
            cancel_only = cancel_only.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if crate::foundations::keyboard::is_escape_key(event) {
                    if let Some(handler) = &on_cancel {
                        handler(window, cx);
                    }
                }
            });
        }

        return cancel_only.into_any_element();
    }

    // Item group container (only reached when items exist).
    let item_group = glass_surface(
        div().w_full().overflow_hidden().rounded(glass_radius),
        theme,
        GlassSize::Large,
    )
    .children(item_rows);

    // Overall container stacks item group and cancel group with spacing.
    let mut container = div()
        .flex()
        .flex_col()
        .w_full()
        .gap(theme.spacing_sm)
        .id(id);

    container = container.child(item_group).child(cancel_group);

    let has_focus = focus_handle.is_some();
    if let Some(ref handle) = focus_handle {
        container = container.track_focus(handle);
    }

    if has_focus {
        let on_cancel = on_cancel_rc;
        container = container.on_key_down(move |event: &KeyDownEvent, window, cx| {
            if crate::foundations::keyboard::is_escape_key(event) {
                if let Some(handler) = &on_cancel {
                    handler(window, cx);
                }
            }
        });
    }

    container.into_any_element()
}

/// macOS centered confirmation-dialog chrome. HIG `#action-sheets`:
/// "No additional considerations for macOS" — the component structure
/// still applies, but the panel is centered over its parent with a
/// translucent backdrop rather than sliding from the bottom.
fn render_centered(
    id: ElementId,
    theme: &TahoeTheme,
    _glass_radius: gpui::Pixels,
    item_rows: Vec<gpui::Stateful<gpui::Div>>,
    cancel_row: gpui::Stateful<gpui::Div>,
    on_cancel_rc: Option<SharedCancel>,
    focus_handle: Option<FocusHandle>,
) -> gpui::AnyElement {
    let mut panel = glass_surface(
        div().w(px(320.0)).overflow_hidden(),
        theme,
        GlassSize::Large,
    )
    .id(ElementId::from((id.clone(), "panel")))
    .flex()
    .flex_col();

    // Separator between items (reuse existing per-row top border on rows > 0)
    for row in item_rows {
        panel = panel.child(row);
    }
    panel = panel.child(cancel_row);

    if let Some(ref handle) = focus_handle {
        panel = panel.track_focus(handle);
    }
    if focus_handle.is_some() {
        let on_cancel = on_cancel_rc.clone();
        panel = panel.on_key_down(move |event: &KeyDownEvent, window, cx| {
            if crate::foundations::keyboard::is_escape_key(event) {
                if let Some(handler) = &on_cancel {
                    handler(window, cx);
                }
            }
        });
    }
    if let Some(handler) = on_cancel_rc.clone() {
        panel = panel.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
            handler(window, cx);
        });
    }

    backdrop_overlay(theme)
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .child(panel)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{ActionSheet, ActionSheetItem, ActionSheetPresentation, ActionSheetStyle};
    use crate::foundations::layout::Platform;
    use core::prelude::v1::test;

    #[test]
    fn action_sheet_new_defaults() {
        let sheet = ActionSheet::new("sheet");
        assert!(sheet.items.is_empty());
        assert_eq!(sheet.cancel_text.as_ref(), "Cancel");
        assert!(sheet.is_open, "default is_open should be true");
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

    use gpui::{Context, FocusHandle, IntoElement, Render, TestAppContext};

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
}
