//! Anchored overlay primitive — shared surface for menus, dropdowns,
//! popovers, and context menus that need to escape parent `overflow_hidden()`
//! clipping and float above the window.
//!
//! Built on GPUI's `anchored()` + `deferred()` + `occlude()` primitives, it
//! reads the trigger's current-frame layout bounds during `prepaint` and
//! positions the overlay at window-absolute coordinates that bypass the
//! parent clip chain. This mirrors Zed's `PopoverMenu` pattern
//! (`crates/ui/src/components/popover_menu.rs`) but with an eager, stateless
//! trigger/content API so existing consumers can migrate without adopting
//! `ManagedView`.
//!
//! Overlay content is laid out independently in prepaint (via
//! `AnyElement::layout_as_root`) so positioning is correct on the first
//! frame — no two-frame bootstrap or caller-side `cx.notify()` dance.
//!
//! See `foundations::layout`'s overflow-clipping audit note for the list
//! of in-crate components that should migrate onto this primitive.
//!
//! # Example
//!
//! ```ignore
//! use tahoe_gpui::foundations::overlay::{AnchoredOverlay, OverlayAnchor};
//! use gpui::{div, point, px};
//!
//! let overlay = AnchoredOverlay::new("my-overlay", div().child("Trigger"))
//!     .anchor(OverlayAnchor::BelowLeft)
//!     .offset(point(px(0.0), px(4.0)))
//!     .content_when(is_open, || div().child("Floating panel").into_any_element());
//! ```

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Corner, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, Pixels, Point, Size, Style, Window, anchored,
    deferred, div, prelude::*,
};

use crate::foundations::layout::DROPDOWN_SNAP_MARGIN;

/// Where an overlay attaches relative to its trigger.
///
/// Each variant encodes two corners: which corner of the trigger the
/// overlay latches onto (the "attach" corner) and which corner of the
/// overlay is placed at that point (the "anchor" corner).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverlayAnchor {
    /// Overlay's top-left meets the trigger's bottom-left.
    #[default]
    BelowLeft,
    /// Overlay's top-right meets the trigger's bottom-right.
    BelowRight,
    /// Overlay's bottom-left meets the trigger's top-left.
    AboveLeft,
    /// Overlay's bottom-right meets the trigger's top-right.
    AboveRight,
    /// Overlay is positioned at a window-absolute point, with no trigger
    /// anchoring. Used by `ContextMenu::show(pos)` and similar.
    WindowPoint(Point<Pixels>),
}

impl OverlayAnchor {
    /// Which corner of the overlay box is the anchor point.
    fn anchor_corner(self) -> Corner {
        match self {
            Self::BelowLeft => Corner::TopLeft,
            Self::BelowRight => Corner::TopRight,
            Self::AboveLeft => Corner::BottomLeft,
            Self::AboveRight => Corner::BottomRight,
            Self::WindowPoint(_) => Corner::TopLeft,
        }
    }

    /// Which corner of the trigger the overlay attaches to. Not applicable
    /// for `WindowPoint`, which carries its own window-absolute point.
    fn attach_corner(self) -> Option<Corner> {
        Some(match self {
            Self::BelowLeft => Corner::BottomLeft,
            Self::BelowRight => Corner::BottomRight,
            Self::AboveLeft => Corner::TopLeft,
            Self::AboveRight => Corner::TopRight,
            Self::WindowPoint(_) => return None,
        })
    }
}

/// Derive a deterministic child [`ElementId`] from a parent id and a static
/// suffix. Uses GPUI's [`Display`](std::fmt::Display) impl on `ElementId`
/// rather than the unstable `Debug` impl, so the resulting name stays
/// predictable across GPUI upgrades. Intended for components that need a
/// per-instance id for a child element (e.g. a popover's floating surface
/// id paired with the popover trigger's id).
pub fn child_id(parent: &ElementId, suffix: &'static str) -> ElementId {
    ElementId::Name(format!("{parent}-{suffix}").into())
}

