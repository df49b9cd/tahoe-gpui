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
//! `AnyElement::layout_as_root`) so a trigger-anchored overlay positions
//! correctly on the first frame — the trigger's bounds are realised inside
//! the same `prepaint` pass, before the overlay subtree is built. The
//! first-frame fallback (no trigger bounds yet) hands off to `anchored()`'s
//! own laid-out-origin behaviour.
//!
//! # Example
//!
//! ```ignore
//! use tahoe_gpui::foundations::overlay::{AnchoredOverlay, OverlayAnchor};
//! use gpui::{IntoElement, div, point, px};
//!
//! let is_open = true;
//! let overlay = AnchoredOverlay::new("my-overlay", div().child("Trigger"))
//!     .anchor(OverlayAnchor::BelowLeft)
//!     .offset(point(px(0.0), px(4.0)))
//!     .content_when(is_open, || div().child("Floating panel").into_any_element());
//! ```
//!
//! When the content depends on the realised (post-flip) anchor — e.g. a
//! directional arrow glyph that must flip orientation — use
//! [`AnchoredOverlay::content_fn`]:
//!
//! ```ignore
//! let overlay = AnchoredOverlay::new("my-overlay", div().child("Trigger"))
//!     .anchor(OverlayAnchor::BelowLeft)
//!     .content_fn(is_open, |realised| {
//!         let arrow = if realised.is_above() { "▲" } else { "▼" };
//!         div().child(arrow).child("Body").into_any_element()
//!     });
//! ```
//!
//! # Notes
//!
//! When both [`.content()`](AnchoredOverlay::content) and
//! [`.content_fn()`](AnchoredOverlay::content_fn) are set, `content`
//! takes precedence; `content_fn` is only invoked when `content` is
//! `None`. This mirrors the `raw_content.take().or_else(content_fn)`
//! drain order inside `prepaint`.

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Corner, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, Pixels, Point, Size, Style, Window, anchored,
    deferred, div, point, prelude::*, px,
};

use crate::foundations::layout::DROPDOWN_SNAP_MARGIN;

/// Named z-order constants for [`AnchoredOverlay::priority`] and the raw
/// GPUI [`deferred`]`(...).with_priority(n)` builder.
///
/// The values are ordinal, not spaced: each constant sits above the
/// previous one, so a new layer inserted between two existing tiers would
/// renumber the ones above it. This is by design — GPUI's deferred-draw
/// priority is a dense `usize` ordering, not a numeric scale. The
/// absolute values carry no meaning beyond their relative ordering, so
/// new consumers should prefer an existing layer over inventing a raw
/// number.
pub struct OverlayLayer;

impl OverlayLayer {
    /// Dropdown menus, list pickers, combo-box popups. The base layer
    /// for trigger-anchored overlay surfaces and the default for
    /// [`AnchoredOverlay`].
    pub const DROPDOWN: usize = 1;
    /// Morphing glass surface transitions (see
    /// [`glass_morph`](crate::foundations::materials::glass_morph)).
    /// Stacks one above dropdowns so an in-flight morph doesn't get
    /// stamped under a simultaneously-opening dropdown.
    pub const GLASS_MORPH: usize = 2;
    /// Tooltip surfaces. Stacks above popovers so a tooltip over a
    /// popover's control renders on top.
    pub const TOOLTIP: usize = 10;
    /// Context menus (right-click / control-click) and `WindowPoint`
    /// popovers. Stacks above tooltips so the menu always reads as the
    /// active input surface once summoned.
    pub const CONTEXT_MENU: usize = 20;
}

