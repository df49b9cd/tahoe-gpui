//! Sheet component (HIG `#sheets`).
//!
//! Distinct from [`Modal`](super::modal::Modal). The rendered chrome
//! varies by platform:
//!
//! - **iOS / iPadOS / watchOS** — bottom-anchored drawer that slides up
//!   from the bottom edge with a drag indicator pill and detent
//!   heights (medium / large / custom fraction).
//! - **macOS / visionOS** — cardlike centered panel that floats on
//!   top of the parent window. HIG `#sheets` macOS: "a sheet is a
//!   cardlike view with rounded corners that floats on top of its
//!   parent window." No drag indicator and no bottom anchoring; the
//!   parent window dims behind the sheet.
//!
//! The variant is selected automatically from [`TahoeTheme::platform`]
//! or explicitly via [`Sheet::presentation`].
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/sheets>

use std::time::Duration;

use gpui::prelude::*;
use gpui::{
    Animation, AnimationExt, AnyElement, App, ElementId, FocusHandle, KeyDownEvent, MouseDownEvent,
    Pixels, SharedString, Window, div, px,
};

use crate::callback_types::{OnMutCallback, rc_wrap};
use crate::foundations::layout::Platform;
use crate::foundations::materials::{backdrop_overlay, glass_surface};
use crate::foundations::motion::REDUCE_MOTION_CROSSFADE;
use crate::foundations::theme::{ActiveTheme, GlassSize, TahoeTheme};

// ── Constants ───────────────────────────────────────────────────────────────

/// Drag indicator pill width (pt).
const DRAG_INDICATOR_WIDTH: f32 = 36.0;
/// Drag indicator pill height (pt).
const DRAG_INDICATOR_HEIGHT: f32 = 5.0;
/// Vertical padding above the drag indicator.
const DRAG_INDICATOR_TOP_PADDING: f32 = 8.0;
/// Vertical padding below the drag indicator.
const DRAG_INDICATOR_BOTTOM_PADDING: f32 = 4.0;

/// macOS cardlike sheet default width (pt).
const MACOS_SHEET_WIDTH: f32 = 480.0;
/// macOS cardlike sheet maximum height as fraction of viewport.
const MACOS_SHEET_MAX_HEIGHT_FRACTION: f32 = 0.82;

/// Shared dismiss callback wrapped in `Rc<Box<..>>` so a single boxed
/// closure can be cloned into several handler sites (mouse-down-out,
/// Escape) without reboxing.
type DismissRc = std::rc::Rc<Box<dyn Fn(&mut Window, &mut App) + 'static>>;

/// Sheet detent — controls how much of the screen the sheet covers on
/// iOS/iPadOS. macOS sheets always use `MACOS_SHEET_WIDTH` and
/// auto-sized height; detents are only consulted in
/// [`SheetPresentation::BottomDrawer`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SheetDetent {
    /// Half the available height.
    Medium,
    /// Near-full height (leaving a small top margin).
    #[default]
    Large,
    /// Explicit height as a fraction of the viewport. Values are
    /// clamped to `(0.0, 1.0]`. HIG iOS 16+: sheets support custom
    /// fractional heights for form-size presentations.
    Custom(f32),
}

impl SheetDetent {
    fn height_fraction(self) -> f32 {
        match self {
            Self::Medium => 0.50,
            Self::Large => 0.92,
            Self::Custom(f) => f.clamp(0.05, 1.0),
        }
    }
}

/// Where the sheet renders on screen. Auto-selected from
/// [`TahoeTheme::platform`] unless [`Sheet::presentation`] is called.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SheetPresentation {
    /// iOS bottom-anchored drawer with drag indicator and detent.
    #[default]
    BottomDrawer,
    /// macOS cardlike centered panel — no drag indicator, no bottom
    /// anchoring. HIG `#sheets` macOS wording.
    Cardlike,
}

impl SheetPresentation {
    /// Default presentation for a given platform per HIG.
    pub fn for_platform(platform: Platform) -> Self {
        match platform {
            Platform::MacOS | Platform::VisionOS | Platform::TvOS => Self::Cardlike,
            Platform::IOS | Platform::WatchOS => Self::BottomDrawer,
        }
    }
}

