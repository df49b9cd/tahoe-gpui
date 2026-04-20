//! Design token system for AI Elements.
//!
//! Provides colors, spacing, typography, and other design tokens as a GPUI
//! global. Components read the theme via `cx.theme()` (from the
//! [`ActiveTheme`] trait). `cx.global::<TahoeTheme>()` is the lower-level
//! form the trait delegates to.

use gpui::{
    App, BoxShadow, FontWeight, Hsla, Pixels, SharedString, Window, WindowAppearance,
    WindowBackgroundAppearance, hsla, point, px,
};

use crate::foundations::color::{Appearance, SystemColor, SystemPalette, text_on_background};

pub mod ansi;
pub use ansi::AnsiColors;

pub mod syntax;
pub use syntax::SyntaxColors;

pub mod semantic;
pub use semantic::SemanticColors;

// Layout types — canonical definitions in `super::layout`
pub use super::layout::{
    CONTENT_MARGIN, CONTENT_MARGIN_WIDE, LayoutDirection, Platform, READABLE_MAX_WIDTH, ShapeType,
};

// Material types — canonical definitions in `super::materials`
pub use super::materials::{
    BlurEffect, ElevationIndex, GlassContainer, GlassLabels, GlassRole, GlassSize, GlassStyle,
    GlassTint, GlassTintColor, GlassTints, GlassVariant, LensEffect, MaterialThickness,
    SCROLL_EDGE_HEIGHT, SCROLL_EDGE_HEIGHT_COMPACT, ScrollEdgeStyle, StandardMaterial,
    SurfaceContext,
};

// System Settings enums — canonical definitions in `super::color` and `super::materials`
pub use super::color::{AccentColor, HighlightColor, IconAndWidgetStyle, SidebarIconSize};
pub use super::materials::LiquidGlassPreference;

// Typography types — canonical definitions in `super::typography`
pub use super::typography::{
    DynamicTypeSize, FontDesign, LeadingStyle, TextStyle, TextStyleAttrs, TextStyledExt, bold_step,
    macos_tracking, platform_text_size,
};

// Accessibility types — canonical definitions in `super::accessibility`
pub use super::accessibility::{
    AccessibilityMode, AccessibilityProps, AccessibilityRole, AccessibilityTokens, AccessibleExt,
};

// Motion types — canonical definition in `super::motion`
pub use super::motion::{MorphState, MotionTokens};

// Contrast utilities — canonical definitions in `super::color`
pub use super::color::{contrast_ratio, meets_contrast};

// Alpha-channel helpers — canonical definitions in `super::color`. The
// `HslaAlphaExt` trait lets call sites chain `color.opacity(0.5)` the same
// way Zed's `Hsla` does, closing Finding 10 of the Zed cross-reference audit.
pub use super::color::{HslaAlphaExt, fade_out, opacity};

/// Design tokens for AI Elements components.
///
/// Register as a GPUI global before rendering any components:
/// ```ignore
/// cx.set_global(TahoeTheme::dark());
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TahoeTheme {
    // --- Appearance ---
    /// Current appearance (light/dark, standard/high-contrast).
    pub appearance: Appearance,
    /// Full HIG system color palette, pre-resolved for the current appearance.
    pub palette: SystemPalette,

    // --- Colors ---
    /// Primary text color.
    pub text: Hsla,
    /// Secondary/muted text color.
    pub text_muted: Hsla,
    /// Accent color (links, active elements).
    pub accent: Hsla,
    /// Ring/focus-ring color (connection lines, focus indicators).
    /// Maps to `--color-ring` in shadcn/ui.
    pub ring: Hsla,
    /// Error/destructive color.
    pub error: Hsla,
    /// Success/confirmation color.
    pub success: Hsla,
    /// Warning color.
    pub warning: Hsla,

    /// Background for the conversation container.
    pub background: Hsla,
    /// Slightly elevated surface (cards, panels).
    pub surface: Hsla,
    /// Assistant message background (typically transparent).
    pub assistant_message_bg: Hsla,
    /// Code block background.
    pub code_bg: Hsla,
    /// Border color.
    pub border: Hsla,
    /// Hover state background.
    pub hover: Hsla,
    /// Active-selection background for rows in lists, trees, and tables.
    ///
    /// Distinct from `hover` per HIG Color: selection must read visually
    /// apart from the transient hover highlight. Finder sidebar, Xcode
    /// navigators, and Zed's project panel all tint the selected row with
    /// a tinted accent fill (~15–20 % alpha) rather than reusing the hover
    /// grey. Callers that previously used `hover` for selection should
    /// migrate to `selected_bg` so Cmd-click / Shift-click multi-selection
    /// stays legible.
    pub selected_bg: Hsla,
    /// Text color for use on accent/colored backgrounds (e.g. primary buttons, badges).
    pub text_on_accent: Hsla,
    /// Scrim/overlay backdrop color (semi-transparent).
    pub overlay_bg: Hsla,

    // --- Spacing ---
    pub spacing_xs: Pixels,
    pub spacing_sm: Pixels,
    pub spacing_md: Pixels,
    pub spacing_lg: Pixels,
    pub spacing_xl: Pixels,
    /// Intermediate spacing between sm (8) and md (16). Apple 12pt.
    pub spacing_sm_md: Pixels,
    /// Intermediate spacing between md (16) and lg (24). Apple 20pt.
    pub spacing_md_lg: Pixels,
    /// Double-large spacing for section breaks. Apple 40pt.
    pub spacing_2xl: Pixels,
    /// Triple-large spacing for hero regions. Apple 48pt.
    pub spacing_3xl: Pixels,

    // --- Border Radius ---
    pub radius_sm: Pixels,
    pub radius_md: Pixels,
    pub radius_lg: Pixels,
    pub radius_full: Pixels,
    /// Corner radius for segmented controls (HIG ~7pt).
    pub radius_segmented: Pixels,

    // --- Component sizing ---
    /// Inline icon size — the small icons used next to labels in search
    /// fields, combo box triggers, text inputs, etc.
    pub icon_size_inline: Pixels,
    /// Small icon size (12 pt) for caption-sized glyphs in tight UI —
    /// chevrons on segmented controls, status dots, inline decorations.
    pub icon_size_small: Pixels,
    /// Extra-small icon size (10 pt) for footnote-sized glyphs such as
    /// tiny status badges or dense table-cell indicators.
    pub icon_size_xs: Pixels,
    /// Vertical offset applied to dropdowns (popover, popup_button,
    /// pulldown_button, search_field, combo_box, picker) from the bottom
    /// of their trigger. Replaces the `MIN_TOUCH_TARGET + 4.0` magic.
    pub dropdown_offset: Pixels,
    /// Hairline thickness used by `Separator::horizontal/vertical`. 1pt
    /// on standard displays; consumers that want sub-pixel hairlines on
    /// Retina can override this.
    pub separator_thickness: Pixels,
    /// Default width for a macOS sidebar (HIG macOS Tahoe: 180–320pt
    /// typical range, 220pt default). Used by `Sidebar::new` when no
    /// explicit width is set. Aligned with
    /// `foundations::layout::SIDEBAR_DEFAULT_WIDTH`.
    pub sidebar_width_default: Pixels,
    /// Track width for the `Toggle` switch. HIG macOS toggles are
    /// ~36pt wide.
    pub toggle_track_width: Pixels,
    /// Track height for the `Toggle` switch. HIG macOS toggles are
    /// ~20pt tall; iOS is closer to 31pt.
    pub toggle_track_height: Pixels,
    /// Horizontal inset between a menu capsule row and the menu container
    /// edge (per macOS Tahoe context menu reference — ~6pt).
    pub menu_inset: Pixels,

    // --- Typography ---
    pub font_sans: SharedString,
    pub font_mono: SharedString,
    /// User-controlled font scale (macOS System Settings → Accessibility →
    /// Display → Text Size). `1.0` is the HIG default; values >1 scale all
    /// system text styles proportionally. `TextStyledExt::text_style` and
    /// `text_style_emphasized` multiply both `size` and `leading` by this
    /// factor so vertical rhythm is preserved. GPUI does not currently
    /// expose `NSFontDescriptor`'s preferred body size, so hosts must drive
    /// this field themselves via `with_font_scale_factor`.
    pub font_scale_factor: f32,
    /// User-controlled Dynamic Type size (iOS-style enum). Defaults to
    /// [`DynamicTypeSize::Large`] — the HIG baseline — so LTR macOS apps
    /// behave identically to the pre-audit behavior. Hosts that read the
    /// user's Dynamic Type preference from `UIApplication.shared
    /// .preferredContentSizeCategory` (or the OS-specific equivalent) can
    /// propagate it here via [`TahoeTheme::with_dynamic_type_size`]; the
    /// matching multiplier is applied by [`TahoeTheme::text_size_for`] so
    /// components can resolve a runtime-scaled point size for any
    /// [`TextStyle`] without reaching past the theme.
    ///
    /// Finding 26 in the Zed cross-reference audit tracks this wiring — it
    /// lands the storage + resolver so future PRs can migrate individual
    /// components off raw `px(14.)` in favour of
    /// `theme.text_size_for(TextStyle::Body)`.
    pub dynamic_type_size: DynamicTypeSize,

    // --- Colors ---
    /// Info/blue color for citations, chain-of-thought.
    pub info: Hsla,
    /// AI/agent color (purple) for AI-specific elements.
    pub ai: Hsla,
    /// Pending/dimmed color for inactive states.
    pub pending: Hsla,
    /// Terminal background (darker than code_bg).
    pub terminal_bg: Hsla,
    /// Vertical connector line color (chain-of-thought, file tree).
    pub connector_line: Hsla,
    /// Tool approval accepted background.
    pub tool_approved_bg: Hsla,
    /// Tool approval rejected background.
    pub tool_rejected_bg: Hsla,

    // --- Colors ---
    pub ansi: AnsiColors,

    // --- Syntax Highlighting ---
    pub syntax: SyntaxColors,

    // --- Colors ---
    /// Color for input token usage indicator.
    pub usage_input: Hsla,
    /// Color for output token usage indicator.
    pub usage_output: Hsla,
    /// Color for reasoning token usage indicator.
    pub usage_reasoning: Hsla,
    /// Color for cached token usage indicator.
    pub usage_cached: Hsla,

    // --- Component-specific ---
    /// Avatar size.
    pub avatar_size: Pixels,
    /// Icon default size.
    pub icon_size: Pixels,
    /// Shimmer animation duration in milliseconds.
    pub shimmer_duration_ms: u64,
    /// Text shimmer spread multiplier (default 2.0).
    /// Higher values create a wider highlight sweep area.
    pub shimmer_spread: f32,
    /// Semi-transparent panel/overlay background. With `WindowBackgroundAppearance::Blurred`,
    /// the macOS window blur shows through for a true glass effect.
    pub panel_surface: Hsla,

    // ─── Liquid Glass ────────────────────────────────────────────────────────
    /// Liquid Glass design tokens. Per HIG, glass is always present
    /// in macOS Tahoe (26). Use `glass_preference` to control Clear/Tinted style.
    pub glass: GlassStyle,

    // ─── Semantic Colors (HIG) ─────────────────────────────────────────
    /// Semantic label and background colors adapted for the current appearance.
    pub semantic: SemanticColors,

    // ─── Accessibility ────────────────────────────────────────────────────────
    /// Current accessibility mode. Components check this to adjust rendering.
    pub accessibility_mode: AccessibilityMode,

    // ─── Layout Direction ───────────────────────────────────────────────
    /// Layout direction for RTL language support.
    pub layout_direction: LayoutDirection,

    /// First day of the week for calendar components: `0` = Sunday (US
    /// convention), `1` = Monday (ISO 8601 / most of Europe). HIG:
    /// "The exact values shown … and their order, depend on the device
    /// language/location." Components like `DatePicker` read this to lay
    /// out their weekday header. Defaults to Sunday; hosts that can read
    /// the user's locale should set it explicitly.
    pub first_weekday: u8,

    /// Target platform. Controls interactive target sizes, text sizes,
    /// and other platform-specific layout values per HIG.
    /// Defaults to `Platform::MacOS` since GPUI targets desktop.
    pub platform: Platform,

    // ─── System Settings (macOS Appearance pane) ────────────────────────
    /// Accent color selection (System Settings > Theme > Color).
    ///
    /// This field stores the user's *intent*: which named accent they
    /// picked. The resolved colour for rendering lives in `accent` (and is
    /// propagated to `ring`, `focus_ring_color`, and `glass.accent_tint`).
    ///
    /// **Limitation:** GPUI does not currently expose the macOS system
    /// accent (`NSColor.controlAccentColor`) at the public API surface, so
    /// constructors like [`TahoeTheme::for_appearance`] cannot read the
    /// user's actual accent and default to [`AccentColor::Multicolor`]
    /// (Blue). To respect the system choice, hosts that have access to
    /// AppKit can build the theme via [`TahoeTheme::with_accent`] explicitly:
    ///
    /// ```ignore
    /// let user_accent = read_ns_control_accent_color(); // host-provided
    /// TahoeTheme::with_accent(appearance, user_accent).apply(cx);
    /// ```
    ///
    /// Once GPUI exposes a `Window::accent_color()` (or similar), the
    /// `install_*_with_system_appearance` helpers should read it and feed
    /// it into the theme alongside `Appearance`.
    pub accent_color: AccentColor,

    /// Highlight color for text selection.
    pub highlight_color: HighlightColor,

    /// Icon and widget style preference.
    pub icon_and_widget_style: IconAndWidgetStyle,

    /// Sidebar icon size (System Settings > Windows).
    pub sidebar_icon_size: SidebarIconSize,

    // ─── Focus Ring ─────────────────────────────────────────────────────
    /// Focus ring color (defaults to accent).
    pub focus_ring_color: Hsla,
    /// Focus ring stroke width in pixels (HIG: 3px).
    pub focus_ring_width: Pixels,
    /// Focus ring offset from the element edge in pixels (2px gap).
    pub focus_ring_offset: Pixels,
}

