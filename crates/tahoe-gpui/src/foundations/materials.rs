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
//! - `glass_surface()` — Liquid Glass panel (controls/navigation)
//! - `glass_or_surface()` — Glass with accessibility fallback
//! - `tinted_glass_surface()` — Colored Liquid Glass
//! - `accent_tinted_glass_surface()` — Accent-colored Liquid Glass
//! - `glass_clear_surface()` — Clear variant for media backgrounds
//! - `glass_shaped_surface()` — Glass with HIG shape (Capsule, Concentric)
//! - `glass_blur_surface()` — Per-element blur (blur primitive pending
//!   GPUI; falls back to `glass_surface`)
//! - `glass_lens_surface()` — Per-element refraction (blur primitive
//!   pending GPUI; falls back to `glass_surface`)
//! - `backdrop_overlay()` — Full-viewport modal backdrop scrim. Routes
//!   through `backdrop_blur_overlay()` so every modal auto-upgrades
//!   once the GPUI blur primitive lands.
//! - `backdrop_blur_overlay()` — Full-viewport backdrop with explicit
//!   [`BlurEffect`]; today emits the tint and records the intent.
//!
//! # Current rendering limitation
//!
//! GPUI exposes no `paint_blur_rect()` / backdrop-filter primitive, so every
//! surface function in this module is a translucent tinted fill plus
//! shadows — no per-element compositing of the content behind the element.
//! On macOS the library installs `WindowBackgroundAppearance::Blurred`
//! (NSVisualEffectView), so glass is translucent to the **desktop wallpaper
//! behind the window** but NOT to sibling GPUI elements in the same window.
//! Place glass directly on the window root for meaningful translucency; over
//! dense content it reads as a tinted rectangle. See [`glass_surface`] for
//! the full caveat and guidance.
//!
//! # Accessibility
//!
//! - **ReduceTransparency**: Glass replaced with opaque fills
//! - **IncreaseContrast**: Visible borders added via `apply_high_contrast_border()`
//! - **ReduceMotion**: Animation durations set to 0 via `effective_duration()`
//!
//! # GPU Pipeline Extension (Future)
//!
//! When GPUI gains render-to-texture support, the blur and lens effects
//! will be implemented as new scene primitives:
//!
//! ## BlurRect -- Dual Kawase Backdrop Blur
//!
//! 1. End current render pass
//! 2. Copy framebuffer region under bounds to downsample chain (3-4 levels)
//! 3. Run Dual Kawase downsample shader per level (5 tex samples, ~10 lines WGSL)
//! 4. Run Dual Kawase upsample shader per level (8 tex samples, ~15 lines WGSL)
//! 5. Composite blurred result with tint overlay and corner_radius SDF mask
//! 6. Resume main render pass
//!
//! ## LensRect -- Glass Refraction (extends BlurRect)
//!
//! Additional fragment shader work:
//! - Parabolic UV distortion: `offset = (1 - dist^2) * direction * strength`
//! - Chromatic aberration: sample R/G/B at offset UVs
//! - Fresnel edge highlight: `smoothstep(edge, 0, sdf) * dot(normal, light)`
//!
//! ## Cross-Platform Shaders
//!
//! | Platform | Shader Lang | Backend |
//! |----------|------------|---------|
//! | macOS    | Metal / WGSL | gpui_macos / gpui_wgpu |
//! | Linux    | WGSL (Vulkan) | gpui_wgpu |
//! | Windows  | HLSL / WGSL | gpui_windows / gpui_wgpu |
//! | Web      | WGSL (WebGPU) | gpui_web |
//!
//! Performance: Dual Kawase is <1ms per element at 1920x1080 (GTX 1060 class).
//! Budget at 120fps (8.3ms/frame) supports multiple glass elements.
//!
//! The wgpu renderer already implements multi-pass rendering for path
//! compositing (`path_intermediate_texture`), so the architectural
//! pattern for blur passes exists -- it needs to be generalized.
//!
//! # Example
//!
//! ```ignore
//! let theme = cx.theme();
//! let card = glass_surface(div().p(px(16.0)), theme, GlassSize::Medium);
//! // For custom colors:
//! let bg = theme.glass.accessible_bg(GlassSize::Small, theme.accessibility_mode);
//! let bg = glass.accessible_bg(GlassSize::Small, theme.accessibility_mode);
//! ```

use gpui::prelude::*;
use gpui::{
    AnimationExt, AnyElement, BoxShadow, Deferred, Div, ElementId, Hsla, Pixels, SharedString,
    deferred, hsla, px,
};

use crate::foundations::accessibility::{AccessibilityMode, AccessibilityTokens};
use crate::foundations::layout::ShapeType;
use crate::foundations::motion::{
    MorphState, MotionTokens, REDUCE_MOTION_CROSSFADE, accessible_spring_animation,
};
use crate::foundations::theme::{ActiveTheme, TahoeTheme};

/// Alpha of the Apple Liquid Glass "Layer 2" tint (`#000000 @ 20%`).
///
/// Applied in linear-light RGB by `apply_glass_chrome` so the glass darkening
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

/// Glass surface size variant per HIG.
/// Small = tab bars, toolbars, buttons. Medium = sidebars, cards. Large = sheets, modals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GlassSize {
    Small,
    Medium,
    Large,
}

/// Glass material variant per HIG.
/// Regular = full adaptive glass with lensing. Clear = more transparent, for media-rich content only.
/// Identity = no glass effect (conditional disable).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GlassVariant {
    Regular,
    Clear,
    /// No glass effect — pass-through. Use to conditionally disable glass.
    Identity,
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
    /// the content under the chrome still reads through. Currently
    /// routes to the same `thin_bg` fill as `Thin`; a dedicated
    /// `chrome_bg` theme token is tracked as a future refinement once
    /// toolbars consume this variant broadly.
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
/// When GPUI gains render-to-texture support, this will drive a Dual Kawase
/// blur pass on the framebuffer region behind the element. Currently falls
/// back to the standard glass surface styling (translucent fill + shadows).
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