/// A sheet overlay following Human Interface Guidelines.
///
/// The sheet renders a semi-transparent backdrop and applies Liquid
/// Glass styling. On iOS it anchors to the bottom and exposes a drag
/// indicator; on macOS it renders as a cardlike centered panel.
///
/// # Example
///
/// ```ignore
/// Sheet::new("my-sheet", div().child("Hello"))
///     .open(true)
///     .detent(SheetDetent::Medium) // iOS only
///     .on_dismiss(|_window, _cx| { /* close state */ })
/// ```
#[derive(IntoElement)]
pub struct Sheet {
    id: SharedString,
    is_open: bool,
    detent: SheetDetent,
    content: AnyElement,
    on_dismiss: OnMutCallback,
    focus_handle: Option<FocusHandle>,
    presentation: Option<SheetPresentation>,
    width: Option<Pixels>,
}

impl Sheet {
    /// Create a new sheet with the given id and content element.
    pub fn new(id: impl Into<SharedString>, content: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            is_open: false,
            detent: SheetDetent::default(),
            content: content.into_any_element(),
            on_dismiss: None,
            focus_handle: None,
            presentation: None,
            width: None,
        }
    }

    /// Control visibility. When `false` the sheet renders nothing.
    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    /// Set the height detent (iOS / iPadOS / watchOS only).
    pub fn detent(mut self, detent: SheetDetent) -> Self {
        self.detent = detent;
        self
    }

    /// Called when the user taps the backdrop or presses Escape.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }

    /// Override the focus handle tracked by the sheet panel.
    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
        self
    }

    /// Force a specific presentation chrome regardless of platform.
    pub fn presentation(mut self, presentation: SheetPresentation) -> Self {
        self.presentation = Some(presentation);
        self
    }

    /// Override the cardlike sheet width (macOS). Ignored on iOS which
    /// always spans the full viewport width.
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// Convenience: pass a child element (equivalent to providing content in `new`).
    pub fn child(self, child: impl IntoElement) -> Self {
        Self {
            content: child.into_any_element(),
            ..self
        }
    }
}

impl RenderOnce for Sheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.is_open {
            return div().into_any_element();
        }

        // Resolve presentation first (pure read of TahoeTheme) so the
        // subsequent `cx.focus_handle()` mutable-borrow doesn't clash.
        let presentation = {
            let theme = cx.theme();
            self.presentation
                .unwrap_or_else(|| SheetPresentation::for_platform(theme.platform))
        };

        // Auto-focus the panel on open so Escape / Tab reach the key handler
        // without a prior click. Mirrors Modal & Alert.
        let focus_handle = self.focus_handle.unwrap_or_else(|| cx.focus_handle());
        if !focus_handle.is_focused(window) {
            focus_handle.focus(window, cx);
        }

        let theme = cx.theme();
        let on_dismiss_rc = rc_wrap(self.on_dismiss);

        match presentation {
            SheetPresentation::BottomDrawer => render_bottom_drawer(
                self.id,
                self.detent,
                self.content,
                theme,
                focus_handle,
                on_dismiss_rc,
            ),
            SheetPresentation::Cardlike => render_cardlike(
                self.id,
                self.content,
                self.width.unwrap_or(px(MACOS_SHEET_WIDTH)),
                theme,
                focus_handle,
                on_dismiss_rc,
            ),
        }
    }
}