// Appearance::resolve() on `crate::foundations::color::Appearance` replaces the
// old `resolve_by_appearance(is_dark, is_hc, ...)` free function. Call sites now
// use `appearance.resolve(light, dark, light_hc, dark_hc)` directly.

// Private groupings used to decompose `with_accent()` into focused builders.
// The public `TahoeTheme` struct fields remain flat; these types only flow
// between the builder methods and the constructor.

struct TextColors {
    text: Hsla,
    text_muted: Hsla,
    accent: Hsla,
    ring: Hsla,
    error: Hsla,
    success: Hsla,
    warning: Hsla,
    text_on_accent: Hsla,
}

struct SpacingTokens {
    xs: Pixels,
    sm: Pixels,
    md: Pixels,
    lg: Pixels,
    xl: Pixels,
    sm_md: Pixels,
    md_lg: Pixels,
    xl2: Pixels,
    xl3: Pixels,
    radius_sm: Pixels,
    radius_md: Pixels,
    radius_lg: Pixels,
    radius_full: Pixels,
    radius_segmented: Pixels,
}

struct TypographyTokens {
    font_sans: SharedString,
    font_mono: SharedString,
}

struct ComponentSizes {
    icon_size_inline: Pixels,
    icon_size_small: Pixels,
    icon_size_xs: Pixels,
    dropdown_offset: Pixels,
    separator_thickness: Pixels,
    sidebar_width_default: Pixels,
    toggle_track_width: Pixels,
    toggle_track_height: Pixels,
    menu_inset: Pixels,
    avatar_size: Pixels,
    icon_size: Pixels,
    shimmer_duration_ms: u64,
    shimmer_spread: f32,
}

impl TahoeTheme {
    /// Create a theme for the given appearance.
    ///
    /// Semantic tokens are grounded in the HIG system color palette.
    /// The palette is pre-resolved and stored on the theme for direct access.
    pub fn new(appearance: Appearance) -> Self {
        Self::with_accent(appearance, AccentColor::default())
    }