/// Content builder that receives the realised anchor (after any
/// overflow-driven flip) so consumers whose rendering depends on
/// placement — such as a popover arrow glyph — can stay in sync with
/// the actual position.
type ContentFn = Box<dyn FnOnce(OverlayAnchor) -> AnyElement>;

/// Anchored, deferred, optionally occluding overlay.
///
/// The trigger element is always laid out in the normal tree. The content
/// element, when present, is built in `prepaint` (after the trigger's
/// bounds are realised by taffy) and wrapped in a
/// `deferred(anchored(...))` subtree so it paints after the rest of the
/// frame at window-absolute coordinates. Positioning is correct on the
/// first frame — no two-frame bootstrap.
pub struct AnchoredOverlay {
    id: ElementId,
    trigger: Option<AnyElement>,
    content: Option<AnyElement>,
    content_fn: Option<ContentFn>,
    anchor: OverlayAnchor,
    offset: Point<Pixels>,
    snap_margin: Pixels,
    priority: usize,
    occlude: bool,
}

impl AnchoredOverlay {
    /// Construct a new overlay wrapping the given trigger element.
    ///
    /// Call `.content(...)` with the floating overlay body when the
    /// overlay should be visible. Omitting `.content(...)` leaves the
    /// overlay closed (only the trigger renders).
    pub fn new(id: impl Into<ElementId>, trigger: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            trigger: Some(trigger.into_any_element()),
            content: None,
            content_fn: None,
            anchor: OverlayAnchor::default(),
            offset: Point::default(),
            snap_margin: DROPDOWN_SNAP_MARGIN,
            priority: 1,
            occlude: true,
        }
    }

    /// Attach the floating content body. Passing any element marks the
    /// overlay as "open"; omit this call (or guard with an `if is_open`
    /// branch on the consumer side) to leave it closed.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    /// Attach the floating content body only when `is_open` is true. This
    /// is a convenience for the common pattern of consumers tracking open
    /// state on an outer entity.
    pub fn content_when(mut self, is_open: bool, content: impl FnOnce() -> AnyElement) -> Self {
        if is_open {
            self.content = Some(content());
        }
        self
    }

    /// Attach content that depends on the realised (post-flip) anchor.
    /// The builder runs inside `prepaint` after [`AnchoredOverlay`] has
    /// picked the side of the trigger the overlay will actually render
    /// on — so arrow glyphs, directional callouts, etc. can track the
    /// realised placement rather than the preferred one.
    ///
    /// Prefer [`Self::content`] / [`Self::content_when`] when the body
    /// doesn't depend on placement (the common case).
    pub fn content_fn<F>(mut self, is_open: bool, builder: F) -> Self
    where
        F: FnOnce(OverlayAnchor) -> AnyElement + 'static,
    {
        if is_open {
            self.content_fn = Some(Box::new(builder));
        }
        self
    }

    /// Placement relative to the trigger. Defaults to [`OverlayAnchor::BelowLeft`].
    pub fn anchor(mut self, anchor: OverlayAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Pixel offset applied to the anchor point before positioning. Useful
    /// for the 4pt gap between a trigger and its menu, or for visually
    /// nudging a context-menu away from the cursor. Defaults to
    /// `Point::default()` (zero offset — the overlay sits flush against the
    /// attach corner).
    pub fn offset(mut self, offset: Point<Pixels>) -> Self {
        self.offset = offset;
        self
    }

    /// Window-edge snap margin. Defaults to [`DROPDOWN_SNAP_MARGIN`] (8pt).
    pub fn snap_margin(mut self, margin: Pixels) -> Self {
        self.snap_margin = margin;
        self
    }

    /// Deferred-draw priority. Defaults to `1` (matches Zed's `PopoverMenu`);
    /// raise for overlays that must stack above other floating surfaces.
    pub fn priority(mut self, priority: usize) -> Self {
        self.priority = priority;
        self
    }

    /// Whether the overlay wrapper should block mouse hit-testing for
    /// elements beneath it. Defaults to `true`. Pass `false` when the
    /// overlay is itself partially transparent and clicks should pass
    /// through (rare).
    pub fn occlude(mut self, occlude: bool) -> Self {
        self.occlude = occlude;
        self
    }

    /// Test-only accessor: whether the builder has captured a content
    /// element. Used to assert that `content_when(is_open, ...)` gates
    /// the content correctly without round-tripping through a full
    /// GPUI render.
    #[cfg(test)]
    fn has_content(&self) -> bool {
        self.content.is_some()
    }

    /// Test-only accessor for the configured snap margin.
    #[cfg(test)]
    fn snap_margin_px(&self) -> Pixels {
        self.snap_margin
    }

    /// Test-only accessor for the configured deferred-draw priority.
    #[cfg(test)]
    fn priority_value(&self) -> usize {
        self.priority
    }

    /// Test-only accessor for the configured occlude flag.
    #[cfg(test)]
    fn occlude_value(&self) -> bool {
        self.occlude
    }
}

