//! Surface-scope context for `IconStyle::Auto` resolution.
//!
//! Per HIG §Foundations / Materials / Liquid Glass (`docs/hig/foundations.md`
//! lines 1034–1045): "When placing content on a Liquid Glass surface, rely
//! on the system's vibrancy effects for text and icons." Icons inherit their
//! appearance from the surface they sit on, not from a global theme mode.
//!
//! GPUI cannot composite `NSVisualEffectView` vibrancy onto an SVG icon
//! layer, so tahoe-gpui approximates the effect by selecting glass color
//! tokens (`theme.glass.icon_*`) and a thicker 1.5pt stroke when an icon is
//! known to sit on a Liquid Glass surface. This module carries that
//! "is on glass" context through the GPUI element tree via a scope guard
//! pattern, mirroring GPUI's own paint-time stacks such as
//! `Window::with_content_mask` and `Window::with_text_style`.
//!
//! Use [`GlassSurfaceScope`] around any element subtree that paints on
//! Liquid Glass chrome; nested `Icon` elements default to
//! [`crate::foundations::icons::IconStyle::LiquidGlass`] inside the scope
//! and [`crate::foundations::icons::IconStyle::Standard`] outside it.
//!
//! # Scope propagation boundaries
//!
//! The scope is maintained by a thread-local depth counter that
//! [`GlassSurfaceScopeElement`] increments around each of its child's
//! `request_layout` / `prepaint` / `paint` phases. This propagates
//! correctly through the *synchronous* element tree inside a single frame,
//! but **does not reach across**:
//!
//! - **Deferred draws** — `gpui::deferred()` and components that use it
//!   (popovers, pulldown menus, combo boxes, tooltips, `glass_morph`
//!   overlays) remove their child from the main paint tree during
//!   `prepaint` and paint it later in `Window::paint_deferred_draws`, at
//!   which point the scope guard has already dropped. Components that
//!   render a glass-surfaced deferred child should re-wrap the deferred
//!   content in its own [`GlassSurfaceScope`] or hold a
//!   [`GlassSurfaceGuard`] across the boundary.
//! - **Sub-windows** — `cx.open_window(...)` runs its own render pipeline
//!   on a different element tree; the outer window's depth counter is not
//!   observed. Wrap the sub-window's root in [`GlassSurfaceScope`] if it
//!   paints on glass chrome.
//! - **Off-thread work** — the depth counter is thread-local. GPUI's
//!   render loop is single-threaded per window on every supported
//!   platform today, but any work that migrates paint off-thread (e.g. a
//!   future background renderer) will see a fresh counter initialised to
//!   zero. [`is_on_glass_surface`] uses `try_with` so it reports `false`
//!   gracefully rather than panicking if called on a never-initialised
//!   thread.

use std::cell::Cell;

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, FocusHandle, GlobalElementId, InspectorElementId,
    IntoElement, LayoutId, Pixels, Window,
};

thread_local! {
    /// Depth counter for nested `GlassSurfaceScope`s.
    ///
    /// GPUI's rendering runs on a single OS thread per window (the platform
    /// main thread on macOS). Each `GlassSurfaceGuard::enter()` bumps the
    /// counter and `Drop` decrements it; the counter being `> 0` means some
    /// ancestor in the current render phase has declared itself a Liquid
    /// Glass surface.
    static GLASS_DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// Returns `true` if the currently-rendering element subtree sits on a
/// Liquid Glass surface (i.e. at least one active [`GlassSurfaceGuard`] is
/// higher in the call stack).
///
/// Consumed by `IconStyle::Auto::resolve` to pick the glass-vibrancy
/// approximation tokens when the icon is composited on glass. Safe to call
/// from any thread; on a thread where `GLASS_DEPTH` was never initialised,
/// returns `false` (the default "not on glass" outcome) without panicking.
#[inline]
pub fn is_on_glass_surface() -> bool {
    GLASS_DEPTH.try_with(|d| d.get() > 0).unwrap_or(false)
}

/// RAII guard that declares "my subtree sits on a Liquid Glass surface."
///
/// [`GlassSurfaceScope`] is the usual entry point — wrap it around an
/// element subtree and the guard's push/pop is handled automatically
/// across each GPUI lifecycle phase. This lower-level guard is exposed
/// for:
///
/// - Downstream components that implement their own custom `Element` and
///   need to re-establish scope manually (e.g. across a deferred-draw
///   boundary — see the "Scope propagation boundaries" section in the
///   module docs).
/// - Unit tests that want to verify scope-dependent behaviour without
///   constructing a full GPUI render frame.
///
/// The guard is intentionally `!Send` — the underlying depth counter is
/// thread-local and sharing a guard across threads would silently
/// desynchronise scope.
pub struct GlassSurfaceGuard {
    _not_send: std::marker::PhantomData<*const ()>,
}

impl GlassSurfaceGuard {
    #[inline]
    pub fn enter() -> Self {
        GLASS_DEPTH.with(|d| d.set(d.get().wrapping_add(1)));
        Self {
            _not_send: std::marker::PhantomData,
        }
    }
}

impl Drop for GlassSurfaceGuard {
    #[inline]
    fn drop(&mut self) {
        GLASS_DEPTH.with(|d| {
            let current = d.get();
            debug_assert!(
                current > 0,
                "GlassSurfaceGuard::drop called with depth == 0; \
                 enter/drop calls are unbalanced",
            );
            d.set(current.wrapping_sub(1));
        });
    }
}

/// Wrapper builder that marks its child's render subtree as sitting on a
/// Liquid Glass surface. Nested `Icon`s with `IconStyle::Auto` resolve to
/// [`crate::foundations::icons::IconStyle::LiquidGlass`] inside the
/// wrapper and [`crate::foundations::icons::IconStyle::Standard`] outside
/// it.
///
/// ```ignore
/// // Icons inside this wrapper auto-resolve to glass coloring.
/// GlassSurfaceScope::new(
///     glass_surface(div(), theme, GlassSize::Medium)
///         .child(Icon::new(IconName::Star))
/// )
/// ```
///
/// Scope propagation has limitations around deferred draws and
/// sub-windows — see the module-level documentation.
pub struct GlassSurfaceScope<E> {
    child: E,
}

impl<E: IntoElement> GlassSurfaceScope<E> {
    pub fn new(child: E) -> Self {
        Self { child }
    }
}

impl<E: IntoElement> IntoElement for GlassSurfaceScope<E> {
    type Element = GlassSurfaceScopeElement;

    fn into_element(self) -> Self::Element {
        GlassSurfaceScopeElement {
            child: self.child.into_any_element(),
        }
    }
}

/// Element produced by [`GlassSurfaceScope`]. Pushes [`GlassSurfaceGuard`]
/// around each phase of its child's lifecycle so that `Icon::render`
/// (which runs inside the child's `request_layout`) observes the active
/// scope.
pub struct GlassSurfaceScopeElement {
    child: AnyElement,
}

impl IntoElement for GlassSurfaceScopeElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for GlassSurfaceScopeElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let _guard = GlassSurfaceGuard::enter();
        let layout_id = self.child.request_layout(window, cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let _guard = GlassSurfaceGuard::enter();
        let _focus: Option<FocusHandle> = self.child.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let _guard = GlassSurfaceGuard::enter();
        self.child.paint(window, cx);
    }
}