    /// Create a theme with a specific accent color.
    pub fn with_accent(appearance: Appearance, accent_color: AccentColor) -> Self {
        let palette = SystemPalette::new(appearance);
        let accent = accent_color.resolve(&palette);
        let is_dark = appearance.is_dark();

        let semantic = SemanticColors::new(appearance);
        let text_colors = Self::build_text_colors(&semantic, &palette, accent);
        let spacing = Self::build_spacing_tokens();
        let typography = Self::build_typography_tokens();
        let sizes = Self::build_component_sizes();

        Self {
            appearance,
            palette,

            text: text_colors.text,
            text_muted: text_colors.text_muted,
            accent: text_colors.accent,
            ring: text_colors.ring,
            error: text_colors.error,
            success: text_colors.success,
            warning: text_colors.warning,

            background: semantic.system_background,
            surface: semantic.secondary_system_background,
            assistant_message_bg: hsla(0.0, 0.0, 0.0, 0.0), // transparent
            code_bg: appearance.resolve(
                hsla(0.0, 0.0, 0.96, 1.0), // light
                hsla(0.0, 0.0, 0.13, 1.0), // dark
                hsla(0.0, 0.0, 0.95, 1.0), // light HC
                hsla(0.0, 0.0, 0.15, 1.0), // dark HC
            ),
            border: semantic.opaque_separator,
            hover: semantic.quaternary_system_fill,
            // Tinted accent fill for selected rows. Mirrors Finder's
            // `selectedContentBackgroundColor` when the window is key: a
            // low-alpha accent tint that stays legible against both the
            // default and high-contrast appearances.
            selected_bg: Self::selected_bg_for(accent, is_dark),
            text_on_accent: text_colors.text_on_accent,
            overlay_bg: if is_dark {
                hsla(0.0, 0.0, 0.0, 0.5)
            } else {
                hsla(0.0, 0.0, 0.0, 0.3)
            },

            spacing_xs: spacing.xs,
            spacing_sm: spacing.sm,
            spacing_md: spacing.md,
            spacing_lg: spacing.lg,
            spacing_xl: spacing.xl,
            spacing_sm_md: spacing.sm_md,
            spacing_md_lg: spacing.md_lg,
            spacing_2xl: spacing.xl2,
            spacing_3xl: spacing.xl3,

            radius_sm: spacing.radius_sm,
            radius_md: spacing.radius_md,
            radius_lg: spacing.radius_lg,
            radius_full: spacing.radius_full,
            radius_segmented: spacing.radius_segmented,

            icon_size_inline: sizes.icon_size_inline,
            icon_size_small: sizes.icon_size_small,
            icon_size_xs: sizes.icon_size_xs,
            dropdown_offset: sizes.dropdown_offset,
            separator_thickness: sizes.separator_thickness,
            sidebar_width_default: sizes.sidebar_width_default,
            toggle_track_width: sizes.toggle_track_width,
            toggle_track_height: sizes.toggle_track_height,
            menu_inset: sizes.menu_inset,

            font_sans: typography.font_sans,
            font_mono: typography.font_mono,
            font_scale_factor: 1.0,
            dynamic_type_size: DynamicTypeSize::default(),

            // Extended colors — `semantic.info`/`semantic.ai` carry the
            // appearance-resolved palette colour with HC overrides applied,
            // so these top-level fields stay in sync with the semantic table.
            info: semantic.info,
            ai: semantic.ai,
            // `pending` is a dimmed indicator label — HC variants pull
            // ~0.10 toward the opposite end of the lightness axis so the
            // dimmed state stays distinguishable when IncreaseContrast is on.
            pending: appearance.resolve(
                hsla(0.0, 0.0, 0.55, 1.0), // light
                hsla(0.0, 0.0, 0.45, 1.0), // dark
                hsla(0.0, 0.0, 0.40, 1.0), // light HC — darker for more contrast
                hsla(0.0, 0.0, 0.55, 1.0), // dark HC — lighter for more contrast
            ),
            // `terminal_bg` deliberately does not vary with HC: the terminal
            // is a content surface whose contrast is controlled by ANSI text
            // colours, not the background. Boosting the bg under HC would
            // re-tint the user's terminal contents.
            terminal_bg: if is_dark {
                hsla(0.0, 0.0, 0.05, 1.0)
            } else {
                hsla(0.0, 0.0, 0.96, 1.0)
            },
            // `connector_line` is a decorative chain-of-thought / file-tree
            // hairline. HC variants widen the lightness delta vs the
            // background so the line remains visible at IncreaseContrast,
            // without becoming a primary visual element.
            connector_line: appearance.resolve(
                hsla(0.0, 0.0, 0.72, 1.0), // light
                hsla(0.0, 0.0, 0.35, 1.0), // dark
                hsla(0.0, 0.0, 0.55, 1.0), // light HC — darker for more contrast
                hsla(0.0, 0.0, 0.50, 1.0), // dark HC — lighter for more contrast
            ),
            tool_approved_bg: Hsla {
                a: if is_dark { 0.20 } else { 0.15 },
                ..palette.green
            },
            tool_rejected_bg: Hsla {
                a: if is_dark { 0.20 } else { 0.15 },
                ..palette.red
            },

            ansi: AnsiColors::new(is_dark),
            syntax: SyntaxColors::new(is_dark),

            usage_input: palette.blue,
            usage_output: palette.green,
            usage_reasoning: palette.purple,
            usage_cached: palette.orange,

            avatar_size: sizes.avatar_size,
            icon_size: sizes.icon_size,
            shimmer_duration_ms: sizes.shimmer_duration_ms,
            shimmer_spread: sizes.shimmer_spread,
            panel_surface: if is_dark {
                hsla(0.0, 0.0, 0.11, 0.80)
            } else {
                hsla(0.0, 0.0, 0.97, 0.80)
            },
            glass: Self::build_glass(appearance, &palette, accent),

            semantic,

            accessibility_mode: AccessibilityMode::DEFAULT,

            layout_direction: LayoutDirection::LeftToRight,
            first_weekday: 0,
            platform: Platform::MacOS,

            accent_color,
            highlight_color: HighlightColor::Automatic,
            icon_and_widget_style: IconAndWidgetStyle::Automatic,
            sidebar_icon_size: SidebarIconSize::Medium,

            focus_ring_color: accent,
            // HIG macOS 14+: solid 3pt accent outline at 3pt offset. Prior
            // versions used a 2pt offset with an outer glow; keep the 3pt
            // width/offset pairing so `focus_ring_shadows` draws a clean
            // ring with a 3pt breathing gap to the element edge.
            focus_ring_width: px(3.0),
            focus_ring_offset: px(3.0),
        }
    }

    /// Tinted accent fill for selected rows. Dark mode uses a
    /// slightly higher alpha so the fill stays visible against the
    /// darker background. Shared between the primary constructor and
    /// [`TahoeTheme::with_accent_color`] so a runtime accent swap
    /// cannot drift from the initial derivation.
    fn selected_bg_for(accent: Hsla, is_dark: bool) -> Hsla {
        Hsla {
            a: if is_dark { 0.28 } else { 0.18 },
            ..accent
        }
    }

    /// Build the text-oriented color tokens (labels, accent, and status colors).
    fn build_text_colors(
        semantic: &SemanticColors,
        palette: &SystemPalette,
        accent: Hsla,
    ) -> TextColors {
        TextColors {
            text: semantic.label,
            text_muted: semantic.secondary_label,
            accent,
            ring: accent,
            error: palette.red,
            success: palette.green,
            warning: palette.orange,
            text_on_accent: text_on_background(accent),
        }
    }

    /// Build the HIG spacing scale and border radii. Static across appearances.
    fn build_spacing_tokens() -> SpacingTokens {
        SpacingTokens {
            xs: px(4.0),
            sm: px(8.0),
            md: px(16.0),
            lg: px(24.0),
            xl: px(32.0),
            sm_md: px(12.0),
            md_lg: px(20.0),
            xl2: px(40.0),
            xl3: px(48.0),
            radius_sm: px(4.0),
            radius_md: px(8.0),
            radius_lg: px(12.0),
            radius_full: px(9999.0),
            radius_segmented: px(7.0),
        }
    }

    /// Build the type family and size scale. Static across appearances.
    fn build_typography_tokens() -> TypographyTokens {
        TypographyTokens {
            font_sans: SharedString::from(".AppleSystemUIFont"),
            // Per HIG: SF Mono is the system monospaced typeface on
            // macOS 10.15 (Catalina) and later. It ships with the system
            // since Tahoe. On earlier macOS releases SF Mono was only
            // bundled with Xcode, so callers targeting <10.15 should
            // override `font_mono` to "Menlo" as a fallback.
            font_mono: SharedString::from("SF Mono"),
        }
    }

    /// Build shared component sizes. Overridden in glass variants where needed.
    fn build_component_sizes() -> ComponentSizes {
        ComponentSizes {
            icon_size_inline: px(14.0),
            icon_size_small: px(12.0),
            icon_size_xs: px(10.0),
            dropdown_offset: px(4.0),
            separator_thickness: px(1.0),
            // HIG macOS Tahoe: sidebar default width is ~220pt
            // (`foundations/layout.rs::SIDEBAR_DEFAULT_WIDTH`). AppKit's
            // `NSSplitViewController` picks 220pt for the primary column
            // when the caller doesn't override it.
            sidebar_width_default: px(220.0),
            toggle_track_width: px(36.0),
            toggle_track_height: px(20.0),
            menu_inset: px(6.0),
            avatar_size: px(28.0),
            icon_size: px(16.0),
            shimmer_duration_ms: 2000,
            shimmer_spread: 2.0,
        }
    }

