//! HIG Materials — Liquid Glass and Standard Materials.
//!
//! Apple platforms define two types of materials (per HIG Materials page):
//!
//! ## Liquid Glass (controls/navigation layer)
//!
//! A dynamic meta-material for controls and navigation — tab bars, sidebars,
//! toolbars — that floats above the content layer. Composed of two layers
//! (from Figma Tahoe UI Kit):
//!
//! **Layer 1 (base):** Fill stack + Drop shadow (X:0 Y:8 Blur:40 `#000@12%`)
//! **Layer 2 (glass):** `#000000 @20%` + Glass effect (Refraction:100 Depth:16
//!   Dispersion:0 Frost:12 Splay:6 Light:-45°/67%)
//!
//! **Don't use Liquid Glass in the content layer.** It works best when it
//! provides a clear distinction between interactive elements and content.
//! Exception: transient interactive elements like sliders/toggles take on
//! Liquid Glass appearance when activated.
//!
//! **Variants:**
//! - **Regular**: Blurs and adjusts luminosity of background content.
//!   Most system components use this variant.
//! - **Clear**: Highly translucent, ideal for media-rich backgrounds.
//!   Add a 35% dark dimming layer for bright backgrounds.
//!
//! ## Standard Materials (content layer)
//!
//! Fill + Background blur (Uniform, 30) for visual differentiation within
//! the content layer. Five thickness levels with decreasing opacity:
//!
//! | Thickness   | Light (`#F6F6F6`) | Dark (`#000000`) |
//! |-------------|-------------------|------------------|
//! | UltraThick  | 84%               | 50%              |
//! | Thick       | 72%               | 40%              |
//! | Medium      | 60%               | 29%              |
//! | Thin        | 48%               | 20%              |
//! | UltraThin   | 36%               | 10%              |
//!
//! Use vibrant colors on top of materials for legibility.
//!
//! # Surface functions
//!
//! Apple-aligned entry points mirroring SwiftUI's `glassEffect(_:in:)`:
//!
//! - [`glass_effect()`] — fill-only Liquid Glass (no render-pass break).
//!   Replaces the legacy `glass_surface` / `tinted_glass_surface` /
//!   `glass_clear_surface` / `glass_shaped_surface` family.
//! - [`glass_effect_blur()`] — backdrop blur without refraction.
//! - [`glass_effect_lens()`] — full Liquid Glass lens composite
//!   (backdrop blur + refraction + chromatic aberration + Fresnel
//!   edge highlight) via [`Window::paint_lens_rect`].
//! - [`glass_surface_hud()`] — HUD surface (`NSPanel.StyleMask.HUDWindow`),
//!   Figma legacy material kept as a bespoke helper.
//! - [`backdrop_overlay()`] — full-viewport modal backdrop scrim.
//! - [`backdrop_blur_overlay()`] — full-viewport backdrop with an
//!   explicit [`BlurEffect`].
//!
//! Pick the [`Glass`] material, the [`Shape`] geometry, and the
//! [`Elevation`] shadow tier at the call site:
//!
//! ```ignore
//! glass_effect_lens(theme, Glass::Regular, Shape::RoundedRectangle(px(10.0)), Elevation::Elevated, None)
//! ```
//!
//! # Rendering pipeline
//!
//! Blur and lens surfaces use the dual-Kawase primitives that live on the
//! vendored `gpui` fork at `.context/zed` (`Primitive::BlurRect` /
//! `Primitive::LensRect`; pending upstream merge — see
//! `crates/tahoe-gpui/Cargo.toml` for the path dep). Each primitive forces
//! the renderer to break the current render pass so the framebuffer can be
//! sampled — keep the count per frame small and prefer one primitive per
//! glass surface. Do not use [`glass_effect_blur`] / [`glass_effect_lens`]
//! for list-row backgrounds. [`glass_effect`] is a translucent tinted
//! fill plus shadows, which composites cheaply without a render-pass
//! break.
//!
//! On macOS, [`TahoeTheme::apply_in_window`] additionally installs the
//! `NSVisualEffectView` window background so glass surfaces at the window
//! root stay translucent against the desktop wallpaper.
//!
//! [`TahoeTheme::apply_in_window`]: crate::foundations::theme::TahoeTheme::apply_in_window
//!
//! # Accessibility
//!
//! - **ReduceTransparency**: Glass replaced with opaque fills
//! - **IncreaseContrast**: Visible borders added via `apply_high_contrast_border()`
//! - **ReduceMotion**: Animation durations set to 0 via `effective_duration()`
//!
//! # Example
//!
//! ```ignore
//! let theme = cx.theme();
//! let card = glass_effect(
//!     div().p(px(16.0)),
//!     theme,
//!     Glass::Regular,
//!     Shape::Default,
//!     Elevation::Elevated,
//! );
//! // For custom fills:
//! let bg = theme.glass.accessible_fill(Glass::Regular, theme.accessibility_mode);
//! ```

use gpui::prelude::*;
use gpui::{
    AnimationExt, AnyElement, Bounds, BoxShadow, Corners, Deferred, Div, ElementId, FocusHandle,
    Hsla, Pixels, SharedString, Window, canvas, deferred, hsla, px,
};

use crate::foundations::accessibility::{AccessibilityMode, AccessibilityTokens};
use crate::foundations::layout::ShapeType;

/// Per-surface geometry — re-exported alias of [`ShapeType`] to mirror
/// SwiftUI's `Shape` parameter on [`glassEffect(_:in:)`][apple].
///
/// Callers pick a shape independently from the glass material, so a
/// [`Glass::Regular`] surface can render as a rounded rectangle or as a
/// capsule without changing the material recipe.
///
/// [apple]: https://developer.apple.com/documentation/SwiftUI/View/glassEffect(_:in:)
pub use crate::foundations::layout::ShapeType as Shape;

use crate::foundations::motion::{
    MorphState, MotionTokens, REDUCE_MOTION_CROSSFADE, accessible_spring_animation,
};
use crate::foundations::theme::{ActiveTheme, TahoeTheme};

/// Alpha of the Apple Liquid Glass "Layer 2" tint (`#000000 @ 20%`).
///
/// Applied in linear-light RGB by [`glass_effect`] so the glass darkening
/// is perceptually consistent from dark to light surfaces.
pub(crate) const GLASS_LAYER_TINT_ALPHA: f32 = 0.20;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Glass Type Definitions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Liquid Glass appearance preference per macOS System Settings.
///
/// Maps to the "Liquid Glass" picker in System Settings > Appearance.
/// `Clear` shows more of the background through the glass.
/// `Tinted` adds the accent color to the glass material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LiquidGlassPreference {
    #[default]
    Clear,
    Tinted,
}

/// Liquid Glass material identity — mirrors SwiftUI's
/// [`Glass`][apple-glass] type.
///
/// - `Regular` — the default variant; adaptive blur + luminosity correction
///   for legibility. Per Apple: "for alerts, sidebars, or popovers."
/// - `Clear` — highly translucent; prioritises visibility of media-rich
///   underlying content.
/// - `Identity` — no glass effect, pass-through. Use to conditionally
///   disable the material.
///
/// Pair with a [`Shape`] and an [`Elevation`] at a call site —
/// [`glass_effect`] / [`glass_effect_lens`] / [`glass_effect_blur`] are
/// the Apple-aligned entry points.
///
/// [apple-glass]: https://developer.apple.com/documentation/SwiftUI/Glass
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Glass {
    Regular,
    Clear,
    /// No glass effect — pass-through. Use to conditionally disable glass.
    Identity,
}

impl Glass {
    /// Attach the `interactive` modifier — returns a [`GlassMaterial`]
    /// with the flag set and defaults for the other fields.
    pub const fn interactive(self, v: bool) -> GlassMaterial {
        GlassMaterial {
            variant: self,
            interactive: v,
            tint: None,
        }
    }

    /// Attach a tint override — returns a [`GlassMaterial`].
    pub const fn tint(self, c: Option<Hsla>) -> GlassMaterial {
        GlassMaterial {
            variant: self,
            interactive: false,
            tint: c,
        }
    }
}

/// Liquid Glass material with optional SwiftUI-style modifiers.
///
/// Mirrors the method-chain form `Glass::Regular.interactive(true).tint(Some(color))`.
/// Most callers pass a bare [`Glass`] variant — the conversion to
/// `GlassMaterial` happens via `impl Into<GlassMaterial> for Glass` at
/// the entry-point boundary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassMaterial {
    pub variant: Glass,
    pub interactive: bool,
    pub tint: Option<Hsla>,
}

impl From<Glass> for GlassMaterial {
    fn from(variant: Glass) -> Self {
        Self {
            variant,
            interactive: false,
            tint: None,
        }
    }
}

impl GlassMaterial {
    /// Set the `interactive` flag — mirrors SwiftUI's `Glass.interactive(_:)`.
    pub const fn interactive(mut self, v: bool) -> Self {
        self.interactive = v;
        self
    }

    /// Set the tint override — mirrors SwiftUI's `Glass.tint(_:)`.
    pub const fn tint(mut self, c: Option<Hsla>) -> Self {
        self.tint = c;
        self
    }
}

/// Material thickness level per HIG.
/// Controls the blur/frost intensity of glass surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum MaterialThickness {
    /// Most transparent — barely visible frosting.
    UltraThin,
    /// Light frosting for subtle background separation.
    Thin,
    /// Standard glass frosting (default).
    #[default]
    Regular,
    /// Heavier frosting for higher contrast.
    Thick,
    /// Most opaque frosting — nearly solid background.
    UltraThick,
    /// HIG `.bar` / Chrome material — tuned for window toolbars, title
    /// bars, and tab bars where content scrolls beneath chrome. Slightly
    /// more opaque than `Thin` so labels stay legible while the tint of
    /// the content under the chrome still reads through. Routes to the
    /// dedicated `chrome_bg` theme token via [`GlassStyle::material_bg`].
    Chrome,
}

/// HIG-named alias for [`MaterialThickness`].
///
/// HIG §Materials names the content-layer Standard-Material tiers
/// directly (`UltraThick`, `Thick`, `Regular`, `Thin`, `UltraThin`). The
/// Zed cross-reference audit (Finding 28) flagged
/// that tahoe-gpui's `GlassStyle::subtle()` / `blur_only()` helpers don't
/// map to any of these named tiers, which leaves callers guessing which
/// thickness they're actually selecting. Exposing `StandardMaterial` as a
/// HIG-named alias keeps the type unified while letting content-layer
/// components read in HIG vocabulary.
pub type StandardMaterial = MaterialThickness;

/// Configuration for per-element backdrop blur.
///
/// Drives a Dual Kawase blur pass on the framebuffer region behind the
/// element via [`Window::paint_blur_rect`].
#[derive(Debug, Clone, Copy)]
pub struct BlurEffect {
    /// Blur radius in points. Higher values = more blur.
    /// Apple's standard glass uses ~20-40pt equivalent.
    pub radius: f32,
    /// Corner radius for the blur region mask.
    pub corner_radius: f32,
    /// Tint color overlaid on the blurred content.
    pub tint: Hsla,
}

impl From<&BlurEffect> for gpui::BlurEffect {
    fn from(effect: &BlurEffect) -> Self {
        Self {
            radius: px(effect.radius),
            kernel_levels: DEFAULT_BLUR_KERNEL_LEVELS,
            tint: effect.tint,
        }
    }
}

/// Configuration for glass refraction/lensing effect.
///
/// Extends [`BlurEffect`] with light-bending distortion that creates Apple's
/// Liquid Glass lens/refraction effect parameters.
///
/// Maps directly to Figma's Glass effect panel:
///
/// | Figma Parameter | Struct Field | Default |
/// |-----------------|--------------|---------|
/// | Refraction | `refraction` | 100 (→ 1.0) |
/// | Depth | `depth` | 16 |
/// | Dispersion | `dispersion` | 0 (→ 0.0) |
/// | Frost | `frost` (blur radius) | 12 |
/// | Splay | `splay` | 6 |
/// | Light angle | `light_angle` | -45° |
/// | Light intensity | `light_intensity` | 67% (→ 0.67) |
///
/// Drives the Liquid Glass composite via [`Window::paint_lens_rect`],
/// which applies dual-Kawase backdrop blur, parabolic refraction,
/// chromatic aberration, and a directional Fresnel edge highlight.
#[derive(Debug, Clone, Copy)]
pub struct LensEffect {
    /// Base blur configuration (frost maps to blur.radius).
    pub blur: BlurEffect,
    /// Refraction strength (Figma: 0–100, normalized 0.0–1.0).
    /// Controls how much background content is distorted/magnified.
    pub refraction: f32,
    /// Depth of the glass surface (Figma: 0–100).
    /// Controls parallax/3D depth illusion.
    pub depth: f32,
    /// Dispersion / chromatic aberration (Figma: 0–100, normalized 0.0–1.0).
    /// Splits R/G/B channels at edges. Apple default: 0 (no fringing).
    pub dispersion: f32,
    /// Edge splay distance in points (Figma: 0–100).
    /// Controls how far the specular edge highlight spreads.
    pub splay: f32,
    /// Directional light angle in degrees (Figma: -180 to 180).
    pub light_angle: f32,
    /// Light intensity (Figma: 0–100%, normalized 0.0–1.0).
    /// Controls brightness of the specular/Fresnel edge highlight.
    pub light_intensity: f32,
}