/// iOS/iPadOS bottom-drawer rendering.
fn render_bottom_drawer(
    id: SharedString,
    detent: SheetDetent,
    content: AnyElement,
    theme: &TahoeTheme,
    focus_handle: FocusHandle,
    on_dismiss_rc: Option<DismissRc>,
) -> gpui::AnyElement {
    // ── Backdrop ────────────────────────────────────────────────────────
    let backdrop = backdrop_overlay(theme).flex().flex_col().justify_end();

    // ── Sheet height based on detent ────────────────────────────────────
    let height_frac = detent.height_fraction();

    // ── Drag indicator pill (iOS only) ──────────────────────────────────
    let indicator_color = theme.text_quaternary();
    let drag_indicator = div()
        .flex()
        .justify_center()
        .pt(px(DRAG_INDICATOR_TOP_PADDING))
        .pb(px(DRAG_INDICATOR_BOTTOM_PADDING))
        .child(
            div()
                .w(px(DRAG_INDICATOR_WIDTH))
                .h(px(DRAG_INDICATOR_HEIGHT))
                .rounded(px(DRAG_INDICATOR_HEIGHT / 2.0))
                .bg(indicator_color),
        );

    // ── Scrollable content area (animated) ──────────────────────────────
    let reduce_motion = theme.accessibility_mode.reduce_motion();
    let (anim_duration, slide_offset_pt) = if reduce_motion {
        (REDUCE_MOTION_CROSSFADE, 0.0)
    } else {
        (
            Duration::from_millis(theme.glass.motion.shape_shift_duration_ms),
            32.0,
        )
    };
    let anim_id = ElementId::Name(format!("{}-present", id).into());
    let scroll_id = ElementId::Name(format!("{}-scroll", id).into());
    let scroll_body = div()
        .id(scroll_id)
        .flex_1()
        .overflow_y_scroll()
        .px(theme.spacing_lg)
        .pb(theme.spacing_lg)
        .child(content)
        .with_animation(anim_id, Animation::new(anim_duration), move |el, delta| {
            let offset = slide_offset_pt * (1.0 - delta);
            el.opacity(delta).mt(px(offset))
        });

    // ── Sheet panel (glass surface) ─────────────────────────────────────
    let top_radius = theme.glass.radius(GlassSize::Large);
    let panel_id = ElementId::Name(format!("{}-panel", id).into());
    let mut panel = glass_surface(div().w_full().overflow_hidden(), theme, GlassSize::Large)
        .rounded_t(top_radius)
        .rounded_b(px(0.0))
        .id(panel_id)
        .track_focus(&focus_handle)
        .flex()
        .flex_col()
        .w_full()
        .h(gpui::relative(height_frac))
        .child(drag_indicator)
        .child(scroll_body);

    if let Some(ref handler) = on_dismiss_rc {
        let h = handler.clone();
        panel = panel.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
            h(window, cx);
        });
    }

    if let Some(ref handler) = on_dismiss_rc {
        let h = handler.clone();
        panel = panel.on_key_down(move |event: &KeyDownEvent, window, cx| {
            if crate::foundations::keyboard::is_escape_key(event) {
                h(window, cx);
            }
        });
    }

    backdrop.child(panel).into_any_element()
}