    /// Build default Liquid Glass tokens for the given appearance mode.
    ///
    /// Per HIG macOS Tahoe: glass is always present. Dark themes get
    /// dark translucent fills; light themes get white translucent fills.
    ///
    /// Values are sourced from the Figma Tahoe UI Kit; geometry (radii,
    /// window backing, motion) is shared across appearances while colors,
    /// shadow alphas, and material fills differ.
    fn build_glass(appearance: Appearance, palette: &SystemPalette, accent: Hsla) -> GlassStyle {
        let is_dark = appearance.is_dark();
        let is_hc = appearance.is_high_contrast();

        // Motion tokens are identical across all appearances.
        const MOTION: MotionTokens = MotionTokens {
            flex_duration_ms: 150,
            lift_duration_ms: 200,
            shape_shift_duration_ms: 350,
            spring_damping: 0.85,
            spring_response: 0.35,
            spring_bounce: 0.0,
        };

        // Label palettes are a light-on-dark / dark-on-light pair. The mapping
        // to `labels_dim` / `labels_bright` swaps between appearances.
        let light_text = GlassLabels {
            primary: hsla(0.0, 0.0, 0.96, 1.0),
            secondary: hsla(0.0, 0.0, 0.54, 1.0),
            tertiary: hsla(0.0, 0.0, 0.25, 1.0),
            quaternary: hsla(0.0, 0.0, 0.18, 1.0),
            quinary: hsla(0.0, 0.0, 0.12, 1.0),
        };
        let dark_text = GlassLabels {
            primary: hsla(0.0, 0.0, 0.10, 1.0),
            secondary: hsla(0.0, 0.0, 0.45, 1.0),
            tertiary: hsla(0.0, 0.0, 0.75, 1.0),
            quaternary: hsla(0.0, 0.0, 0.82, 1.0),
            quinary: hsla(0.0, 0.0, 0.88, 1.0),
        };
        let (labels_dim, labels_bright) = if is_dark {
            (light_text, dark_text)
        } else {
            (dark_text, light_text)
        };

        // Per-size surface fills (Figma Tahoe UI Kit).
        //
        // Dark:
        //   Small:  #CCCCCC@50% + #000000@60% + #FFFFFF@6% composite → L≈0.17, a=0.80
        //   Medium: #CCCCCC@100% + #000000@67% + #FFFFFF@3% with 67% container opacity
        //   Large:  #CCCCCC@100% + #000000@85% + #FFFFFF@3% with 67% container opacity
        //
        // Light:
        //   Small:  #F7F7F7 + #FFFFFF@50% + #333333 → near-white, opaque
        //   Medium: #F5F5F5@67% + #262626 → translucent white (backdrop blur)
        //   Large:  #FAFAFA@80% + #262626
        // Clear-variant fills differ by size per Adopting Liquid Glass:
        // smaller controls receive a lighter fill, larger panels a heavier
        // fill, so the media-rich backdrop reads differently at each depth
        // level. Values target Figma Tahoe UI Kit clear tokens.
        let (small_bg, medium_bg, large_bg, hover_bg, root_bg) = if is_dark {
            (
                hsla(0.0, 0.0, 0.17, 0.80),
                hsla(0.0, 0.0, 0.28, 0.67),
                hsla(0.0, 0.0, 0.13, 0.67),
                hsla(0.0, 0.0, 0.0, 0.50),
                hsla(0.0, 0.0, 0.0, 0.80),
            )
        } else {
            (
                hsla(0.0, 0.0, 0.969, 1.0),
                hsla(0.0, 0.0, 0.961, 0.67),
                hsla(0.0, 0.0, 0.98, 0.80),
                hsla(0.0, 0.0, 0.0, 0.04),
                hsla(0.0, 0.0, 1.0, 0.95),
            )
        };
        let (clear_small_bg, clear_medium_bg, clear_large_bg) = if is_dark {
            (
                hsla(0.0, 0.0, 1.0, 0.09),
                hsla(0.0, 0.0, 1.0, 0.12),
                hsla(0.0, 0.0, 1.0, 0.17),
            )
        } else {
            (
                hsla(0.0, 0.0, 1.0, 0.32),
                hsla(0.0, 0.0, 1.0, 0.40),
                hsla(0.0, 0.0, 1.0, 0.48),
            )
        };

        // Standard material fills (content layer) — same base tone per mode,
        // varying alpha. Dark uses `#000000` at 10/20/29/40/50%; light uses
        // `#F6F6F6` at 36/48/60/72/84%. All share Background blur: Uniform,
        // 30. These are distinct from the Liquid Glass fills above — the
        // standard-material Medium (29% / 60%) is the fill for
        // `MaterialThickness::Regular` and should not be confused with the
        // Liquid Glass Medium (`medium_bg`).
        let (ultra_thin_bg, thin_bg, medium_standard_bg, thick_bg, ultra_thick_bg) = if is_dark {
            (
                hsla(0.0, 0.0, 0.0, 0.10),
                hsla(0.0, 0.0, 0.0, 0.20),
                hsla(0.0, 0.0, 0.0, 0.29),
                hsla(0.0, 0.0, 0.0, 0.40),
                hsla(0.0, 0.0, 0.0, 0.50),
            )
        } else {
            (
                hsla(0.0, 0.0, 0.965, 0.36),
                hsla(0.0, 0.0, 0.965, 0.48),
                hsla(0.0, 0.0, 0.965, 0.60),
                hsla(0.0, 0.0, 0.965, 0.72),
                hsla(0.0, 0.0, 0.965, 0.84),
            )
        };
        // HIG `.bar` / Chrome material: denser than Thin so toolbar
        // labels remain legible when content scrolls beneath the chrome.
        // Dark ≈ `#000 @34%`, light ≈ `#F6F6F6 @65%` (values between
        // `Thin` and `Regular`).
        let chrome_bg = if is_dark {
            hsla(0.0, 0.0, 0.0, 0.34)
        } else {
            hsla(0.0, 0.0, 0.965, 0.65)
        };

        // Drop shadow shape is identical across appearances; only the #000
        // alpha (and medium's y-offset) differ.
        let shadow = |offset_y: f32, blur: f32, alpha: f32| BoxShadow {
            color: hsla(0.0, 0.0, 0.0, alpha),
            offset: point(px(0.), px(offset_y)),
            blur_radius: px(blur),
            spread_radius: px(0.),
        };
        let (small_shadow_a, medium_shadow_a, large_shadow_a, medium_shadow_y) = if is_dark {
            (0.06, 0.10, 0.12, 4.0)
        } else {
            (0.04, 0.06, 0.10, 3.0)
        };

        // Colored tints share a canonical hue/saturation per color, with
        // per-appearance alpha and a couple of hue/saturation tweaks for
        // green/orange/red on light backgrounds.
        let (tint_bg_a, tint_hover_a) = if is_dark { (0.08, 0.16) } else { (0.10, 0.18) };
        let tint = |h: f32, s: f32, l: f32| GlassTint {
            bg: hsla(h, s, l, tint_bg_a),
            bg_hover: hsla(h, s, l, tint_hover_a),
        };
        let tints = if is_dark {
            GlassTints::new(
                tint(0.37, 0.64, 0.50),
                tint(0.57, 1.0, 0.50),
                tint(0.81, 0.88, 0.58),
                tint(0.08, 1.0, 0.59),
                tint(0.999, 1.0, 0.63),
                tint(0.54, 0.99, 0.62),
                tint(0.48, 0.70, 0.50),
                tint(0.71, 0.80, 0.50),
            )
        } else {
            GlassTints::new(
                tint(0.37, 0.70, 0.42),
                tint(0.57, 1.0, 0.50),
                tint(0.81, 0.88, 0.58),
                tint(0.08, 1.0, 0.55),
                tint(0.999, 0.85, 0.50),
                tint(0.54, 0.99, 0.62),
                tint(0.48, 0.70, 0.50),
                tint(0.71, 0.80, 0.50),
            )
        };

        // Accessibility: reduced-transparency backdrop and HC border flip
        // between near-black and near-white.
        let accessibility = if is_dark {
            AccessibilityTokens {
                reduced_transparency_bg: hsla(0.0, 0.0, 0.0, 0.85),
                high_contrast_border: hsla(0.0, 0.0, 1.0, 0.60),
                reduced_motion_scale: 0.0,
            }
        } else {
            AccessibilityTokens {
                reduced_transparency_bg: hsla(0.0, 0.0, 1.0, 0.90),
                high_contrast_border: hsla(0.0, 0.0, 0.0, 0.60),
                reduced_motion_scale: 0.0,
            }
        };

        // Glass-surface icon colors — pastel variants on dark; richer on light.
        let icon_text = if is_dark {
            hsla(0.0, 0.0, 1.0, 0.85)
        } else {
            hsla(0.0, 0.0, 0.15, 0.85)
        };
        let icon = |dark_l: f32, dark_s: f32, light_l: f32, light_s: f32, base: Hsla| Hsla {
            l: if is_dark { dark_l } else { light_l },
            s: if is_dark { dark_s } else { light_s },
            ..base
        };

        // Tile chrome: white-over-dark vs black-over-light, both low-alpha.
        let (tile_bg, tile_border) = if is_dark {
            (hsla(0.0, 0.0, 1.0, 0.05), hsla(0.0, 0.0, 1.0, 0.08))
        } else {
            (hsla(0.0, 0.0, 0.0, 0.04), hsla(0.0, 0.0, 0.0, 0.06))
        };

        let mut glass = GlassStyle {
            variant: GlassVariant::Regular,
            small_bg,
            medium_bg,
            large_bg,
            clear_small_bg,
            clear_medium_bg,
            clear_large_bg,
            hover_bg,
            ultra_thin_bg,
            thin_bg,
            medium_standard_bg,
            thick_bg,
            ultra_thick_bg,
            chrome_bg,
            small_shadows: vec![shadow(1.0, 4.0, small_shadow_a)],
            medium_shadows: vec![shadow(medium_shadow_y, 16.0, medium_shadow_a)],
            large_shadows: vec![shadow(8.0, 40.0, large_shadow_a)],
            small_radius: px(20.0),
            medium_radius: px(34.0),
            // Large panels (sheets, alerts, modals) sit on a slightly bigger
            // radius than Medium so their rounded corners stay concentric
            // with macOS 26 window chrome (system window corners ≈ 12–14pt
            // outer → ~40pt inner panel). See Figma Tahoe UI Kit.
            large_radius: px(40.0),
            // macOS 26 ships `WindowBackgroundAppearance::Blurred` via
            // NSVisualEffectView. Linux and Windows GPUI backends fall back
            // to opaque silently, which produces incorrect rendering unless
            // we gate the request here. Non-macOS platforms receive the
            // `ultra_thick_bg` standard-material fill as their backing.
            window_background: if cfg!(target_os = "macos") {
                WindowBackgroundAppearance::Blurred
            } else {
                WindowBackgroundAppearance::Opaque
            },
            root_bg,
            labels_dim,
            labels_bright,
            font_sans: SharedString::from(".AppleSystemUIFont"),
            tints,
            accessibility,
            motion: MOTION,
            preference: LiquidGlassPreference::default(),
            accent_tint: GlassTint {
                bg: accent,
                bg_hover: crate::foundations::color::lighten(accent, 0.08),
            },
            icon_text,
            icon_success: icon(0.82, 0.75, 0.55, 0.60, palette.green),
            icon_info: icon(0.84, 0.95, 0.55, 0.70, palette.cyan),
            icon_warning: icon(0.74, 0.95, 0.55, 0.80, palette.orange),
            icon_error: icon(0.71, 0.90, 0.55, 0.75, palette.red),
            icon_ai: icon(0.85, 0.70, 0.55, 0.55, palette.purple),
            tile_bg,
            tile_border,
        };

        // Boost glass label contrast for high-contrast mode
        if is_hc {
            if is_dark {
                glass.labels_dim.secondary = hsla(0.0, 0.0, 0.80, 1.0);
                glass.labels_dim.tertiary = hsla(0.0, 0.0, 0.70, 1.0);
            } else {
                glass.labels_dim.secondary = hsla(0.0, 0.0, 0.20, 1.0);
                glass.labels_dim.tertiary = hsla(0.0, 0.0, 0.30, 1.0);
            }
        }

        glass
    }