impl From<&LensEffect> for gpui::LensEffect {
    fn from(effect: &LensEffect) -> Self {
        // `tahoe-gpui` stores refraction/dispersion normalized to 0.0..1.0 to
        // match how callers think in fractions; GPUI's `LensEffect` now
        // uses the same 0..1 convention (upstream normalized refraction /
        // dispersion / depth in v0.231.1-pre), so refraction and dispersion
        // pass through directly. `depth` is still stored on the Figma 0..100
        // scale at call sites (e.g. `depth: 16.0`) — divide by 100 to reach
        // GPUI's normalized depth. Angle is stored in degrees for ergonomics;
        // GPUI consumes radians.
        Self {
            radius: px(effect.blur.radius),
            kernel_levels: DEFAULT_BLUR_KERNEL_LEVELS,
            refraction: effect.refraction,
            depth: effect.depth / 100.0,
            dispersion: effect.dispersion,
            splay: px(effect.splay),
            light_angle: gpui::radians(effect.light_angle.to_radians()),
            light_intensity: effect.light_intensity,
            tint: effect.blur.tint,
        }
    }
}

/// Text colors for content on Liquid Glass surfaces.
///
/// Apple adapts label colors based on whether underlying content is bright
/// or dim. The hierarchy matches the 5-level system from the Apple Tahoe
/// UI Kit: Primary → Secondary → Tertiary → Quaternary → Quinary.
#[derive(Debug, Clone)]
pub struct GlassLabels {
    pub primary: Hsla,
    pub secondary: Hsla,
    pub tertiary: Hsla,
    pub quaternary: Hsla,
    pub quinary: Hsla,
}

/// Tinted glass variant colors for colored glass surfaces.
#[derive(Debug, Clone)]
pub struct GlassTint {
    pub bg: Hsla,
    pub bg_hover: Hsla,
}

/// Named tint colors for glass surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlassTintColor {
    Green,
    Blue,
    Purple,
    Amber,
    Red,
    Cyan,
    Teal,
    Indigo,
}

/// Collection of tinted glass variants, keyed by color name.
#[derive(Debug, Clone)]
pub struct GlassTints {
    tints: [GlassTint; 8],
}

impl GlassTints {
    /// Create a new tints collection.
    pub fn new(
        green: GlassTint,
        blue: GlassTint,
        purple: GlassTint,
        amber: GlassTint,
        red: GlassTint,
        cyan: GlassTint,
        teal: GlassTint,
        indigo: GlassTint,
    ) -> Self {
        Self {
            tints: [green, blue, purple, amber, red, cyan, teal, indigo],
        }
    }

    /// Get the tint for a specific color.
    pub fn get(&self, color: GlassTintColor) -> &GlassTint {
        &self.tints[color as usize]
    }
}

/// Surface context for automatic label color resolution.
/// Components pass this to indicate what surface they render on,
/// and the correct label color (opaque semantic vs vibrant glass) is returned.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceContext {
    /// Standard opaque surface (use semantic label hierarchy).
    Opaque,
    /// Glass surface over dim underlying content (use vibrant dim labels).
    GlassDim,
    /// Glass surface over bright underlying content (use vibrant bright labels).
    GlassBright,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ElevationIndex — semantic elevation tiers (Finding 9)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Semantic elevation level of a surface.
///
/// Mirrors Zed's `ElevationIndex` (see `crates/ui/src/styles/elevation.rs`
/// in zed-industries/zed) so every surface has a named tier rather than
/// choosing shadow parameters ad hoc. The tiers cover all of the layering
/// tahoe-gpui renders — both opaque surfaces and Liquid Glass surfaces
/// route through this enum.
///
/// ```text
/// Background      — app background, lowest layer
/// Surface         — primary panel / pane
/// ElevatedSurface — hovers just above a Surface (popovers, context cards)
/// ModalSurface    — sheets, alerts, floating panels
/// OverlaySurface  — drag previews, tooltips, other above-modal content
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ElevationIndex {
    /// App background — nothing is below this.
    Background,
    /// Primary surface — panels, panes, containers.
    #[default]
    Surface,
    /// Surface elevated just above [`Self::Surface`] — popovers,
    /// context cards, hover cards.
    ElevatedSurface,
    /// Modal layer — sheets, dialogs, floating panels, inspectors.
    ModalSurface,
    /// Transient overlay layer above modals — drag previews, tooltips,
    /// scroll-edge effects, scrim washes. Matches Zed's
    /// `OverlaySurface` tier.
    OverlaySurface,
}

impl ElevationIndex {
    /// Returns the [`GlassRole`] that applies at this elevation tier.
    ///
    /// HIG prohibits Liquid Glass in the content layer — that is, the
    /// opaque [`ElevationIndex::Background`] / [`ElevationIndex::Surface`]
    /// strata — so those tiers return [`GlassRole::Controls`] with a
    /// `content_layer` flag the caller must negate. Elevated, modal, and
    /// overlay tiers map to [`GlassRole::Navigation`] and
    /// [`GlassRole::Overlay`] respectively. Keeps the
    /// "content vs. controls" HIG distinction legible at a glance.
    pub fn glass_role(self) -> GlassRole {
        match self {
            Self::Background | Self::Surface => GlassRole::ContentLayer,
            Self::ElevatedSurface => GlassRole::Controls,
            Self::ModalSurface => GlassRole::Navigation,
            Self::OverlaySurface => GlassRole::Overlay,
        }
    }