/// Per-frame scratch state threaded between GPUI lifecycle phases. The
/// trigger is laid out in `request_layout`; the overlay's anchored content
/// (if any) is built and laid out lazily in `prepaint` once the trigger's
/// bounds are known.
pub struct OverlayFrameState {
    trigger_layout_id: Option<LayoutId>,
    trigger: Option<AnyElement>,
    raw_content: Option<AnyElement>,
    content_fn: Option<ContentFn>,
    anchored_content: Option<AnyElement>,
}

impl IntoElement for AnchoredOverlay {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for AnchoredOverlay {
    type RequestLayoutState = OverlayFrameState;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut trigger = self.trigger.take();
        let trigger_layout_id = trigger.as_mut().map(|el| el.request_layout(window, cx));

        // Content is deferred to `prepaint` — we need the trigger's
        // computed bounds before we can construct the anchored subtree.
        let raw_content = self.content.take();
        let content_fn = self.content_fn.take();

        let layout_id = window.request_layout(Style::default(), trigger_layout_id, cx);

        (
            layout_id,
            OverlayFrameState {
                trigger_layout_id,
                trigger,
                raw_content,
                content_fn,
                anchored_content: None,
            },
        )
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if let Some(trigger) = request_layout.trigger.as_mut() {
            trigger.prepaint(window, cx);
        }

        // Trigger layout is realised now — read its bounds before
        // building the anchored subtree so the overlay positions
        // correctly on the very first frame.
        let trigger_bounds = request_layout
            .trigger_layout_id
            .map(|id| window.layout_bounds(id));
        let window_size = window.viewport_size();
        let realised = realise_anchor(self.anchor, trigger_bounds, window_size);

        let raw_content = request_layout
            .raw_content
            .take()
            .or_else(|| request_layout.content_fn.take().map(|f| f(realised)));