    /// Dark theme (default).
    pub fn dark() -> Self {
        Self::new(Appearance::Dark)
    }

    /// Light theme.
    pub fn light() -> Self {
        Self::new(Appearance::Light)
    }

    /// Dark theme with increased contrast (accessibility).
    pub fn dark_high_contrast() -> Self {
        Self::new(Appearance::DarkHighContrast)
    }

    /// Light theme with increased contrast (accessibility).
    pub fn light_high_contrast() -> Self {
        Self::new(Appearance::LightHighContrast)
    }

    /// Liquid Glass theme (dark variant) aligned with Apple iOS 26 design system.
    ///
    /// Uses `WindowBackgroundAppearance::Blurred` to enable macOS NSVisualEffectView.
    /// Token values sourced from the official Apple iOS 26 UI Kit design resource.
    ///
    /// When opening a window with this theme, set:
    /// ```ignore
    /// WindowOptions {
    ///     window_background: theme.glass.window_background,
    ///     ..
    /// }
    /// ```
    pub fn liquid_glass() -> Self {
        let mut theme = Self::dark();

        // Apple Dark Base backgrounds. Per HIG `foundations.md:338` and the
        // `dark_mode.rs:19` rule we don't use pure black — the system dark
        // gray (L≈0.07) is the substrate when the blurred backdrop fades or
        // the user disables transparency. `theme.background` is kept in sync
        // with the semantic override at the bottom of this constructor.
        theme.surface = hsla(0.0, 0.0, 0.11, 1.0); // #1C1C1E
        theme.border = hsla(0.0, 0.0, 1.0, 0.12); // #FFFFFF1F separator
        theme.panel_surface = hsla(0.0, 0.0, 0.0, 0.40); // matches glass fill

        // Apple accent colors (Dark)
        let accent = hsla(0.57, 1.0, 0.50, 1.0); // #0091FF Blue
        theme.accent = accent;
        theme.text_on_accent = text_on_background(accent);
        theme.ring = accent;
        theme.error = Hsla {
            l: 0.63,
            ..theme.palette.red
        };
        theme.success = Hsla {
            l: 0.50,
            ..theme.palette.green
        };
        theme.warning = Hsla {
            l: 0.59,
            ..theme.palette.orange
        };
        // `theme.info` is synced from `semantic.info` below — semantic owns it
        // so HC overrides flow through automatically.

        // Override accent tint on glass
        theme.glass.accent_tint = GlassTint {
            bg: accent,
            bg_hover: crate::foundations::color::lighten(accent, 0.08),
        };

        // Glass radii: scale sm/md/lg by the same factor so concentric and
        // padded rounds stay visually coherent on glass surfaces.
        let glass_radius_scale = 20.0 / 12.0;
        theme.radius_sm = px(f32::from(theme.radius_sm) * glass_radius_scale);
        theme.radius_md = px(f32::from(theme.radius_md) * glass_radius_scale);
        theme.radius_lg = px(20.0);

        // Apple semantic colors for dark glass.
        // System backgrounds use the dark gray substrate (L=0.07) per
        // `dark_mode.rs:19`, not pure black. The blurred window backdrop
        // provides the perceived translucency when present.
        // (sync_shorthands derives text/text_muted from these)
        theme.semantic = SemanticColors {
            label: hsla(0.0, 0.0, 0.96, 1.0),            // #F5F5F5
            secondary_label: hsla(0.0, 0.0, 1.0, 0.70),  // White 70%
            tertiary_label: hsla(0.0, 0.0, 1.0, 0.50),   // White 50%
            quaternary_label: hsla(0.0, 0.0, 1.0, 0.25), // White 25%
            quinary_label: hsla(0.0, 0.0, 1.0, 0.18),    // White 18%
            system_background: hsla(0.0, 0.0, 0.07, 1.0),
            secondary_system_background: hsla(0.0, 0.0, 0.11, 1.0),
            tertiary_system_background: hsla(0.0, 0.0, 0.15, 1.0),
            separator: hsla(0.0, 0.0, 0.33, 0.60),
            opaque_separator: hsla(0.0, 0.0, 0.23, 1.0),
            placeholder_text: hsla(0.0, 0.0, 1.0, 0.30),
            link: SystemColor::Blue.resolve(Appearance::Dark),
            system_fill: hsla(0.0, 0.0, 0.47, 0.36),
            secondary_system_fill: hsla(0.0, 0.0, 0.47, 0.32),
            tertiary_system_fill: hsla(0.0, 0.0, 0.46, 0.24),
            quaternary_system_fill: hsla(0.0, 0.0, 0.46, 0.18),
            quinary_system_fill: hsla(0.0, 0.0, 0.44, 0.12),
            system_grouped_background: hsla(0.0, 0.0, 0.07, 1.0),
            secondary_system_grouped_background: hsla(0.0, 0.0, 0.11, 1.0),
            tertiary_system_grouped_background: hsla(0.0, 0.0, 0.17, 1.0),
            elevated_system_background: hsla(0.0, 0.0, 0.11, 1.0), // #1C1C1E
            elevated_secondary_system_background: hsla(0.0, 0.0, 0.17, 1.0), // #2C2C2E
            info: SystemColor::Cyan.resolve(Appearance::Dark),
            ai: SystemColor::Purple.resolve(Appearance::Dark),
        };

        // Re-sync background from the updated semantic override.
        theme.background = theme.semantic.system_background;
        theme.sync_shorthands();

        theme
    }

    /// Sync shorthand colour fields from `semantic` to eliminate dual-write paths.
    ///
    /// Call at the end of any constructor that overrides `self.semantic` after
    /// initial construction. Syncs `text`, `text_muted`, `info`, and `ai` —
    /// the values that always track their semantic counterparts. Other
    /// shorthands (`background`, `surface`, `border`, `hover`) are intentionally
    /// NOT synced because liquid_glass themes set them to values that differ
    /// from their semantic equivalents.
    fn sync_shorthands(&mut self) {
        self.text = self.semantic.label;
        self.text_muted = self.semantic.secondary_label;
        self.info = self.semantic.info;
        self.ai = self.semantic.ai;
    }

    /// Returns the hover background color from glass tokens.
    pub fn hover_bg(&self) -> Hsla {
        self.glass.hover_bg
    }

    /// Returns the separator color for modal surfaces, adapting for contrast modes.
    pub fn modal_separator_color(&self) -> Hsla {
        if self.accessibility_mode.increase_contrast() {
            self.glass.accessibility.high_contrast_border
        } else {
            crate::foundations::color::with_alpha(self.border, 0.15)
        }
    }