    /// The matching HIG Standard-Material thickness for opaque rendering
    /// on this elevation tier. Mirrors the mapping Zed uses between
    /// `ElevationIndex` and Apple's thickness ladder.
    ///
    /// HIG §Standard materials advances from `.thin` at lower
    /// elevations to `.ultraThick` as we move up so elevated surfaces
    /// feel progressively more substantial when the platform can't
    /// render real depth. Content-layer surfaces (`Background` /
    /// `Surface`) stay on the `Regular` tier to match AppKit default
    /// `NSVisualEffectMaterial.contentBackground`.
    pub fn standard_material(self) -> MaterialThickness {
        match self {
            Self::Background => MaterialThickness::Thin,
            Self::Surface => MaterialThickness::Regular,
            Self::ElevatedSurface => MaterialThickness::Thick,
            Self::ModalSurface => MaterialThickness::UltraThick,
            // Overlay surfaces borrow the modal thickness — they should
            // read as opaque even over translucent modals.
            Self::OverlaySurface => MaterialThickness::UltraThick,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Elevation — shadow tier axis (orthogonal to Glass material)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Shadow tier for a Liquid Glass surface. Decoupled from [`Glass`]
/// because SwiftUI's `glassEffect(_:in:)` does not bundle shadow with
/// material — shadows come from the view's elevation context, not from
/// the material recipe itself.
///
/// Maps to the three shadow stacks on [`GlassStyle`]:
/// `resting_shadows` / `elevated_shadows` / `floating_shadows`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Elevation {
    /// No shadow.
    None,
    /// Resting controls — Figma "Liquid Glass – Small UI" tier (single
    /// 4pt drop shadow). Buttons, toggles, segmented-control tracks.
    #[default]
    Resting,
    /// Elevated panels — Figma "BG - Medium UI" tier (ambient Y=8
    /// Blur=40 @12% + 1pt rim @23%). Alerts, modals, dropdowns,
    /// popovers, action-sheet groups, non-HUD panels.
    Elevated,
    /// Floating sheets — full-screen sheets, large overlays. Shares the
    /// 40pt ambient with `Elevated` today; reserved as a separate tier
    /// so sheets can diverge if Apple's Large-UI spec lands.
    Floating,
}

impl Elevation {
    /// Returns the shadow stack from the active theme that matches this
    /// elevation tier.
    pub fn shadows(self, theme: &TahoeTheme) -> &[gpui::BoxShadow] {
        match self {
            Self::None => &[],
            Self::Resting => &theme.glass.resting_shadows,
            Self::Elevated => &theme.glass.elevated_shadows,
            Self::Floating => &theme.glass.floating_shadows,
        }
    }
}

impl From<ElevationIndex> for Elevation {
    fn from(index: ElevationIndex) -> Self {
        match index {
            ElevationIndex::Background | ElevationIndex::Surface => Self::Resting,
            ElevationIndex::ElevatedSurface => Self::Elevated,
            ElevationIndex::ModalSurface | ElevationIndex::OverlaySurface => Self::Floating,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// GlassRole — HIG layering guard (Finding 21)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Semantic role of a surface in the macOS Tahoe layering stack.
///
/// HIG §Materials prohibits using Liquid Glass in the **content layer**
/// (regular document body, list rows, grid cells). Only the **controls
/// layer** (buttons, toolbars, segmented controls), **navigation layer**
/// (sidebars, modal sheets, floating panels), and **overlay layer**
/// (tooltips, drag previews, scrim washes) may adopt Liquid Glass.
/// Content surfaces must use Standard Materials.
///
/// Components that render a background thread a [`GlassRole`] through
/// the API so callers can detect the "content layer" case and fall back
/// to a Standard Material or opaque fill. Glass helpers log a debug
/// assertion when invoked with [`GlassRole::ContentLayer`], matching the
/// lint-comment pattern described in Finding 21 of the Zed cross-reference
/// audit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlassRole {
    /// Content surfaces — document body, list rows, grid cells. HIG
    /// forbids Liquid Glass here. Callers landing in this case should
    /// use a Standard Material (see [`MaterialThickness`]) or an
    /// opaque fill.
    #[default]
    ContentLayer,
    /// Controls layer — buttons, segmented controls, toolbar items.
    /// Liquid Glass is appropriate.
    Controls,
    /// Navigation layer — sidebars, floating panels, modal sheets.
    /// Liquid Glass is appropriate.
    Navigation,
    /// Overlay layer — tooltips, drag previews, scrim washes. Liquid
    /// Glass is appropriate (often at the `Overlay` tier from
    /// [`ElevationIndex`]).
    Overlay,
}

impl GlassRole {
    /// Returns `true` when Liquid Glass is permitted for this role.
    pub fn permits_liquid_glass(self) -> bool {
        !matches!(self, Self::ContentLayer)
    }
}

/// Resolve a label color based on surface context and hierarchy level.
///
/// Level 0 = primary, 1 = secondary, 2 = tertiary, 3 = quaternary, 4 = quinary.
pub fn resolve_label(theme: &TahoeTheme, context: SurfaceContext, level: usize) -> Hsla {
    match context {
        SurfaceContext::Opaque => match level {
            0 => theme.semantic.label,
            1 => theme.semantic.secondary_label,
            2 => theme.semantic.tertiary_label,
            3 => theme.semantic.quaternary_label,
            _ => theme.semantic.quinary_label,
        },
        SurfaceContext::GlassDim | SurfaceContext::GlassBright => {
            let labels = theme.glass.labels(context);
            match level {
                0 => labels.primary,
                1 => labels.secondary,
                2 => labels.tertiary,
                3 => labels.quaternary,
                _ => labels.quinary,
            }
        }
    }
}

/// Liquid Glass design tokens aligned with Apple iOS 26 design system.
///
/// Mirrors SwiftUI's `Glass` + `glassEffect(_:in:)` split — material
/// identity (`variant`) is independent of per-surface geometry and
/// shadow tier. The struct exposes one canonical fill per Liquid Glass
/// variant and three shadow tiers keyed by [`Elevation`]; corner radius
/// is per-surface via [`Shape`] / [`compute_shape_radius`] and is no
/// longer a GlassStyle field.
///
/// # Usage
///
/// ```ignore
/// let glass = &theme.glass;
/// let bg = glass.fill(Glass::Regular);
/// let shadows = Elevation::Elevated.shadows(theme).to_vec();
/// ```
#[derive(Debug, Clone)]
pub struct GlassStyle {
    /// Theme-level default variant (`Regular` or `Clear`). Per-surface
    /// callers override this by passing a [`Glass`] to
    /// [`glass_effect`]/[`glass_effect_lens`]/[`glass_effect_blur`].
    pub variant: Glass,

    /// Canonical Regular Liquid Glass fill — Figma Tahoe UI Kit
    /// "BG - Medium UI".
    pub regular_fill: Hsla,
    /// Canonical Clear Liquid Glass fill — high-translucency variant
    /// for media-rich backdrops (Apple: "highly translucent").
    pub clear_fill: Hsla,
    pub hover_bg: Hsla,

    // Per-thickness fills (HIG Standard Materials).
    //
    // These are *standard-material* fills for the content layer
    // (backdrop-blur + #F6F6F6 / #000000 at varying alpha) — distinct from
    // the Liquid Glass fills above (`regular_fill` / `clear_fill`), which
    // are for the controls/navigation layer. Mixing the two violates the
    // HIG layering rule: glass sits above content, content sits above
    // the window background.
    pub ultra_thin_bg: Hsla,
    pub thin_bg: Hsla,
    /// Regular-thickness standard material. Dark: `#000000 @29%`,
    /// light: `#F6F6F6 @60%`. Used by [`GlassStyle::material_bg`] for
    /// [`MaterialThickness::Regular`] — *not* the Liquid Glass Medium fill.
    pub medium_standard_bg: Hsla,
    pub thick_bg: Hsla,
    pub ultra_thick_bg: Hsla,
    /// HIG `.bar` / Chrome fill for toolbars, title bars, and tab bars.
    /// Darker/denser than `thin_bg` so labels stay legible when content
    /// scrolls behind the chrome. Dark ≈ `#000 @ 34%`, light ≈ `#F6F6F6
    /// @ 65%`. Consumed by [`GlassStyle::material_bg`] for
    /// [`MaterialThickness::Chrome`].
    pub chrome_bg: Hsla,

    /// Shadow stack for the [`Elevation::Resting`] tier — single 4pt
    /// drop shadow. Controls and toolbar tracks.
    pub resting_shadows: Vec<BoxShadow>,
    /// Shadow stack for the [`Elevation::Elevated`] tier — Figma
    /// "BG - Medium UI": ambient Y=8 Blur=40 @12% + 1pt rim @23%.
    /// Alerts, modals, dropdowns, popovers.
    pub elevated_shadows: Vec<BoxShadow>,
    /// Shadow stack for the [`Elevation::Floating`] tier — full-screen
    /// sheets and large overlays.
    pub floating_shadows: Vec<BoxShadow>,

    // Window
    pub window_background: gpui::WindowBackgroundAppearance,
    pub root_bg: Hsla,

    // Labels on glass
    pub labels_dim: GlassLabels,
    pub labels_bright: GlassLabels,

    // Font family for glass surfaces (use TextStyle for type scale)
    pub font_sans: SharedString,

    // Tinted variants
    pub tints: GlassTints,

    // Accessibility
    pub accessibility: AccessibilityTokens,

    // Motion
    pub motion: MotionTokens,

    // Glass preference (from macOS System Settings)
    pub preference: LiquidGlassPreference,

    // Accent-colored glass tint
    pub accent_tint: GlassTint,

    // Glass icon/semantic colors (pastel variants for glass surfaces)
    pub icon_text: Hsla,
    pub icon_success: Hsla,
    pub icon_info: Hsla,
    pub icon_warning: Hsla,
    pub icon_error: Hsla,
    pub icon_ai: Hsla,
    pub tile_bg: Hsla,
    pub tile_border: Hsla,
}

impl GlassStyle {
    /// Returns the canonical fill for the given Liquid Glass variant.
    ///
    /// `Regular` → [`Self::regular_fill`]; `Clear` → [`Self::clear_fill`];
    /// `Identity` → transparent. Mirrors SwiftUI's `Glass` material
    /// identity — fill is a function of the material variant, not a
    /// per-surface tier.
    pub fn fill(&self, glass: Glass) -> Hsla {
        match glass {
            Glass::Regular => self.regular_fill,
            Glass::Clear => self.clear_fill,
            Glass::Identity => hsla(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Returns the fill for `glass`, adjusted for
    /// [`AccessibilityMode::REDUCE_TRANSPARENCY`] — falls back to the
    /// opaque accessibility fill when transparency is reduced.
    pub fn accessible_fill(&self, glass: Glass, mode: AccessibilityMode) -> Hsla {
        if mode.reduce_transparency() {
            self.accessibility.reduced_transparency_bg
        } else {
            self.fill(glass)
        }
    }

    /// Returns the glass label colors for the given glass surface context.
    ///
    /// # Contract
    ///
    /// `context` MUST be [`SurfaceContext::GlassDim`] or
    /// [`SurfaceContext::GlassBright`]. Opaque callers must use
    /// [`resolve_label`] instead — glass labels are tuned for vibrant
    /// compositing over a blurred backdrop and do not match the semantic
    /// label hierarchy used by opaque surfaces, especially under
    /// IncreaseContrast. Passing `Opaque` here trips a `debug_assert!`; in
    /// release builds it returns the dim palette as a conservative fallback.
    ///
    /// Per HIG Materials: "Rely on the system's vibrancy effects for text
    /// and icons on Liquid Glass. Don't use opaque fills on top of Liquid
    /// Glass."
    pub fn labels(&self, context: SurfaceContext) -> &GlassLabels {
        match context {
            SurfaceContext::GlassDim => &self.labels_dim,
            SurfaceContext::GlassBright => &self.labels_bright,
            SurfaceContext::Opaque => {
                debug_assert!(
                    false,
                    "GlassStyle::labels() requires a glass surface context; \
                     call resolve_label(theme, SurfaceContext::Opaque, level) \
                     to route through the semantic label hierarchy instead."
                );
                &self.labels_dim
            }
        }
    }

    /// Returns the fill for a given standard material thickness level.
    ///
    /// UltraThin is most transparent, UltraThick is most opaque. These fills
    /// are distinct from the Liquid Glass fills returned by [`Self::fill`]:
    /// standard materials belong to the *content* layer (HIG Materials —
    /// Standard), Liquid Glass to the *controls/navigation* layer (HIG
    /// Materials — Liquid Glass). The two should not be conflated, so
    /// `Regular` routes to its own `medium_standard_bg` token rather than
    /// the Liquid Glass `regular_fill`.
    pub fn material_bg(&self, thickness: MaterialThickness) -> Hsla {
        match thickness {
            MaterialThickness::UltraThin => self.ultra_thin_bg,
            MaterialThickness::Thin => self.thin_bg,
            MaterialThickness::Chrome => self.chrome_bg,
            MaterialThickness::Regular => self.medium_standard_bg,
            MaterialThickness::Thick => self.thick_bg,
            MaterialThickness::UltraThick => self.ultra_thick_bg,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Glass Surface Functions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Dark translucent tint applied on top of [`glass_effect`] so HUD
/// surfaces render dark regardless of the current appearance.
///
/// This is the **effective visible tint** — `black @ 60%` to match
/// `NSPanel.StyleMask.HUDWindow` per HIG `#panels` — i.e. the tint a
/// viewer sees after [`glass_surface_hud`]'s Layer 2
/// (`GLASS_LAYER_TINT_ALPHA`) has stacked on top. The actual alpha
/// `hud_fill` hands to `compose_black_tint_linear` is lower
/// (`HUD_PRE_COMPOSE_ALPHA`); Layer 2 fills the gap.
///
/// Exposed as a constant so callers that need the raw value
/// (e.g. tinting a sub-element consistently with the HUD backdrop)
/// can re-use the same effective tint.
pub const HUD_TINT_ALPHA: f32 = 0.6;

/// Pre-composition alpha used inside [`hud_fill`]. Chosen so that
/// after [`glass_surface_hud`]'s Layer 2 ([`GLASS_LAYER_TINT_ALPHA`])
/// stacks on top, the effective visible tint lands at
/// [`HUD_TINT_ALPHA`].
///
/// Linear-light Porter–Duff src-over:
/// `1 - (1 - pre)(1 - layer2) = effective` →
/// `pre = 1 - (1 - effective)/(1 - layer2)`.
/// With `effective = 0.60` and `layer2 = 0.20`, `pre = 0.50`.
const HUD_PRE_COMPOSE_ALPHA: f32 = 1.0 - (1.0 - HUD_TINT_ALPHA) / (1.0 - GLASS_LAYER_TINT_ALPHA);

/// Resolve the HUD surface fill — the base Regular Liquid Glass fill
/// pre-composed with a black tint that, after `glass_effect` layers the
/// universal Layer 2 tint on top, lands at the spec-documented
/// [`HUD_TINT_ALPHA`] effective visible tint.
///
/// Uses [`GlassStyle::accessible_fill`] so `ReduceTransparency` routes
/// through the opaque fallback before the HUD tint applies — the
/// accessibility path darkens the opaque fill rather than a
/// translucent one.
fn hud_fill(theme: &TahoeTheme) -> Hsla {
    let base = theme
        .glass
        .accessible_fill(Glass::Regular, theme.accessibility_mode);
    crate::foundations::color::compose_black_tint_linear(base, HUD_PRE_COMPOSE_ALPHA)
}

/// Apply Liquid Glass HUD surface styling to a div.
///
/// Pre-composes the dark HUD tint into the Regular Liquid Glass fill
/// via [`hud_fill`], then hands the result to [`glass_effect`] with a
/// caller-supplied [`Shape`] and [`Elevation`] so the surface picks up
/// the shared Layer 2 tint + shadows + border contract, plus
/// [`TahoeTheme::background`] as the text color so the surface reads as
/// a dark HUD regardless of the current appearance. Matches
/// `NSPanel.StyleMask.HUDWindow` per HIG `#panels`.
///
/// The tint is pre-composed rather than layered via a second `.bg()`
/// call because GPUI's `bg()` is last-write-wins — chaining a second
/// `.bg()` would discard the glass chrome and collapse the surface to
/// a flat black rectangle.
///
/// Respects accessibility the same way [`glass_effect`] does:
/// ReduceTransparency routes through the opaque fallback fill (the
/// HUD tint then darkens that opaque color), and IncreaseContrast
/// adds a visible border. Uses the fill-only rendering path; pair with
/// [`glass_effect_blur`] or [`glass_effect_lens`] when a HUD needs a
/// blurred backdrop.
pub fn glass_surface_hud(el: Div, theme: &TahoeTheme, shape: Shape, elevation: Elevation) -> Div {
    let bg = hud_fill(theme);
    let radius = compute_shape_radius(theme, shape, None);
    let composited =
        crate::foundations::color::compose_black_tint_linear(bg, GLASS_LAYER_TINT_ALPHA);
    let shadows = elevation.shadows(theme).to_vec();
    let el = el
        .bg(composited)
        .rounded(radius)
        .shadow(shadows)
        .text_color(theme.background);
    apply_glass_border_by_elevation(el, theme, elevation)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Liquid Glass surface scope
//
// Per HIG §Materials / Liquid Glass (`docs/hig/foundations.md:1045`),
// content placed on a Liquid Glass surface should inherit vibrancy. Any
// `Icon` descendant using the default `IconStyle::Auto` automatically
// resolves to the glass variant when it sits inside a
// [`crate::foundations::surface_scope::GlassSurfaceScope`]. To opt a
// subtree into that behaviour, wrap the glass-surface Div:
//
// ```ignore
// use tahoe_gpui::foundations::surface_scope::GlassSurfaceScope;
// use tahoe_gpui::foundations::materials::{Elevation, Glass, Shape, glass_effect};
//
// GlassSurfaceScope::new(
//     glass_effect(div(), theme, Glass::Regular, Shape::Default, Elevation::Elevated)
//         .child(Icon::new(IconName::Star))
// )
// ```
//
// Keeping scope separate from the non-scoped `glass_effect*` functions
// means callers who only need the chrome (no icon propagation) keep the
// `Div -> Div` signature and its chain-ability; and callers who want the
// full propagation compose with one extra `GlassSurfaceScope::new(…)`.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Kawase pass count used by every `tahoe-gpui` glass helper.
///
/// 3 matches Apple's HIG default for heavy sheet / inspector backdrops and
/// GPUI's own default. `kernel_levels` is the Kawase downsample/upsample
/// pass count *inside* the blur post-process (clamped to 1..=5 by the
/// renderer), not the number of render-pass breaks — each `BlurRect` /
/// `LensRect` primitive always breaks the render pass exactly once, so
/// raising this value widens the blur without adding more pass-breaks.
const DEFAULT_BLUR_KERNEL_LEVELS: u32 = 3;

/// Returns a styled [`gpui::Canvas`] that fills its parent and invokes
/// `paint` during the paint phase. The canvas is `.absolute()` + `.size_full()`
/// so it covers the parent's box without participating in the flex flow;
/// attached as the *first* child so the paint callback samples the
/// framebuffer before sibling content paints on top.
fn paint_canvas(paint: impl FnOnce(Bounds<Pixels>, &mut Window) + 'static) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| paint(bounds, window),
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

/// Wrap [`Window::paint_blur_rect`] in a [`paint_canvas`] that fills its parent.
fn blur_rect_canvas(effect: gpui::BlurEffect, corner_radius: Pixels) -> impl IntoElement {
    paint_canvas(move |bounds, window| {
        window.paint_blur_rect(bounds, Corners::all(corner_radius), effect);
    })
}

/// Wrap [`Window::paint_lens_rect`] in a [`paint_canvas`] that fills its parent.
fn lens_rect_canvas(effect: gpui::LensEffect, corner_radius: Pixels) -> impl IntoElement {
    paint_canvas(move |bounds, window| {
        window.paint_lens_rect(bounds, Corners::all(corner_radius), effect);
    })
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// glass_effect — Apple-aligned entry points
// (mirrors SwiftUI's `glassEffect(_:in:)` — material × shape × elevation)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Resolve the fill color for a [`GlassMaterial`], respecting
/// ReduceTransparency. One canonical fill per Liquid Glass variant —
/// Regular uses the Figma "BG - Medium UI" fill; Clear uses the
/// highest-translucency fill; Identity is transparent.
fn glass_fill_for(theme: &TahoeTheme, material: GlassMaterial) -> Hsla {
    if theme.accessibility_mode.reduce_transparency() {
        return theme.glass.accessibility.reduced_transparency_bg;
    }
    if let Some(tint) = material.tint {
        return tint;
    }
    theme.glass.fill(material.variant)
}

/// Build the lens recipe for a [`GlassMaterial`]. Regular picks the full
/// Figma "BG - Medium UI" refractive params; Clear picks a HIG-aligned
/// high-translucency recipe (lighter frost, lower refraction).
fn lens_effect_for(theme: &TahoeTheme, material: GlassMaterial, radius: Pixels) -> LensEffect {
    let corner_radius = f32::from(radius);
    let tint = glass_fill_for(theme, material);
    match material.variant {
        Glass::Regular => LensEffect {
            blur: BlurEffect {
                radius: 12.0,
                corner_radius,
                tint,
            },
            refraction: 1.0,
            depth: 16.0,
            dispersion: 0.0,
            splay: 6.0,
            light_angle: -45.0,
            light_intensity: 0.67,
        },
        // Apple "Clear": highly translucent — lighter frost, lower
        // refraction, softer edge highlight. Keeps the lens readable
        // over media-rich backdrops without dominating.
        Glass::Clear => LensEffect {
            blur: BlurEffect {
                radius: 6.0,
                corner_radius,
                tint,
            },
            refraction: 0.4,
            depth: 4.0,
            dispersion: 0.0,
            splay: 2.0,
            light_angle: -45.0,
            light_intensity: 0.30,
        },
        // Identity: callers short-circuit before reaching this path.
        Glass::Identity => LensEffect {
            blur: BlurEffect {
                radius: 0.0,
                corner_radius,
                tint: hsla(0.0, 0.0, 0.0, 0.0),
            },
            refraction: 0.0,
            depth: 0.0,
            dispersion: 0.0,
            splay: 0.0,
            light_angle: -45.0,
            light_intensity: 0.0,
        },
    }
}

/// Build the blur recipe for a [`GlassMaterial`] — same frost as the
/// lens recipe but without refraction.
fn blur_effect_for(theme: &TahoeTheme, material: GlassMaterial, radius: Pixels) -> BlurEffect {
    let lens = lens_effect_for(theme, material, radius);
    lens.blur
}

/// Apply the elevation-driven border contract. Elevated/Floating tiers
/// receive the 1pt specular top-edge highlight (HIG "frosted edge");
/// Resting/None tiers only get the IncreaseContrast fallback border.
fn apply_glass_border_by_elevation(mut el: Div, theme: &TahoeTheme, elevation: Elevation) -> Div {
    let mode = theme.accessibility_mode;
    if mode.increase_contrast() {
        el = el
            .border_1()
            .border_color(theme.glass.accessibility.high_contrast_border);
    } else if !mode.reduce_transparency()
        && matches!(elevation, Elevation::Elevated | Elevation::Floating)
    {
        el = el.border_t(px(1.0)).border_color(hsla(0.0, 0.0, 1.0, 0.18));
    }
    el
}

/// Fill-only Liquid Glass — no render-pass break. Apple-aligned entry
/// point mirroring SwiftUI's `glassEffect(_:in:)` for the cheap path.
///
/// Use this for content-dense surfaces (input rows, upload dropzones)
/// where the full lens composite would cost a render-pass break for
/// little visual gain. For real refraction + blur sampling, use
/// [`glass_effect_lens`]. For blur without refraction, use
/// [`glass_effect_blur`].
pub fn glass_effect(
    el: Div,
    theme: &TahoeTheme,
    glass: impl Into<GlassMaterial>,
    shape: Shape,
    elevation: Elevation,
) -> Div {
    let material = glass.into();
    if matches!(material.variant, Glass::Identity) {
        return el;
    }
    let radius = compute_shape_radius(theme, shape, None);
    let bg = glass_fill_for(theme, material);
    let composited =
        crate::foundations::color::compose_black_tint_linear(bg, GLASS_LAYER_TINT_ALPHA);
    let shadows = elevation.shadows(theme).to_vec();
    let el = el.bg(composited).rounded(radius).shadow(shadows);
    apply_glass_border_by_elevation(el, theme, elevation)
}

/// Real Liquid Glass lens composite (refraction + blur + specular edge).
/// Breaks one render pass — keep the total number of concurrent lens
/// surfaces in a frame bounded per the rendering-pipeline guidance at
/// the top of this module.
///
/// Apple-aligned entry point mirroring SwiftUI's
/// `glassEffect(_:in:)` with the lens compositing default.
pub fn glass_effect_lens(
    theme: &TahoeTheme,
    glass: impl Into<GlassMaterial>,
    shape: Shape,
    elevation: Elevation,
    container_height: Option<Pixels>,
) -> Div {
    let material = glass.into();
    let radius = compute_shape_radius(theme, shape, container_height);

    if matches!(material.variant, Glass::Identity) {
        return gpui::div().rounded(radius);
    }

    if theme.accessibility_mode.reduce_transparency() {
        return apply_glass_border_by_elevation(
            gpui::div()
                .bg(theme.glass.accessibility.reduced_transparency_bg)
                .rounded(radius),
            theme,
            elevation,
        );
    }

    let effect = lens_effect_for(theme, material, radius);
    let shadows = elevation.shadows(theme).to_vec();
    apply_glass_border_by_elevation(
        gpui::div()
            .relative()
            .rounded(radius)
            .shadow(shadows)
            .child(lens_rect_canvas(gpui::LensEffect::from(&effect), radius)),
        theme,
        elevation,
    )
}

/// Backdrop blur without refraction — cheaper than [`glass_effect_lens`]
/// but still breaks one render pass. Use for surfaces that want a
/// backdrop blur and tint but don't need the full Liquid Glass lens
/// (status-indicator HUDs, simple frosted overlays).
pub fn glass_effect_blur(
    theme: &TahoeTheme,
    glass: impl Into<GlassMaterial>,
    shape: Shape,
    elevation: Elevation,
    container_height: Option<Pixels>,
) -> Div {
    let material = glass.into();
    let radius = compute_shape_radius(theme, shape, container_height);

    if matches!(material.variant, Glass::Identity) {
        return gpui::div().rounded(radius);
    }

    if theme.accessibility_mode.reduce_transparency() {
        return apply_glass_border_by_elevation(
            gpui::div()
                .bg(theme.glass.accessibility.reduced_transparency_bg)
                .rounded(radius),
            theme,
            elevation,
        );
    }

    let effect = blur_effect_for(theme, material, radius);
    let shadows = elevation.shadows(theme).to_vec();
    apply_glass_border_by_elevation(
        gpui::div()
            .relative()
            .rounded(radius)
            .shadow(shadows)
            .child(blur_rect_canvas(gpui::BlurEffect::from(&effect), radius)),
        theme,
        elevation,
    )
}

/// Computes the corner radius for an HIG shape type.
///
/// - **Fixed** / **RoundedRectangle**: Returns the constant radius.
/// - **Capsule**: Returns half the container height (pill shape).
/// - **Concentric**: Returns `parent_radius - padding`, minimum 0.
/// - **Default**: Returns `theme.radius_md` when no `container_height`
///   is supplied, or the capsule radius (`height / 2`) when one is —
///   mirrors SwiftUI's `DefaultGlassEffectShape` behaviour for
///   interactive vs embedded surfaces.
pub fn compute_shape_radius(
    theme: &TahoeTheme,
    shape: ShapeType,
    container_height: Option<Pixels>,
) -> Pixels {
    match shape {
        ShapeType::Fixed(r) | ShapeType::RoundedRectangle(r) => r,
        ShapeType::Capsule => container_height.map_or(theme.radius_full, |h| h / 2.0),
        ShapeType::Concentric {
            parent_radius,
            padding,
        } => (parent_radius - padding).max(px(0.0)),
        ShapeType::Default => container_height.map_or(theme.radius_md, |h| h / 2.0),
    }
}

/// Returns tint background adjusted for ReduceTransparency (alpha × 3, capped at 0.5).
/// For default mode, returns the original tint background unchanged.
pub fn accessible_tint_bg(tint: &GlassTint, mode: AccessibilityMode) -> gpui::Hsla {
    if mode.reduce_transparency() {
        let mut bg = tint.bg;
        bg.a = (bg.a * 3.0).min(0.5);
        bg
    } else {
        tint.bg
    }
}

pub use super::accessibility::{apply_high_contrast_border, effective_duration};

/// Apply the standard glass-control styling triplet.
///
/// Fills the control with [`Glass::Regular`] at [`Elevation::Resting`],
/// rounds it with `shape` ([`Shape::Default`] for rectangular triggers,
/// [`Shape::Capsule`] for pill controls), stacks the resting shadows,
/// layers the focus ring when `focused`, and applies the IncreaseContrast
/// border fallback. Use for any control whose chrome is a glass trigger —
/// popup buttons, pickers, date/time pickers, combo boxes, steppers,
/// segmented controls, and similar.
///
/// Matches the fill-only contract of [`glass_effect`] but stays
/// [`gpui::Styled`]-generic so callers can chain it onto typed builders
/// (buttons, sliders) rather than bare [`Div`]s. The high-contrast
/// border is applied exactly once; the focus ring layers on top of the
/// resting shadows without re-assigning them.
pub fn apply_standard_control_styling<E: gpui::Styled>(
    mut el: E,
    theme: &TahoeTheme,
    shape: Shape,
    focused: bool,
) -> E {
    let glass = &theme.glass;
    let bg = glass.accessible_fill(Glass::Regular, theme.accessibility_mode);
    let composited =
        crate::foundations::color::compose_black_tint_linear(bg, GLASS_LAYER_TINT_ALPHA);
    let radius = compute_shape_radius(theme, shape, None);
    let mut shadows = Elevation::Resting.shadows(theme).to_vec();
    if focused {
        shadows.extend(theme.focus_ring_shadows());
    }
    el = el.bg(composited).rounded(radius).shadow(shadows);
    apply_high_contrast_border(el, theme)
}

/// Applies Liquid Glass interactive hover behavior to a div per HIG.
///
/// When a glass theme is active:
/// - **Hover**: background shifts to `hover_bg` for lift effect, cursor becomes pointer
///
/// Active/press states require an element ID (use `.id().active()` on the caller).
///
/// Glass is always present; uses `glass.hover_bg` for lift effect.
pub fn glass_interactive_hover(mut el: Div, theme: &TahoeTheme) -> Div {
    let hover_bg = theme.glass.hover_bg;
    el = el.hover(|style| style.bg(hover_bg).cursor_pointer());
    el
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// GlassContainer Component
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A container that applies glass surface to itself and renders children
/// without glass, as a convention around Apple's "no glass on glass" rule.
///
/// Per HIG, glass elements should never be stacked on other glass
/// elements. `GlassContainer` provides the single glass layer and renders
/// all children as standard content within it. The rule is *documented*
/// here and honored by all in-tree components, but it is not enforced at
/// render time — a caller that wraps a `glass_effect(...)` child inside
/// a `GlassContainer` will silently nest.
///
/// # Example
/// ```ignore
/// GlassContainer::new("toolbar-group")
///     .shape(Shape::Capsule)
///     .elevation(Elevation::Resting)
///     .spacing(theme.spacing_sm)
///     .child(button_a)
///     .child(button_b)
/// ```
#[derive(IntoElement)]
pub struct GlassContainer {
    id: ElementId,
    shape: Shape,
    elevation: Elevation,
    pub(crate) spacing: Option<Pixels>,
    pub(crate) children: Vec<AnyElement>,
}

impl GlassContainer {
    /// Create a new glass container with the given element ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            shape: Shape::Default,
            elevation: Elevation::Resting,
            spacing: None,
            children: Vec::new(),
        }
    }

    /// Set the glass shape (Default, Capsule, RoundedRectangle, …).
    pub fn shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }

    /// Set the elevation tier (Resting, Elevated, Floating).
    pub fn elevation(mut self, elevation: Elevation) -> Self {
        self.elevation = elevation;
        self
    }

    /// Set the spacing between children.
    pub fn spacing(mut self, spacing: Pixels) -> Self {
        self.spacing = Some(spacing);
        self
    }

    /// Add a child element to the container.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    /// Add multiple children.
    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for GlassContainer {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme();

        let mut inner = gpui::div().flex().flex_row().items_center();

        if let Some(spacing) = self.spacing {
            inner = inner.gap(spacing);
        }

        for child in self.children {
            inner = inner.child(child);
        }

        glass_effect(inner, theme, Glass::Regular, self.shape, self.elevation).id(self.id)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Scroll Edge Effects
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Default height for a scroll-edge overlay.
///
/// HIG §scroll-edge-effects (July 2025) does not mandate a specific
/// pixel value — the rule is that each scroll view applies *one* edge
/// effect and that split-view panes keep their edge-effect heights
/// consistent with each other. We default to 40 pt to cover the unified
/// title-bar-plus-toolbar region on macOS 26 Tahoe (see
/// [`MACOS_TOOLBAR_UNIFIED_HEIGHT`](super::layout::MACOS_TOOLBAR_UNIFIED_HEIGHT))
/// while staying visually subtle for iPad/iPhone toolbars. Callers that
/// want the original 16 pt behavior can pass
/// [`SCROLL_EDGE_HEIGHT_COMPACT`] for the compact variant.
pub const SCROLL_EDGE_HEIGHT: Pixels = px(40.0);

/// Compact scroll-edge height (16 pt) — the pre-audit default, retained
/// for callers that do not need the taller macOS Tahoe region.
pub const SCROLL_EDGE_HEIGHT_COMPACT: Pixels = px(16.0);

/// Style of a scroll-edge effect per HIG §scroll-edge-effects.
///
/// HIG distinguishes a **soft** effect (the default: a gentle blur fade
/// between scrolling content and the navigation bar) from a **hard**
/// effect (used by macOS interactive text surfaces to keep text readable
/// all the way to the edge). Apple's real effect fades blur *strength*
/// across the edge band — `Window::paint_blur_rect` today takes a single
/// scalar radius, so the implementation below still approximates both
/// with a bounded linear gradient. Expose the enum now so callers record
/// intent and the rendering can upgrade in one place when a
/// variable-radius blur primitive lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollEdgeStyle {
    /// Default soft fade — subtle gradient between content and toolbar.
    #[default]
    Soft,
    /// Hard edge — used by macOS interactive text surfaces where the
    /// text must remain legible up to the toolbar's leading edge.
    Hard,
}

/// Create a top scroll edge overlay.
///
/// Returns an absolutely-positioned overlay that fades from the theme
/// background to transparent at the given `height`. Use
/// [`SCROLL_EDGE_HEIGHT`] for the default 40 pt tier or
/// [`SCROLL_EDGE_HEIGHT_COMPACT`] for the narrower 16 pt variant;
/// callers rendering into split views should pass identical heights to
/// every pane so the edges stay consistent (HIG requirement).
///
/// `style` selects between [`ScrollEdgeStyle::Soft`] (gradient fade,
/// matching the pre-audit behavior) and [`ScrollEdgeStyle::Hard`]
/// (abrupt cutoff — near-instant fade used by interactive text
/// surfaces). Variable-radius blur matching the HIG's modern
/// scroll-edge effect needs a primitive GPUI does not yet expose
/// (`paint_blur_rect` takes a single scalar radius); this gradient
/// fallback is documented at the call site and can upgrade when a
/// variable-radius primitive ships.
pub fn scroll_edge_top(theme: &TahoeTheme, height: Pixels, style: ScrollEdgeStyle) -> Div {
    scroll_edge(theme, height, style, ScrollEdgeSide::Top)
}

/// Create a bottom scroll edge overlay. See [`scroll_edge_top`] for
/// the parameter semantics; the only difference here is that the
/// gradient originates from the bottom instead of the top.
pub fn scroll_edge_bottom(theme: &TahoeTheme, height: Pixels, style: ScrollEdgeStyle) -> Div {
    scroll_edge(theme, height, style, ScrollEdgeSide::Bottom)
}

enum ScrollEdgeSide {
    Top,
    Bottom,
}

fn scroll_edge(
    theme: &TahoeTheme,
    height: Pixels,
    style: ScrollEdgeStyle,
    side: ScrollEdgeSide,
) -> Div {
    let bg = theme.background;

    // Soft variant: fade across the full height so the transition
    // reads as a gentle smear. Hard variant: confine the fade to the
    // last ~10 % of the gradient so the opaque region meets the
    // scroll content nearly at the edge — the closest approximation
    // of HIG's "hard" scroll edge effect using a gradient, since
    // `paint_blur_rect` does not yet accept a variable-radius mask.
    let (top_color, top_stop, bottom_color, bottom_stop) = match style {
        ScrollEdgeStyle::Soft => (bg, 0.0, hsla(bg.h, bg.s, bg.l, 0.0), 1.0),
        ScrollEdgeStyle::Hard => (bg, 0.9, hsla(bg.h, bg.s, bg.l, 0.0), 1.0),
    };

    let (angle, first_stop, second_stop) = match side {
        ScrollEdgeSide::Top => (
            180.0,
            gpui::LinearColorStop {
                color: top_color,
                percentage: top_stop,
            },
            gpui::LinearColorStop {
                color: bottom_color,
                percentage: bottom_stop,
            },
        ),
        ScrollEdgeSide::Bottom => (
            0.0,
            gpui::LinearColorStop {
                color: top_color,
                percentage: top_stop,
            },
            gpui::LinearColorStop {
                color: bottom_color,
                percentage: bottom_stop,
            },
        ),
    };

    let base = gpui::div().absolute().left_0().w_full().h(height);
    let positioned = match side {
        ScrollEdgeSide::Top => base.top_0(),
        ScrollEdgeSide::Bottom => base.bottom_0(),
    };

    positioned.bg(gpui::linear_gradient(angle, first_stop, second_stop))
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Glass Morphing Transitions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Apply a Reduce-Motion-aware glass morphing animation to an element.
///
/// Interpolates between `from` and `to` morph states using spring timing.
/// The element renders in a deferred overlay layer during the transition.
///
/// When `theme.accessibility_mode` has `REDUCE_MOTION` set, the spring morph
/// is replaced with a short linear cross-fade (per HIG:
/// "replace large, dramatic transitions with subtle cross-fades") — the
/// element jumps to `to`'s geometry and only opacity animates. This matches
/// the guidance in `foundations.md:1100`.
///
/// The child is wrapped in a `div()` that receives the animation styles,
/// then placed in a `deferred()` with priority 2 for overlay rendering.
///
/// # Example
/// ```ignore
/// let morph = glass_morph(
///     "card-expand",
///     MorphState::new(100.0, 200.0, 44.0, 44.0, 22.0),
///     MorphState::new(50.0, 100.0, 300.0, 400.0, 20.0),
///     theme,
///     div().child("Card content"),
/// );
/// ```
pub fn glass_morph(
    id: impl Into<ElementId>,
    from: MorphState,
    to: MorphState,
    theme: &TahoeTheme,
    child: impl IntoElement,
) -> Deferred {
    let reduce_motion = theme.accessibility_mode.reduce_motion();
    let animation = accessible_spring_animation(&theme.glass.motion, reduce_motion);
    let child_el = child.into_any_element();

    deferred(gpui::div().size_full().child(child_el).with_animation(
        id,
        animation,
        move |el, delta| {
            if reduce_motion {
                // Cross-fade only: snap geometry to `to`, animate opacity.
                el.absolute()
                    .left(px(to.x))
                    .top(px(to.y))
                    .w(px(to.width))
                    .h(px(to.height))
                    .rounded(px(to.corner_radius))
                    .opacity(delta * to.opacity)
            } else {
                let state = MorphState::lerp(&from, &to, delta);
                el.absolute()
                    .left(px(state.x))
                    .top(px(state.y))
                    .w(px(state.width))
                    .h(px(state.height))
                    .rounded(px(state.corner_radius))
                    .opacity(state.opacity)
            }
        },
    ))
    .with_priority(crate::foundations::overlay::OverlayLayer::GLASS_MORPH)
}

/// Cross-fade a glass surface between two [`Elevation`] tiers when its
/// shadow stack or material identity changes.
///
/// Per Apple's Tahoe Liquid Glass spec, a surface that changes its material
/// tier should smoothly blend between blur/opacity levels rather than
/// snapping. GPUI does not yet expose animated blur, so this helper
/// approximates the tier blend with a duration-based opacity cross-fade
/// over `shape_shift_duration_ms`. Callers that don't animate layout
/// should keep calling [`glass_effect`] directly — this helper is opt-in.
///
/// The element renders the `to`-tier surface throughout; only opacity
/// animates from 0→1 on tier change. Under Reduce Motion the animation
/// still runs but at the short 150ms cross-fade duration.
pub fn glass_tier_transition<E>(
    el: E,
    id: impl Into<ElementId>,
    theme: &TahoeTheme,
) -> gpui::AnimationElement<E>
where
    E: IntoElement + gpui::Styled + 'static,
{
    use std::time::Duration;
    let reduce_motion = theme.accessibility_mode.reduce_motion();
    let duration = if reduce_motion {
        REDUCE_MOTION_CROSSFADE
    } else {
        Duration::from_millis(theme.glass.motion.shape_shift_duration_ms)
    };
    el.with_animation(id, gpui::Animation::new(duration), |el, delta| {
        el.opacity(delta)
    })
}

/// Resolve the effective focused state for a control with an optional
/// host-supplied [`FocusHandle`].
///
/// When `handle` is `Some`, its live focus state (`handle.is_focused(window)`)
/// wins and `fallback` is ignored — this keeps the focus ring in sync with
/// the host's focus graph instead of a stale `.focused(bool)` cache. When
/// `handle` is `None`, `fallback` is returned unchanged so hosts that have
/// not wired a handle keep their existing `.focused(bool)` behaviour.
///
/// Paired with [`apply_focus_ring`]: resolve the bool here, then pass it to
/// the ring. Centralizing the rule lets callers avoid re-implementing the
/// `Option::map(is_focused).unwrap_or(fallback)` dance inline.
pub fn resolve_focused(handle: Option<&FocusHandle>, window: &Window, fallback: bool) -> bool {
    match handle {
        Some(h) => h.is_focused(window),
        None => fallback,
    }
}

/// Apply the HIG focus ring to an element, preserving any base shadows.
///
/// When `focused`: sets shadows to `base_shadows` + the two focus-ring layers
/// returned by [`TahoeTheme::focus_ring_shadows`] (outer accent + inner gap).
/// When not focused: sets shadows to `base_shadows` (if non-empty), or no-op.
///
/// This is the single entry point for focus ring + shadow composition.
/// - Non-glass components: `apply_focus_ring(el, theme, focused, &[])`
/// - Glass components: `apply_focus_ring(el, theme, focused, &theme.glass.shadows(size))`
///
/// When the host wires a [`FocusHandle`], resolve the bool via
/// [`resolve_focused`] so the ring tracks live focus instead of a prop cache.
pub fn apply_focus_ring<E: gpui::Styled>(
    mut el: E,
    theme: &crate::foundations::theme::TahoeTheme,
    focused: bool,
    base_shadows: &[gpui::BoxShadow],
) -> E {
    if focused {
        let mut shadows = base_shadows.to_vec();
        shadows.extend(theme.focus_ring_shadows());
        el = el.shadow(shadows);
    } else if !base_shadows.is_empty() {
        el = el.shadow(base_shadows.to_vec());
    }
    el
}

/// Apply standard interactive hover styling (background highlight + pointer cursor).
///
/// Consolidates the common `.hover(|s| s.bg(hover).cursor_pointer())` pattern.
///
/// GPUI's `.hover()` style API is a binary CSS-style swap with no transition
/// duration, so `theme.glass.motion.flex_duration_ms` (HIG short-ramp hover
/// target) is not consumed here. To animate hover transitions properly
/// today a consumer must wrap the element with an `AnimationElement` keyed
/// off a hover state flag; this is tracked as open question #3 on
/// the internal tracker.
pub fn interactive_hover(
    el: gpui::Div,
    theme: &crate::foundations::theme::TahoeTheme,
) -> gpui::Div {
    let hover = theme.hover_bg();
    el.hover(move |style| style.bg(hover).cursor_pointer())
}

/// Fade-in + slide-from-top animation for collapsible content, Reduce-Motion
/// aware.
///
/// Used by collapsible sections (reasoning, chain-of-thought) and panel
/// mount animations. Derives duration from `theme.glass.motion.shape_shift_duration_ms`
/// (long ramp). When `REDUCE_MOTION` is active, the vertical slide is
/// suppressed and the element cross-fades over a short 150ms window
/// instead (per HIG: subtle cross-fades replace dramatic transitions).
pub fn fade_slide_in(
    el: gpui::Div,
    id: gpui::ElementId,
    theme: &crate::foundations::theme::TahoeTheme,
) -> gpui::AnimationElement<gpui::Div> {
    use std::time::Duration;
    let reduce_motion = theme.accessibility_mode.reduce_motion();
    let duration = if reduce_motion {
        REDUCE_MOTION_CROSSFADE
    } else {
        // `effective_duration` would return 0 when reduce_motion is on,
        // but Animation::new(0ms) produces NaN deltas — branch here.
        Duration::from_millis(crate::foundations::accessibility::effective_duration(
            theme,
            theme.glass.motion.shape_shift_duration_ms,
        ))
    };
    el.with_animation(id, gpui::Animation::new(duration), move |el, delta| {
        let el = el.opacity(delta);
        if reduce_motion {
            el
        } else {
            el.mt(gpui::px((1.0 - delta) * -8.0))
        }
    })
}

/// Opaque content card surface per HIG.
///
/// Returns a `Div` styled as a content-layer card with `theme.surface` background,
/// rounded corners, 1px border, and hidden overflow. Used by code module cards
/// (agent, artifact, sandbox, test_results, commit).
pub fn card_surface(theme: &crate::foundations::theme::TahoeTheme) -> gpui::Div {
    gpui::div()
        .flex()
        .flex_col()
        .bg(theme.surface)
        .rounded(theme.radius_lg)
        .border_1()
        .border_color(theme.border)
        .overflow_hidden()
}

/// Create a full-screen backdrop overlay for modal components.
///
/// Returns an absolutely positioned div covering the full viewport,
/// filled with [`TahoeTheme::overlay_bg`] — the standard modal dim scrim.
///
/// # No Kawase blur
///
/// Per the Figma Tahoe UI Kit overlay spec the scrim is a flat tint with
/// no backdrop blur of its own. Every modal panel in the crate now paints
/// its own lens composite via [`glass_effect_lens`], so blurring the
/// full viewport beneath would both double-blur (once under the scrim and
/// once under the modal) and cost a second render-pass break. Callers
/// that want an explicit blurred scrim can still use
/// [`backdrop_blur_overlay`] directly.
pub fn backdrop_overlay(theme: &crate::foundations::theme::TahoeTheme) -> gpui::Div {
    gpui::div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .bg(theme.overlay_bg)
}

/// Create a full-screen backdrop overlay with an explicit backdrop-blur
/// effect — the blur-aware analog of [`backdrop_overlay`].
///
/// - With `ReduceTransparency`: tints with the opaque
///   [`AccessibilityTokens::reduced_transparency_bg`] so motion-sensitive
///   and high-contrast users get a solid scrim with no blur.
/// - Otherwise: paints a full-viewport dual-Kawase blur via
///   [`Window::paint_blur_rect`], with `effect.tint` composited on top.
///
/// `effect.corner_radius` is intentionally ignored — a modal backdrop is
/// always full-bleed, and rounding the blur rect over a `size_full`
/// overlay would leave four triangular corners of unblurred content
/// bleeding through. Callers that want the scrim to clip to a rounded
/// window chrome must render this overlay inside a rounded,
/// `overflow_hidden` ancestor.
pub fn backdrop_blur_overlay(
    theme: &crate::foundations::theme::TahoeTheme,
    effect: &BlurEffect,
) -> gpui::Div {
    if theme.accessibility_mode.reduce_transparency() {
        return gpui::div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(theme.glass.accessibility.reduced_transparency_bg);
    }

    gpui::div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .child(blur_rect_canvas(gpui::BlurEffect::from(effect), px(0.0)))
}

/// HIG-default backdrop-blur effect for full-viewport modal overlays.
///
/// - `radius`: 40 pt — matches Apple's heavy sheet/inspector backdrop blur.
/// - `corner_radius`: 0 — backdrops are always full-bleed, no rounded mask.
/// - `tint`: [`TahoeTheme::overlay_bg`] — the standard dim scrim.
///
/// Exposed so callers that want to customize only one axis (e.g. a
/// lighter blur for a popover) can do
/// `BlurEffect { radius: 20.0, ..default_backdrop_blur_effect(theme) }`.
pub fn default_backdrop_blur_effect(theme: &crate::foundations::theme::TahoeTheme) -> BlurEffect {
    BlurEffect {
        radius: 40.0,
        corner_radius: 0.0,
        tint: theme.overlay_bg,
    }
}

/// Wrap a Clear Liquid Glass surface with a 35%-opacity dark dimming layer
/// behind it.
///
/// Per HIG: "If the underlying content is bright, consider adding a dark
/// dimming layer of 35% opacity behind Liquid Glass in the clear style."
///
/// The dimming layer is rendered before the glass element in the element
/// tree, so z-order is guaranteed by the wrapper — callers cannot reorder
/// the two children. Pair with [`glass_effect`] / [`glass_effect_lens`]
/// / [`glass_effect_blur`] under [`Glass::Clear`].
pub fn clear_glass_dimmed(glass: Div) -> Div {
    gpui::div()
        .relative()
        .child(
            gpui::div()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .bg(gpui::hsla(0.0, 0.0, 0.0, 0.35)),
        )
        .child(glass)
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;
    use gpui::px;

    use super::{
        Elevation, ElevationIndex, GLASS_LAYER_TINT_ALPHA, Glass, GlassMaterial, GlassRole,
        GlassTint, GlassTintColor, GlassTints, HUD_PRE_COMPOSE_ALPHA, HUD_TINT_ALPHA, Shape,
        accessible_tint_bg, blur_effect_for, compute_shape_radius, effective_duration,
        glass_effect, glass_effect_blur, glass_effect_lens, glass_fill_for, hud_fill,
        lens_effect_for,
    };
    use crate::foundations::accessibility::AccessibilityMode;
    use crate::foundations::theme::TahoeTheme;

    // ── Shape / compute_shape_radius ─────────────────────────────────────

    #[test]
    fn fixed_radius_returns_input() {
        let theme = TahoeTheme::dark();
        let r = compute_shape_radius(&theme, Shape::Fixed(px(12.0)), None);
        assert!((f32::from(r) - 12.0).abs() < f32::EPSILON);
    }

    #[test]
    fn rounded_rectangle_equals_fixed() {
        let theme = TahoeTheme::dark();
        let r_fixed = compute_shape_radius(&theme, Shape::Fixed(px(17.0)), None);
        let r_rr = compute_shape_radius(&theme, Shape::RoundedRectangle(px(17.0)), None);
        assert_eq!(r_fixed, r_rr);
    }

    #[test]
    fn capsule_returns_half_height() {
        let theme = TahoeTheme::dark();
        let r = compute_shape_radius(&theme, Shape::Capsule, Some(px(44.0)));
        assert!((f32::from(r) - 22.0).abs() < f32::EPSILON);
    }

    #[test]
    fn capsule_no_height_uses_full() {
        let theme = TahoeTheme::dark();
        let r = compute_shape_radius(&theme, Shape::Capsule, None);
        assert_eq!(r, theme.radius_full);
    }

    #[test]
    fn concentric_subtracts_padding() {
        let theme = TahoeTheme::dark();
        let r = compute_shape_radius(
            &theme,
            Shape::Concentric {
                parent_radius: px(20.0),
                padding: px(4.0),
            },
            None,
        );
        assert!((f32::from(r) - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn concentric_minimum_zero() {
        let theme = TahoeTheme::dark();
        let r = compute_shape_radius(
            &theme,
            Shape::Concentric {
                parent_radius: px(4.0),
                padding: px(10.0),
            },
            None,
        );
        assert!((f32::from(r) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn default_shape_without_height_resolves_to_radius_md() {
        let theme = TahoeTheme::dark();
        let r = compute_shape_radius(&theme, Shape::Default, None);
        assert_eq!(r, theme.radius_md);
    }

    #[test]
    fn default_shape_with_height_acts_as_capsule() {
        let theme = TahoeTheme::dark();
        let r = compute_shape_radius(&theme, Shape::Default, Some(px(28.0)));
        assert!((f32::from(r) - 14.0).abs() < f32::EPSILON);
    }

    // ── Glass material identity ──────────────────────────────────────────

    #[test]
    fn glass_material_from_bare_variant_sets_sane_defaults() {
        let material: GlassMaterial = Glass::Regular.into();
        assert_eq!(material.variant, Glass::Regular);
        assert!(!material.interactive);
        assert!(material.tint.is_none());
    }

    #[test]
    fn glass_material_builder_chain_round_trip() {
        let color = gpui::hsla(0.6, 0.5, 0.5, 0.5);
        let material = Glass::Clear.interactive(true).tint(Some(color));
        assert_eq!(material.variant, Glass::Clear);
        assert!(material.interactive);
        assert_eq!(material.tint, Some(color));
    }

    #[test]
    fn glass_fill_respects_reduce_transparency_fallback() {
        let mut theme = TahoeTheme::liquid_glass();
        theme.accessibility_mode = AccessibilityMode::REDUCE_TRANSPARENCY;
        let bg = glass_fill_for(&theme, Glass::Regular.into());
        assert_eq!(bg, theme.glass.accessibility.reduced_transparency_bg);
    }

    #[test]
    fn glass_fill_regular_uses_canonical_regular_fill() {
        let theme = TahoeTheme::liquid_glass();
        let bg = glass_fill_for(&theme, Glass::Regular.into());
        assert_eq!(bg, theme.glass.regular_fill);
    }

    #[test]
    fn glass_fill_clear_uses_canonical_clear_fill() {
        let theme = TahoeTheme::liquid_glass();
        let bg = glass_fill_for(&theme, Glass::Clear.into());
        assert_eq!(bg, theme.glass.clear_fill);
    }

    #[test]
    fn glass_fill_identity_is_transparent() {
        let theme = TahoeTheme::liquid_glass();
        let bg = glass_fill_for(&theme, Glass::Identity.into());
        assert_eq!(bg.a, 0.0);
    }

    #[test]
    fn glass_fill_honours_material_tint_override() {
        let theme = TahoeTheme::liquid_glass();
        let tint = gpui::hsla(0.33, 0.7, 0.5, 0.4);
        let bg = glass_fill_for(&theme, Glass::Regular.tint(Some(tint)));
        assert_eq!(bg, tint);
    }

    // ── GlassStyle::fill / accessible_fill ───────────────────────────────

    #[test]
    fn glass_style_fill_regular_matches_regular_fill() {
        let theme = TahoeTheme::liquid_glass();
        assert_eq!(theme.glass.fill(Glass::Regular), theme.glass.regular_fill);
    }

    #[test]
    fn glass_style_fill_clear_matches_clear_fill() {
        let theme = TahoeTheme::liquid_glass();
        assert_eq!(theme.glass.fill(Glass::Clear), theme.glass.clear_fill);
    }

    #[test]
    fn glass_style_accessible_fill_matches_fill_by_default() {
        let theme = TahoeTheme::liquid_glass();
        let bg = theme
            .glass
            .accessible_fill(Glass::Regular, theme.accessibility_mode);
        assert_eq!(bg, theme.glass.fill(Glass::Regular));
    }

    #[test]
    fn glass_style_accessible_fill_reduce_transparency_falls_back() {
        let mut theme = TahoeTheme::liquid_glass();
        theme.accessibility_mode = AccessibilityMode::REDUCE_TRANSPARENCY;
        let bg = theme
            .glass
            .accessible_fill(Glass::Regular, theme.accessibility_mode);
        assert_eq!(bg, theme.glass.accessibility.reduced_transparency_bg);
    }

    // ── Elevation enum + From<ElevationIndex> ────────────────────────────

    #[test]
    fn elevation_default_is_resting() {
        assert_eq!(Elevation::default(), Elevation::Resting);
    }

    #[test]
    fn elevation_none_has_no_shadows() {
        let theme = TahoeTheme::liquid_glass();
        assert!(Elevation::None.shadows(&theme).is_empty());
    }

    #[test]
    fn elevation_resting_shadows_match_theme() {
        let theme = TahoeTheme::liquid_glass();
        assert_eq!(
            Elevation::Resting.shadows(&theme),
            theme.glass.resting_shadows.as_slice()
        );
    }

    #[test]
    fn elevation_elevated_shadows_match_theme() {
        let theme = TahoeTheme::liquid_glass();
        assert_eq!(
            Elevation::Elevated.shadows(&theme),
            theme.glass.elevated_shadows.as_slice()
        );
    }

    #[test]
    fn elevation_floating_shadows_match_theme() {
        let theme = TahoeTheme::liquid_glass();
        assert_eq!(
            Elevation::Floating.shadows(&theme),
            theme.glass.floating_shadows.as_slice()
        );
    }

    #[test]
    fn elevation_index_maps_to_elevation() {
        assert_eq!(
            Elevation::from(ElevationIndex::Background),
            Elevation::Resting
        );
        assert_eq!(Elevation::from(ElevationIndex::Surface), Elevation::Resting);
        assert_eq!(
            Elevation::from(ElevationIndex::ElevatedSurface),
            Elevation::Elevated
        );
        assert_eq!(
            Elevation::from(ElevationIndex::ModalSurface),
            Elevation::Floating
        );
        assert_eq!(
            Elevation::from(ElevationIndex::OverlaySurface),
            Elevation::Floating
        );
    }

    // ── Internal lens / blur recipes ─────────────────────────────────────

    #[test]
    fn lens_effect_regular_matches_figma_params() {
        let theme = TahoeTheme::liquid_glass();
        let radius = theme.radius_md;
        let lens = lens_effect_for(&theme, Glass::Regular.into(), radius);
        assert_eq!(lens.refraction, 1.0);
        assert_eq!(lens.depth, 16.0);
        assert_eq!(lens.dispersion, 0.0);
        assert_eq!(lens.splay, 6.0);
        assert_eq!(lens.light_angle, -45.0);
        assert!((lens.light_intensity - 0.67).abs() < 0.01);
        assert_eq!(lens.blur.radius, 12.0);
    }

    #[test]
    fn lens_effect_clear_is_more_translucent_than_regular() {
        let theme = TahoeTheme::liquid_glass();
        let radius = theme.radius_md;
        let regular = lens_effect_for(&theme, Glass::Regular.into(), radius);
        let clear = lens_effect_for(&theme, Glass::Clear.into(), radius);
        assert!(clear.refraction < regular.refraction);
        assert!(clear.depth < regular.depth);
        assert!(clear.light_intensity < regular.light_intensity);
        assert!(clear.blur.radius < regular.blur.radius);
    }

    #[test]
    fn lens_effect_identity_produces_noop_recipe() {
        let theme = TahoeTheme::liquid_glass();
        let lens = lens_effect_for(&theme, Glass::Identity.into(), theme.radius_md);
        assert_eq!(lens.refraction, 0.0);
        assert_eq!(lens.depth, 0.0);
        assert_eq!(lens.light_intensity, 0.0);
        assert_eq!(lens.blur.tint.a, 0.0);
    }

    #[test]
    fn blur_effect_matches_lens_blur() {
        let theme = TahoeTheme::liquid_glass();
        let radius = theme.radius_md;
        let material: GlassMaterial = Glass::Regular.into();
        let lens = lens_effect_for(&theme, material, radius);
        let blur = blur_effect_for(&theme, material, radius);
        assert_eq!(blur.radius, lens.blur.radius);
        assert_eq!(blur.tint, lens.blur.tint);
    }

    // ── glass_effect* entry-point smoke tests ────────────────────────────

    #[test]
    fn glass_effect_builds_for_regular_and_clear() {
        let theme = TahoeTheme::liquid_glass();
        let _: gpui::Div = glass_effect(
            gpui::div(),
            &theme,
            Glass::Regular,
            Shape::Default,
            Elevation::Elevated,
        );
        let _: gpui::Div = glass_effect(
            gpui::div(),
            &theme,
            Glass::Clear,
            Shape::Capsule,
            Elevation::Resting,
        );
    }

    #[test]
    fn glass_effect_identity_passes_through() {
        let theme = TahoeTheme::liquid_glass();
        let _: gpui::Div = glass_effect(
            gpui::div(),
            &theme,
            Glass::Identity,
            Shape::Default,
            Elevation::Resting,
        );
    }

    #[test]
    fn glass_effect_lens_builds_across_accessibility_modes() {
        for mode in [
            AccessibilityMode::DEFAULT,
            AccessibilityMode::REDUCE_TRANSPARENCY,
            AccessibilityMode::INCREASE_CONTRAST,
        ] {
            let mut theme = TahoeTheme::liquid_glass();
            theme.accessibility_mode = mode;
            let _: gpui::Div = glass_effect_lens(
                &theme,
                Glass::Regular,
                Shape::Default,
                Elevation::Elevated,
                None,
            );
        }
    }

    #[test]
    fn glass_effect_blur_builds_across_accessibility_modes() {
        for mode in [
            AccessibilityMode::DEFAULT,
            AccessibilityMode::REDUCE_TRANSPARENCY,
            AccessibilityMode::INCREASE_CONTRAST,
        ] {
            let mut theme = TahoeTheme::liquid_glass();
            theme.accessibility_mode = mode;
            let _: gpui::Div = glass_effect_blur(
                &theme,
                Glass::Regular,
                Shape::Default,
                Elevation::Elevated,
                None,
            );
        }
    }

    // ── (Glass, Elevation) matrix pin test ───────────────────────────────

    #[test]
    fn glass_elevation_matrix_builds() {
        let theme = TahoeTheme::liquid_glass();
        for glass in [Glass::Regular, Glass::Clear, Glass::Identity] {
            for elevation in [
                Elevation::None,
                Elevation::Resting,
                Elevation::Elevated,
                Elevation::Floating,
            ] {
                let _: gpui::Div =
                    glass_effect(gpui::div(), &theme, glass, Shape::Default, elevation);
                let _: gpui::Div =
                    glass_effect_lens(&theme, glass, Shape::Default, elevation, None);
                let _: gpui::Div =
                    glass_effect_blur(&theme, glass, Shape::Default, elevation, None);
            }
        }
    }

    // ── GlassRole (HIG layering guard) ───────────────────────────────────

    #[test]
    fn glass_role_permits_liquid_glass_except_content_layer() {
        assert!(!GlassRole::ContentLayer.permits_liquid_glass());
        assert!(GlassRole::Controls.permits_liquid_glass());
        assert!(GlassRole::Navigation.permits_liquid_glass());
        assert!(GlassRole::Overlay.permits_liquid_glass());
    }

    #[test]
    fn glass_role_default_is_safest_choice() {
        assert_eq!(GlassRole::default(), GlassRole::ContentLayer);
    }

    #[test]
    fn elevation_index_maps_to_glass_role() {
        assert_eq!(
            ElevationIndex::Background.glass_role(),
            GlassRole::ContentLayer
        );
        assert_eq!(
            ElevationIndex::Surface.glass_role(),
            GlassRole::ContentLayer
        );
        assert_eq!(
            ElevationIndex::ElevatedSurface.glass_role(),
            GlassRole::Controls
        );
        assert_eq!(
            ElevationIndex::ModalSurface.glass_role(),
            GlassRole::Navigation
        );
        assert_eq!(
            ElevationIndex::OverlaySurface.glass_role(),
            GlassRole::Overlay
        );
    }

    #[test]
    fn elevation_index_standard_material_ladder_is_monotonic() {
        use super::MaterialThickness;
        fn rank(m: MaterialThickness) -> u8 {
            match m {
                MaterialThickness::UltraThin => 0,
                MaterialThickness::Thin | MaterialThickness::Chrome => 1,
                MaterialThickness::Regular => 2,
                MaterialThickness::Thick => 3,
                MaterialThickness::UltraThick => 4,
            }
        }
        assert!(
            rank(ElevationIndex::Background.standard_material())
                <= rank(ElevationIndex::Surface.standard_material())
        );
        assert!(
            rank(ElevationIndex::Surface.standard_material())
                <= rank(ElevationIndex::ElevatedSurface.standard_material())
        );
        assert!(
            rank(ElevationIndex::ElevatedSurface.standard_material())
                <= rank(ElevationIndex::ModalSurface.standard_material())
        );
    }

    // ── Motion & Accessibility ───────────────────────────────────────────

    #[test]
    fn effective_duration_dark_theme_unchanged() {
        let theme = TahoeTheme::dark();
        assert_eq!(effective_duration(&theme, 200), 200);
    }

    #[test]
    fn effective_duration_glass_default_unchanged() {
        let theme = TahoeTheme::liquid_glass();
        assert_eq!(effective_duration(&theme, 200), 200);
    }

    #[test]
    fn effective_duration_glass_reduce_motion_is_zero() {
        let mut theme = TahoeTheme::liquid_glass();
        theme.accessibility_mode = AccessibilityMode::REDUCE_MOTION;
        assert_eq!(effective_duration(&theme, 200), 0);
    }

    #[test]
    fn effective_duration_dark_theme_reduce_motion_is_zero() {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::REDUCE_MOTION;
        assert_eq!(effective_duration(&theme, 200), 0);
    }

    // ── Tint helpers ─────────────────────────────────────────────────────

    #[test]
    fn accessible_tint_bg_multiplies_alpha_for_reduce_transparency() {
        let tint = GlassTint {
            bg: gpui::hsla(0.0, 0.0, 0.0, 0.08),
            bg_hover: gpui::hsla(0.0, 0.0, 0.0, 0.16),
        };
        let bg = accessible_tint_bg(&tint, AccessibilityMode::REDUCE_TRANSPARENCY);
        assert!((bg.a - 0.24).abs() < f32::EPSILON);
    }

    #[test]
    fn accessible_tint_bg_returns_original_for_default() {
        let tint = GlassTint {
            bg: gpui::hsla(0.0, 0.0, 0.0, 0.08),
            bg_hover: gpui::hsla(0.0, 0.0, 0.0, 0.16),
        };
        let bg = accessible_tint_bg(&tint, AccessibilityMode::DEFAULT);
        assert!((bg.a - 0.08).abs() < f32::EPSILON);
    }

    #[test]
    fn glass_tints_round_trip_by_name() {
        let mk = |l: f32| GlassTint {
            bg: gpui::hsla(0.0, 0.0, l, 0.1),
            bg_hover: gpui::hsla(0.0, 0.0, l, 0.2),
        };
        let tints = GlassTints::new(
            mk(0.1),
            mk(0.2),
            mk(0.3),
            mk(0.4),
            mk(0.5),
            mk(0.6),
            mk(0.7),
            mk(0.8),
        );
        assert_eq!(tints.get(GlassTintColor::Green).bg.l, 0.1);
        assert_eq!(tints.get(GlassTintColor::Indigo).bg.l, 0.8);
    }

    // ── Focus ring ───────────────────────────────────────────────────────

    #[test]
    fn focus_ring_shadows_default_uses_accent_color_and_is_solid() {
        let theme = TahoeTheme::dark();
        let shadows = theme.focus_ring_shadows();
        assert_eq!(shadows.len(), 2);
        let outer = &shadows[0];
        let inner = &shadows[1];
        assert_eq!(outer.color.h, theme.focus_ring_color.h);
        assert!((outer.color.a - 1.0).abs() < f32::EPSILON);
        assert_eq!(f32::from(outer.blur_radius), 0.0);
        let expected_outer = f32::from(theme.focus_ring_offset) + f32::from(theme.focus_ring_width);
        assert!((f32::from(outer.spread_radius) - expected_outer).abs() < f32::EPSILON);
        assert!((f32::from(inner.blur_radius)).abs() < f32::EPSILON);
        assert!(
            (f32::from(inner.spread_radius) - f32::from(theme.focus_ring_offset)).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn focus_ring_shadows_increase_contrast_remains_solid() {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::INCREASE_CONTRAST;
        let shadows = theme.focus_ring_shadows();
        assert!((shadows[0].color.a - 1.0).abs() < f32::EPSILON);
        assert_eq!(f32::from(shadows[0].blur_radius), 0.0);
    }

    #[test]
    fn focus_ring_shadows_glass_is_solid_not_translucent() {
        let theme = TahoeTheme::liquid_glass();
        let shadows = theme.focus_ring_shadows();
        assert!((shadows[0].color.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn focus_ring_offset_defaults_to_three_points() {
        let theme = TahoeTheme::dark();
        assert!((f32::from(theme.focus_ring_offset) - 3.0).abs() < f32::EPSILON);
        assert!((f32::from(theme.focus_ring_width) - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn layout_direction_default_is_ltr() {
        let theme = TahoeTheme::dark();
        assert!(!theme.is_rtl());
    }

    // ── BlurEffect / LensEffect → gpui::* conversion ─────────────────────

    #[test]
    fn blur_effect_from_sets_kernel_levels_and_pixel_wraps_radius() {
        use super::{BlurEffect, DEFAULT_BLUR_KERNEL_LEVELS};
        let effect = BlurEffect {
            radius: 24.0,
            corner_radius: 12.0,
            tint: gpui::hsla(0.0, 0.0, 0.0, 0.2),
        };
        let gpui_effect = gpui::BlurEffect::from(&effect);
        assert_eq!(gpui_effect.radius, px(24.0));
        assert_eq!(gpui_effect.kernel_levels, DEFAULT_BLUR_KERNEL_LEVELS);
        assert_eq!(gpui_effect.tint, effect.tint);
    }

    #[test]
    fn lens_effect_from_passes_normalized_refraction_and_dispersion() {
        use super::{BlurEffect, LensEffect};
        let effect = LensEffect {
            blur: BlurEffect {
                radius: 12.0,
                corner_radius: 16.0,
                tint: gpui::hsla(0.0, 0.0, 0.0, 0.2),
            },
            refraction: 1.0,
            dispersion: 0.25,
            depth: 16.0,
            splay: 6.0,
            light_angle: -45.0,
            light_intensity: 0.67,
        };
        let gpui_effect = gpui::LensEffect::from(&effect);
        assert!((gpui_effect.refraction - 1.0).abs() < f32::EPSILON);
        assert!((gpui_effect.dispersion - 0.25).abs() < f32::EPSILON);
        assert!((gpui_effect.depth - 0.16).abs() < f32::EPSILON);
    }

    #[test]
    fn lens_effect_from_converts_degrees_to_radians() {
        use super::{BlurEffect, LensEffect};
        let effect = LensEffect {
            blur: BlurEffect {
                radius: 12.0,
                corner_radius: 16.0,
                tint: gpui::hsla(0.0, 0.0, 0.0, 0.2),
            },
            refraction: 1.0,
            dispersion: 0.0,
            depth: 16.0,
            splay: 6.0,
            light_angle: -45.0,
            light_intensity: 0.67,
        };
        let gpui_effect = gpui::LensEffect::from(&effect);
        let expected = -std::f32::consts::FRAC_PI_4;
        assert!((gpui_effect.light_angle.0 - expected).abs() < 1e-6);
    }

    #[test]
    fn lens_effect_from_wraps_splay_in_pixels_and_sets_kernel_levels() {
        use super::{BlurEffect, DEFAULT_BLUR_KERNEL_LEVELS, LensEffect};
        let effect = LensEffect {
            blur: BlurEffect {
                radius: 12.0,
                corner_radius: 16.0,
                tint: gpui::hsla(0.0, 0.0, 0.0, 0.2),
            },
            refraction: 1.0,
            dispersion: 0.0,
            depth: 16.0,
            splay: 6.0,
            light_angle: -45.0,
            light_intensity: 0.67,
        };
        let gpui_effect = gpui::LensEffect::from(&effect);
        assert_eq!(gpui_effect.splay, px(6.0));
        assert_eq!(gpui_effect.kernel_levels, DEFAULT_BLUR_KERNEL_LEVELS);
    }

    // ── Standard Material layering ───────────────────────────────────────

    #[test]
    fn material_regular_uses_standard_fill_not_glass_regular() {
        use super::MaterialThickness;
        let theme = TahoeTheme::dark();
        let regular = theme.glass.material_bg(MaterialThickness::Regular);
        let glass_regular = theme.glass.fill(Glass::Regular);
        assert_ne!(
            regular, glass_regular,
            "MaterialThickness::Regular must not alias Glass::Regular fill"
        );
        assert!(
            (regular.a - 0.29).abs() < 1e-3,
            "dark Regular alpha {}",
            regular.a
        );
        assert!(
            regular.l < 0.05,
            "dark Regular should be near-black, got {}",
            regular.l
        );
    }

    #[test]
    fn material_regular_light_matches_f6_at_60_percent() {
        use super::MaterialThickness;
        let theme = TahoeTheme::light();
        let regular = theme.glass.material_bg(MaterialThickness::Regular);
        assert!(
            (regular.a - 0.60).abs() < 1e-3,
            "light Regular alpha {}",
            regular.a
        );
        assert!(
            regular.l > 0.95,
            "light Regular should be near-white, got {}",
            regular.l
        );
    }

    #[test]
    fn material_regular_ordering_is_monotonic() {
        use super::MaterialThickness;
        let theme = TahoeTheme::dark();
        let a = |t: MaterialThickness| theme.glass.material_bg(t).a;
        assert!(a(MaterialThickness::UltraThin) < a(MaterialThickness::Thin));
        assert!(a(MaterialThickness::Thin) < a(MaterialThickness::Regular));
        assert!(a(MaterialThickness::Regular) < a(MaterialThickness::Thick));
        assert!(a(MaterialThickness::Thick) < a(MaterialThickness::UltraThick));
    }

    // ── HUD tint algebra ─────────────────────────────────────────────────

    #[test]
    fn hud_tint_alpha_matches_nspanel_hud_window() {
        assert!((HUD_TINT_ALPHA - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn hud_fill_is_darker_than_regular_fill_across_themes() {
        use crate::foundations::color::relative_luminance;

        for theme in [
            TahoeTheme::liquid_glass(),
            TahoeTheme::dark(),
            TahoeTheme::light(),
        ] {
            let base = theme.glass.fill(Glass::Regular);
            let hud = hud_fill(&theme);
            assert!(
                relative_luminance(hud) < relative_luminance(base),
                "HUD fill must be darker than Regular base"
            );
        }
    }

    #[test]
    fn hud_fill_preserves_base_alpha() {
        for theme in [
            TahoeTheme::liquid_glass(),
            TahoeTheme::dark(),
            TahoeTheme::light(),
        ] {
            let base = theme.glass.fill(Glass::Regular);
            let hud = hud_fill(&theme);
            assert!((hud.a - base.a).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn hud_fill_over_reduced_transparency_inherits_fallback_alpha() {
        for base_theme in [TahoeTheme::liquid_glass(), TahoeTheme::light()] {
            let mut theme = base_theme;
            theme.accessibility_mode = AccessibilityMode::REDUCE_TRANSPARENCY;
            let base = theme.glass.accessibility.reduced_transparency_bg;
            let hud = hud_fill(&theme);
            assert!(
                (hud.a - base.a).abs() < f32::EPSILON,
                "reduced-transparency HUD fill must inherit fallback alpha",
            );
        }
    }

    #[test]
    fn hud_fill_plus_layer_two_lands_at_effective_hud_tint_alpha() {
        let effective = 1.0 - (1.0 - HUD_PRE_COMPOSE_ALPHA) * (1.0 - GLASS_LAYER_TINT_ALPHA);
        assert!(
            (effective - HUD_TINT_ALPHA).abs() < 1e-6,
            "effective tint ({}) must match HUD_TINT_ALPHA ({})",
            effective,
            HUD_TINT_ALPHA,
        );
    }

    // ── GlassStyle::labels() ──────────────────────────────────────────────

    #[test]
    fn labels_returns_dim_for_glass_dim() {
        use super::SurfaceContext;
        let theme = TahoeTheme::dark();
        let labels = theme.glass.labels(SurfaceContext::GlassDim);
        assert_eq!(labels.primary, theme.glass.labels_dim.primary);
    }

    #[test]
    fn labels_returns_bright_for_glass_bright() {
        use super::SurfaceContext;
        let theme = TahoeTheme::dark();
        let labels = theme.glass.labels(SurfaceContext::GlassBright);
        assert_eq!(labels.primary, theme.glass.labels_bright.primary);
    }

    // ── Backdrop helpers ────────────────────────────────────────────────

    #[test]
    fn default_backdrop_blur_effect_uses_overlay_bg_tint() {
        let theme = TahoeTheme::dark();
        let effect = super::default_backdrop_blur_effect(&theme);
        assert_eq!(effect.tint, theme.overlay_bg);
    }

    #[test]
    fn default_backdrop_blur_effect_is_heavy_and_full_bleed() {
        let theme = TahoeTheme::liquid_glass();
        let effect = super::default_backdrop_blur_effect(&theme);
        assert!((effect.radius - 40.0).abs() < f32::EPSILON);
        assert!(effect.corner_radius.abs() < f32::EPSILON);
    }

    #[test]
    fn backdrop_blur_overlay_builds_without_panic() {
        let theme = TahoeTheme::liquid_glass();
        let effect = super::default_backdrop_blur_effect(&theme);
        let _div: gpui::Div = super::backdrop_blur_overlay(&theme, &effect);
    }

    #[test]
    fn backdrop_blur_overlay_reduce_transparency_builds_without_panic() {
        let mut theme = TahoeTheme::liquid_glass();
        theme.accessibility_mode = AccessibilityMode::REDUCE_TRANSPARENCY;
        let effect = super::default_backdrop_blur_effect(&theme);
        let _div: gpui::Div = super::backdrop_blur_overlay(&theme, &effect);
    }

    // ── Scroll edge ──────────────────────────────────────────────────────

    #[test]
    fn scroll_edge_height_constants_are_finite_and_ordered() {
        use super::{SCROLL_EDGE_HEIGHT, SCROLL_EDGE_HEIGHT_COMPACT};
        let default_h: f32 = SCROLL_EDGE_HEIGHT.into();
        let compact_h: f32 = SCROLL_EDGE_HEIGHT_COMPACT.into();
        assert!(default_h.is_finite() && default_h > 0.0);
        assert!(compact_h.is_finite() && compact_h > 0.0);
        assert!(default_h >= compact_h);
    }

    #[test]
    fn scroll_edge_style_defaults_to_soft() {
        use super::ScrollEdgeStyle;
        assert_eq!(ScrollEdgeStyle::default(), ScrollEdgeStyle::Soft);
    }

    #[test]
    fn scroll_edge_overlays_build_with_custom_height() {
        use super::{
            SCROLL_EDGE_HEIGHT_COMPACT, ScrollEdgeStyle, scroll_edge_bottom, scroll_edge_top,
        };
        let theme = TahoeTheme::dark();
        let _top = scroll_edge_top(&theme, SCROLL_EDGE_HEIGHT_COMPACT, ScrollEdgeStyle::Soft);
        let _bottom = scroll_edge_bottom(&theme, px(24.0), ScrollEdgeStyle::Hard);
    }

    // ── clear_glass_dimmed ───────────────────────────────────────────────

    #[test]
    fn clear_glass_dimmed_builds_without_panic() {
        let theme = TahoeTheme::liquid_glass();
        let glass = glass_effect(
            gpui::div(),
            &theme,
            Glass::Clear,
            Shape::Default,
            Elevation::Resting,
        );
        let _wrapped: gpui::Div = super::clear_glass_dimmed(glass);
    }
}

#[cfg(test)]
mod resolve_focused_tests {
    use super::resolve_focused;
    use crate::test_helpers::helpers::setup_test_window;
    use gpui::prelude::*;
    use gpui::{Context, FocusHandle, IntoElement, Render, TestAppContext, Window, div, px};

    struct Harness {
        handle: FocusHandle,
    }

    impl Harness {
        fn new(cx: &mut Context<Self>) -> Self {
            Self {
                handle: cx.focus_handle(),
            }
        }
    }

    impl Render for Harness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(px(200.0))
                .h(px(80.0))
                .id("harness-root")
                .track_focus(&self.handle)
        }
    }

    #[gpui::test]
    async fn none_handle_returns_fallback(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| Harness::new(cx));
        host.update_in(cx, |_host, window, _cx| {
            assert!(resolve_focused(None, window, true));
            assert!(!resolve_focused(None, window, false));
        });
    }

    #[gpui::test]
    async fn some_focused_handle_overrides_fallback(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| Harness::new(cx));
        host.update_in(cx, |host, window, cx| {
            window.focus(&host.handle, cx);
            assert!(host.handle.is_focused(window));
            assert!(resolve_focused(Some(&host.handle), window, false));
        });
    }

    #[gpui::test]
    async fn some_unfocused_handle_overrides_true_fallback(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| Harness::new(cx));
        host.update_in(cx, |host, window, _cx| {
            assert!(!host.handle.is_focused(window));
            assert!(!resolve_focused(Some(&host.handle), window, true));
        });
    }
}