        if let Some(raw_content) = raw_content {
            let mut anchored_content = build_overlay_subtree(
                raw_content,
                realised,
                self.offset,
                self.snap_margin,
                self.priority,
                self.occlude,
                trigger_bounds,
            );

            // Lay the overlay content out as an independent subtree
            // against the full window — `anchored()` repositions the
            // result to window-absolute coordinates inside its own
            // prepaint. Offering the full viewport as available space is
            // conservative; a tighter budget (e.g. trigger-relative
            // remaining space) could shave some taffy work on huge
            // viewports, but the win is negligible for current consumers.
            let available = Size {
                width: AvailableSpace::Definite(window_size.width),
                height: AvailableSpace::Definite(window_size.height),
            };
            anchored_content.layout_as_root(available, window, cx);
            anchored_content.prepaint(window, cx);

            request_layout.anchored_content = Some(anchored_content);
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if let Some(trigger) = request_layout.trigger.as_mut() {
            trigger.paint(window, cx);
        }
        if let Some(content) = request_layout.anchored_content.as_mut() {
            content.paint(window, cx);
        }
    }
}

/// Resolve the window-absolute anchor point for an overlay. Returns
/// `None` when the anchor is trigger-relative but no trigger bounds
/// have been captured yet — in that case `anchored()` falls back to
/// its laid-out origin, which is the "first-frame bootstrap" path.
fn resolve_anchor_point(
    anchor: OverlayAnchor,
    trigger_bounds: Option<Bounds<Pixels>>,
) -> Option<Point<Pixels>> {
    match anchor {
        OverlayAnchor::WindowPoint(pt) => Some(pt),
        _ => {
            let bounds = trigger_bounds?;
            let attach = anchor.attach_corner()?;
            Some(bounds.corner(attach))
        }
    }
}

/// Decide which side of the trigger the overlay should actually render on.
///
/// When the preferred anchor would put the overlay in a region with
/// materially less space than the opposite side, flip Below↔Above so
/// consumers that adapt their rendering to the realised placement (e.g.
/// a popover arrow glyph) stay in sync. The threshold (opposite side
/// must have strictly more than twice the space of the preferred side)
/// is deliberately conservative so near-equal splits don't flicker.
///
/// Returns the preferred anchor unchanged when:
/// - No trigger bounds have been captured yet.
/// - The anchor is a free-floating [`OverlayAnchor::WindowPoint`].
/// - Both sides have comparable room.
fn realise_anchor(
    preferred: OverlayAnchor,
    trigger_bounds: Option<Bounds<Pixels>>,
    window_size: Size<Pixels>,
) -> OverlayAnchor {
    let Some(trigger) = trigger_bounds else {
        return preferred;
    };
    let space_above = trigger.origin.y;
    let space_below = window_size.height - trigger.bottom();
    match preferred {
        OverlayAnchor::BelowLeft if space_above > space_below * 2.0 => OverlayAnchor::AboveLeft,
        OverlayAnchor::BelowRight if space_above > space_below * 2.0 => OverlayAnchor::AboveRight,
        OverlayAnchor::AboveLeft if space_below > space_above * 2.0 => OverlayAnchor::BelowLeft,
        OverlayAnchor::AboveRight if space_below > space_above * 2.0 => OverlayAnchor::BelowRight,
        _ => preferred,
    }
}

/// Wraps `content` in `deferred(anchored().child(occluded_content))`
/// ready for layout/prepaint/paint. Positioning falls back to the
/// anchored element's laid-out origin until `trigger_bounds` is populated
/// on the next frame.
fn build_overlay_subtree(
    content: AnyElement,
    anchor: OverlayAnchor,
    offset: Point<Pixels>,
    snap_margin: Pixels,
    priority: usize,
    occlude: bool,
    trigger_bounds: Option<Bounds<Pixels>>,
) -> AnyElement {
    let mut anchored_el = anchored()
        .snap_to_window_with_margin(snap_margin)
        .anchor(anchor.anchor_corner())
        .offset(offset);

    // `anchored()` adds our `.offset()` to this position during its own
    // prepaint, so we only pass the trigger corner here.
    if let Some(pt) = resolve_anchor_point(anchor, trigger_bounds) {
        anchored_el = anchored_el.position(pt);
    }

    let wrapper = div()
        .map(|w| if occlude { w.occlude() } else { w })
        .child(content);
    deferred(anchored_el.child(wrapper))
        .with_priority(priority)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{
        AnchoredOverlay, OverlayAnchor, build_overlay_subtree, child_id, realise_anchor,
        resolve_anchor_point,
    };
    use core::prelude::v1::test;
    use gpui::{AnyElement, Bounds, Corner, ElementId, IntoElement, Pixels, div, point, px, size};

    #[test]
    fn anchor_corner_mapping() {
        assert_eq!(OverlayAnchor::BelowLeft.anchor_corner(), Corner::TopLeft);
        assert_eq!(OverlayAnchor::BelowRight.anchor_corner(), Corner::TopRight);
        assert_eq!(OverlayAnchor::AboveLeft.anchor_corner(), Corner::BottomLeft);
        assert_eq!(
            OverlayAnchor::AboveRight.anchor_corner(),
            Corner::BottomRight
        );
        assert_eq!(
            OverlayAnchor::WindowPoint(point(px(10.0), px(20.0))).anchor_corner(),
            Corner::TopLeft,
        );
    }

    #[test]
    fn attach_corner_mapping() {
        assert_eq!(
            OverlayAnchor::BelowLeft.attach_corner(),
            Some(Corner::BottomLeft)
        );
        assert_eq!(
            OverlayAnchor::BelowRight.attach_corner(),
            Some(Corner::BottomRight)
        );
        assert_eq!(
            OverlayAnchor::AboveLeft.attach_corner(),
            Some(Corner::TopLeft)
        );
        assert_eq!(
            OverlayAnchor::AboveRight.attach_corner(),
            Some(Corner::TopRight)
        );
        assert_eq!(
            OverlayAnchor::WindowPoint(point(px(10.0), px(20.0))).attach_corner(),
            None,
        );
    }

    #[test]
    fn default_anchor_is_below_left() {
        assert_eq!(OverlayAnchor::default(), OverlayAnchor::BelowLeft);
    }

    #[test]
    fn resolve_anchor_point_returns_none_without_bounds() {
        // With no trigger bounds captured yet, trigger-relative anchors
        // have nothing to resolve against — `anchored()` falls back to
        // its laid-out origin. This covers the first-frame bootstrap.
        for anchor in [
            OverlayAnchor::BelowLeft,
            OverlayAnchor::BelowRight,
            OverlayAnchor::AboveLeft,
            OverlayAnchor::AboveRight,
        ] {
            assert_eq!(resolve_anchor_point(anchor, None), None);
        }
    }

    #[test]
    fn resolve_anchor_point_uses_window_point_even_without_bounds() {
        // `WindowPoint` carries its own absolute position and never
        // needs trigger bounds.
        let pt = point(px(100.0), px(50.0));
        assert_eq!(
            resolve_anchor_point(OverlayAnchor::WindowPoint(pt), None),
            Some(pt),
        );
    }

    #[test]
    fn resolve_anchor_point_below_left_returns_trigger_bottom_left() {
        let bounds: Bounds<Pixels> = Bounds {
            origin: point(px(10.0), px(20.0)),
            size: size(px(100.0), px(30.0)),
        };
        // bottom-left of a box at (10,20) size 100x30 is (10, 50)
        assert_eq!(
            resolve_anchor_point(OverlayAnchor::BelowLeft, Some(bounds)),
            Some(point(px(10.0), px(50.0))),
        );
    }

    #[test]
    fn resolve_anchor_point_below_right_returns_trigger_bottom_right() {
        let bounds: Bounds<Pixels> = Bounds {
            origin: point(px(10.0), px(20.0)),
            size: size(px(100.0), px(30.0)),
        };
        // bottom-right of a box at (10,20) size 100x30 is (110, 50)
        assert_eq!(
            resolve_anchor_point(OverlayAnchor::BelowRight, Some(bounds)),
            Some(point(px(110.0), px(50.0))),
        );
    }

    #[test]
    fn resolve_anchor_point_above_left_returns_trigger_top_left() {
        let bounds: Bounds<Pixels> = Bounds {
            origin: point(px(10.0), px(20.0)),
            size: size(px(100.0), px(30.0)),
        };
        assert_eq!(
            resolve_anchor_point(OverlayAnchor::AboveLeft, Some(bounds)),
            Some(point(px(10.0), px(20.0))),
        );
    }

    #[test]
    fn resolve_anchor_point_above_right_returns_trigger_top_right() {
        let bounds: Bounds<Pixels> = Bounds {
            origin: point(px(10.0), px(20.0)),
            size: size(px(100.0), px(30.0)),
        };
        // top-right of a box at (10,20) size 100x30 is (110, 20)
        assert_eq!(
            resolve_anchor_point(OverlayAnchor::AboveRight, Some(bounds)),
            Some(point(px(110.0), px(20.0))),
        );
    }

    #[test]
    fn build_overlay_subtree_emits_element_for_all_anchor_kinds() {
        // Exercising every anchor variant ensures no panic paths
        // (unwrap/expect) exist in the construction pipeline. Positioning
        // correctness is covered by `resolve_anchor_point_*` tests above.
        let bounds: Bounds<Pixels> = Bounds {
            origin: point(px(10.0), px(20.0)),
            size: size(px(100.0), px(30.0)),
        };
        let cases: [(OverlayAnchor, Option<Bounds<Pixels>>); 6] = [
            (OverlayAnchor::BelowLeft, None),
            (OverlayAnchor::BelowLeft, Some(bounds)),
            (OverlayAnchor::BelowRight, Some(bounds)),
            (OverlayAnchor::AboveLeft, Some(bounds)),
            (OverlayAnchor::AboveRight, Some(bounds)),
            (OverlayAnchor::WindowPoint(point(px(100.0), px(50.0))), None),
        ];
        for (anchor, trigger_bounds) in cases {
            let content: AnyElement = div().into_any_element();
            let el = build_overlay_subtree(
                content,
                anchor,
                point(px(0.0), px(4.0)),
                px(8.0),
                1,
                true,
                trigger_bounds,
            );
            // An empty (non-constructed) AnyElement wouldn't have a
            // non-None source location from `deferred()`; getting this
            // far without panic means the subtree was built.
            drop(el);
        }
    }

    #[test]
    fn child_id_differs_from_parent_and_suffix() {
        let parent = ElementId::Name("my-widget".into());
        let derived = child_id(&parent, "overlay");
        assert_ne!(derived, parent);
        assert_eq!(derived, ElementId::Name("my-widget-overlay".into()));
    }

    #[test]
    fn child_id_distinguishes_suffixes() {
        let parent = ElementId::Name("my-widget".into());
        assert_ne!(child_id(&parent, "overlay"), child_id(&parent, "surface"));
    }

    #[test]
    fn child_id_works_with_integer_parents() {
        // Non-Name parent ids should still format via Display without the
        // Debug-fallback branch of the old call site.
        let parent: ElementId = 42usize.into();
        let derived = child_id(&parent, "overlay");
        assert_eq!(derived, ElementId::Name("42-overlay".into()));
    }

    #[test]
    fn content_when_false_leaves_content_unset() {
        let overlay = AnchoredOverlay::new("o", div()).content_when(false, || {
            panic!("content builder should not run when is_open is false")
        });
        assert!(!overlay.has_content());
    }

    #[test]
    fn content_when_true_captures_content() {
        let overlay =
            AnchoredOverlay::new("o", div()).content_when(true, || div().into_any_element());
        assert!(overlay.has_content());
    }

    #[test]
    fn content_when_replaces_prior_content_on_true() {
        // `content_when(true, ...)` should behave like `content(...)` —
        // the closure runs and its element is stored.
        let overlay = AnchoredOverlay::new("o", div())
            .content(div())
            .content_when(true, || div().into_any_element());
        assert!(overlay.has_content());
    }

    #[test]
    fn snap_margin_builder_stores_value() {
        let overlay = AnchoredOverlay::new("o", div()).snap_margin(px(16.0));
        assert_eq!(overlay.snap_margin_px(), px(16.0));
    }

    #[test]
    fn snap_margin_default_matches_dropdown_token() {
        use crate::foundations::layout::DROPDOWN_SNAP_MARGIN;
        let overlay = AnchoredOverlay::new("o", div());
        assert_eq!(overlay.snap_margin_px(), DROPDOWN_SNAP_MARGIN);
    }

    #[test]
    fn priority_builder_stores_value() {
        let overlay = AnchoredOverlay::new("o", div()).priority(5);
        assert_eq!(overlay.priority_value(), 5);
    }

    #[test]
    fn priority_default_is_one() {
        // Matches Zed's `PopoverMenu` default (see module docs).
        let overlay = AnchoredOverlay::new("o", div());
        assert_eq!(overlay.priority_value(), 1);
    }

    #[test]
    fn occlude_builder_stores_value() {
        let overlay = AnchoredOverlay::new("o", div()).occlude(false);
        assert!(!overlay.occlude_value());
    }

    #[test]
    fn occlude_default_is_true() {
        let overlay = AnchoredOverlay::new("o", div());
        assert!(overlay.occlude_value());
    }

    #[test]
    fn realise_anchor_returns_preferred_without_bounds() {
        // No trigger bounds means no informed flip decision.
        let window = size(px(1000.0), px(1000.0));
        assert_eq!(
            realise_anchor(OverlayAnchor::BelowLeft, None, window),
            OverlayAnchor::BelowLeft,
        );
    }

    #[test]
    fn realise_anchor_keeps_window_point_unchanged() {
        let window = size(px(1000.0), px(1000.0));
        let pt = point(px(100.0), px(900.0));
        assert_eq!(
            realise_anchor(OverlayAnchor::WindowPoint(pt), None, window),
            OverlayAnchor::WindowPoint(pt),
        );
    }

    #[test]
    fn realise_anchor_flips_below_to_above_when_trigger_is_near_bottom() {
        // Trigger near the bottom: space_below is small, space_above is
        // large — flip Below→Above.
        let window = size(px(1000.0), px(1000.0));
        let trigger: Bounds<Pixels> = Bounds {
            origin: point(px(100.0), px(900.0)),
            size: size(px(80.0), px(32.0)),
        };
        // space_above = 900, space_below = 1000 - 932 = 68; ratio 13x
        assert_eq!(
            realise_anchor(OverlayAnchor::BelowLeft, Some(trigger), window),
            OverlayAnchor::AboveLeft,
        );
        assert_eq!(
            realise_anchor(OverlayAnchor::BelowRight, Some(trigger), window),
            OverlayAnchor::AboveRight,
        );
    }

    #[test]
    fn realise_anchor_flips_above_to_below_when_trigger_is_near_top() {
        let window = size(px(1000.0), px(1000.0));
        let trigger: Bounds<Pixels> = Bounds {
            origin: point(px(100.0), px(10.0)),
            size: size(px(80.0), px(32.0)),
        };
        // space_above = 10, space_below = 958; ratio 95x
        assert_eq!(
            realise_anchor(OverlayAnchor::AboveLeft, Some(trigger), window),
            OverlayAnchor::BelowLeft,
        );
        assert_eq!(
            realise_anchor(OverlayAnchor::AboveRight, Some(trigger), window),
            OverlayAnchor::BelowRight,
        );
    }

    #[test]
    fn realise_anchor_keeps_preferred_near_middle() {
        // Trigger near the middle: both sides comparable, should NOT flip.
        let window = size(px(1000.0), px(1000.0));
        let trigger: Bounds<Pixels> = Bounds {
            origin: point(px(100.0), px(450.0)),
            size: size(px(80.0), px(32.0)),
        };
        // space_above = 450, space_below = 1000 - 482 = 518; ratio ~1.15x
        // well below the 2x flip threshold.
        assert_eq!(
            realise_anchor(OverlayAnchor::BelowLeft, Some(trigger), window),
            OverlayAnchor::BelowLeft,
        );
        assert_eq!(
            realise_anchor(OverlayAnchor::AboveLeft, Some(trigger), window),
            OverlayAnchor::AboveLeft,
        );
    }

    #[test]
    fn realise_anchor_does_not_flip_at_threshold() {
        // Exactly 2x — must NOT flip (strict >).
        let window = size(px(300.0), px(300.0));
        let trigger: Bounds<Pixels> = Bounds {
            origin: point(px(0.0), px(200.0)),
            size: size(px(10.0), px(0.0)),
        };
        // space_above = 200, space_below = 100; ratio exactly 2x
        assert_eq!(
            realise_anchor(OverlayAnchor::BelowLeft, Some(trigger), window),
            OverlayAnchor::BelowLeft,
        );
    }
}