    /// Tertiary label color (HIG label hierarchy level 3).
    pub fn text_tertiary(&self) -> Hsla {
        self.semantic.tertiary_label
    }
    /// Quaternary label color (HIG label hierarchy level 4).
    pub fn text_quaternary(&self) -> Hsla {
        self.semantic.quaternary_label
    }
    /// Quinary label color (HIG label hierarchy level 5).
    pub fn text_quinary(&self) -> Hsla {
        self.semantic.quinary_label
    }
    /// Semi-transparent separator (use for hairlines).
    pub fn separator_color(&self) -> Hsla {
        self.semantic.separator
    }
    /// Placeholder text color for text fields.
    pub fn placeholder_text(&self) -> Hsla {
        self.semantic.placeholder_text
    }
    /// Primary system fill (for thin/small elements like slider tracks).
    pub fn system_fill(&self) -> Hsla {
        self.semantic.system_fill
    }
    /// Secondary system fill.
    pub fn secondary_system_fill(&self) -> Hsla {
        self.semantic.secondary_system_fill
    }
    /// Tertiary system fill.
    pub fn tertiary_system_fill(&self) -> Hsla {
        self.semantic.tertiary_system_fill
    }

    /// Muted surface tint for subtle fills — e.g. zebra-striped rows in
    /// a bordered list/table. Maps to the quinary system fill (lightest
    /// of the HIG semantic fill ladder) so alternating rows stay legible
    /// without competing with selection or hover states.
    pub fn surface_muted(&self) -> Hsla {
        self.semantic.quinary_system_fill
    }

    /// Resolve primary label color for the given surface context.
    pub fn label_color(&self, context: crate::foundations::materials::SurfaceContext) -> Hsla {
        crate::foundations::materials::resolve_label(self, context, 0)
    }
    /// Resolve secondary label color for the given surface context.
    pub fn secondary_label_color(
        &self,
        context: crate::foundations::materials::SurfaceContext,
    ) -> Hsla {
        crate::foundations::materials::resolve_label(self, context, 1)
    }
    /// Resolve tertiary label color for the given surface context.
    pub fn tertiary_label_color(
        &self,
        context: crate::foundations::materials::SurfaceContext,
    ) -> Hsla {
        crate::foundations::materials::resolve_label(self, context, 2)
    }

    /// Disabled-state label color (HIG: fixed muted tint, not proportional
    /// opacity).
    ///
    /// HIG disabled controls use a fixed muted foreground rather than
    /// halving the enabled opacity, because 50 % of low-contrast variants
    /// (Ghost / Outline) would fail WCAG 4.5:1 on the window background.
    /// This returns `theme.text` at 30 % alpha, which composites against
    /// the background to a ratio that clears WCAG AA 3:1 for non-text UI
    /// (which is what "disabled" signals — the label is not meant to be
    /// read, it is meant to be seen as inert).
    pub fn text_disabled(&self) -> Hsla {
        crate::foundations::color::with_alpha(self.text, 0.3)
    }

    /// Specular rim highlight for bordered buttons (macOS 26 Liquid Glass).
    ///
    /// Liquid Glass bordered controls carry a 0.5pt inset highlight rim in
    /// addition to the flat shadow — a subtle specular-white border that
    /// separates the control's fill from the surrounding surface and is
    /// distinct from the HighContrast border (which is opaque).
    ///
    /// Returns `white` at 15 % alpha on light appearances and 8 % on dark,
    /// per Apple's Tahoe reference controls.
    pub fn specular_rim(&self) -> Hsla {
        let a = if self.appearance.is_dark() {
            0.08
        } else {
            0.15
        };
        hsla(0.0, 0.0, 1.0, a)
    }

    /// Returns true if the layout direction is right-to-left.
    pub fn is_rtl(&self) -> bool {
        self.layout_direction.is_rtl()
    }

    /// Set the first day of the week for calendar components. `0` = Sunday
    /// (US), `1` = Monday (ISO 8601 / Europe). Values outside 0..=6 are
    /// clamped to 0.
    pub fn with_first_weekday(mut self, day: u8) -> Self {
        self.first_weekday = day.min(6);
        self
    }

    /// Returns the BoxShadow layers used to render a focus ring.
    ///
    /// Per HIG (macOS 14+ / macOS 26), the focus ring is a solid
    /// `focus_ring_width`-point accent-color outline at `focus_ring_offset`
    /// points from the element edge — no soft glow. The outline is rendered
    /// as two stacked shadows: an outer accent layer whose spread equals
    /// `offset + width`, covered by an inner layer in the element's
    /// background colour whose spread equals `offset`. The inner layer
    /// "erases" the accent fill inside the gap zone, leaving only the
    /// band between `offset` and `offset + width`.
    ///
    /// The shadows are returned in back-to-front order — callers should
    /// extend their shadow vector rather than replace it, preserving any
    /// base drop-shadows (e.g. glass container shadows).
    ///
    /// `IncreaseContrast` does not change the ring geometry; it already
    /// renders solid and fully opaque so no per-mode adjustment is needed.
    pub fn focus_ring_shadows(&self) -> Vec<BoxShadow> {
        let mut ring_color = self.focus_ring_color;
        ring_color.a = 1.0;
        let offset = self.focus_ring_offset;
        let width = self.focus_ring_width;
        let outer_spread = offset + width;
        let gap_color = self.background;
        vec![
            BoxShadow {
                color: ring_color,
                offset: point(px(0.), px(0.)),
                blur_radius: px(0.),
                spread_radius: outer_spread,
            },
            BoxShadow {
                color: gap_color,
                offset: point(px(0.), px(0.)),
                blur_radius: px(0.),
                spread_radius: offset,
            },
        ]
    }

    /// Returns the effective font weight, respecting BoldText accessibility mode.
    /// When BoldText is active, bumps the weight one step (Normal -> Medium, etc.).
    pub fn effective_weight(&self, base: FontWeight) -> FontWeight {
        if self.accessibility_mode.bold_text() {
            bold_step(base)
        } else {
            base
        }
    }

    /// Returns the vertical offset for dropdown menus from their trigger.
    /// Uses the theme's `dropdown_offset` field (defaults to 4pt above the touch target).
    pub fn dropdown_top(&self) -> Pixels {
        px(self.platform.default_target_size() + f32::from(self.dropdown_offset))
    }

    /// Returns the default interactive target size for the current platform.
    ///
    /// Equivalent to `theme.control_height(ControlSize::Regular)`.
    pub fn target_size(&self) -> f32 {
        self.platform.default_target_size()
    }

    /// Returns the minimum interactive target size for the current platform.
    ///
    /// Equivalent to `theme.control_height(ControlSize::Mini)`.
    pub fn min_target_size(&self) -> f32 {
        self.platform.min_target_size()
    }

    /// Visual height in points for a
    /// [`crate::foundations::layout::ControlSize`] tier on the active
    /// platform. Prefer this over `target_size` / `min_target_size` when
    /// authoring a new component so the control picks up any future
    /// platform-scaling tweaks without code churn.
    pub fn control_height(&self, size: crate::foundations::layout::ControlSize) -> f32 {
        size.height(self.platform)
    }

    /// Returns the standard row height for menus and lists on the current platform.
    pub fn row_height(&self) -> f32 {
        self.platform.row_height()
    }