impl BlurEffect {
    /// Create a blur effect matching the given glass size.
    pub fn for_glass_size(size: GlassSize, theme: &TahoeTheme) -> Self {
        let (radius, tint) = match size {
            GlassSize::Small => (20.0, theme.glass.bg(size)),
            GlassSize::Medium => (30.0, theme.glass.bg(size)),
            GlassSize::Large => (40.0, theme.glass.bg(size)),
        };
        Self {
            radius,
            corner_radius: f32::from(theme.glass.radius(size)),
            tint,
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
/// Currently falls back to standard glass surface styling until GPUI
/// gains render-to-texture support for real-time refraction.
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

impl LensEffect {
    /// Figma's default Liquid Glass parameters.
    pub fn liquid_glass(size: GlassSize, theme: &TahoeTheme) -> Self {
        Self {
            blur: BlurEffect {
                radius: 12.0, // Figma Frost: 12
                corner_radius: f32::from(theme.glass.radius(size)),
                tint: theme.glass.bg(size),
            },
            refraction: 1.0,       // Figma: 100
            depth: 16.0,           // Figma: 16
            dispersion: 0.0,       // Figma: 0
            splay: 6.0,            // Figma: 6
            light_angle: -45.0,    // Figma: -45°
            light_intensity: 0.67, // Figma: 67%
        }
    }

    /// Subtle lens effect for small UI elements (buttons, pills).
    pub fn subtle(size: GlassSize, theme: &TahoeTheme) -> Self {
        Self {
            blur: BlurEffect {
                radius: 8.0,
                corner_radius: f32::from(theme.glass.radius(size)),
                tint: theme.glass.bg(size),
            },
            refraction: 0.5,
            depth: 8.0,
            dispersion: 0.0,
            splay: 3.0,
            light_angle: -45.0,
            light_intensity: 0.4,
        }
    }

    /// No refraction — just blur + tint.
    pub fn blur_only(size: GlassSize, theme: &TahoeTheme) -> Self {
        Self {
            blur: BlurEffect::for_glass_size(size, theme),
            refraction: 0.0,
            depth: 0.0,
            dispersion: 0.0,
            splay: 0.0,
            light_angle: -45.0,
            light_intensity: 0.0,
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

    /// Convenience alias — the Liquid Glass size to use at this tier.
    /// `Background` / `Surface` tiers route through `Small` since they
    /// are content-layer and should not actually adopt Liquid Glass
    /// without an explicit override; callers that honour
    /// [`Self::glass_role`] will short-circuit before reading this.
    pub fn glass_size(self) -> GlassSize {
        match self {
            Self::Background | Self::Surface => GlassSize::Small,
            Self::ElevatedSurface => GlassSize::Small,
            Self::ModalSurface => GlassSize::Medium,
            Self::OverlaySurface => GlassSize::Large,
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
/// Apple defines three size variants (Small/Medium/Large), each with distinct
/// fill opacities and shadow sets. The specular edge effect comes from
/// multi-layer box shadows, not gradient lines or CSS borders.
///
/// # Usage
///
/// ```ignore
/// let glass = &theme.glass;
/// let bg = glass.bg(GlassSize::Medium);
/// let shadows = glass.shadows(GlassSize::Medium).to_vec();
/// ```
#[derive(Debug, Clone)]
pub struct GlassStyle {
    // Material variant
    pub variant: GlassVariant,

    // Per-size surface fills (Regular)
    pub small_bg: Hsla,
    pub medium_bg: Hsla,
    pub large_bg: Hsla,
    // Per-size surface fills (Clear)
    pub clear_small_bg: Hsla,
    pub clear_medium_bg: Hsla,
    pub clear_large_bg: Hsla,
    pub hover_bg: Hsla,

    // Per-thickness fills (HIG Standard Materials).
    //
    // These are *standard-material* fills for the content layer
    // (backdrop-blur + #F6F6F6 / #000000 at varying alpha) — distinct from
    // the Liquid Glass fills above (`small_bg` / `medium_bg` / `large_bg`),
    // which are for the controls/navigation layer. Mixing the two violates
    // the HIG layering rule: glass sits above content, content sits above
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

    // Pre-built shadow sets per size (Apple's specular edge effect)
    pub small_shadows: Vec<BoxShadow>,
    pub medium_shadows: Vec<BoxShadow>,
    pub large_shadows: Vec<BoxShadow>,

    // Per-size corner radii (from Figma Tahoe UI Kit)
    pub small_radius: Pixels,
    pub medium_radius: Pixels,
    pub large_radius: Pixels,

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
    /// Returns the corner radius for the given glass size.
    /// Per Figma Tahoe UI Kit: Small = 20px, Medium/Large = 34px.
    pub fn radius(&self, size: GlassSize) -> Pixels {
        match size {
            GlassSize::Small => self.small_radius,
            GlassSize::Medium => self.medium_radius,
            GlassSize::Large => self.large_radius,
        }
    }

    /// Returns the background fill for the given size variant, respecting the material variant.
    /// `Identity` returns fully transparent (no glass).
    pub fn bg(&self, size: GlassSize) -> Hsla {
        match self.variant {
            GlassVariant::Regular => match size {
                GlassSize::Small => self.small_bg,
                GlassSize::Medium => self.medium_bg,
                GlassSize::Large => self.large_bg,
            },
            GlassVariant::Clear => match size {
                GlassSize::Small => self.clear_small_bg,
                GlassSize::Medium => self.clear_medium_bg,
                GlassSize::Large => self.clear_large_bg,
            },
            GlassVariant::Identity => hsla(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Returns the Regular fill for the given size (ignores variant).
    pub fn regular_bg(&self, size: GlassSize) -> Hsla {
        match size {
            GlassSize::Small => self.small_bg,
            GlassSize::Medium => self.medium_bg,
            GlassSize::Large => self.large_bg,
        }
    }

    /// Returns the Clear fill for the given size (ignores variant).
    pub fn clear_fill(&self, size: GlassSize) -> Hsla {
        match size {
            GlassSize::Small => self.clear_small_bg,
            GlassSize::Medium => self.clear_medium_bg,
            GlassSize::Large => self.clear_large_bg,
        }
    }

    /// Returns the shadow set for the given size variant.
    ///
    /// Callers typically hand this straight to GPUI's `Styled::shadow` via
    /// `.shadow(glass.shadows(size).to_vec())`. The per-frame `Vec` allocation
    /// is a GPUI API constraint — `Styled::shadow` takes `Vec<BoxShadow>` by
    /// value, so a theme-owned `Arc<[BoxShadow]>` can't be reused without an
    /// upstream API change. Keep the borrow-return here so we're not
    /// double-cloning (one internal clone in the theme + one in `.shadow`).
    pub fn shadows(&self, size: GlassSize) -> &[BoxShadow] {
        match size {
            GlassSize::Small => &self.small_shadows,
            GlassSize::Medium => &self.medium_shadows,
            GlassSize::Large => &self.large_shadows,
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

    /// Returns the background for `size`, adjusted for accessibility mode.
    pub fn accessible_bg(&self, size: GlassSize, mode: AccessibilityMode) -> Hsla {
        if mode.reduce_transparency() {
            self.accessibility.reduced_transparency_bg
        } else {
            self.bg(size)
        }
    }

    /// Returns the fill for a given standard material thickness level.
    ///
    /// UltraThin is most transparent, UltraThick is most opaque. These fills
    /// are distinct from the Liquid Glass fills returned by [`Self::bg`]:
    /// standard materials belong to the *content* layer (HIG Materials —
    /// Standard), Liquid Glass to the *controls/navigation* layer (HIG
    /// Materials — Liquid Glass). The two should not be conflated, so
    /// `Regular` routes to its own `medium_standard_bg` token rather than the
    /// Liquid Glass `medium_bg`.
    ///
    /// Note: `UltraThick` dark (`#000000 @50%`) has the same alpha as the
    /// Liquid Glass `small_bg` dark (`#CCCCCC @50%` composite at L≈0.17),
    /// but the fills differ — `UltraThick` is a content-layer background,
    /// not a `GlassSize::Large` alias.
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

/// Private helper: apply the shared glass chrome (bg + radius + shadow +
/// high-contrast border) or the opaque fallback. All three public surface
/// functions delegate to this, eliminating ~40 LOC of duplication.
/// Apple's Liquid Glass is a 2-layer composition (from Figma Tahoe UI Kit):
///
/// **Layer 1 (base):** 3 stacked fills + drop shadow
///   - Bottom fill: `#333333` at 100% — dark underlay
///   - Middle fill: `#FFFFFF` at 50% — white translucent overlay
///   - Top fill: `#F7F7F7` at 100% — light gray surface
///   - Drop shadow for depth
///
///   (Same values for both light and dark — the window bg shows through)
///
/// **Layer 2 (glass effect):** Tint overlay + Figma Glass effect
///   - Fill: `#000000` at 20% — glass tint
///   - Figma "Glass" effect (refraction — approximated with translucency)
///
/// Since GPUI only supports one `bg()` per div, we approximate by rendering
/// the composited result as the primary fill color (stored in `GlassStyle`),
/// plus a subtle tint overlay div for the glass effect layer.
fn apply_glass_chrome(
    mut el: Div,
    theme: &TahoeTheme,
    bg: gpui::Hsla,
    radius: Pixels,
    size: GlassSize,
) -> Div {
    let glass = &theme.glass;

    // Apple Liquid Glass is a 2-layer composition (from Figma Tahoe UI Kit):
    //
    // Layer 1 (base): Fill stack + drop shadow
    //   Default light small: #F7F7F7 @100% + #FFFFFF @50% + #333333 @100%
    //   Default dark small:  #FFFFFF @6% + #000000 @60% + #CCCCCC @50%
    //   Default light/dark medium and large: see theme/mod.rs for exact values
    //   Primary (both light AND dark): #0091FF @100% + #999999 @100% + #FFFFFF @100% + #FFFFFF @50%
    //
    // Layer 2 (glass): #000000 @20% + Figma "Glass" effect
    //   Identical across ALL variants (default, primary, tinted, light, dark).
    //
    // We composite Layer 2 into the `bg` parameter in linear-light RGB so the
    // src-over blend matches Porter–Duff. A naive HSL-space blend diverges
    // from linear-light by up to ~17% on bright surfaces.
    let composited =
        crate::foundations::color::compose_black_tint_linear(bg, GLASS_LAYER_TINT_ALPHA);

    el = el
        .bg(composited)
        .rounded(radius)
        .shadow(glass.shadows(size).to_vec());

    if theme.accessibility_mode.increase_contrast() {
        el = el
            .border_1()
            .border_color(glass.accessibility.high_contrast_border);
    } else if matches!(size, GlassSize::Medium | GlassSize::Large) {
        // Approximate Apple's specular inner-edge highlight on Liquid Glass.
        // A real inset highlight needs `BoxShadow::inset`, which GPUI does
        // not yet expose; until then a 1px translucent top border gives the
        // perceived "frosted edge" on large panels without producing a
        // distinct outlined look. Skipped at `GlassSize::Small` because the
        // 20pt corner radius leaves too little straight-edge surface for
        // the highlight to read cleanly — small pill/tab controls stay
        // visually clean.
        el = el.border_t(px(1.0)).border_color(hsla(0.0, 0.0, 1.0, 0.18));
    }
    el
}

/// Resolve the glass background color respecting ReduceTransparency.
///
/// Accepts a `&GlassStyle` reference directly so the precondition is
/// enforced by the type system rather than a runtime assertion.
fn default_glass_bg(glass: &GlassStyle, mode: AccessibilityMode, size: GlassSize) -> gpui::Hsla {
    if mode.reduce_transparency() {
        glass.accessibility.reduced_transparency_bg
    } else {
        glass.bg(size)
    }
}

/// Applies Liquid Glass surface styling to a div.
///
/// Applies glass background, radius, and per-size Apple shadow set
/// (no border -- Apple uses shadows for edge definition).
/// Respects accessibility mode: ReduceTransparency uses frosted fills,
/// IncreaseContrast adds a visible border.
///
/// # ⚠️ Current limitation: no per-element compositing
///
/// GPUI exposes no `paint_blur_rect()` / backdrop-filter primitive, so this
/// function cannot composite a blurred sample of the content behind the
/// element into its fill. The rendering is a translucent tinted fill plus
/// per-size shadows — nothing more. On macOS, the library installs
/// `WindowBackgroundAppearance::Blurred` (NSVisualEffectView) at the window
/// level, so glass surfaces are translucent to the **desktop wallpaper
/// behind the window** but NOT to sibling GPUI elements inside the same
/// window. A glass card placed directly over a list renders as a tinted
/// rectangle, not true Liquid Glass.
///
/// For meaningful translucency, place glass surfaces directly on the window
/// root background (see `examples/liquid_glass_gallery.rs` for the pattern).
/// [`glass_blur_surface`] and [`glass_lens_surface`] fall back to this
/// function for the same reason; [`backdrop_blur_overlay`] documents the
/// same gap for full-viewport scrims. The upstream tracking task is a GPUI
/// PR that lands a rect-level blur entry point.
///
/// **Note per HIG:** Don't use Liquid Glass in the content layer.
/// Use glass surfaces only for controls, navigation elements, and overlays.
/// For content backgrounds, use `theme.background` or `theme.surface` instead.
pub fn glass_surface(el: Div, theme: &TahoeTheme, size: GlassSize) -> Div {
    let glass = &theme.glass;
    let bg = default_glass_bg(glass, theme.accessibility_mode, size);
    let radius = glass.radius(size);
    apply_glass_chrome(el, theme, bg, radius, size)
}

/// Applies Liquid Glass at a specific material thickness level.
///
/// Per HIG Materials: thickness controls the frosting intensity.
/// UltraThin is most transparent, UltraThick is most opaque.
/// Use this when you need explicit control over material depth
/// (e.g., background panels vs floating overlays).
///
/// Glass is always present per HIG macOS Tahoe.
///
/// See [`glass_surface`] for the current GPUI backdrop-blur limitation.
pub fn glass_surface_thick(
    el: Div,
    theme: &TahoeTheme,
    thickness: MaterialThickness,
    size: GlassSize,
) -> Div {
    let glass = &theme.glass;
    let bg = if theme.accessibility_mode.reduce_transparency() {
        glass.accessibility.reduced_transparency_bg
    } else {
        glass.material_bg(thickness)
    };
    let radius = glass.radius(size);
    apply_glass_chrome(el, theme, bg, radius, size)
}

/// Applies Liquid Glass tinted surface styling to a div.
///
/// Uses the provided `GlassTint` background color instead of the neutral glass fill.
/// Respects accessibility mode: ReduceTransparency increases tint opacity
/// (alpha × 3, capped at 0.5), IncreaseContrast adds a visible border.
///
/// See [`glass_surface`] for the current GPUI backdrop-blur limitation.
pub fn tinted_glass_surface(el: Div, theme: &TahoeTheme, tint: &GlassTint, size: GlassSize) -> Div {
    let bg = if theme.accessibility_mode.reduce_transparency() {
        let mut higher = tint.bg;
        higher.a = (higher.a * 3.0).min(0.5);
        higher
    } else {
        tint.bg
    };
    let radius = theme.glass.radius(size);
    apply_glass_chrome(el, theme, bg, radius, size)
}

/// Apply Clear glass surface styling -- higher transparency for media-rich content.
/// Per HIG, consider adding a dark dimming layer for bright backgrounds.
///
/// Respects accessibility mode: ReduceTransparency uses an opaque fallback,
/// IncreaseContrast adds a visible border (via `apply_glass_chrome`).
///
/// See [`glass_surface`] for the current GPUI backdrop-blur limitation.
pub fn glass_clear_surface(el: Div, theme: &TahoeTheme, size: GlassSize) -> Div {
    let glass = &theme.glass;
    let bg = if theme.accessibility_mode.reduce_transparency() {
        glass.accessibility.reduced_transparency_bg
    } else {
        glass.clear_fill(size)
    };
    let radius = glass.radius(size);
    apply_glass_chrome(el, theme, bg, radius, size)
}

/// Dark translucent tint applied on top of [`glass_surface`] so HUD
/// surfaces render dark regardless of the current appearance.
///
/// Composed as `black @ 60%` to match `NSPanel.StyleMask.HUDWindow`
/// per HIG `#panels`. Exposed as a constant so callers that need the
/// raw value (e.g. tinting a sub-element consistently with the HUD
/// backdrop) can re-use the exact recipe.
pub const HUD_TINT_ALPHA: f32 = 0.6;

/// Apply Liquid Glass HUD surface styling to a div.
///
/// Composes the standard [`glass_surface`] chrome (bg + radius +
/// shadows + high-contrast border) with the dark translucent HUD tint
/// ([`HUD_TINT_ALPHA`]) and [`TahoeTheme::background`] as the text
/// color, so the surface reads as a dark HUD regardless of the
/// current appearance. Matches `NSPanel.StyleMask.HUDWindow` per HIG
/// `#panels`.
///
/// Respects accessibility the same way [`glass_surface`] does:
/// ReduceTransparency routes through the opaque fallback fill and
/// IncreaseContrast adds a visible border. Inherits the current GPUI
/// backdrop-blur limitation from [`glass_surface`].
pub fn glass_surface_hud(el: Div, theme: &TahoeTheme, size: GlassSize) -> Div {
    glass_surface(el, theme, size)
        .bg(hsla(0.0, 0.0, 0.0, HUD_TINT_ALPHA))
        .text_color(theme.background)
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
// use tahoe_gpui::foundations::materials::glass_surface;
//
// GlassSurfaceScope::new(
//     glass_surface(div(), theme, GlassSize::Medium)
//         .child(Icon::new(IconName::Star))
// )
// ```
//
// Keeping scope separate from the non-scoped `glass_surface*` functions
// means callers who only need the chrome (no icon propagation) keep the
// `Div -> Div` signature and its chain-ability; and callers who want the
// full propagation compose with one extra `GlassSurfaceScope::new(…)`.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Apply per-element glass blur effect to a div.
///
/// # ⚠️ Backdrop blur is not yet implemented
///
/// GPUI does not currently expose `paint_blur_rect()`, so this function falls
/// back to [`glass_surface`] — a translucent fill plus shadows. Callers get
/// the correct glass *chrome* but no real backdrop blur; on dark backgrounds
/// the visual difference is subtle, on content-heavy backgrounds it is not.
/// The `effect.radius` is unused in the fallback path.
///
/// The reference shader lives at `src/foundations/shaders/dual_kawase.wgsl`
/// and is the target of the GPUI upstream PR that unblocks this call.
///
/// Accessibility: falls back to the opaque ReduceTransparency fill
/// irrespective of blur support — users who disable transparency never see
/// the blur either way.
pub fn glass_blur_surface(
    el: Div,
    theme: &TahoeTheme,
    effect: &BlurEffect,
    size: GlassSize,
) -> Div {
    // Accessibility: opaque fallback for ReduceTransparency
    if theme.accessibility_mode.reduce_transparency() {
        return el
            .bg(theme.glass.accessibility.reduced_transparency_bg)
            .rounded(px(effect.corner_radius));
    }

    warn_blur_fallback_once("glass_blur_surface");

    // GPUI blocker: no per-element backdrop-blur API exists today.
    // GPUI ships `Window::set_background_appearance(WindowBackgroundAppearance::Blurred)`
    // which wraps the private `CGSSetWindowBackgroundBlurRadius` on
    // macOS (see Zed `crates/gpui_macos/src/window.rs:~1050`); that is a
    // window-level blur, not a rect-level one. Fall back to
    // [`glass_surface`] which pairs that window blur with translucent
    // fills — visually close on simple surfaces but loses the
    // per-element corner radius / tint control a future
    // `paint_blur_rect` would provide.
    glass_surface(el, theme, size)
}

/// Apply per-element glass lens effect to a div.
///
/// # ⚠️ Refraction/lens rendering is not yet implemented
///
/// GPUI does not currently expose a render-to-texture path, so this function
/// falls back to [`glass_surface`] (translucent fill + shadows). The
/// `effect.refraction`, `effect.dispersion`, `effect.depth`, `effect.splay`,
/// `effect.light_angle`, and `effect.light_intensity` values are encoded but
/// not sampled until the shader at `src/foundations/shaders/glass_composite.wgsl`
/// is compiled and wired up.
///
/// When the pipeline lands, this function should apply:
/// 1. Dual Kawase backdrop blur (`dual_kawase.wgsl`)
/// 2. Parabolic UV distortion (refraction, depth-scaled)
/// 3. Chromatic aberration (dispersion)
/// 4. Directional Fresnel edge highlight (light_angle, light_intensity,
///    splay)
///
/// Accessibility: falls back to the opaque ReduceTransparency fill
/// irrespective of lens support.
pub fn glass_lens_surface(
    el: Div,
    theme: &TahoeTheme,
    effect: &LensEffect,
    size: GlassSize,
) -> Div {
    // Accessibility: opaque fallback
    if theme.accessibility_mode.reduce_transparency() {
        return el
            .bg(theme.glass.accessibility.reduced_transparency_bg)
            .rounded(px(effect.blur.corner_radius));
    }

    warn_blur_fallback_once("glass_lens_surface");

    // GPUI blocker: lens rendering needs a render-to-texture pass GPUI
    // doesn't expose today. The shader stack (dual Kawase blur,
    // parabolic UV distortion, chromatic aberration, directional
    // Fresnel) lives in `src/foundations/shaders/glass_composite.wgsl`
    // and is staged for when GPUI lands a `paint_lens_rect` entry
    // point. Until then fall through to the standard glass surface —
    // equivalent coverage for the blur + tint layers, no refraction or
    // dispersion. Zed's materials system makes the same trade-off
    // (window-level blur only; see investigation notes in commit
    // history).
    let _ = (
        effect.refraction,
        effect.depth,
        effect.dispersion,
        effect.splay,
        effect.light_intensity,
        effect.light_angle,
    );
    glass_surface(el, theme, size)
}

/// Emit a one-shot warning the first time a glass blur/lens surface falls
/// back to the non-blurred implementation. A per-fn `OnceLock` keeps the log
/// out of hot paths while still flagging the missing GPU pipeline clearly to
/// anyone running with `RUST_LOG`-style diagnostics.
#[cfg(debug_assertions)]
fn warn_blur_fallback_once(fn_name: &'static str) {
    use std::sync::OnceLock;
    static BLUR_WARN: OnceLock<()> = OnceLock::new();
    static LENS_WARN: OnceLock<()> = OnceLock::new();
    let slot = match fn_name {
        "glass_blur_surface" => &BLUR_WARN,
        "glass_lens_surface" => &LENS_WARN,
        _ => return,
    };
    slot.get_or_init(|| {
        eprintln!(
            "[tahoe-gpui] {fn_name}: per-element backdrop blur is not yet \
             implemented; falling back to glass_surface(). Track the GPUI \
             upstream paint_blur_rect() / paint_lens_rect() contribution to \
             re-enable real refractive rendering."
        );
    });
}

#[cfg(not(debug_assertions))]
fn warn_blur_fallback_once(_fn_name: &'static str) {}

/// Apply accent-tinted glass surface styling.
/// Uses the theme's accent color as the glass tint, suitable for
/// primary action areas like toolbars and navigation bars.
///
/// See [`glass_surface`] for the current GPUI backdrop-blur limitation.
pub fn accent_tinted_glass_surface(el: Div, theme: &TahoeTheme, size: GlassSize) -> Div {
    tinted_glass_surface(el, theme, &theme.glass.accent_tint, size)
}

/// Applies Liquid Glass surface styling with a specific shape type.
///
/// Combines concentricity-based radius calculation with glass surface styling.
/// Use this when a component needs a specific HIG shape (Fixed, Capsule, Concentric).
///
/// See [`glass_surface`] for the current GPUI backdrop-blur limitation.
pub fn glass_shaped_surface(
    el: Div,
    theme: &TahoeTheme,
    size: GlassSize,
    shape: ShapeType,
    container_height: Option<Pixels>,
) -> Div {
    let glass = &theme.glass;
    let bg = default_glass_bg(glass, theme.accessibility_mode, size);
    let radius = compute_shape_radius(theme, shape, container_height);
    apply_glass_chrome(el, theme, bg, radius, size)
}

/// Computes the corner radius for an HIG shape type.
///
/// - **Fixed**: Returns the constant radius.
/// - **Capsule**: Returns half the container height (pill shape).
/// - **Concentric**: Returns `parent_radius - padding`, minimum 0.
pub fn compute_shape_radius(
    theme: &TahoeTheme,
    shape: ShapeType,
    container_height: Option<Pixels>,
) -> Pixels {
    match shape {
        ShapeType::Fixed(r) => r,
        ShapeType::Capsule => container_height.map_or(theme.radius_full, |h| h / 2.0),
        ShapeType::Concentric {
            parent_radius,
            padding,
        } => (parent_radius - padding).max(px(0.0)),
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

/// Apply Liquid Glass surface styling per HIG.
///
/// Applies glass background, radius, shadows, and high-contrast border.
/// Per HIG macOS Tahoe, glass is always present.
///
/// This is the most common styling pattern in the crate -- use it for any container
/// that needs glass surface treatment.
///
/// See [`glass_surface`] for the current GPUI backdrop-blur limitation.
pub fn glass_or_surface<E: gpui::Styled>(mut el: E, theme: &TahoeTheme, size: GlassSize) -> E {
    let glass = &theme.glass;
    el = el
        .bg(glass.accessible_bg(size, theme.accessibility_mode))
        .rounded(glass.radius(size))
        .shadow(glass.shadows(size).to_vec());
    el = apply_high_contrast_border(el, theme);
    el
}

pub use super::accessibility::{apply_high_contrast_border, effective_duration};

/// Apply the standard glass-control styling triplet.
///
/// Combines [`glass_or_surface`] (bg + radius + shadow + high-contrast border)
/// with the focus ring when `focused`. Use this for any control whose chrome
/// is a glass trigger at `size` -- popup buttons, pickers, date/time pickers,
/// combo boxes, steppers, and similar.
///
/// This replaces the prior triplet:
/// ```ignore
/// el = glass_or_surface(el, theme, size);
/// el = apply_focus_ring(el, theme, focused, theme.glass.shadows(size));
/// el = apply_high_contrast_border(el, theme);
/// ```
///
/// The high-contrast border is applied exactly once (inside `glass_or_surface`);
/// the focus ring layers on top of the base shadows without re-assigning them.
pub fn apply_standard_control_styling<E: gpui::Styled>(
    mut el: E,
    theme: &TahoeTheme,
    size: GlassSize,
    focused: bool,
) -> E {
    el = glass_or_surface(el, theme, size);
    if focused {
        let mut shadows = theme.glass.shadows(size).to_vec();
        shadows.extend(theme.focus_ring_shadows());
        el = el.shadow(shadows);
    }
    el
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
/// without glass, enforcing Apple's "no glass on glass" rule.
///
/// Per HIG, glass elements should never be stacked on other glass
/// elements. `GlassContainer` provides the single glass layer and renders
/// all children as standard content within it.
///
/// # Example
/// ```ignore
/// GlassContainer::new("toolbar-group")
///     .size(GlassSize::Small)
///     .spacing(theme.spacing_sm)
///     .child(button_a)
///     .child(button_b)
/// ```
#[derive(IntoElement)]
pub struct GlassContainer {
    id: ElementId,
    size: GlassSize,
    pub(crate) spacing: Option<Pixels>,
    pub(crate) children: Vec<AnyElement>,
}

impl GlassContainer {
    /// Create a new glass container with the given element ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            size: GlassSize::Small,
            spacing: None,
            children: Vec::new(),
        }
    }

    /// Set the glass size variant (Small, Medium, Large).
    pub fn size(mut self, size: GlassSize) -> Self {
        self.size = size;
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

        glass_surface(inner, theme, self.size).id(self.id)
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
/// all the way to the edge). GPUI currently lacks a backdrop-blur
/// primitive, so the implementation below approximates both with a
/// bounded linear gradient until that lands. Expose the enum now so
/// callers record intent and the rendering can upgrade in one place.
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
/// surfaces). Variable blur matching the HIG's modern scroll-edge
/// effect is unavailable until GPUI ships a backdrop-blur primitive;
/// this fallback is documented at the call site.
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
    // of HIG's "hard" scroll edge effect without a backdrop-blur
    // primitive.
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
    .with_priority(2)
}

/// Cross-fade a glass surface between two tiers when its `GlassSize` changes.
///
/// Per Apple's Tahoe Liquid Glass spec, a surface that changes its material
/// tier should smoothly blend between blur/opacity levels rather than
/// snapping. GPUI does not yet expose animated blur, so this helper
/// approximates the tier blend with a duration-based opacity cross-fade
/// over `shape_shift_duration_ms`. Callers that don't animate layout
/// should keep calling `glass_surface` directly — this helper is opt-in.
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

/// Apply the HIG focus ring to an element, preserving any base shadows.
///
/// When `focused`: sets shadows to `base_shadows` + the two focus-ring layers
/// returned by [`TahoeTheme::focus_ring_shadows`] (outer accent + inner gap).
/// When not focused: sets shadows to `base_shadows` (if non-empty), or no-op.
///
/// This is the single entry point for focus ring + shadow composition.
/// - Non-glass components: `apply_focus_ring(el, theme, focused, &[])`
/// - Glass components: `apply_focus_ring(el, theme, focused, &theme.glass.shadows(size))`
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
/// Returns an absolutely positioned div covering the full viewport. The
/// tint is [`TahoeTheme::overlay_bg`] — the standard modal dim scrim.
///
/// # Backdrop blur (HIG Materials)
///
/// Apple inspector / sheet / alert backdrops composite a blurred view of
/// the window content *behind* the overlay tint (vibrancy / material
/// layer). Today this function emits only the tint because GPUI lacks a
/// `paint_blur_rect()` primitive — there is no way to blur an arbitrary
/// sub-region of the framebuffer. This is the same GPUI gap that makes
/// [`glass_blur_surface`] / [`glass_lens_surface`] fall back to
/// [`glass_surface`].
///
/// Callers that want to *express* backdrop-blur intent (so they can
/// audit which overlays need the upgrade) should use
/// [`backdrop_blur_overlay`] instead. `backdrop_overlay` itself delegates
/// to `backdrop_blur_overlay` with a HIG-default [`BlurEffect`], so every
/// site that calls this function is automatically upgraded the moment a
/// real blur primitive ships upstream.
pub fn backdrop_overlay(theme: &crate::foundations::theme::TahoeTheme) -> gpui::Div {
    backdrop_blur_overlay(theme, &default_backdrop_blur_effect(theme))
}

/// Create a full-screen backdrop overlay with an explicit backdrop-blur
/// effect — the blur-aware analog of [`backdrop_overlay`].
///
/// # Current behaviour (pending GPUI `paint_blur_rect`)
///
/// - With `ReduceTransparency`: tints with the opaque
///   [`AccessibilityTokens::reduced_transparency_bg`] so motion-sensitive
///   and high-contrast users get a solid scrim instead of any translucency.
/// - Otherwise: tints with [`TahoeTheme::overlay_bg`]. The `effect.radius`
///   is recorded but not rendered — GPUI has no render-to-texture
///   primitive for arbitrary sub-region blur yet. See the module-level
///   "GPU Pipeline Extension" section for the shader design.
///
/// # Future
///
/// When GPUI lands `paint_blur_rect()`, this function will compose
/// `effect.radius`-point Dual Kawase blur over the covered region and
/// then overlay `effect.tint`. Every caller of [`backdrop_overlay`] gains
/// real blur without any additional edits because `backdrop_overlay`
/// routes through here.
pub fn backdrop_blur_overlay(
    theme: &crate::foundations::theme::TahoeTheme,
    effect: &BlurEffect,
) -> gpui::Div {
    let bg = if theme.accessibility_mode.reduce_transparency() {
        theme.glass.accessibility.reduced_transparency_bg
    } else {
        effect.tint
    };

    // GPUI blocker: without `paint_blur_rect`, the backdrop blur comes
    // from the window-level `WindowBackgroundAppearance::Blurred` set
    // at install time. The `effect.radius` / `corner_radius` values are
    // retained on the struct so callers can still express intent; once
    // GPUI lands a rect-level blur entry, populate it here before the
    // tint fill without changing the call-site API.
    let _ = effect.radius;
    let _ = effect.corner_radius;

    gpui::div().absolute().top_0().left_0().size_full().bg(bg)
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

/// Apply a dark dimming layer behind Clear Liquid Glass for bright content.
///
/// Per HIG: "If the underlying content is bright, consider adding a
/// dark dimming layer of 35% opacity behind Liquid Glass in the clear style."
///
/// Returns a div with 35% black background to be placed behind a Clear glass surface.
pub fn clear_glass_dimming_layer() -> gpui::Div {
    gpui::div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .bg(gpui::hsla(0.0, 0.0, 0.0, 0.35))
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;
    use gpui::px;

    use super::GlassSize;
    use crate::foundations::accessibility::AccessibilityMode;
    use crate::foundations::layout::ShapeType;
    use crate::foundations::theme::TahoeTheme;

    use super::{compute_shape_radius, effective_duration};

    #[test]
    fn fixed_radius_returns_input() {
        let theme = TahoeTheme::dark();
        let r = compute_shape_radius(&theme, ShapeType::Fixed(px(12.0)), None);
        assert!((f32::from(r) - 12.0).abs() < f32::EPSILON);
    }

    #[test]
    fn capsule_returns_half_height() {
        let theme = TahoeTheme::dark();
        let r = compute_shape_radius(&theme, ShapeType::Capsule, Some(px(44.0)));
        assert!((f32::from(r) - 22.0).abs() < f32::EPSILON);
    }

    #[test]
    fn capsule_no_height_uses_full() {
        let theme = TahoeTheme::dark();
        let r = compute_shape_radius(&theme, ShapeType::Capsule, None);
        assert_eq!(r, theme.radius_full);
    }

    #[test]
    fn concentric_subtracts_padding() {
        let theme = TahoeTheme::dark();
        let r = compute_shape_radius(
            &theme,
            ShapeType::Concentric {
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
            ShapeType::Concentric {
                parent_radius: px(4.0),
                padding: px(10.0),
            },
            None,
        );
        assert!((f32::from(r) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn concentric_exact_equals_zero() {
        let theme = TahoeTheme::dark();
        let r = compute_shape_radius(
            &theme,
            ShapeType::Concentric {
                parent_radius: px(20.0),
                padding: px(20.0),
            },
            None,
        );
        assert!((f32::from(r) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn glass_surface_uses_glass_radius_for_glass_theme() {
        // Smoke test: glass_surface with a glass theme should not panic
        let theme = TahoeTheme::liquid_glass();
        // We can't easily test Div rendering in a unit test, but we verify
        // the theme has the right tokens
        assert!((f32::from(theme.glass.small_radius) - 20.0).abs() < f32::EPSILON);
        assert!((f32::from(theme.glass.radius(GlassSize::Small)) - 20.0).abs() < f32::EPSILON);
    }

    // ─── Motion & Accessibility Tests ────────────────────────────────────────

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

    // ─── Accessibility Helper Tests ─────────────────────────────────────────

    #[test]
    fn accessible_bg_returns_reduced_transparency_for_reduce_transparency() {
        let mut theme = TahoeTheme::liquid_glass();
        theme.accessibility_mode = AccessibilityMode::REDUCE_TRANSPARENCY;
        let bg = theme
            .glass
            .accessible_bg(GlassSize::Small, theme.accessibility_mode);
        assert_eq!(bg, theme.glass.accessibility.reduced_transparency_bg);
    }

    #[test]
    fn accessible_bg_returns_standard_for_default() {
        let theme = TahoeTheme::liquid_glass();
        let bg = theme
            .glass
            .accessible_bg(GlassSize::Small, theme.accessibility_mode);
        assert_eq!(bg, theme.glass.bg(GlassSize::Small));
    }

    #[test]
    fn accessible_tint_bg_multiplies_alpha_for_reduce_transparency() {
        use super::GlassTint;
        use super::accessible_tint_bg;
        let tint = GlassTint {
            bg: gpui::hsla(0.0, 0.0, 0.0, 0.08),
            bg_hover: gpui::hsla(0.0, 0.0, 0.0, 0.16),
        };
        let bg = accessible_tint_bg(&tint, AccessibilityMode::REDUCE_TRANSPARENCY);
        assert!((bg.a - 0.24).abs() < f32::EPSILON);
    }

    #[test]
    fn accessible_tint_bg_returns_original_for_default() {
        use super::GlassTint;
        use super::accessible_tint_bg;
        let tint = GlassTint {
            bg: gpui::hsla(0.0, 0.0, 0.0, 0.08),
            bg_hover: gpui::hsla(0.0, 0.0, 0.0, 0.16),
        };
        let bg = accessible_tint_bg(&tint, AccessibilityMode::DEFAULT);
        assert!((bg.a - 0.08).abs() < f32::EPSILON);
    }

    // ─── Focus Ring Tests ───────────────────────────────────────────────────

    #[test]
    fn focus_ring_shadows_default_uses_accent_color_and_is_solid() {
        let theme = TahoeTheme::dark();
        let shadows = theme.focus_ring_shadows();
        // Two layers: outer accent ring, inner gap.
        assert_eq!(shadows.len(), 2);
        let outer = &shadows[0];
        let inner = &shadows[1];

        // Outer: accent hue, solid (alpha=1.0), no blur, spread = offset + width.
        assert_eq!(outer.color.h, theme.focus_ring_color.h);
        assert!((outer.color.a - 1.0).abs() < f32::EPSILON);
        assert_eq!(f32::from(outer.blur_radius), 0.0);
        let expected_outer = f32::from(theme.focus_ring_offset) + f32::from(theme.focus_ring_width);
        assert!((f32::from(outer.spread_radius) - expected_outer).abs() < f32::EPSILON);

        // Inner: gap in background colour, no blur, spread = offset only.
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
        // Outer accent layer should remain fully opaque.
        assert!((shadows[0].color.a - 1.0).abs() < f32::EPSILON);
        assert_eq!(f32::from(shadows[0].blur_radius), 0.0);
    }

    #[test]
    fn focus_ring_shadows_glass_is_solid_not_translucent() {
        // HIG macOS 14+ removed the soft glow — even on glass the ring is
        // solid accent. The inner layer uses the theme's background colour
        // to carve the 3pt gap.
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

    // --- Blur & Lens Effect Tests ────────────────────────────────────────────

    #[test]
    fn blur_effect_for_glass_size() {
        use super::BlurEffect;
        let theme = TahoeTheme::dark();
        let small = BlurEffect::for_glass_size(GlassSize::Small, &theme);
        let large = BlurEffect::for_glass_size(GlassSize::Large, &theme);
        assert!(
            small.radius < large.radius,
            "Large glass should have more blur"
        );
        assert!(small.corner_radius > 0.0);
    }

    #[test]
    fn lens_effect_liquid_glass_defaults() {
        use super::LensEffect;
        let theme = TahoeTheme::dark();
        let lens = LensEffect::liquid_glass(GlassSize::Medium, &theme);
        assert_eq!(lens.refraction, 1.0); // Figma: 100
        assert_eq!(lens.depth, 16.0); // Figma: 16
        assert_eq!(lens.dispersion, 0.0); // Figma: 0
        assert_eq!(lens.splay, 6.0); // Figma: 6
        assert_eq!(lens.light_angle, -45.0); // Figma: -45°
        assert!((lens.light_intensity - 0.67).abs() < 0.01); // Figma: 67%
        assert_eq!(lens.blur.radius, 12.0); // Figma Frost: 12
    }

    #[test]
    fn lens_effect_blur_only_no_refraction() {
        use super::LensEffect;
        let theme = TahoeTheme::dark();
        let lens = LensEffect::blur_only(GlassSize::Small, &theme);
        assert_eq!(lens.refraction, 0.0);
        assert_eq!(lens.dispersion, 0.0);
        assert_eq!(lens.light_intensity, 0.0);
        assert!(lens.blur.radius > 0.0); // blur still active
    }

    #[test]
    fn lens_effect_subtle_has_lower_refraction() {
        use super::LensEffect;
        let theme = TahoeTheme::dark();
        let full = LensEffect::liquid_glass(GlassSize::Medium, &theme);
        let subtle = LensEffect::subtle(GlassSize::Medium, &theme);
        assert!(subtle.refraction < full.refraction);
        assert!(subtle.depth < full.depth);
        assert!(subtle.light_intensity < full.light_intensity);
    }

    #[test]
    fn blur_effect_radius_per_size() {
        use super::BlurEffect;
        let theme = TahoeTheme::dark();
        let small = BlurEffect::for_glass_size(GlassSize::Small, &theme);
        let medium = BlurEffect::for_glass_size(GlassSize::Medium, &theme);
        let large = BlurEffect::for_glass_size(GlassSize::Large, &theme);
        assert!(small.radius < medium.radius);
        assert!(medium.radius < large.radius);
    }

    #[test]
    fn blur_effect_corner_radius_matches_glass() {
        use super::BlurEffect;
        let theme = TahoeTheme::dark();
        let effect = BlurEffect::for_glass_size(GlassSize::Medium, &theme);
        assert!(
            (effect.corner_radius - f32::from(theme.glass.radius(GlassSize::Medium))).abs()
                < f32::EPSILON,
        );
    }

    // ── Standard Material Layering Tests ──────────────────────────────────

    #[test]
    fn material_regular_uses_standard_fill_not_glass_medium() {
        // Regression guard for HIG layering: MaterialThickness::Regular is
        // the 4-tier standard material, not the Liquid Glass Medium fill.
        use super::MaterialThickness;
        let theme = TahoeTheme::dark();
        let regular = theme.glass.material_bg(MaterialThickness::Regular);
        let glass_medium = theme.glass.regular_bg(GlassSize::Medium);
        assert_ne!(
            regular, glass_medium,
            "MaterialThickness::Regular must not alias GlassSize::Medium fill"
        );
        // Dark standard-material Regular is #000000 @ 29%.
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
        // UltraThin < Thin < Regular < Thick < UltraThick (by alpha on dark).
        use super::MaterialThickness;
        let theme = TahoeTheme::dark();
        let a = |t: MaterialThickness| theme.glass.material_bg(t).a;
        assert!(a(MaterialThickness::UltraThin) < a(MaterialThickness::Thin));
        assert!(a(MaterialThickness::Thin) < a(MaterialThickness::Regular));
        assert!(a(MaterialThickness::Regular) < a(MaterialThickness::Thick));
        assert!(a(MaterialThickness::Thick) < a(MaterialThickness::UltraThick));
    }

    // ── Clear Variant Per-Size Differentiation ────────────────────────────

    #[test]
    fn clear_variant_differentiates_by_size() {
        let theme = TahoeTheme::dark();
        let small = theme.glass.clear_fill(GlassSize::Small);
        let medium = theme.glass.clear_fill(GlassSize::Medium);
        let large = theme.glass.clear_fill(GlassSize::Large);
        assert!(
            small.a < medium.a,
            "clear Small ({}) >= Medium ({})",
            small.a,
            medium.a
        );
        assert!(
            medium.a < large.a,
            "clear Medium ({}) >= Large ({})",
            medium.a,
            large.a
        );
    }

    #[test]
    fn clear_variant_light_also_differentiates() {
        let theme = TahoeTheme::light();
        let small = theme.glass.clear_fill(GlassSize::Small);
        let medium = theme.glass.clear_fill(GlassSize::Medium);
        let large = theme.glass.clear_fill(GlassSize::Large);
        assert!(small.a < medium.a);
        assert!(medium.a < large.a);
    }

    // ── Large Radius Tests ────────────────────────────────────────────────

    #[test]
    fn large_radius_exceeds_medium_for_concentric_window_corners() {
        // Per Figma Tahoe UI Kit, large panels get a slightly bigger radius
        // than medium so they stay concentric with macOS 26 window corners.
        let theme = TahoeTheme::liquid_glass();
        assert!(
            f32::from(theme.glass.large_radius) > f32::from(theme.glass.medium_radius),
            "large_radius ({}) must exceed medium_radius ({})",
            f32::from(theme.glass.large_radius),
            f32::from(theme.glass.medium_radius),
        );
    }

    // ── HUD tint ──────────────────────────────────────────────────────────

    #[test]
    fn hud_tint_alpha_matches_nspanel_hud_window() {
        // HIG `#panels` HUD overlays compose glass with a black-60% tint
        // to match `NSPanel.StyleMask.HUDWindow`. A drift here would
        // silently brighten every HUD across the crate.
        use super::HUD_TINT_ALPHA;
        assert!((HUD_TINT_ALPHA - 0.6).abs() < f32::EPSILON);
    }

    // ── GlassStyle::labels() contract ─────────────────────────────────────

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

    // ─── Backdrop helper tests ──────────────────────────────────────────────

    #[test]
    fn default_backdrop_blur_effect_uses_overlay_bg_tint() {
        let theme = TahoeTheme::dark();
        let effect = super::default_backdrop_blur_effect(&theme);
        assert_eq!(effect.tint, theme.overlay_bg);
    }

    #[test]
    fn default_backdrop_blur_effect_is_heavy_and_full_bleed() {
        // HIG inspector backdrops use a heavy blur radius and no corner
        // mask (they cover the full viewport).
        let theme = TahoeTheme::liquid_glass();
        let effect = super::default_backdrop_blur_effect(&theme);
        assert!(
            (effect.radius - 40.0).abs() < f32::EPSILON,
            "default backdrop blur radius should be 40pt (HIG heavy)"
        );
        assert!(
            effect.corner_radius.abs() < f32::EPSILON,
            "default backdrop has no corner mask"
        );
    }

    #[test]
    fn scroll_edge_height_constants_are_finite_and_ordered() {
        use super::{SCROLL_EDGE_HEIGHT, SCROLL_EDGE_HEIGHT_COMPACT};
        let default_h: f32 = SCROLL_EDGE_HEIGHT.into();
        let compact_h: f32 = SCROLL_EDGE_HEIGHT_COMPACT.into();
        assert!(default_h.is_finite() && default_h > 0.0);
        assert!(compact_h.is_finite() && compact_h > 0.0);
        // Default should be at least as tall as compact — if a future
        // retune swaps them, this catches it before callers get
        // inconsistent scroll-edge effects between split panes.
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
        // Smoke test — confirms the public signature accepts a custom
        // `height` + `style` pair without panicking on theme access.
        let _top = scroll_edge_top(&theme, SCROLL_EDGE_HEIGHT_COMPACT, ScrollEdgeStyle::Soft);
        let _bottom = scroll_edge_bottom(&theme, px(24.0), ScrollEdgeStyle::Hard);
    }

    #[test]
    fn glass_role_rejects_content_layer() {
        use super::GlassRole;
        assert!(!GlassRole::ContentLayer.permits_liquid_glass());
        assert!(GlassRole::Controls.permits_liquid_glass());
        assert!(GlassRole::Navigation.permits_liquid_glass());
        assert!(GlassRole::Overlay.permits_liquid_glass());
    }

    #[test]
    fn glass_role_default_is_safest_choice() {
        use super::GlassRole;
        // Default should be the layer where Liquid Glass is NOT allowed
        // so callers that forget to specify a role don't accidentally
        // violate the HIG content-layer restriction.
        assert_eq!(GlassRole::default(), GlassRole::ContentLayer);
    }

    #[test]
    fn elevation_index_maps_to_glass_role() {
        use super::{ElevationIndex, GlassRole};
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
        use super::{ElevationIndex, MaterialThickness};
        // Higher elevations use thicker materials — tripping this
        // means a retune silently inverted the ladder.
        fn rank(m: MaterialThickness) -> u8 {
            match m {
                MaterialThickness::UltraThin => 0,
                // Chrome currently aliases to Thin's fill, so they share a rank.
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
}