/// macOS cardlike (centered) rendering.
fn render_cardlike(
    id: SharedString,
    content: AnyElement,
    width: Pixels,
    theme: &TahoeTheme,
    focus_handle: FocusHandle,
    on_dismiss_rc: Option<DismissRc>,
) -> gpui::AnyElement {
    let backdrop = backdrop_overlay(theme)
        .flex()
        .items_center()
        .justify_center();

    let reduce_motion = theme.accessibility_mode.reduce_motion();
    let anim_duration = if reduce_motion {
        REDUCE_MOTION_CROSSFADE
    } else {
        Duration::from_millis(theme.glass.motion.lift_duration_ms)
    };
    let anim_id = ElementId::Name(format!("{}-present", id).into());
    let scroll_id = ElementId::Name(format!("{}-scroll", id).into());

    let animated_body = div()
        .id(scroll_id)
        .overflow_y_scroll()
        .p(theme.spacing_lg)
        .child(content)
        .with_animation(anim_id, Animation::new(anim_duration), |el, delta| {
            el.opacity(delta)
        });

    let panel_id = ElementId::Name(format!("{}-panel", id).into());
    let mut panel = glass_surface(div().w(width).overflow_hidden(), theme, GlassSize::Large)
        .id(panel_id)
        .track_focus(&focus_handle)
        .flex()
        .flex_col()
        .max_h(gpui::relative(MACOS_SHEET_MAX_HEIGHT_FRACTION))
        .child(animated_body);

    if let Some(ref handler) = on_dismiss_rc {
        let h = handler.clone();
        panel = panel.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
            h(window, cx);
        });
    }

    if let Some(ref handler) = on_dismiss_rc {
        let h = handler.clone();
        panel = panel.on_key_down(move |event: &KeyDownEvent, window, cx| {
            if crate::foundations::keyboard::is_escape_key(event) {
                h(window, cx);
            }
        });
    }

    backdrop.child(panel).into_any_element()
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;
    use gpui::prelude::*;

    use crate::components::presentation::sheet::{
        DRAG_INDICATOR_HEIGHT, DRAG_INDICATOR_WIDTH, Sheet, SheetDetent, SheetPresentation,
    };
    use crate::foundations::layout::Platform;

    #[test]
    fn sheet_defaults() {
        let sheet = Sheet::new("test-sheet", gpui::div().child("hello"));
        assert!(!sheet.is_open);
        assert_eq!(sheet.detent, SheetDetent::Large);
        assert!(sheet.on_dismiss.is_none());
        assert!(sheet.focus_handle.is_none());
        assert!(sheet.presentation.is_none());
        assert!(sheet.width.is_none());
    }

    #[test]
    fn sheet_open_builder() {
        let sheet = Sheet::new("s", gpui::div()).open(true);
        assert!(sheet.is_open);
    }

    #[test]
    fn sheet_detent_medium() {
        let sheet = Sheet::new("s", gpui::div()).detent(SheetDetent::Medium);
        assert_eq!(sheet.detent, SheetDetent::Medium);
    }

    #[test]
    fn sheet_detent_large() {
        let sheet = Sheet::new("s", gpui::div()).detent(SheetDetent::Large);
        assert_eq!(sheet.detent, SheetDetent::Large);
    }

    #[test]
    fn sheet_detent_custom_clamps() {
        assert!((SheetDetent::Custom(0.65).height_fraction() - 0.65).abs() < f32::EPSILON);
        // Out-of-range values are clamped into (0.05, 1.0].
        assert!((SheetDetent::Custom(2.0).height_fraction() - 1.0).abs() < f32::EPSILON);
        assert!((SheetDetent::Custom(0.0).height_fraction() - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn sheet_on_dismiss_sets_callback() {
        let sheet = Sheet::new("s", gpui::div()).on_dismiss(|_w, _cx| {});
        assert!(sheet.on_dismiss.is_some());
    }

    #[test]
    fn sheet_child_builder() {
        let _sheet = Sheet::new("s", gpui::div()).child(gpui::div().child("content"));
    }

    #[test]
    fn sheet_full_builder_chain() {
        let _sheet = Sheet::new("s", gpui::div())
            .open(true)
            .detent(SheetDetent::Medium)
            .on_dismiss(|_w, _cx| {})
            .child(gpui::div().child("body"));
    }

    #[test]
    fn sheet_presentation_builder() {
        let sheet = Sheet::new("s", gpui::div()).presentation(SheetPresentation::Cardlike);
        assert_eq!(sheet.presentation, Some(SheetPresentation::Cardlike));
    }

    #[test]
    fn sheet_presentation_for_platform_matches_hig() {
        assert_eq!(
            SheetPresentation::for_platform(Platform::MacOS),
            SheetPresentation::Cardlike
        );
        assert_eq!(
            SheetPresentation::for_platform(Platform::IOS),
            SheetPresentation::BottomDrawer
        );
        assert_eq!(
            SheetPresentation::for_platform(Platform::WatchOS),
            SheetPresentation::BottomDrawer
        );
    }

    #[test]
    fn default_detent_is_large() {
        assert_eq!(SheetDetent::default(), SheetDetent::Large);
    }

    #[test]
    fn detent_variants_are_distinct() {
        assert_ne!(SheetDetent::Medium, SheetDetent::Large);
        assert_ne!(SheetDetent::Medium, SheetDetent::Custom(0.5));
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn drag_indicator_dimensions_are_positive() {
        assert!(
            DRAG_INDICATOR_WIDTH > 0.0,
            "drag indicator width must be positive"
        );
        assert!(
            DRAG_INDICATOR_HEIGHT > 0.0,
            "drag indicator height must be positive"
        );
    }
}