    /// Liquid Glass theme (light variant) aligned with Apple iOS 26 design system.
    ///
    /// Uses semi-transparent white glass fills on a light background.
    /// Same SF Pro type scale, motion tokens, and accessibility support as the dark variant.
    pub fn liquid_glass_light() -> Self {
        let mut theme = Self::light();

        // Apple Light Base backgrounds
        theme.background = hsla(0.0, 0.0, 0.95, 1.0); // Light base
        theme.surface = hsla(0.0, 0.0, 0.93, 1.0);
        theme.border = hsla(0.0, 0.0, 0.0, 0.08); // Light separator
        theme.panel_surface = hsla(0.0, 0.0, 1.0, 0.50); // Semi-transparent white

        // Apple accent colors (Light -- same hues, adjusted for light background)
        let accent = hsla(0.57, 1.0, 0.50, 1.0); // #0091FF Blue
        theme.accent = accent;
        theme.text_on_accent = text_on_background(accent);
        theme.ring = accent;
        theme.error = Hsla {
            l: 0.50,
            ..theme.palette.red
        };
        theme.success = Hsla {
            l: 0.42,
            ..theme.palette.green
        };
        theme.warning = Hsla {
            l: 0.55,
            ..theme.palette.orange
        };
        // `theme.info` is synced from `semantic.info` below — semantic owns it
        // so HC overrides flow through automatically.

        // Override accent tint on glass
        theme.glass.accent_tint = GlassTint {
            bg: accent,
            bg_hover: crate::foundations::color::lighten(accent, 0.08),
        };

        // Glass radii: scale sm/md/lg by the same factor so concentric and
        // padded rounds stay visually coherent on glass surfaces.
        let glass_radius_scale = 20.0 / 12.0;
        theme.radius_sm = px(f32::from(theme.radius_sm) * glass_radius_scale);
        theme.radius_md = px(f32::from(theme.radius_md) * glass_radius_scale);
        theme.radius_lg = px(20.0);

        // Apple semantic colors for light glass
        // (sync_shorthands derives text/text_muted from these)
        theme.semantic = SemanticColors {
            label: hsla(0.0, 0.0, 0.0, 1.0),
            secondary_label: hsla(0.0, 0.0, 0.0, 0.60),
            tertiary_label: hsla(0.0, 0.0, 0.0, 0.40),
            quaternary_label: hsla(0.0, 0.0, 0.0, 0.18),
            quinary_label: hsla(0.0, 0.0, 0.0, 0.10),
            system_background: hsla(0.0, 0.0, 0.95, 1.0),
            secondary_system_background: hsla(0.0, 0.0, 0.93, 1.0),
            tertiary_system_background: hsla(0.0, 0.0, 0.90, 1.0),
            separator: hsla(0.0, 0.0, 0.24, 0.29),
            opaque_separator: hsla(0.0, 0.0, 0.78, 1.0),
            placeholder_text: hsla(0.0, 0.0, 0.24, 0.30),
            link: SystemColor::Blue.resolve(Appearance::Light),
            system_fill: hsla(0.0, 0.0, 0.47, 0.20),
            secondary_system_fill: hsla(0.0, 0.0, 0.47, 0.16),
            tertiary_system_fill: hsla(0.0, 0.0, 0.46, 0.12),
            quaternary_system_fill: hsla(0.0, 0.0, 0.45, 0.08),
            quinary_system_fill: hsla(0.0, 0.0, 0.44, 0.05),
            system_grouped_background: hsla(0.0, 0.0, 0.95, 1.0),
            secondary_system_grouped_background: hsla(0.0, 0.0, 1.0, 1.0),
            tertiary_system_grouped_background: hsla(0.0, 0.0, 0.95, 1.0),
            elevated_system_background: hsla(0.0, 0.0, 0.95, 1.0), // same as system_background
            elevated_secondary_system_background: hsla(0.0, 0.0, 0.93, 1.0), // same as secondary
            info: SystemColor::Cyan.resolve(Appearance::Light),
            ai: SystemColor::Purple.resolve(Appearance::Light),
        };

        theme.sync_shorthands();

        theme
    }

    /// Liquid Glass Clear theme (dark variant).
    ///
    /// Uses `GlassVariant::Clear` for higher transparency -- suitable for
    /// media-rich content per HIG.
    pub fn liquid_glass_clear() -> Self {
        let mut theme = Self::liquid_glass();
        theme.glass.variant = GlassVariant::Clear;
        theme.glass.preference = LiquidGlassPreference::Clear;
        theme
    }

    /// Liquid Glass Clear theme (light variant).
    ///
    /// Uses `GlassVariant::Clear` for higher transparency on a light background.
    pub fn liquid_glass_clear_light() -> Self {
        let mut theme = Self::liquid_glass_light();
        theme.glass.variant = GlassVariant::Clear;
        theme.glass.preference = LiquidGlassPreference::Clear;
        theme
    }

    /// Builder: set the Liquid Glass preference (Clear or Tinted).
    pub fn with_glass_preference(mut self, pref: LiquidGlassPreference) -> Self {
        self.glass.preference = pref;
        self
    }

    /// Apply this theme globally and refresh all windows.
    ///
    /// Use this instead of `cx.set_global()` + `cx.notify()` for runtime theme switching.
    /// Calling `refresh_windows` ensures every component — stateful or stateless — picks
    /// up the new tokens on the next frame.
    ///
    /// `refresh_windows` is cheap (~1 wm-level invalidation per open window), so it is
    /// always called.
    pub fn apply(self, cx: &mut App) {
        cx.set_global(self);
        cx.refresh_windows();
    }

    /// Apply this theme and update the window's background appearance.
    ///
    /// Sets the macOS `NSVisualEffectView` blur from the glass window_background token.
    pub fn apply_in_window(self, window: &mut Window, cx: &mut App) {
        let appearance = self.glass.window_background;
        window.set_background_appearance(appearance);
        self.apply(cx);
    }

    /// Create a theme matching the system appearance (non-glass).
    pub fn for_appearance(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::light(),
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::dark(),
        }
    }

    /// Like [`Self::for_appearance`] but promotes to a HighContrast appearance
    /// when `mode.increase_contrast()` is set.
    pub fn for_appearance_with_a11y(
        appearance: WindowAppearance,
        mode: crate::foundations::accessibility::AccessibilityMode,
    ) -> Self {
        let base = Self::for_appearance(appearance);
        if mode.increase_contrast() {
            let hc_appearance = if base.appearance.is_dark() {
                Appearance::DarkHighContrast
            } else {
                Appearance::LightHighContrast
            };
            Self::with_accent(hc_appearance, base.accent_color)
        } else {
            base
        }
    }

    /// Create a glass theme matching the system appearance.
    pub fn for_appearance_glass(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::liquid_glass_light(),
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::liquid_glass(),
        }
    }

    /// Install a non-glass theme that tracks the window's system appearance.
    ///
    /// Applies `Self::for_appearance(window.appearance())` immediately and returns
    /// a subscription that swaps the global theme whenever the OS toggles between
    /// light and dark mode. Keep the returned `Subscription` alive (typically on
    /// the root view) for the observer to remain active.
    ///
    /// **Accent colour:** GPUI does not expose the macOS system accent
    /// (`NSColor.controlAccentColor`) yet, so this helper falls back to
    /// [`AccentColor::Multicolor`] (Blue). Hosts that have AppKit access
    /// should call [`Self::install_with_system_appearance_and_accent`] with
    /// the value they read out themselves. See the doc comment on
    /// [`TahoeTheme::accent_color`] for the broader rationale.
    pub fn install_with_system_appearance(window: &mut Window, cx: &mut App) -> gpui::Subscription {
        Self::install_with_system_appearance_and_accent(window, cx, AccentColor::default())
    }

    /// Like [`Self::install_with_system_appearance`] but accepts an explicit
    /// accent colour. Use this when the host can read the user's macOS
    /// accent preference (e.g. via AppKit FFI) — the accent persists across
    /// light/dark switches.
    pub fn install_with_system_appearance_and_accent(
        window: &mut Window,
        cx: &mut App,
        accent: AccentColor,
    ) -> gpui::Subscription {
        let theme_for = move |w: &Window| {
            let appearance = match w.appearance() {
                WindowAppearance::Light | WindowAppearance::VibrantLight => Appearance::Light,
                WindowAppearance::Dark | WindowAppearance::VibrantDark => Appearance::Dark,
            };
            Self::with_accent(appearance, accent)
        };
        theme_for(window).apply(cx);
        window.observe_window_appearance(move |window, cx| {
            theme_for(window).apply(cx);
        })
    }

    /// Install a Liquid Glass theme that tracks the window's system appearance.
    ///
    /// Applies `Self::for_appearance_glass(window.appearance())` immediately and
    /// returns a subscription that swaps the global theme when the OS toggles
    /// between light and dark. Keep the returned `Subscription` alive.
    ///
    /// **Accent colour:** GPUI does not expose the macOS system accent
    /// (`NSColor.controlAccentColor`) yet, so this helper leaves the accent
    /// baked into the Liquid Glass presets
    /// ([`Self::liquid_glass`] / [`Self::liquid_glass_light`]) untouched.
    /// Hosts that have AppKit access and want to override with the user's
    /// system accent should call
    /// [`Self::install_glass_with_system_appearance_and_accent`].
    pub fn install_glass_with_system_appearance(
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Subscription {
        Self::for_appearance_glass(window.appearance()).apply_in_window(window, cx);
        window.observe_window_appearance(|window, cx| {
            Self::for_appearance_glass(window.appearance()).apply_in_window(window, cx);
        })
    }

    /// Like [`Self::install_glass_with_system_appearance`] but accepts an
    /// explicit accent colour. Use this when the host can read the user's
    /// macOS accent preference (e.g. via AppKit FFI) — the accent persists
    /// across light/dark switches and propagates through the glass theme's
    /// accent-derived tokens (`accent`, `ring`, `focus_ring_color`,
    /// `glass.accent_tint`, `text_on_accent`).
    pub fn install_glass_with_system_appearance_and_accent(
        window: &mut Window,
        cx: &mut App,
        accent: AccentColor,
    ) -> gpui::Subscription {
        let theme_for =
            move |w: &Window| Self::for_appearance_glass(w.appearance()).with_accent_color(accent);
        theme_for(window).apply_in_window(window, cx);
        window.observe_window_appearance(move |window, cx| {
            theme_for(window).apply_in_window(window, cx);
        })
    }

    /// Set the user's preferred font scale factor.
    ///
    /// macOS exposes a global text size preference in System Settings →
    /// Accessibility → Display → Text Size. `1.0` is the HIG default; values
    /// above 1 scale all system text styles proportionally. Hosts that can
    /// read `NSFontDescriptor`'s preferred body size via AppKit should feed
    /// the ratio (`preferred / default`) into this builder so the type scale
    /// respects the user's choice.
    ///
    /// Non-positive or non-finite inputs are clamped to `1.0`.
    pub fn with_font_scale_factor(mut self, factor: f32) -> Self {
        self.font_scale_factor = if factor.is_finite() && factor > 0.0 {
            factor
        } else {
            1.0
        };
        self
    }

    /// Set the Dynamic Type size the theme reports via
    /// [`TahoeTheme::text_size_for`]. Default is
    /// [`DynamicTypeSize::Large`] (the HIG baseline); hosts that read
    /// the user's Dynamic Type preference from the OS should propagate
    /// it here. Finding 26 in the Zed cross-reference audit.
    pub fn with_dynamic_type_size(mut self, size: DynamicTypeSize) -> Self {
        self.dynamic_type_size = size;
        self
    }

    /// Returns the point size for `style` at the current Dynamic Type
    /// size, honouring both [`TahoeTheme::dynamic_type_size`] and
    /// [`TahoeTheme::font_scale_factor`]. Equivalent to reading
    /// `style.attrs().size` scaled by the current user preferences.
    ///
    /// # Single source of truth
    ///
    /// Platform-aware so the output stays consistent with the explicit
    /// [`TextStyle::ios_attrs`] and [`TextStyle::attrs`] tables — callers
    /// that mix `text_size_for` with direct `style.ios_attrs(level).size`
    /// reads see identical values.
    ///
    /// * On iOS / iPadOS / visionOS / watchOS Dynamic Type is a first-class
    ///   system feature; we route through [`TextStyle::ios_attrs`] so every
    ///   style × level combination matches Apple's published HIG table
    ///   exactly.
    /// * On macOS / tvOS there is no native Dynamic Type; we instead scale
    ///   the macOS baseline ([`TextStyle::attrs`]) by an iOS-style
    ///   multiplier so hosts exposing a "text size" slider still get
    ///   proportional sizing.
    ///
    /// In both branches the result is multiplied by `font_scale_factor`
    /// so the accessibility "Text Size" preference flows through uniformly.
    pub fn text_size_for(&self, style: TextStyle) -> Pixels {
        let base_pt: f32 = match self.platform {
            Platform::IOS | Platform::VisionOS | Platform::WatchOS => {
                f32::from(style.ios_attrs(self.dynamic_type_size).size)
            }
            Platform::MacOS | Platform::TvOS => {
                f32::from(style.attrs().size) * dynamic_type_multiplier(self.dynamic_type_size)
            }
        };
        Pixels::from(base_pt * self.font_scale_factor)
    }

    /// Replace the theme's accent colour and propagate it through the
    /// derived tokens (`accent`, `ring`, `focus_ring_color`,
    /// `glass.accent_tint`, `text_on_accent`, and `selected_bg`).
    ///
    /// Useful when the host detects a runtime accent change after the
    /// theme has been built — call this before `apply` (or on a clone),
    /// then re-apply.
    ///
    /// Note: `tool_approved_bg` and `tool_rejected_bg` are palette-keyed
    /// (green / red), not accent-keyed, so they intentionally stay put.
    pub fn with_accent_color(mut self, accent: AccentColor) -> Self {
        let resolved = accent.resolve(&self.palette);
        self.accent_color = accent;
        self.accent = resolved;
        self.ring = resolved;
        self.focus_ring_color = resolved;
        self.text_on_accent = text_on_background(resolved);
        self.glass.accent_tint = GlassTint {
            bg: resolved,
            bg_hover: crate::foundations::color::lighten(resolved, 0.08),
        };
        self.selected_bg = Self::selected_bg_for(resolved, self.appearance.is_dark());
        self
    }
}