/// Where an overlay attaches relative to its trigger.
///
/// Each trigger-relative variant encodes two corners: which corner of the
/// trigger the overlay latches onto (the "attach" corner) and which corner
/// of the overlay is placed at that point (the "anchor" corner).
///
/// # Choosing a variant
///
/// - **`BelowLeft` / `BelowRight`** — dropdowns, auto-complete menus,
///   popovers below a toolbar button. The default for anything triggered
///   by a control with more room below than above.
/// - **`AboveLeft` / `AboveRight`** — tooltip-style popovers attached to a
///   trigger near the bottom of a pane. The overlay flips a
///   preferred `Below*` to `Above*` (and vice-versa) automatically when
///   the opposite side has materially more room, so most callers can
///   leave the preferred side set to whatever reads best in the common
///   case.
/// - **`WindowPoint`** — context menus and popovers summoned at the
///   pointer position. See the variant's own docs for the coordinate
///   space.
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
    ///
    /// The coordinate is viewport-relative — (0, 0) is the window's
    /// top-left corner, matching GPUI's `Window::viewport_size`. The
    /// overlay's top-left is placed at this point (modulo offset);
    /// the overlay does not flip `WindowPoint` overlays, and
    /// [`AnchoredOverlay::gap`] is a no-op because there is no trigger
    /// to clear.
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

/// Content builder that receives the realised anchor (after any
/// overflow-driven flip) so consumers whose rendering depends on
/// placement — such as a popover arrow glyph — can stay in sync with
/// the actual position.
type ContentFn = Box<dyn FnOnce(OverlayAnchor) -> AnyElement>;

