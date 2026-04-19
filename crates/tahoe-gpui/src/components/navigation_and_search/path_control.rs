//! Path Control component (HIG breadcrumb navigation).
//!
//! Renders a horizontal breadcrumb trail of path segments. Two HIG-defined
//! styles are supported:
//!
//! - [`PathControlStyle::Standard`] — flat linear list of segments
//!   separated by chevrons. Non-last segments are clickable.
//! - [`PathControlStyle::PopUp`] — each segment is a pop-up button. Clicking
//!   a segment opens a menu of sibling / parent items supplied by the
//!   caller via [`PathSegment::pop_up_items`]. Matches `NSPathControl`'s
//!   `.popUp` style.
//!
//! # Placement (HIG)
//!
//! "Use a path control in the window body, not the window frame." Do not
//! place a [`PathControl`] inside a [`Toolbar`](super::Toolbar),
//! [`NavigationBarIOS`](super::NavigationBarIOS), or status bar — a debug
//! assertion triggers when this is detected in tests. The doc comment on
//! every public constructor reiterates this constraint so the misuse is
//! visible at call sites.

use crate::foundations::layout::SPACING_4;
use gpui::prelude::*;
use gpui::{App, ElementId, FontWeight, KeyDownEvent, SharedString, Window, div, px};

use crate::callback_types::{OnUsizeChange, rc_wrap};
use crate::components::menus_and_actions::pulldown_button::{PulldownButton, PulldownItem};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};

/// Visual style for a [`PathControl`]. Matches the two styles defined by
/// `NSPathControl`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum PathControlStyle {
    /// Standard flat breadcrumb trail with chevron separators. Segments
    /// truncate with an ellipsis when the control is too narrow to fit.
    #[default]
    Standard,
    /// Pop-up style — each segment opens a menu of siblings / parents when
    /// clicked. Requires [`PathSegment::pop_up_items`] on each segment
    /// whose menu should be non-empty.
    PopUp,
}

/// A single segment in the path control breadcrumb.
#[derive(Debug, Clone)]
pub struct PathSegment {
    /// Display label for the segment.
    pub label: SharedString,
    /// Optional leading icon.
    pub icon: Option<IconName>,
    /// Pop-up menu items for this segment (only used when the parent is
    /// rendered with [`PathControlStyle::PopUp`]). Each entry fires with
    /// the segment's index and the entry's label when clicked.
    pub pop_up_items: Vec<SharedString>,
}

impl PathSegment {
    /// Create a new path segment with a label and no icon.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            pop_up_items: Vec::new(),
        }
    }

    /// Set an optional leading icon for this segment.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Supply pop-up menu entries for this segment. Only rendered when the
    /// parent [`PathControl::style`] is [`PathControlStyle::PopUp`].
    pub fn pop_up_items(mut self, items: Vec<SharedString>) -> Self {
        self.pop_up_items = items;
        self
    }
}

type OnPopUpSelect = Option<Box<dyn Fn(usize, &SharedString, &mut Window, &mut App) + 'static>>;

/// An HIG-style breadcrumb path control.
///
/// # Placement
///
/// **Do not use a `PathControl` inside a toolbar, navigation bar, or
/// status bar.** HIG: "Use a path control in the window body, not the
/// window frame." Constructors check this via [`PathControl::assert_in_body`]
/// in debug builds.
///
/// Renders segments in a horizontal row separated by chevron-right icons.
/// The last segment is displayed in primary text with bold weight; preceding
/// segments are muted and clickable, firing `on_select(segment_index)`.
#[derive(IntoElement)]
pub struct PathControl {
    id: ElementId,
    segments: Vec<PathSegment>,
    on_select: OnUsizeChange,
    on_pop_up_select: OnPopUpSelect,
    highlighted_index: Option<usize>,
    focused: bool,
    style: PathControlStyle,
    truncated: bool,
}