impl Default for TahoeTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl gpui::Global for TahoeTheme {}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Architectural plan: Arc-wrap the global to make theme swaps cheap.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//
// `TahoeTheme` is ~1 KiB (`SystemPalette` + `SemanticColors` + `SyntaxColors`
// + `AnsiColors` + `GlassStyle` + ~30 tokens). `cx.set_global(self)` above
// stores the whole value, so every `apply()` call copies the struct.
//
// Zed solves this by wrapping the theme in `GlobalTheme(Arc<Theme>)`.
// We can adopt the same pattern *without* touching the 112 call sites that
// currently read `cx.global::<TahoeTheme>()` by restructuring as:
//
// ```rust
// pub struct TahoeTheme { inner: Arc<TahoeThemeData> }
// pub struct TahoeThemeData { /* all current pub fields */ }
// impl Deref for TahoeTheme { type Target = TahoeThemeData; ... }
// impl gpui::Global for TahoeTheme {}
// ```
//
// `cx.global::<TahoeTheme>().background` continues to compile via
// `Deref<Target = TahoeThemeData>`. `cx.set_global(theme)` stores ~8
// bytes (Arc pointer) instead of the full struct. Mutating builders
// (`with_accent_color`, `…`) take `Arc::make_mut(&mut self.inner)` — the
// only invasive change is in the methods that assign fields.
//
// Tracked here rather than landed because the refactor needs to walk
// every `TahoeTheme::*` method and rewrite `mut self`-style builders to
// thread `Arc::make_mut`; doing it in the same PR as the present sweep
// would risk subtle perf or aliasing regressions. Acceptance criteria
// for the follow-up:
//   1. `cx.global::<TahoeTheme>()` and `cx.theme()` keep returning the
//      same data through the Deref shim (no call-site changes).
//   2. `mem::size_of::<TahoeTheme>() <= 16` (Arc pointer + niche).
//   3. `apply()` cost is one `Arc::clone` + one `cx.refresh_windows()`.

/// Multiplier applied to the default Large body size for each Dynamic
/// Type level.
///
/// Values approximate Apple's documented iOS type scale
/// (`UIContentSizeCategory`): XSmall ≈ 0.82, Small ≈ 0.88, Medium ≈ 0.94,
/// Large = 1.0 (baseline), XLarge ≈ 1.06, XXLarge ≈ 1.12, XXXLarge ≈ 1.18,
/// AX1 ≈ 1.35, AX2 ≈ 1.59, AX3 ≈ 1.94, AX4 ≈ 2.35, AX5 ≈ 2.76. The AX*
/// range implements Apple's "larger accessibility sizes" — hosts that
/// opt out of the accessibility scale should cap at XXXLarge.
fn dynamic_type_multiplier(size: DynamicTypeSize) -> f32 {
    match size {
        DynamicTypeSize::XSmall => 0.82,
        DynamicTypeSize::Small => 0.88,
        DynamicTypeSize::Medium => 0.94,
        DynamicTypeSize::Large => 1.0,
        DynamicTypeSize::XLarge => 1.06,
        DynamicTypeSize::XXLarge => 1.12,
        DynamicTypeSize::XXXLarge => 1.18,
        DynamicTypeSize::AX1 => 1.35,
        DynamicTypeSize::AX2 => 1.59,
        DynamicTypeSize::AX3 => 1.94,
        DynamicTypeSize::AX4 => 2.35,
        DynamicTypeSize::AX5 => 2.76,
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ActiveTheme — Zed-style `cx.theme()` accessor
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//
// Finding 1 in the Zed cross-reference audit. The
// crate still has ~200 call-sites that reach for the theme via
// `cx.global::<TahoeTheme>()` — a concrete-type coupling that makes future
// theme-backing-store changes a per-file sweep. Zed paid the same tax and
// resolved it with a tiny `ActiveTheme` trait so every caller uses
// `cx.theme()` instead.
//
// This trait is the additive first step: new code should reach for
// `cx.theme()`, and the existing `cx.global::<TahoeTheme>()` form is kept as
// a working alias so we can migrate call-sites incrementally without a
// single mega-PR. The raw form may also still be needed inside `TahoeTheme`
// itself (e.g. `theme::apply`), which is why we don't deprecate it yet.

/// Implementing this trait lets callers reach for the active [`TahoeTheme`]
/// via a uniform `cx.theme()` accessor — the same shape Zed uses in
/// `crates/theme/src/theme.rs`.
///
/// All GPUI contexts (`App`, `Window`-bound contexts, `Context<Entity>`)
/// deref down to `App`, so implementing the trait on `App` makes it work
/// everywhere a component would previously have written
/// `cx.global::<TahoeTheme>()`.
///
/// ```ignore
/// use tahoe_gpui::foundations::theme::ActiveTheme;
///
/// fn render(cx: &mut Context<Self>) {
///     let theme = cx.theme();
///     // …use `theme` like before…
/// }
/// ```
pub trait ActiveTheme {
    /// Returns the currently-registered [`TahoeTheme`] global.
    ///
    /// Panics if no theme has been registered via `cx.set_global(TahoeTheme::…)`
    /// — same semantics as `cx.global::<TahoeTheme>()`.
    fn theme(&self) -> &TahoeTheme;
}

impl ActiveTheme for gpui::App {
    fn theme(&self) -> &TahoeTheme {
        self.global::<TahoeTheme>()
    }
}

#[cfg(test)]
mod tests;