/// Anchored, deferred, optionally occluding overlay.
///
/// The trigger element is always laid out in the normal tree. The content
/// element, when present, is built in `prepaint` (after the trigger's
/// bounds are realised by taffy in the same pass) and wrapped in a
/// `deferred(anchored(...))` subtree so it paints after the rest of the
/// frame at window-absolute coordinates.
///
/// The common case is first-frame correct: `prepaint` reads the trigger's
/// freshly-realised bounds before building the anchored subtree. The edge
/// case where `trigger_bounds` isn't available yet (e.g. window just
/// opened, taffy hasn't measured the trigger) falls back to
/// `anchored()`'s own laid-out-origin behaviour — the overlay appears at
/// its natural layout position until the next frame resolves bounds.
pub struct AnchoredOverlay {
    id: ElementId,
    trigger: Option<AnyElement>,
    content: Option<AnyElement>,
    content_fn: Option<ContentFn>,
    anchor: OverlayAnchor,
    offset: Point<Pixels>,
    gap: Option<Pixels>,
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
            gap: None,
            snap_margin: DROPDOWN_SNAP_MARGIN,
            priority: OverlayLayer::DROPDOWN,
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
    ///
    /// # Flipping
    ///
    /// `prepaint` may flip the preferred side when the opposite side has
    /// strictly more than twice the available space. Callers that need
    /// the actual side the overlay landed on (e.g. to orient an arrow
    /// glyph) should use [`Self::content_fn`], which receives the
    /// realised anchor. See [`OverlayAnchor`] docs for the flip threshold.
    pub fn anchor(mut self, anchor: OverlayAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Pixel offset applied to the anchor point before positioning. Free-form
    /// — the sign is honoured as given, independent of the realised placement.
    /// Use this for a fixed nudge (e.g. inset from a context-menu cursor
    /// position). For the canonical "gap between trigger and overlay" use
    /// [`Self::gap`] instead so the sign tracks the realised side after
    /// any overflow-driven flip. Defaults to `Point::default()`.
    pub fn offset(mut self, offset: Point<Pixels>) -> Self {
        self.offset = offset;
        self
    }

    /// Vertical gap (in pixels) between the trigger edge and the overlay.
    /// The sign is resolved in `prepaint` against the *realised* anchor:
    /// `Below*` placements shift the overlay downward by `magnitude`;
    /// `Above*` placements shift it upward. An overflow-driven Below↔Above
    /// flip therefore preserves the gap on whichever side the overlay
    /// actually lands on — callers don't need to re-sign the offset
    /// themselves. Ignored for [`OverlayAnchor::WindowPoint`]. Composes
    /// additively with [`Self::offset`] when both are set.
    pub fn gap(mut self, magnitude: Pixels) -> Self {
        self.gap = Some(magnitude);
        self
    }

    /// Window-edge snap margin. Defaults to [`DROPDOWN_SNAP_MARGIN`] (8pt).
    ///
    /// The configured value is an *upper bound*: on compact viewports the
    /// overlay clamps the effective margin against the viewport's shorter
    /// side so a wide margin plus a narrow window doesn't shove the
    /// overlay on top of its trigger.
    pub fn snap_margin(mut self, margin: Pixels) -> Self {
        self.snap_margin = margin;
        self
    }

    /// Deferred-draw priority. Defaults to [`OverlayLayer::DROPDOWN`]
    /// (matches Zed's `PopoverMenu`); pass one of the other
    /// [`OverlayLayer`] constants (or a raw `usize` for bespoke stacks)
    /// to place the overlay above other floating surfaces.
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

    /// Test-only mirror of [`Self::has_content`] for the flip-aware
    /// `content_fn` path. Lets tests assert that `content_fn(is_open, ...)`
    /// gates the builder closure correctly (and that it doesn't run
    /// eagerly when `is_open` is false).
    #[cfg(test)]
    fn has_content_fn(&self) -> bool {
        self.content_fn.is_some()
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

    /// Test-only accessor for the configured gap magnitude.
    #[cfg(test)]
    fn gap_value(&self) -> Option<Pixels> {
        self.gap
    }

    /// Test-only shim that replicates the anchor-resolution + content
    /// precedence path inside `prepaint`, without running the full GPUI
    /// element lifecycle. Returns the realised anchor so tests can assert
    /// both `realise_anchor`'s decision and the `content_fn` contract
    /// (the closure is invoked with the realised anchor, not the preferred
    /// one).
    ///
    /// Mirrors prepaint's precedence: if `content` is set it's drained
    /// first and `content_fn` is left untouched; otherwise the
    /// `content_fn` builder is consumed and invoked with the realised
    /// anchor. Matches the `raw_content.take().or_else(...)` shape in
    /// `prepaint`.
    #[cfg(test)]
    fn simulate_resolve(
        &mut self,
        trigger_bounds: Option<Bounds<Pixels>>,
        window_size: Size<Pixels>,
    ) -> OverlayAnchor {
        let realised = realise_anchor(self.anchor, trigger_bounds, window_size);
        if self.content.take().is_none()
            && let Some(builder) = self.content_fn.take()
        {
            let _ = builder(realised);
        }
        realised
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
            let effective_offset = apply_gap_to_offset(self.offset, self.gap, realised);
            let effective_snap_margin = clamp_snap_margin(self.snap_margin, window_size);
            let mut anchored_content = build_overlay_subtree(
                raw_content,
                realised,
                effective_offset,
                effective_snap_margin,
                self.priority,
                self.occlude,
                trigger_bounds,
            );

            // Use `MinContent` so the overlay sizes to its intrinsic
            // content rather than filling the viewport. `anchored()`
            // repositions the result to window-absolute coordinates inside
            // its own prepaint.
            let available = Size {
                width: AvailableSpace::MinContent,
                height: AvailableSpace::MinContent,
            };
            // TODO(perf): cache the layout pass keyed by
            // `(trigger_bounds, content_id)` so repeated frames with
            // identical inputs skip the taffy traversal.
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
/// Flips Below↔Above when the opposite side has strictly more than twice
/// the space of the preferred side — i.e. `space_opposite > space_preferred * 2.0`.
/// The `> 2.0` (not `>= 2.0`) is deliberate: a trigger sitting exactly on
/// the 2:1 split stays on the preferred side so small viewport resizes
/// don't flicker the overlay across the trigger.
///
/// Space is measured as the pixel distance from the trigger's edge to
/// the viewport edge on each side: `trigger.origin.y` for above, and
/// `window.height - trigger.bottom()` for below. The horizontal suffix
/// (`Left`/`Right`) is preserved across a vertical flip so a
/// `BelowLeft` → `AboveLeft` realisation keeps the overlay pinned to
/// the trigger's left edge.
///
/// Returns the preferred anchor unchanged when:
/// - No trigger bounds have been captured yet (first-frame bootstrap).
/// - The anchor is a free-floating [`OverlayAnchor::WindowPoint`] —
///   these carry their own window-absolute point and never flip.
/// - Both sides have comparable room (ratio ≤ 2:1 either way).
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

/// Resolve the effective offset for an overlay, folding in the flip-aware
/// gap set via [`AnchoredOverlay::gap`] against the realised anchor.
///
/// The y-axis points downward in GPUI coordinates, so Below* placements
/// shift the overlay *down* (positive) to clear the trigger, and Above*
/// placements shift *up* (negative). Using the realised anchor here —
/// rather than the preferred one the caller set — means the gap lands on
/// the correct side even after [`realise_anchor`] flips Below↔Above.
/// [`OverlayAnchor::WindowPoint`] has no trigger to clear, so the gap is
/// ignored for it.
fn apply_gap_to_offset(
    base: Point<Pixels>,
    gap: Option<Pixels>,
    realised: OverlayAnchor,
) -> Point<Pixels> {
    let Some(magnitude) = gap else {
        return base;
    };
    let signed = match realised {
        OverlayAnchor::BelowLeft | OverlayAnchor::BelowRight => magnitude,
        OverlayAnchor::AboveLeft | OverlayAnchor::AboveRight => -magnitude,
        OverlayAnchor::WindowPoint(_) => return base,
    };
    point(base.x, base.y + signed)
}

/// Clamp the configured snap margin against the viewport so a compact
/// window (Stage Manager tile, narrow inspector, 1/4-width split view)
/// doesn't eat its own interior.
///
/// The rule: the snap margin may consume at most ~3% of the viewport's
/// shorter side. On a 1200 pt desktop window that lets the full 8 pt
/// default through; on a 200 pt compact viewport it floors to ~6 pt, and
/// on a 40 pt sliver it floors to ~1 pt. The upper bound is the caller's
/// configured `configured`, so passing a large value (say 16 pt) still
/// honours that on wide viewports but degrades gracefully on narrow ones.
fn clamp_snap_margin(configured: Pixels, window_size: Size<Pixels>) -> Pixels {
    let shorter = window_size.width.min(window_size.height);
    // `0.03` keeps the default 8 pt intact above a ~267 pt viewport,
    // which is well below anything macOS or iPadOS would render at.
    let ceiling = shorter * 0.03;
    if ceiling <= px(0.0) {
        return configured;
    }
    if configured < ceiling {
        configured
    } else {
        ceiling
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
        AnchoredOverlay, OverlayAnchor, apply_gap_to_offset, build_overlay_subtree,
        clamp_snap_margin, realise_anchor, resolve_anchor_point,
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
    fn build_overlay_subtree_wraps_content_in_deferred_for_all_anchor_kinds() {
        // `build_overlay_subtree` must wrap the content in a `deferred()`
        // element so the overlay paints after its ancestors and escapes
        // parent `overflow_hidden()` clipping — the whole point of the
        // primitive. Asserting the outer element downcasts to `Deferred`
        // for every anchor variant proves the wrapper was actually
        // applied (and that no variant silently returns the inner
        // content inline). Positioning correctness is covered by
        // `resolve_anchor_point_*` above; this test guards the shape of
        // the construction pipeline.
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
            let mut el = build_overlay_subtree(
                content,
                anchor,
                point(px(0.0), px(4.0)),
                px(8.0),
                1,
                true,
                trigger_bounds,
            );
            assert!(
                el.downcast_mut::<gpui::Deferred>().is_some(),
                "outer element for {:?} should be a gpui::Deferred wrapper",
                anchor,
            );
        }
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
    fn content_fn_when_false_leaves_builder_unset() {
        let overlay = AnchoredOverlay::new("o", div()).content_fn(false, |_realised| {
            panic!("content_fn builder should not be stored when is_open is false")
        });
        assert!(!overlay.has_content_fn());
        assert!(!overlay.has_content());
    }

    #[test]
    fn content_fn_when_true_captures_builder() {
        let overlay =
            AnchoredOverlay::new("o", div()).content_fn(true, |_realised| div().into_any_element());
        assert!(overlay.has_content_fn());
        // `content_fn` is a separate channel from `content`; it must not
        // bleed into the plain-content slot.
        assert!(!overlay.has_content());
    }

    #[test]
    fn content_has_precedence_over_content_fn_in_prepaint() {
        // Both channels set — prepaint's `raw_content.take().or_else(...)`
        // drains `content` and skips `content_fn`. A panicking closure in
        // `content_fn` proves the builder was not invoked. `content_fn`
        // remains captured on the overlay afterwards — prepaint only
        // consumes the branch it actually runs.
        let window = size(px(1000.0), px(1000.0));
        let trigger: Bounds<Pixels> = Bounds {
            origin: point(px(100.0), px(450.0)),
            size: size(px(80.0), px(32.0)),
        };

        let mut overlay = AnchoredOverlay::new(ElementId::Name("both".into()), div())
            .content(div())
            .content_fn(true, |_realised| {
                panic!("content_fn should not run when content() is also set")
            });
        assert!(overlay.has_content());
        assert!(overlay.has_content_fn());

        overlay.simulate_resolve(Some(trigger), window);

        // `content` drained; `content_fn` left for a subsequent frame
        // (matches the real prepaint's branch-specific `take`).
        assert!(!overlay.has_content());
        assert!(overlay.has_content_fn());
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
    fn realise_anchor_does_not_flip_below_at_threshold() {
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

    #[test]
    fn realise_anchor_does_not_flip_above_at_threshold() {
        // Symmetric to the Below case: a preferred Above at exactly 2x
        // in favour of below must also stay on the preferred side. This
        // guards against the flip condition being asymmetric between
        // Above→Below and Below→Above.
        let window = size(px(300.0), px(300.0));
        let trigger: Bounds<Pixels> = Bounds {
            origin: point(px(0.0), px(100.0)),
            size: size(px(10.0), px(0.0)),
        };
        // space_above = 100, space_below = 200; ratio exactly 2x
        assert_eq!(
            realise_anchor(OverlayAnchor::AboveLeft, Some(trigger), window),
            OverlayAnchor::AboveLeft,
        );
    }

    #[test]
    fn realise_anchor_flips_below_just_over_threshold() {
        // Just over 2x — must flip. Uses a 1pt nudge of the trigger
        // (space_above = 201, space_below = 99; ratio ≈ 2.03x).
        let window = size(px(300.0), px(300.0));
        let trigger: Bounds<Pixels> = Bounds {
            origin: point(px(0.0), px(201.0)),
            size: size(px(10.0), px(0.0)),
        };
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
    fn realise_anchor_flips_above_just_over_threshold() {
        // Mirror case: trigger just above the 2x split in favour of below.
        let window = size(px(300.0), px(300.0));
        let trigger: Bounds<Pixels> = Bounds {
            origin: point(px(0.0), px(99.0)),
            size: size(px(10.0), px(0.0)),
        };
        // space_above = 99, space_below = 201; ratio ≈ 2.03x
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
    fn apply_gap_signs_positive_for_below_negative_for_above() {
        let base = point(px(0.0), px(0.0));
        let gap = px(4.0);

        let below_left = apply_gap_to_offset(base, Some(gap), OverlayAnchor::BelowLeft);
        assert_eq!(below_left, point(px(0.0), px(4.0)));

        let below_right = apply_gap_to_offset(base, Some(gap), OverlayAnchor::BelowRight);
        assert_eq!(below_right, point(px(0.0), px(4.0)));

        let above_left = apply_gap_to_offset(base, Some(gap), OverlayAnchor::AboveLeft);
        assert_eq!(above_left, point(px(0.0), px(-4.0)));

        let above_right = apply_gap_to_offset(base, Some(gap), OverlayAnchor::AboveRight);
        assert_eq!(above_right, point(px(0.0), px(-4.0)));
    }

    #[test]
    fn apply_gap_flips_sign_when_realised_anchor_flips() {
        // The bug the gap API exists to prevent: caller's preferred
        // placement decides a sign, the primitive flips Below↔Above, and
        // the old offset-based approach kept the pre-flip sign — producing
        // an overlap with the trigger instead of a gap. `.gap()` resolves
        // against the realised anchor, so flipping preserves the gap on
        // the correct side.
        let base = point(px(0.0), px(0.0));
        let gap = px(4.0);

        // Preferred Below, realised Above (flipped because trigger is near
        // the bottom of the viewport): the old code kept +4 (wrong side);
        // the gap API emits -4, pushing the overlay up and away from the
        // trigger.
        let flipped_to_above = apply_gap_to_offset(base, Some(gap), OverlayAnchor::AboveLeft);
        assert_eq!(flipped_to_above, point(px(0.0), px(-4.0)));

        // Preferred Above, realised Below (flipped because trigger is near
        // the top): old code kept -4 (wrong side); gap API emits +4.
        let flipped_to_below = apply_gap_to_offset(base, Some(gap), OverlayAnchor::BelowLeft);
        assert_eq!(flipped_to_below, point(px(0.0), px(4.0)));
    }

    #[test]
    fn apply_gap_returns_base_when_no_gap_set() {
        let base = point(px(3.0), px(7.0));
        let out = apply_gap_to_offset(base, None, OverlayAnchor::BelowLeft);
        assert_eq!(out, base);
    }

    #[test]
    fn apply_gap_is_additive_with_base_offset() {
        let base = point(px(1.0), px(2.0));
        let gap = px(4.0);

        let below = apply_gap_to_offset(base, Some(gap), OverlayAnchor::BelowRight);
        assert_eq!(below, point(px(1.0), px(6.0)));

        let above = apply_gap_to_offset(base, Some(gap), OverlayAnchor::AboveRight);
        assert_eq!(above, point(px(1.0), px(-2.0)));
    }

    #[test]
    fn apply_gap_is_ignored_for_window_point_anchor() {
        let base = point(px(10.0), px(20.0));
        let out = apply_gap_to_offset(
            base,
            Some(px(4.0)),
            OverlayAnchor::WindowPoint(point(px(0.0), px(0.0))),
        );
        assert_eq!(out, base);
    }

    #[test]
    fn gap_builder_stores_magnitude() {
        let overlay = AnchoredOverlay::new(ElementId::Name("o".into()), div()).gap(px(6.0));
        assert_eq!(overlay.gap_value(), Some(px(6.0)));
    }

    #[test]
    fn content_fn_receives_realised_anchor_after_flip() {
        // Simulate the prepaint path: preferred BelowLeft with the trigger
        // pinned near the bottom edge must flip to AboveLeft, and the
        // content_fn closure must observe the post-flip anchor so consumers
        // (e.g. popover arrow glyphs) can track the realised side.
        use std::cell::Cell;
        use std::rc::Rc;

        let received: Rc<Cell<Option<OverlayAnchor>>> = Rc::new(Cell::new(None));
        let received_clone = received.clone();

        let mut overlay = AnchoredOverlay::new(ElementId::Name("flip-probe".into()), div())
            .anchor(OverlayAnchor::BelowLeft)
            .content_fn(true, move |realised| {
                received_clone.set(Some(realised));
                div().into_any_element()
            });

        let window = size(px(1000.0), px(1000.0));
        let trigger: Bounds<Pixels> = Bounds {
            origin: point(px(100.0), px(900.0)),
            size: size(px(80.0), px(32.0)),
        };
        let realised = overlay.simulate_resolve(Some(trigger), window);

        assert_eq!(realised, OverlayAnchor::AboveLeft);
        assert_eq!(received.get(), Some(OverlayAnchor::AboveLeft));
    }

    #[test]
    fn content_fn_receives_preferred_anchor_when_no_flip() {
        use std::cell::Cell;
        use std::rc::Rc;

        let received: Rc<Cell<Option<OverlayAnchor>>> = Rc::new(Cell::new(None));
        let received_clone = received.clone();

        let mut overlay = AnchoredOverlay::new(ElementId::Name("no-flip".into()), div())
            .anchor(OverlayAnchor::BelowLeft)
            .content_fn(true, move |realised| {
                received_clone.set(Some(realised));
                div().into_any_element()
            });

        // Trigger mid-viewport: both sides have comparable room, no flip.
        let window = size(px(1000.0), px(1000.0));
        let trigger: Bounds<Pixels> = Bounds {
            origin: point(px(100.0), px(450.0)),
            size: size(px(80.0), px(32.0)),
        };
        overlay.simulate_resolve(Some(trigger), window);

        assert_eq!(received.get(), Some(OverlayAnchor::BelowLeft));
    }

    #[test]
    fn flip_handles_trigger_extending_past_viewport_bottom() {
        // Resize-during-animation path: the viewport shrank mid-frame so
        // the trigger's bottom is below the new viewport height, which
        // makes `space_below = window.height - trigger.bottom()` negative.
        // The `> space_preferred * 2.0` comparison must not silently flip
        // against a negative number (which would always satisfy the check
        // for preferred Below placements).
        let window = size(px(100.0), px(100.0));
        let trigger: Bounds<Pixels> = Bounds {
            origin: point(px(0.0), px(50.0)),
            size: size(px(20.0), px(150.0)),
        };
        // space_above = 50, space_below = -100. Preferred BelowLeft must
        // flip to AboveLeft because there is strictly no room below.
        assert_eq!(
            realise_anchor(OverlayAnchor::BelowLeft, Some(trigger), window),
            OverlayAnchor::AboveLeft,
        );
        // Preferred AboveLeft must stay: negative space_below cannot beat
        // a positive space_above * 2.0.
        assert_eq!(
            realise_anchor(OverlayAnchor::AboveLeft, Some(trigger), window),
            OverlayAnchor::AboveLeft,
        );
    }

    #[test]
    fn flip_handles_trigger_with_negative_origin_y() {
        // Scrolled-above-viewport path: the trigger sits above the
        // viewport's top edge (origin.y < 0), so `space_above =
        // trigger.origin.y` is negative. Preferred AboveLeft must flip
        // because there is no room above.
        let window = size(px(100.0), px(100.0));
        let trigger: Bounds<Pixels> = Bounds {
            origin: point(px(0.0), px(-20.0)),
            size: size(px(20.0), px(15.0)),
        };
        // space_above = -20, space_below = 100 - (-20 + 15) = 105. Flip.
        assert_eq!(
            realise_anchor(OverlayAnchor::AboveLeft, Some(trigger), window),
            OverlayAnchor::BelowLeft,
        );
    }

    #[test]
    fn flip_noop_for_zero_sized_trigger() {
        // Degenerate zero-size trigger (possible during first-frame
        // bootstrap when layout hasn't run). `space_above + space_below`
        // should cover the whole viewport; neither side should dominate
        // by more than 2:1 unless the trigger is far from centre.
        let window = size(px(100.0), px(100.0));
        let trigger: Bounds<Pixels> = Bounds {
            origin: point(px(50.0), px(50.0)),
            size: size(px(0.0), px(0.0)),
        };
        // space_above = 50, space_below = 50. 2:1 check fails (ratio 1:1),
        // so the preferred placement survives.
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
    fn clamp_snap_margin_passes_through_on_wide_viewport() {
        // Default 8 pt margin on a 1200 pt window: shorter-side 0.03 ceiling
        // is 36 pt, so the 8 pt configured value is returned unchanged.
        let configured = px(8.0);
        let window = size(px(1200.0), px(800.0));
        assert_eq!(clamp_snap_margin(configured, window), configured);
    }

    #[test]
    fn clamp_snap_margin_floors_on_compact_viewport() {
        // 8 pt margin on a 200 pt-wide compact viewport: 3% of 200 = 6 pt,
        // which is under the configured 8 pt — so the margin clamps to 6 pt.
        let configured = px(8.0);
        let window = size(px(200.0), px(800.0));
        assert_eq!(clamp_snap_margin(configured, window), px(6.0));
    }

    #[test]
    fn clamp_snap_margin_preserves_configured_on_degenerate_viewport() {
        // Zero-size viewport (shouldn't happen in practice, but defensive
        // against first-frame-before-layout). The ceiling is 0; rather
        // than collapsing to 0 we fall through to the configured value —
        // `anchored()` will snap against whatever viewport it finds at
        // paint time.
        let configured = px(8.0);
        let window = size(px(0.0), px(0.0));
        assert_eq!(clamp_snap_margin(configured, window), configured);
    }

    #[test]
    fn clamp_snap_margin_at_exact_ceiling() {
        // 300 pt viewport: shorter side 300, ceiling = 300 * 0.03 = 9.0.
        // Configured == ceiling must return the ceiling (the `else` branch
        // where configured >= ceiling).
        let configured = px(9.0);
        let window = size(px(300.0), px(300.0));
        assert_eq!(clamp_snap_margin(configured, window), px(9.0));
    }
}