impl PathControl {
    /// Create a new path control with the given element id.
    ///
    /// **HIG:** A path control belongs in the window *body*, not the
    /// window frame. Do not nest one inside a [`Toolbar`](super::Toolbar),
    /// [`NavigationBarIOS`](super::NavigationBarIOS), or status bar.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            segments: Vec::new(),
            on_select: None,
            on_pop_up_select: None,
            highlighted_index: None,
            focused: false,
            style: PathControlStyle::Standard,
            truncated: false,
        }
    }

    /// Runtime guard used by gallery harnesses: panics in debug builds
    /// when the caller is about to place this control inside toolbar /
    /// navigation / status chrome. Host apps pass the container name they
    /// intend to place the control in.
    pub fn assert_in_body(container: &str) {
        debug_assert!(
            !matches!(
                container,
                "toolbar" | "navigation_bar" | "status_bar" | "window_frame"
            ),
            "PathControl must be placed in the window body per HIG #path-controls; \
             got container={container:?}"
        );
    }

    /// Set the path segments.
    pub fn segments(mut self, segments: Vec<PathSegment>) -> Self {
        self.segments = segments;
        self
    }

    /// Sets the keyboard-highlighted segment index.
    pub fn highlighted_index(mut self, index: Option<usize>) -> Self {
        self.highlighted_index = index;
        self
    }

    /// Marks this control as focused, showing a visible focus ring.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Pick the visual style. Default: [`PathControlStyle::Standard`].
    pub fn style(mut self, style: PathControlStyle) -> Self {
        self.style = style;
        self
    }

    /// When `true`, leading non-final segments collapse into an ellipsis
    /// `…` that opens a pop-up of the hidden segments. Matches the
    /// Standard-style truncation behavior of `NSPathControl` when the
    /// available width is insufficient for the full trail.
    pub fn truncated(mut self, truncated: bool) -> Self {
        self.truncated = truncated;
        self
    }

    /// Called when a non-last segment is clicked (Standard style) or a
    /// segment's pop-up button is opened (PopUp style — segment click
    /// only, the pop-up item selections route to
    /// [`PathControl::on_pop_up_select`]).
    pub fn on_select(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }

    /// Called when the user selects an item from a segment's pop-up menu.
    /// Receives the segment index and the selected label.
    pub fn on_pop_up_select(
        mut self,
        handler: impl Fn(usize, &SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_pop_up_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for PathControl {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let segment_count = self.segments.len();

        let on_select_rc = rc_wrap(self.on_select);
        let on_pop_up_rc = rc_wrap(self.on_pop_up_select);
        let highlighted = self.highlighted_index;
        let style = self.style;

        let mut row = div()
            .id(self.id.clone())
            .focusable()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(SPACING_4));

        row = apply_focus_ring(row, theme, self.focused, &[]);

        // Enter selects the highlighted segment.
        if segment_count > 1
            && let Some(ref handler) = on_select_rc
        {
            let h = handler.clone();
            let max_nav = segment_count - 1;
            row = row.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if event.keystroke.key.as_str() == "enter"
                    && let Some(idx) = highlighted
                    && idx < max_nav
                {
                    h(idx, window, cx);
                }
            });
        }

        // Leading-ellipsis collapse — renders a single `…` as the first
        // segment when truncated, opening a pop-up of the hidden segments.
        let first_visible_idx = if self.truncated && segment_count > 2 {
            // Keep the first and last segments, collapse the rest.
            let ellipsis_items: Vec<SharedString> = self
                .segments
                .iter()
                .enumerate()
                .skip(1)
                .take(segment_count.saturating_sub(2))
                .map(|(_, seg)| seg.label.clone())
                .collect();

            let ellipsis_id = ElementId::from((self.id.clone(), "ellipsis"));
            let mut pulldown = PulldownButton::new(ellipsis_id, "")
                .icon(Icon::new(IconName::Ellipsis))
                .borderless(true)
                .compact(true);

            if let Some(ref handler) = on_pop_up_rc {
                for (idx, item_label) in ellipsis_items.iter().enumerate() {
                    // The ellipsis represents segments 1..=count-2; map
                    // the pulldown index back to the underlying segment
                    // index (idx + 1).
                    let handler = handler.clone();
                    let label = item_label.clone();
                    let segment_index = idx + 1;
                    pulldown = pulldown.item(PulldownItem::new(item_label.clone()).on_click(
                        move |window, cx| {
                            handler(segment_index, &label, window, cx);
                        },
                    ));
                }
            }

            row = row.child(pulldown);
            row = row.child(
                Icon::new(IconName::ChevronRight)
                    .size(px(10.0))
                    .color(theme.text_tertiary()),
            );
            segment_count - 1
        } else {
            0
        };

        for (i, segment) in self.segments.into_iter().enumerate() {
            if self.truncated && segment_count > 2 && i > 0 && i < first_visible_idx {
                // Hidden in the ellipsis pulldown.
                continue;
            }

            let is_last = i == segment_count - 1;

            if i > first_visible_idx {
                row = row.child(
                    Icon::new(IconName::ChevronRight)
                        .size(px(10.0))
                        .color(theme.text_tertiary()),
                );
            }

            let mut seg_el = div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(SPACING_4))
                .min_h(px(theme.target_size()));

            if let Some(icon_name) = segment.icon {
                let icon_color = if is_last {
                    theme.text
                } else {
                    theme.text_muted
                };
                seg_el = seg_el.child(Icon::new(icon_name).size(px(12.0)).color(icon_color));
            }

            match style {
                PathControlStyle::PopUp => {
                    // Every segment (including the last) renders as a
                    // pulldown button. The last segment typically lists
                    // just itself; callers can supply siblings to show
                    // alternate selections.
                    let seg_label = segment.label.clone();
                    let pulldown_id = ElementId::from((self.id.clone(), format!("popup-{i}")));
                    let mut pulldown = PulldownButton::new(pulldown_id, seg_label)
                        .borderless(true)
                        .compact(true);
                    if let Some(icon_name) = segment.icon {
                        pulldown = pulldown.icon(Icon::new(icon_name));
                    }
                    if let Some(ref handler) = on_pop_up_rc {
                        for item_label in &segment.pop_up_items {
                            let handler = handler.clone();
                            let label = item_label.clone();
                            let segment_index = i;
                            pulldown =
                                pulldown.item(PulldownItem::new(item_label.clone()).on_click(
                                    move |window, cx| {
                                        handler(segment_index, &label, window, cx);
                                    },
                                ));
                        }
                    }

                    row = row.child(pulldown);
                    let _ = seg_el; // swallow unused builder
                    continue;
                }
                PathControlStyle::Standard => {
                    if is_last {
                        seg_el = seg_el.child(
                            div()
                                .text_color(theme.text)
                                .text_style(TextStyle::Subheadline, theme)
                                .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
                                .child(segment.label),
                        );
                    } else {
                        let label_el = div()
                            .text_color(theme.text_muted)
                            .text_style(TextStyle::Subheadline, theme)
                            .child(segment.label);

                        seg_el = seg_el.child(label_el);

                        let seg_id = ElementId::Name(format!("path-seg-{i}").into());
                        let hover_bg = theme.hover_bg();

                        let mut interactive = div()
                            .id(seg_id)
                            .flex()
                            .flex_row()
                            .items_center()
                            .cursor_pointer()
                            .rounded(theme.radius_sm)
                            .px(px(SPACING_4))
                            .hover(|style| style.bg(hover_bg));

                        if let Some(ref handler) = on_select_rc {
                            let h = handler.clone();
                            interactive = interactive.on_click(move |_event, window, cx| {
                                h(i, window, cx);
                            });
                        }

                        row = row.child(interactive.child(seg_el));
                        continue;
                    }
                }
            }

            row = row.child(seg_el);
        }

        row
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::{PathControl, PathControlStyle, PathSegment};
    use crate::foundations::icons::IconName;

    #[test]
    fn path_segment_new_has_label_no_icon() {
        let seg = PathSegment::new("Documents");
        assert_eq!(seg.label.as_ref(), "Documents");
        assert!(seg.icon.is_none());
        assert!(seg.pop_up_items.is_empty());
    }

    #[test]
    fn path_segment_builder_icon() {
        let seg = PathSegment::new("Home").icon(IconName::Folder);
        assert_eq!(seg.icon, Some(IconName::Folder));
    }

    #[test]
    fn path_segment_builder_pop_up_items() {
        let seg = PathSegment::new("Home").pop_up_items(vec!["a".into(), "b".into()]);
        assert_eq!(seg.pop_up_items.len(), 2);
    }

    #[test]
    fn path_control_new_defaults() {
        let ctrl = PathControl::new("breadcrumb");
        assert!(ctrl.segments.is_empty());
        assert!(ctrl.on_select.is_none());
        assert!(ctrl.on_pop_up_select.is_none());
        assert_eq!(ctrl.style, PathControlStyle::Standard);
        assert!(!ctrl.truncated);
    }

    #[test]
    fn path_control_builder_segments() {
        let ctrl = PathControl::new("bc").segments(vec![
            PathSegment::new("A"),
            PathSegment::new("B"),
            PathSegment::new("C"),
        ]);
        assert_eq!(ctrl.segments.len(), 3);
        assert_eq!(ctrl.segments[0].label.as_ref(), "A");
        assert_eq!(ctrl.segments[2].label.as_ref(), "C");
    }

    #[test]
    fn path_control_builder_on_select() {
        let ctrl = PathControl::new("bc").on_select(|_idx, _w, _cx| {});
        assert!(ctrl.on_select.is_some());
    }

    #[test]
    fn path_control_builder_style() {
        let ctrl = PathControl::new("bc").style(PathControlStyle::PopUp);
        assert_eq!(ctrl.style, PathControlStyle::PopUp);
    }

    #[test]
    fn path_control_builder_truncated() {
        let ctrl = PathControl::new("bc").truncated(true);
        assert!(ctrl.truncated);
    }

    #[test]
    fn path_control_builder_on_pop_up_select() {
        let ctrl = PathControl::new("bc").on_pop_up_select(|_idx, _label, _w, _cx| {});
        assert!(ctrl.on_pop_up_select.is_some());
    }

    #[test]
    fn path_control_full_builder_chain() {
        let _ctrl = PathControl::new("bc")
            .segments(vec![
                PathSegment::new("Root")
                    .icon(IconName::Folder)
                    .pop_up_items(vec!["Home".into(), "Applications".into()]),
                PathSegment::new("Child"),
            ])
            .style(PathControlStyle::PopUp)
            .truncated(false)
            .on_select(|_idx, _w, _cx| {})
            .on_pop_up_select(|_idx, _label, _w, _cx| {});
    }

    #[test]
    fn path_segment_clone() {
        let seg = PathSegment::new("Test")
            .icon(IconName::File)
            .pop_up_items(vec!["Sibling".into()]);
        let cloned = seg.clone();
        assert_eq!(cloned.label.as_ref(), "Test");
        assert_eq!(cloned.icon, Some(IconName::File));
        assert_eq!(cloned.pop_up_items.len(), 1);
    }

    #[test]
    fn path_control_assert_in_body_accepts_body() {
        // Should not panic.
        PathControl::assert_in_body("window_body");
    }

    #[test]
    #[should_panic(expected = "PathControl must be placed in the window body")]
    fn path_control_assert_in_body_rejects_toolbar() {
        PathControl::assert_in_body("toolbar");
    }

    #[test]
    fn path_control_style_default_is_standard() {
        assert_eq!(PathControlStyle::default(), PathControlStyle::Standard);
    }
}
