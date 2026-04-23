use super::{
    AccessibilityMode, ActiveTheme, DynamicTypeSize, Elevation, FontDesign, Glass, GlassTintColor,
    LabelLevel, LeadingStyle, TahoeTheme, TextStyle, TextStyledExt, bold_step, contrast_ratio,
    macos_tracking, meets_contrast,
};
use crate::foundations::color::{AccentColor, Appearance};
use core::prelude::v1::test;
use gpui::{FontFallbacks, FontWeight, SharedString, Styled, div, hsla};

#[test]
fn dark_theme_has_dark_background() {
    let theme = TahoeTheme::dark();
    assert!(theme.background.l < 0.2);
}

#[test]
fn light_theme_has_light_background() {
    let theme = TahoeTheme::light();
    assert!(theme.background.l > 0.8);
}

#[test]
fn selected_bg_distinct_from_hover() {
    // Finding N6/#7: selection and hover must be visually distinct.
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        assert_ne!(
            theme.selected_bg, theme.hover,
            "selected_bg must not alias hover"
        );
    }
}

#[test]
fn selected_bg_is_tinted_accent() {
    // The selected fill is a low-alpha accent tint — hue and saturation
    // should track the accent so appearance-aware theming stays
    // consistent.
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        assert_eq!(theme.selected_bg.h, theme.accent.h);
        assert_eq!(theme.selected_bg.s, theme.accent.s);
        assert!(theme.selected_bg.a > 0.0 && theme.selected_bg.a < 1.0);
    }
}

#[test]
fn all_spacing_values_positive() {
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        assert!(f32::from(theme.spacing_xs) > 0.0);
        assert!(f32::from(theme.spacing_sm) > 0.0);
        assert!(f32::from(theme.spacing_md) > 0.0);
        assert!(f32::from(theme.spacing_lg) > 0.0);
        assert!(f32::from(theme.spacing_xl) > 0.0);
        assert!(f32::from(theme.spacing_2xl) > 0.0);
        assert!(f32::from(theme.spacing_3xl) > 0.0);
    }
}

#[test]
fn spacing_ordered() {
    let theme = TahoeTheme::dark();
    assert!(theme.spacing_xs < theme.spacing_sm);
    assert!(theme.spacing_sm < theme.spacing_md);
    assert!(theme.spacing_md < theme.spacing_lg);
    assert!(theme.spacing_lg < theme.spacing_xl);
    assert!(theme.spacing_xl < theme.spacing_2xl);
    assert!(theme.spacing_2xl < theme.spacing_3xl);
}

#[test]
fn radius_values_positive() {
    let theme = TahoeTheme::dark();
    assert!(f32::from(theme.radius_sm) > 0.0);
    assert!(f32::from(theme.radius_md) > 0.0);
    assert!(f32::from(theme.radius_lg) > 0.0);
    assert!(f32::from(theme.radius_full) > 0.0);
    assert!(f32::from(theme.radius_segmented) > 0.0);
}

#[test]
fn radius_ordered() {
    let theme = TahoeTheme::dark();
    assert!(theme.radius_sm < theme.radius_md);
    assert!(theme.radius_md < theme.radius_lg);
    assert!(theme.radius_lg < theme.radius_full);
}

#[test]
fn component_sizing_tokens_positive() {
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        assert!(f32::from(theme.icon_size_inline) > 0.0);
        assert!(f32::from(theme.dropdown_offset) > 0.0);
        assert!(f32::from(theme.separator_thickness) > 0.0);
        assert!(f32::from(theme.sidebar_width_default) > 0.0);
    }
}

#[test]
fn default_is_dark() {
    let default = TahoeTheme::default();
    let dark = TahoeTheme::dark();
    assert_eq!(default.background.l, dark.background.l);
}

#[test]
fn dark_and_light_themes_have_different_backgrounds() {
    let dark = TahoeTheme::dark();
    let light = TahoeTheme::light();
    assert_ne!(dark.background.l, light.background.l);
}

#[test]
fn ansi_colors_present() {
    let theme = TahoeTheme::dark();
    assert_eq!(theme.ansi.black.a, 1.0);
    assert_eq!(theme.ansi.red.a, 1.0);
    assert_eq!(theme.ansi.green.a, 1.0);
    assert_eq!(theme.ansi.white.a, 1.0);
    assert_eq!(theme.ansi.bright_white.a, 1.0);
}

#[test]
fn syntax_colors_present() {
    let theme = TahoeTheme::dark();
    assert_eq!(theme.syntax.keyword.a, 1.0);
    assert_eq!(theme.syntax.string.a, 1.0);
    assert_eq!(theme.syntax.comment.a, 1.0);
    assert_eq!(theme.syntax.function.a, 1.0);
}

#[test]
fn component_sizes_positive() {
    let theme = TahoeTheme::dark();
    assert!(f32::from(theme.avatar_size) > 0.0);
    assert!(f32::from(theme.icon_size) > 0.0);
    assert!(theme.shimmer_duration_ms > 0);
}

#[test]
fn shimmer_duration_matches_reference() {
    assert_eq!(TahoeTheme::dark().shimmer_duration_ms, 2000);
    assert_eq!(TahoeTheme::light().shimmer_duration_ms, 2000);
}

#[test]
fn panel_surface_is_semi_transparent() {
    let dark = TahoeTheme::dark();
    let light = TahoeTheme::light();
    assert!(
        dark.panel_surface.a < 1.0,
        "dark panel_surface should be semi-transparent"
    );
    assert!(
        light.panel_surface.a < 1.0,
        "light panel_surface should be semi-transparent"
    );
}

// ─── Liquid Glass Tests ──────────────────────────────────────────────────

#[test]
fn all_themes_have_glass_tokens() {
    // Per HIG macOS Tahoe, glass is always present — every theme must
    // populate the canonical Regular + Clear fills so glass surfaces
    // composite correctly regardless of the theme the caller picked.
    for theme in [
        TahoeTheme::dark(),
        TahoeTheme::light(),
        TahoeTheme::liquid_glass(),
    ] {
        assert!(theme.glass.regular_fill.a > 0.0);
        assert!(theme.glass.clear_fill.a > 0.0);
    }
}

#[test]
fn glass_regular_fill_is_semi_transparent() {
    // The primary Regular fill must be translucent: the window-level
    // NSVisualEffectView blur can only show through fractional-alpha fills.
    let glass = TahoeTheme::liquid_glass().glass;
    assert!(
        glass.regular_fill.a > 0.0 && glass.regular_fill.a < 1.0,
        "regular_fill should be semi-transparent, got {}",
        glass.regular_fill.a
    );
    assert!(
        glass.clear_fill.a > 0.0 && glass.clear_fill.a < 1.0,
        "clear_fill should be semi-transparent, got {}",
        glass.clear_fill.a
    );
}

#[test]
fn glass_hover_more_opaque_than_clear_fill() {
    // Hover is an additive overlay on the glass surface — it should have
    // substantial alpha so the hover state is visible over the Clear
    // (thinnest) fill.
    let glass = TahoeTheme::liquid_glass().glass;
    assert!(
        glass.hover_bg.a > 0.0,
        "hover_bg should have positive alpha"
    );
    assert!(
        glass.hover_bg.a > glass.clear_fill.a,
        "hover_bg alpha ({}) should exceed clear_fill alpha ({})",
        glass.hover_bg.a,
        glass.clear_fill.a
    );
}

#[test]
fn glass_tints_all_semi_transparent() {
    let glass = TahoeTheme::liquid_glass().glass;
    for color in [
        GlassTintColor::Green,
        GlassTintColor::Blue,
        GlassTintColor::Purple,
        GlassTintColor::Amber,
        GlassTintColor::Red,
    ] {
        let tint = glass.tints.get(color);
        assert!(tint.bg.a > 0.0 && tint.bg.a < 0.2);
        assert!(tint.bg_hover.a > tint.bg.a);
    }
}

#[test]
fn liquid_glass_background_is_dark() {
    let theme = TahoeTheme::liquid_glass();
    assert!(theme.background.l < 0.15);
}

#[test]
fn for_appearance_glass_with_a11y_promotes_to_hc_when_requested() {
    let mode = AccessibilityMode::INCREASE_CONTRAST;
    let dark = TahoeTheme::for_appearance_glass_with_a11y(gpui::WindowAppearance::Dark, mode);
    assert!(dark.appearance.is_high_contrast());
    assert!(dark.appearance.is_dark());
    // Palette is actually swapped, not just the appearance flag.
    let dark_base = TahoeTheme::for_appearance_glass(gpui::WindowAppearance::Dark);
    assert_ne!(dark.palette.red, dark_base.palette.red);

    let light = TahoeTheme::for_appearance_glass_with_a11y(gpui::WindowAppearance::Light, mode);
    assert!(light.appearance.is_high_contrast());
    assert!(!light.appearance.is_dark());
    let light_base = TahoeTheme::for_appearance_glass(gpui::WindowAppearance::Light);
    assert_ne!(light.palette.red, light_base.palette.red);
}

#[test]
fn for_appearance_glass_with_a11y_no_hc_matches_base() {
    let base = TahoeTheme::for_appearance_glass(gpui::WindowAppearance::Dark);
    let same = TahoeTheme::for_appearance_glass_with_a11y(
        gpui::WindowAppearance::Dark,
        AccessibilityMode::DEFAULT,
    );
    assert_eq!(base.appearance, same.appearance);
    assert!(!same.appearance.is_high_contrast());
}

#[test]
fn for_appearance_glass_with_a11y_propagates_full_mode() {
    // Flags other than INCREASE_CONTRAST must be written through to
    // `theme.accessibility_mode` so downstream motion/bold-text branches
    // see the caller's intent.
    let mode = AccessibilityMode::REDUCE_MOTION | AccessibilityMode::BOLD_TEXT;
    let theme = TahoeTheme::for_appearance_glass_with_a11y(gpui::WindowAppearance::Dark, mode);
    assert_eq!(theme.accessibility_mode, mode);
    assert!(!theme.appearance.is_high_contrast());

    let hc = mode | AccessibilityMode::INCREASE_CONTRAST;
    let theme = TahoeTheme::for_appearance_glass_with_a11y(gpui::WindowAppearance::Light, hc);
    assert_eq!(theme.accessibility_mode, hc);
    assert!(theme.appearance.is_high_contrast());
}

#[test]
fn for_appearance_with_a11y_propagates_full_mode() {
    // Same guarantee as the glass sibling: the full AccessibilityMode flows
    // into `theme.accessibility_mode`, not just the INCREASE_CONTRAST bit.
    let mode = AccessibilityMode::REDUCE_MOTION | AccessibilityMode::BOLD_TEXT;
    let theme = TahoeTheme::for_appearance_with_a11y(gpui::WindowAppearance::Dark, mode);
    assert_eq!(theme.accessibility_mode, mode);
    assert!(!theme.appearance.is_high_contrast());
}

#[cfg(target_os = "macos")]
#[test]
fn liquid_glass_window_is_blurred() {
    let glass = TahoeTheme::liquid_glass().glass;
    assert!(matches!(
        glass.window_background,
        gpui::WindowBackgroundAppearance::Blurred
    ));
}

#[test]
fn liquid_glass_root_bg_is_semi_transparent() {
    let glass = TahoeTheme::liquid_glass().glass;
    assert!(glass.root_bg.a > 0.0 && glass.root_bg.a < 1.0);
}

#[test]
fn resting_shadow_count() {
    let glass = TahoeTheme::liquid_glass().glass;
    // Per Figma: single 4pt drop shadow for the Resting tier (controls).
    assert_eq!(glass.resting_shadows.len(), 1);
}

#[test]
fn floating_shadow_count() {
    let glass = TahoeTheme::liquid_glass().glass;
    // Per Figma: single 40pt drop shadow for Floating tier.
    assert_eq!(glass.floating_shadows.len(), 1);
}

#[test]
fn elevated_shadow_stack_matches_figma() {
    // Figma Tahoe UI Kit "BG - Medium UI" uses a two-layer shadow stack:
    //   1. Ambient  — Y=8, Blur=40, Spread=0, #000 @ 12%
    //   2. Rim      — Y=0, Blur=0, Spread=1, #000 @ 23%
    // The rim keeps the panel edge legible against low-contrast backdrops
    // where the ambient blur fades into the underlying content.
    let glass = TahoeTheme::liquid_glass().glass;
    assert_eq!(glass.elevated_shadows.len(), 2, "expected ambient + rim");

    let ambient = &glass.elevated_shadows[0];
    assert!((f32::from(ambient.offset.y) - 8.0).abs() < f32::EPSILON);
    assert!((f32::from(ambient.blur_radius) - 40.0).abs() < f32::EPSILON);
    assert!((f32::from(ambient.spread_radius) - 0.0).abs() < f32::EPSILON);
    assert!((ambient.color.a - 0.12).abs() < 1e-5);

    let rim = &glass.elevated_shadows[1];
    assert!((f32::from(rim.offset.y) - 0.0).abs() < f32::EPSILON);
    assert!((f32::from(rim.blur_radius) - 0.0).abs() < f32::EPSILON);
    assert!((f32::from(rim.spread_radius) - 1.0).abs() < f32::EPSILON);
    assert!((rim.color.a - 0.23).abs() < 1e-5);
}

#[test]
fn shadows_scale_by_elevation() {
    // Figma spec: Resting is a tight 4pt drop shadow (controls), Elevated
    // and Floating both use the 40pt ambient per the Tahoe UI Kit (Medium
    // UI is the canonical elevated panel tier). Assert the ordering
    // Resting < Elevated == Floating for the ambient layer (index 0).
    let glass = TahoeTheme::liquid_glass().glass;
    let resting_blur = f32::from(glass.resting_shadows[0].blur_radius);
    let elevated_blur = f32::from(glass.elevated_shadows[0].blur_radius);
    let floating_blur = f32::from(glass.floating_shadows[0].blur_radius);
    assert!(
        resting_blur < elevated_blur,
        "resting blur ({resting_blur}) should be less than elevated ({elevated_blur})"
    );
    assert!(
        (elevated_blur - floating_blur).abs() < f32::EPSILON,
        "elevated blur ({elevated_blur}) should match floating ({floating_blur}) per Figma Medium UI"
    );
}

#[test]
fn glass_labels_dim_primary_light() {
    let glass = TahoeTheme::liquid_glass().glass;
    assert!((glass.labels_dim.primary.l - 0.96).abs() < 0.01);
}

#[test]
fn glass_labels_bright_primary_dark() {
    let glass = TahoeTheme::liquid_glass().glass;
    assert!((glass.labels_bright.primary.l - 0.10).abs() < 0.01);
}

#[test]
fn apple_accent_blue() {
    let theme = TahoeTheme::liquid_glass();
    // #0091FF → h: 0.57, s: 1.0, l: 0.50
    assert!((theme.accent.h - 0.57).abs() < 0.02);
    assert!((theme.accent.s - 1.0).abs() < 0.05);
    assert!((theme.accent.l - 0.50).abs() < 0.02);
}

#[test]
fn liquid_glass_accent_color_enum_matches_accent() {
    // Regression for #24: liquid_glass/liquid_glass_light must report
    // AccentColor::Blue (not the default Multicolor) because they override
    // the accent to a specific blue. The enum must not lie.
    assert_eq!(TahoeTheme::liquid_glass().accent_color, AccentColor::Blue);
    assert_eq!(
        TahoeTheme::liquid_glass_light().accent_color,
        AccentColor::Blue
    );

    // Replaying with_accent_color with the theme's own accent_color
    // preserves the enum value (pins the self-consistency invariant the
    // bug was really about).
    let theme = TahoeTheme::liquid_glass();
    let replayed = theme.clone().with_accent_color(theme.accent_color);
    assert_eq!(replayed.accent_color, theme.accent_color);

    // The constructor's hardcoded accent flows through to ring,
    // focus_ring_color, and the glass accent tint — pinning the
    // "no pixel change" contract against a future refactor that
    // accidentally routes these through palette.blue.
    assert_eq!(theme.ring, theme.accent);
    assert_eq!(theme.focus_ring_color, theme.accent);
    assert_eq!(theme.glass.accent_tint.bg, theme.accent);
}

#[test]
fn with_accent_color_propagates_to_derived_tokens() {
    // Switching the accent updates accent / ring / focus_ring / text_on_accent,
    // the glass accent tint, and selected_bg, so a host that detects a runtime
    // accent change can rebuild without losing the rest of the theme.
    let base = TahoeTheme::dark();
    let purple = base.clone().with_accent_color(AccentColor::Purple);
    assert_ne!(purple.accent, base.accent);
    assert_eq!(purple.accent, purple.palette.purple);
    assert_eq!(purple.ring, purple.accent);
    assert_eq!(purple.focus_ring_color, purple.accent);
    assert_eq!(purple.glass.accent_tint.bg, purple.accent);
    assert_eq!(purple.accent_color, AccentColor::Purple);
    // selected_bg tracks the new accent with the dark-mode alpha.
    assert_eq!(purple.selected_bg.h, purple.accent.h);
    assert_eq!(purple.selected_bg.s, purple.accent.s);
    assert_eq!(purple.selected_bg.l, purple.accent.l);
    assert!((purple.selected_bg.a - 0.28).abs() < f32::EPSILON);
    assert_ne!(purple.selected_bg, base.selected_bg);
    // Tool tints are palette-keyed (green/red), not accent-keyed — invariant.
    assert_eq!(purple.tool_approved_bg, base.tool_approved_bg);
    assert_eq!(purple.tool_rejected_bg, base.tool_rejected_bg);
    // Non-accent fields stay put.
    assert_eq!(purple.background, base.background);
    assert_eq!(purple.text, base.text);
    // Light mode exercises the other arm of the appearance-conditional alpha (0.18).
    let light_purple = TahoeTheme::light().with_accent_color(AccentColor::Purple);
    assert!((light_purple.selected_bg.a - 0.18).abs() < f32::EPSILON);
}

#[test]
fn background_uses_system_dark_gray() {
    // Per dark_mode.rs:19 we don't render pure black backgrounds — the
    // liquid-glass theme uses the system dark gray substrate so the surface
    // remains legible when the blurred backdrop is unavailable or
    // transparency is disabled.
    let theme = TahoeTheme::liquid_glass();
    assert!((theme.background.l - 0.07).abs() < 0.005);
    assert_eq!(theme.background, theme.semantic.system_background);
}

// ─── Phase 1: New Token Tests ────────────────────────────────────────────

#[test]
fn semantic_colors_all_themes() {
    for theme in [
        TahoeTheme::dark(),
        TahoeTheme::light(),
        TahoeTheme::liquid_glass(),
    ] {
        assert!(theme.semantic.label.a > 0.0);
        assert!(theme.semantic.secondary_label.a > 0.0);
        assert!(theme.semantic.tertiary_label.a > 0.0);
        assert!(theme.semantic.quaternary_label.a > 0.0);
    }
}

#[test]
fn semantic_label_hierarchy() {
    // Primary label should be more prominent than secondary, etc.
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        assert!(theme.semantic.label.a >= theme.semantic.secondary_label.a);
        assert!(theme.semantic.secondary_label.a >= theme.semantic.tertiary_label.a);
        assert!(theme.semantic.tertiary_label.a >= theme.semantic.quaternary_label.a);
    }
}

#[test]
fn glass_colors_have_full_alpha() {
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        assert_eq!(
            theme.glass.icon_success.a, 1.0,
            "icon_success should be fully opaque"
        );
        assert_eq!(
            theme.glass.icon_info.a, 1.0,
            "icon_info should be fully opaque"
        );
        assert_eq!(
            theme.glass.icon_warning.a, 1.0,
            "icon_warning should be fully opaque"
        );
        assert_eq!(
            theme.glass.icon_error.a, 1.0,
            "icon_error should be fully opaque"
        );
        assert_eq!(theme.glass.icon_ai.a, 1.0, "icon_ai should be fully opaque");
    }
}

#[test]
fn glass_variant_is_regular() {
    let glass = TahoeTheme::liquid_glass().glass;
    assert_eq!(glass.variant, Glass::Regular);
}

#[test]
fn glass_clear_fill_is_semi_transparent() {
    let glass = TahoeTheme::liquid_glass().glass;
    assert!(glass.clear_fill.a > 0.0 && glass.clear_fill.a < 0.3);
}

#[test]
fn glass_tiles_are_semi_transparent() {
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        assert!(
            (0.0..1.0).contains(&theme.glass.tile_bg.a),
            "tile_bg should be semi-transparent, got {}",
            theme.glass.tile_bg.a,
        );
        assert!(
            (0.0..1.0).contains(&theme.glass.tile_border.a),
            "tile_border should be semi-transparent, got {}",
            theme.glass.tile_border.a,
        );
    }
}

#[test]
fn glass_fill_differs_between_regular_and_clear() {
    // SwiftUI's `Glass` material split: Regular and Clear each hold a
    // single canonical fill that must not alias — the two variants are
    // tunes for different translucency targets (adaptive blur +
    // luminosity vs. maximum translucency).
    let glass = TahoeTheme::liquid_glass().glass;
    let regular = glass.fill(Glass::Regular);
    let clear = glass.fill(Glass::Clear);
    assert_ne!(regular, clear);
    assert_eq!(regular, glass.regular_fill);
    assert_eq!(clear, glass.clear_fill);
}

#[test]
fn glass_fill_identity_is_transparent() {
    let glass = TahoeTheme::liquid_glass().glass;
    assert_eq!(glass.fill(Glass::Identity).a, 0.0);
}

#[test]
fn glass_all_tints_semi_transparent() {
    let glass = TahoeTheme::liquid_glass().glass;
    for color in [
        GlassTintColor::Green,
        GlassTintColor::Blue,
        GlassTintColor::Purple,
        GlassTintColor::Amber,
        GlassTintColor::Red,
        GlassTintColor::Cyan,
        GlassTintColor::Teal,
        GlassTintColor::Indigo,
    ] {
        let tint = glass.tints.get(color);
        assert!(
            tint.bg.a > 0.0 && tint.bg.a < 0.2,
            "tint bg alpha should be between 0 and 0.2, got {}",
            tint.bg.a
        );
        assert!(tint.bg_hover.a > tint.bg.a);
    }
}

#[test]
fn accessibility_tokens_valid() {
    let glass = TahoeTheme::liquid_glass().glass;
    assert!(glass.accessibility.reduced_transparency_bg.a > 0.5);
    assert!(glass.accessibility.high_contrast_border.a > 0.0);
    assert!(
        glass.accessibility.reduced_motion_scale >= 0.0
            && glass.accessibility.reduced_motion_scale <= 1.0
    );
}

#[test]
fn motion_tokens_positive() {
    let glass = TahoeTheme::liquid_glass().glass;
    assert!(glass.motion.flex_duration_ms > 0);
    assert!(glass.motion.lift_duration_ms > 0);
    assert!(glass.motion.shape_shift_duration_ms > 0);
}

#[test]
fn accessibility_mode_default() {
    // Static constructors return DEFAULT accessibility mode.
    // System-detected mode is set by the install_* methods and _with_a11y
    // constructors, not by the plain dark()/light()/liquid_glass() paths.
    assert_eq!(
        TahoeTheme::dark().accessibility_mode,
        AccessibilityMode::DEFAULT
    );
    assert_eq!(
        TahoeTheme::light().accessibility_mode,
        AccessibilityMode::DEFAULT
    );
    assert_eq!(
        TahoeTheme::liquid_glass().accessibility_mode,
        AccessibilityMode::DEFAULT
    );
}

// ─── Liquid Glass Light Variant Tests ─────────────────────────────────────

#[test]
fn liquid_glass_light_has_glass_tokens() {
    // Glass is always present — verify the canonical fills have valid alpha.
    let glass = TahoeTheme::liquid_glass_light().glass;
    assert!(glass.regular_fill.a > 0.0);
    assert!(glass.clear_fill.a > 0.0);
}

#[test]
fn liquid_glass_light_background_is_light() {
    let theme = TahoeTheme::liquid_glass_light();
    assert!(theme.background.l > 0.5);
}

#[cfg(target_os = "macos")]
#[test]
fn liquid_glass_light_window_is_blurred() {
    let glass = TahoeTheme::liquid_glass_light().glass;
    assert!(matches!(
        glass.window_background,
        gpui::WindowBackgroundAppearance::Blurred
    ));
}

#[test]
fn liquid_glass_light_root_bg_semi_transparent() {
    let glass = TahoeTheme::liquid_glass_light().glass;
    assert!(glass.root_bg.a > 0.0 && glass.root_bg.a < 1.0);
}

#[test]
fn liquid_glass_light_regular_fill_semi_transparent() {
    // Regression for #62: light-mode Regular fill must have fractional
    // alpha so the window-level NSVisualEffectView blur shows through.
    let glass = TahoeTheme::liquid_glass_light().glass;
    assert!(
        glass.regular_fill.a > 0.0 && glass.regular_fill.a < 1.0,
        "light regular_fill should be semi-transparent, got {}",
        glass.regular_fill.a
    );
}

#[test]
fn liquid_glass_light_fills_are_white_tinted() {
    let glass = TahoeTheme::liquid_glass_light().glass;
    // Light glass uses semi-transparent white, not black.
    assert!(glass.regular_fill.l > 0.5);
    assert!(glass.clear_fill.l > 0.5);
}

#[test]
fn liquid_glass_light_all_tints_semi_transparent() {
    let glass = TahoeTheme::liquid_glass_light().glass;
    for color in [
        GlassTintColor::Green,
        GlassTintColor::Blue,
        GlassTintColor::Purple,
        GlassTintColor::Amber,
        GlassTintColor::Red,
        GlassTintColor::Cyan,
        GlassTintColor::Teal,
        GlassTintColor::Indigo,
    ] {
        let tint = glass.tints.get(color);
        assert!(tint.bg.a > 0.0 && tint.bg.a < 0.2);
        assert!(tint.bg_hover.a > tint.bg.a);
    }
}

#[test]
fn liquid_glass_light_shadow_stacks_match_dark() {
    // Corner radii are per-surface under the new API (Shape on the call
    // site), so the equivalent parity check is on the shared shadow
    // stacks — both light and dark themes must agree on the three
    // Elevation tiers' shadow parameters.
    let dark_glass = TahoeTheme::liquid_glass().glass;
    let light_glass = TahoeTheme::liquid_glass_light().glass;
    assert_eq!(
        dark_glass.resting_shadows.len(),
        light_glass.resting_shadows.len()
    );
    assert_eq!(
        dark_glass.elevated_shadows.len(),
        light_glass.elevated_shadows.len()
    );
    assert_eq!(
        dark_glass.floating_shadows.len(),
        light_glass.floating_shadows.len()
    );
}

#[test]
fn liquid_glass_light_labels_dim_primary_dark() {
    let glass = TahoeTheme::liquid_glass_light().glass;
    // Light glass dim labels should be dark text
    assert!(glass.labels_dim.primary.l < 0.3);
}

#[test]
fn liquid_glass_light_motion_tokens() {
    let glass = TahoeTheme::liquid_glass_light().glass;
    assert!(glass.motion.flex_duration_ms > 0);
    assert!(glass.accessibility.reduced_transparency_bg.a > 0.5);
}

// ─── Accessibility Mode Tests ─────────────────────────────────────────────

#[test]
fn accessibility_mode_reduce_transparency() {
    let mut theme = TahoeTheme::liquid_glass();
    theme.accessibility_mode = AccessibilityMode::REDUCE_TRANSPARENCY;
    assert_eq!(
        theme.accessibility_mode,
        AccessibilityMode::REDUCE_TRANSPARENCY
    );
}

#[test]
fn accessibility_mode_increase_contrast() {
    let mut theme = TahoeTheme::liquid_glass();
    theme.accessibility_mode = AccessibilityMode::INCREASE_CONTRAST;
    assert_eq!(
        theme.accessibility_mode,
        AccessibilityMode::INCREASE_CONTRAST
    );
}

#[test]
fn accessibility_mode_reduce_motion() {
    let mut theme = TahoeTheme::liquid_glass();
    theme.accessibility_mode = AccessibilityMode::REDUCE_MOTION;
    assert_eq!(theme.accessibility_mode, AccessibilityMode::REDUCE_MOTION);
}

#[test]
fn reduced_transparency_bg_more_opaque_than_regular() {
    // The reduced-transparency fallback should be more opaque than the
    // canonical Regular fill so the accessibility path actually lands
    // on a solid surface.
    let glass = TahoeTheme::liquid_glass().glass;
    assert!(
        glass.accessibility.reduced_transparency_bg.a > glass.regular_fill.a,
        "reduced_transparency_bg alpha ({}) should exceed regular_fill alpha ({})",
        glass.accessibility.reduced_transparency_bg.a,
        glass.regular_fill.a
    );
}

#[test]
fn high_contrast_border_is_visible() {
    let glass = TahoeTheme::liquid_glass().glass;
    assert!(glass.accessibility.high_contrast_border.a > 0.3);
}

#[test]
fn light_variant_accessibility_tokens() {
    let glass = TahoeTheme::liquid_glass_light().glass;
    assert!(glass.accessibility.reduced_transparency_bg.a > 0.5);
    assert!(glass.accessibility.high_contrast_border.a > 0.3);
    assert!(glass.accessibility.reduced_motion_scale >= 0.0);
}

#[test]
fn glass_text_has_reduced_alpha() {
    let dark = TahoeTheme::dark();
    let light = TahoeTheme::light();
    assert!(
        (0.7..1.0).contains(&dark.glass.icon_text.a),
        "dark icon_text should have reduced alpha, got {}",
        dark.glass.icon_text.a,
    );
    assert!(
        (0.7..1.0).contains(&light.glass.icon_text.a),
        "light icon_text should have reduced alpha, got {}",
        light.glass.icon_text.a,
    );
}

// ─── HIG Alignment: TextStyle Tests ──────────────────────────────────────

#[test]
fn text_style_body_is_13pt() {
    let attrs = TextStyle::Body.attrs();
    assert!((f32::from(attrs.size) - 13.0).abs() < f32::EPSILON);
    assert_eq!(attrs.weight, FontWeight::NORMAL);
}

#[test]
fn text_style_headline_is_bold() {
    let attrs = TextStyle::Headline.attrs();
    assert!((f32::from(attrs.size) - 13.0).abs() < f32::EPSILON);
    assert_eq!(attrs.weight, FontWeight::BOLD);
}

#[test]
fn text_style_large_title_is_26pt_bold() {
    // macOS Tahoe aligned LargeTitle with iOS: the default weight is
    // Bold (not Regular). Size stays at 26pt.
    let attrs = TextStyle::LargeTitle.attrs();
    assert!((f32::from(attrs.size) - 26.0).abs() < f32::EPSILON);
    assert_eq!(attrs.weight, FontWeight::BOLD);
}

#[test]
fn text_style_callout_is_12pt() {
    let attrs = TextStyle::Callout.attrs();
    assert!((f32::from(attrs.size) - 12.0).abs() < f32::EPSILON);
}

#[test]
fn text_style_sizes_non_decreasing() {
    let styles = [
        TextStyle::Caption2,
        TextStyle::Caption1,
        TextStyle::Footnote,
        TextStyle::Subheadline,
        TextStyle::Callout,
        TextStyle::Body,
        TextStyle::Headline,
        TextStyle::Title3,
        TextStyle::Title2,
        TextStyle::Title1,
        TextStyle::LargeTitle,
    ];
    for pair in styles.windows(2) {
        assert!(
            f32::from(pair[0].attrs().size) <= f32::from(pair[1].attrs().size),
            "{:?} ({}) should be <= {:?} ({})",
            pair[0],
            f32::from(pair[0].attrs().size),
            pair[1],
            f32::from(pair[1].attrs().size),
        );
    }
}

#[test]
fn high_contrast_themes_build() {
    let hc_dark = TahoeTheme::dark_high_contrast();
    let hc_light = TahoeTheme::light_high_contrast();
    assert!(hc_dark.background.l < 0.2);
    assert!(hc_light.background.l > 0.8);
}

#[test]
fn appearance_stored_on_theme() {
    assert_eq!(TahoeTheme::dark().appearance, Appearance::Dark);
    assert_eq!(TahoeTheme::light().appearance, Appearance::Light);
    assert_eq!(
        TahoeTheme::dark_high_contrast().appearance,
        Appearance::DarkHighContrast,
    );
    assert_eq!(
        TahoeTheme::light_high_contrast().appearance,
        Appearance::LightHighContrast,
    );
}

#[test]
fn palette_available_on_theme() {
    let theme = TahoeTheme::dark();
    assert_eq!(theme.palette.red.a, 1.0);
    assert_eq!(theme.palette.blue.a, 1.0);
    assert_eq!(theme.palette.gray6.a, 1.0);
}

#[test]
fn control_thumb_varies_by_appearance() {
    let light = TahoeTheme::light().control_thumb;
    let dark = TahoeTheme::dark().control_thumb;
    let light_hc = TahoeTheme::light_high_contrast().control_thumb;
    let dark_hc = TahoeTheme::dark_high_contrast().control_thumb;
    let glass_dark = TahoeTheme::liquid_glass().control_thumb;
    let glass_light = TahoeTheme::liquid_glass_light().control_thumb;

    // Light thumb must be dark enough to read on near-white track.
    assert!(light.l < 0.6, "light thumb too pale: {}", light.l);
    // Dark thumb preserves current white-puck behaviour.
    assert!(dark.l > 0.95);
    // HC pushes farther from the track than the default light value.
    assert!(light_hc.l < light.l);
    // Dark HC keeps max contrast against the dark track.
    assert!(dark_hc.l > 0.95);
    // Liquid-glass variants inherit from their parent constructors —
    // pin the contract so a future override in liquid_glass*() that
    // accidentally reintroduces a white puck in light mode is caught.
    assert!(glass_dark.l > 0.95, "glass dark too dim: {}", glass_dark.l);
    assert!(
        glass_light.l < 0.6,
        "glass light too pale: {}",
        glass_light.l
    );
    assert_ne!(light, dark);
}

#[test]
fn new_builds_all_four_appearances() {
    for appearance in [
        Appearance::Light,
        Appearance::Dark,
        Appearance::LightHighContrast,
        Appearance::DarkHighContrast,
    ] {
        let theme = TahoeTheme::new(appearance);
        assert_eq!(theme.appearance, appearance);
        assert_eq!(theme.error.a, 1.0);
        assert_eq!(theme.success.a, 1.0);
    }
}

#[test]
fn overlay_bg_is_semi_transparent() {
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        assert!(
            (0.0..1.0).contains(&theme.overlay_bg.a),
            "overlay_bg should be semi-transparent",
        );
    }
}

#[test]
fn text_style_leading_is_reasonable() {
    for style in [
        TextStyle::Body,
        TextStyle::Headline,
        TextStyle::LargeTitle,
        TextStyle::Caption1,
        TextStyle::Footnote,
    ] {
        let attrs = style.attrs();
        let size = f32::from(attrs.size);
        let leading = f32::from(attrs.leading);
        assert!(
            leading > size && leading < size * 2.0,
            "{:?} leading ({leading}) should be between size ({size}) and 2*size",
            style
        );
    }
}

#[test]
fn text_style_bold_bumps_weight() {
    let normal = TextStyle::Body.attrs();
    let bold = TextStyle::Body.attrs_bold();
    assert_eq!(normal.weight, FontWeight::NORMAL);
    assert_eq!(bold.weight, FontWeight::MEDIUM);
}

#[test]
fn text_style_headline_bold_bumps_bold_to_extra_bold() {
    let bold = TextStyle::Headline.attrs_bold();
    assert_eq!(bold.weight, FontWeight::EXTRA_BOLD);
}

// ─── HIG Alignment: Bold Step Tests ──────────────────────────────────────

#[test]
fn bold_step_normal_to_medium() {
    assert_eq!(bold_step(FontWeight::NORMAL), FontWeight::MEDIUM);
}

#[test]
fn bold_step_medium_to_semibold() {
    assert_eq!(bold_step(FontWeight::MEDIUM), FontWeight::SEMIBOLD);
}

#[test]
fn bold_step_black_stays_black() {
    assert_eq!(bold_step(FontWeight::BLACK), FontWeight::BLACK);
}

#[test]
fn bold_step_nan_returns_nan() {
    let result = bold_step(FontWeight(f32::NAN));
    assert!(result.0.is_nan(), "NaN input should pass through unchanged");
}

#[test]
fn bold_step_positive_infinity_returns_infinity() {
    let result = bold_step(FontWeight(f32::INFINITY));
    assert_eq!(result.0, f32::INFINITY);
}

#[test]
fn bold_step_negative_infinity_returns_negative_infinity() {
    let result = bold_step(FontWeight(f32::NEG_INFINITY));
    assert_eq!(result.0, f32::NEG_INFINITY);
}

#[test]
fn bold_step_negative_finite_clamps_to_extra_light() {
    assert_eq!(bold_step(FontWeight(-100.0)), FontWeight::EXTRA_LIGHT);
}

#[test]
fn bold_step_huge_positive_saturates_to_black() {
    assert_eq!(bold_step(FontWeight(10_000.0)), FontWeight::BLACK);
}

// ─── HIG Alignment: Contrast Ratio Tests ────────────────────────────────

#[test]
fn contrast_white_on_black() {
    let ratio = contrast_ratio(
        hsla(0.0, 0.0, 1.0, 1.0), // white
        hsla(0.0, 0.0, 0.0, 1.0), // black
    );
    assert!(
        ratio > 20.0,
        "White on black should be ~21:1, got {}",
        ratio
    );
}

#[test]
fn contrast_same_color_is_one() {
    let color = hsla(0.58, 0.80, 0.50, 1.0);
    let ratio = contrast_ratio(color, color);
    assert!(
        (ratio - 1.0).abs() < 0.01,
        "Same color contrast should be 1:1, got {}",
        ratio
    );
}

#[test]
fn meets_contrast_aa_text() {
    assert!(meets_contrast(
        hsla(0.0, 0.0, 1.0, 1.0), // white
        hsla(0.0, 0.0, 0.0, 1.0), // black
        4.5,
    ));
}

#[test]
fn dark_theme_text_on_bg_has_sufficient_contrast() {
    let theme = TahoeTheme::dark();
    let ratio = contrast_ratio(theme.text, theme.background);
    assert!(
        ratio >= 4.5,
        "Dark theme text/bg contrast should be >= 4.5:1, got {:.1}:1",
        ratio
    );
}

#[test]
fn light_theme_text_on_bg_has_sufficient_contrast() {
    let theme = TahoeTheme::light();
    let ratio = contrast_ratio(theme.text, theme.background);
    assert!(
        ratio >= 4.5,
        "Light theme text/bg contrast should be >= 4.5:1, got {:.1}:1",
        ratio
    );
}

// ─── HIG Alignment: Glass::Identity Tests ───────────────────────────────

#[test]
fn glass_identity_returns_transparent() {
    let glass = TahoeTheme::liquid_glass().glass;
    let bg = glass.fill(Glass::Identity);
    assert!(
        (bg.a - 0.0).abs() < f32::EPSILON,
        "Identity variant should be fully transparent"
    );
}

// ─── HIG Alignment: Extended Semantic Colors ─────────────────────────────

#[test]
fn semantic_separator_is_semi_transparent() {
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        assert!(
            theme.semantic.separator.a < 1.0 && theme.semantic.separator.a > 0.0,
            "separator should be semi-transparent"
        );
    }
}

#[test]
fn code_bg_lighter_than_background_in_dark() {
    let theme = TahoeTheme::dark();
    assert!(
        theme.code_bg.l > theme.background.l,
        "dark code_bg (L={}) should be lighter than background (L={})",
        theme.code_bg.l,
        theme.background.l
    );
}

#[test]
fn code_bg_darker_than_background_in_light() {
    let theme = TahoeTheme::light();
    assert!(
        theme.code_bg.l < theme.background.l,
        "light code_bg (L={}) should be darker than background (L={})",
        theme.code_bg.l,
        theme.background.l
    );
}

#[test]
fn high_contrast_text_stronger_than_standard() {
    // HC variants use fully opaque, lightness-based values with stronger
    // contrast than the standard semantic defaults (which use alpha for hierarchy).
    let dark = TahoeTheme::dark();
    let dark_hc = TahoeTheme::dark_high_contrast();
    // Dark: both standard and HC text are at L=1.0 (pure white); HC uses
    // fully opaque a=1.0 which is at least as strong.
    assert!(
        dark_hc.text.l >= dark.text.l,
        "dark HC text should be at least as bright"
    );
    assert_eq!(dark_hc.text.a, 1.0, "dark HC text should be fully opaque");
    // HC text_muted is opaque (a=1.0) vs standard which uses alpha (a=0.70)
    assert_eq!(
        dark_hc.text_muted.a, 1.0,
        "dark HC text_muted should be fully opaque"
    );

    let light = TahoeTheme::light();
    let light_hc = TahoeTheme::light_high_contrast();
    assert!(
        light_hc.text.l <= light.text.l,
        "light HC text should be at least as dark"
    );
    assert_eq!(light_hc.text.a, 1.0, "light HC text should be fully opaque");
    assert_eq!(
        light_hc.text_muted.a, 1.0,
        "light HC text_muted should be fully opaque"
    );
}

#[test]
fn tool_bg_is_semi_transparent() {
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        assert!(
            (0.0..1.0).contains(&theme.tool_approved_bg.a),
            "tool_approved_bg should be semi-transparent"
        );
        assert!(
            (0.0..1.0).contains(&theme.tool_rejected_bg.a),
            "tool_rejected_bg should be semi-transparent"
        );
    }
}

#[test]
fn semantic_opaque_separator_is_opaque() {
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        assert!(
            (theme.semantic.opaque_separator.a - 1.0).abs() < f32::EPSILON,
            "opaque_separator should be fully opaque"
        );
    }
}

#[test]
fn semantic_fill_hierarchy() {
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        assert!(theme.semantic.system_fill.a >= theme.semantic.secondary_system_fill.a);
        assert!(theme.semantic.secondary_system_fill.a >= theme.semantic.tertiary_system_fill.a);
        assert!(theme.semantic.tertiary_system_fill.a >= theme.semantic.quaternary_system_fill.a);
    }
}

#[test]
fn semantic_placeholder_text_is_semi_transparent() {
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        assert!(theme.semantic.placeholder_text.a < 0.5 && theme.semantic.placeholder_text.a > 0.0);
    }
}

#[test]
fn semantic_link_has_full_alpha() {
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        assert_eq!(
            theme.semantic.link.a, 1.0,
            "link color should be fully opaque"
        );
    }
}

#[test]
fn semantic_grouped_backgrounds_present() {
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        assert!(theme.semantic.system_grouped_background.a > 0.0);
        assert!(theme.semantic.secondary_system_grouped_background.a > 0.0);
        assert!(theme.semantic.tertiary_system_grouped_background.a > 0.0);
    }
}

// ─── HIG Alignment: BoldText Accessibility Mode ──────────────────────────

#[test]
fn accessibility_mode_bold_text() {
    let mut theme = TahoeTheme::liquid_glass();
    theme.accessibility_mode = AccessibilityMode::BOLD_TEXT;
    assert_eq!(theme.accessibility_mode, AccessibilityMode::BOLD_TEXT);
}

// ─── HIG Phase 3: effective_weight Tests ─────────────────────────────

#[test]
fn effective_weight_default_mode_passes_through() {
    let theme = TahoeTheme::dark();
    assert_eq!(
        theme.effective_weight(FontWeight::NORMAL),
        FontWeight::NORMAL
    );
    assert_eq!(
        theme.effective_weight(FontWeight::SEMIBOLD),
        FontWeight::SEMIBOLD
    );
}

#[test]
fn effective_weight_bold_text_bumps() {
    let mut theme = TahoeTheme::dark();
    theme.accessibility_mode = AccessibilityMode::BOLD_TEXT;
    assert_eq!(
        theme.effective_weight(FontWeight::NORMAL),
        FontWeight::MEDIUM
    );
    assert_eq!(
        theme.effective_weight(FontWeight::SEMIBOLD),
        FontWeight::BOLD
    );
}

#[test]
fn effective_weight_other_modes_pass_through() {
    let mut theme = TahoeTheme::dark();
    theme.accessibility_mode = AccessibilityMode::REDUCE_MOTION;
    assert_eq!(
        theme.effective_weight(FontWeight::NORMAL),
        FontWeight::NORMAL
    );
}

// ─── HIG: Full Keyboard Access helper ────────────────────────────────

#[test]
fn full_keyboard_access_default_mode_is_false() {
    let theme = TahoeTheme::dark();
    assert!(!theme.full_keyboard_access());
}

#[test]
fn full_keyboard_access_reports_when_flag_set() {
    let mut theme = TahoeTheme::dark();
    theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
    assert!(theme.full_keyboard_access());
}

#[test]
fn full_keyboard_access_reports_when_combined_with_other_flags() {
    let mut theme = TahoeTheme::dark();
    theme.accessibility_mode =
        AccessibilityMode::FULL_KEYBOARD_ACCESS | AccessibilityMode::BOLD_TEXT;
    assert!(theme.full_keyboard_access());
    assert_eq!(
        theme.effective_weight(FontWeight::NORMAL),
        FontWeight::MEDIUM
    );
}

#[test]
fn full_keyboard_access_other_flags_do_not_enable() {
    let mut theme = TahoeTheme::dark();
    theme.accessibility_mode = AccessibilityMode::REDUCE_MOTION;
    assert!(!theme.full_keyboard_access());
}

// ─── HIG Phase 3: Spring Token Tests ─────────────────────────────────

#[test]
fn spring_tokens_present() {
    let glass = TahoeTheme::liquid_glass().glass;
    assert!(glass.motion.spring_damping > 0.0);
    assert!(glass.motion.spring_response > 0.0);
    assert!(glass.motion.spring_bounce >= 0.0);
}

#[test]
fn spring_duration_ms_is_reasonable() {
    let glass = TahoeTheme::liquid_glass().glass;
    let dur = glass.motion.spring_duration_ms();
    // Duration must be within the HIG 250-500ms system animation range,
    // capped at MotionRamp::Long (450ms).
    assert!(
        (250..=500).contains(&dur),
        "spring_duration_ms should be within HIG range (250-500ms), got {}",
        dur
    );
}

#[test]
fn spring_tokens_match_across_variants() {
    let dark = TahoeTheme::liquid_glass().glass;
    let light = TahoeTheme::liquid_glass_light().glass;
    assert!((dark.motion.spring_damping - light.motion.spring_damping).abs() < f32::EPSILON);
    assert!((dark.motion.spring_response - light.motion.spring_response).abs() < f32::EPSILON);
}

#[test]
fn palette_colors_all_opaque() {
    for theme in [TahoeTheme::dark(), TahoeTheme::light()] {
        let p = &theme.palette;
        for color in [
            p.red, p.orange, p.yellow, p.green, p.mint, p.teal, p.cyan, p.blue, p.indigo, p.purple,
            p.pink, p.brown, p.gray, p.gray2, p.gray3, p.gray4, p.gray5, p.gray6,
        ] {
            assert_eq!(color.a, 1.0, "all palette colors should be fully opaque");
        }
    }
}

#[test]
fn material_bg_thickness_ordering() {
    let glass = TahoeTheme::liquid_glass().glass;
    // The four dedicated thickness fills must be strictly ordered by alpha.
    // Per Figma: Dark uses #000000 @10%/@20%/@40%/@50%
    let ut = glass.ultra_thin_bg.a;
    let t = glass.thin_bg.a;
    let tk = glass.thick_bg.a;
    let utk = glass.ultra_thick_bg.a;
    assert!(
        ut < t,
        "UltraThin ({ut}) should be less opaque than Thin ({t})"
    );
    assert!(t < tk, "Thin ({t}) should be less opaque than Thick ({tk})");
    assert!(
        tk < utk,
        "Thick ({tk}) should be less opaque than UltraThick ({utk})"
    );
}

#[test]
fn text_style_tracking_in_range() {
    for style in [
        TextStyle::LargeTitle,
        TextStyle::Title1,
        TextStyle::Title2,
        TextStyle::Title3,
        TextStyle::Headline,
        TextStyle::Body,
        TextStyle::Callout,
        TextStyle::Subheadline,
        TextStyle::Footnote,
        TextStyle::Caption1,
        TextStyle::Caption2,
    ] {
        let t = style.attrs().tracking;
        assert!(
            t > -1.0 && t < 1.0,
            "{:?} tracking {t} should be between -1.0 and 1.0",
            style
        );
    }
}

#[test]
fn text_on_accent_is_opaque() {
    for appearance in [
        Appearance::Light,
        Appearance::Dark,
        Appearance::LightHighContrast,
        Appearance::DarkHighContrast,
    ] {
        let theme = TahoeTheme::new(appearance);
        assert_eq!(
            theme.text_on_accent.a, 1.0,
            "text_on_accent should be fully opaque for {:?}",
            appearance
        );
    }
}

#[test]
fn text_style_headline_tracking() {
    assert!((TextStyle::Headline.attrs().tracking - (-0.08)).abs() < f32::EPSILON);
}

#[test]
fn text_style_emphasized_weights() {
    assert_eq!(TextStyle::LargeTitle.emphasized().weight, FontWeight::BOLD);
    assert_eq!(TextStyle::Title1.emphasized().weight, FontWeight::BOLD);
    assert_eq!(TextStyle::Title2.emphasized().weight, FontWeight::BOLD);
    assert_eq!(TextStyle::Title3.emphasized().weight, FontWeight::SEMIBOLD);
    // Per HIG macOS table: Headline's emphasized weight is Heavy (≈ BLACK).
    assert_eq!(TextStyle::Headline.emphasized().weight, FontWeight::BLACK);
    assert_eq!(TextStyle::Body.emphasized().weight, FontWeight::SEMIBOLD);
    assert_eq!(TextStyle::Callout.emphasized().weight, FontWeight::SEMIBOLD);
    assert_eq!(
        TextStyle::Subheadline.emphasized().weight,
        FontWeight::SEMIBOLD
    );
    assert_eq!(
        TextStyle::Footnote.emphasized().weight,
        FontWeight::SEMIBOLD
    );
    assert_eq!(TextStyle::Caption1.emphasized().weight, FontWeight::MEDIUM);
    assert_eq!(
        TextStyle::Caption2.emphasized().weight,
        FontWeight::SEMIBOLD
    );
}

#[test]
fn text_style_emphasized_preserves_size() {
    let normal = TextStyle::Body.attrs();
    let emph = TextStyle::Body.emphasized();
    assert_eq!(normal.size, emph.size);
    assert_eq!(normal.leading, emph.leading);
}

/// Leading tolerance: allow ~100 ULPs of f32 multiplication error
/// (≈1.19e-5). `0.95` / `1.15` are not exactly representable in f32, so the
/// product drifts by a few ULPs from the arithmetic ideal — anything wider
/// than this would hide a real bug in the multiplier.
const LEADING_TOLERANCE: f32 = f32::EPSILON * 100.0;

#[test]
fn leading_style_tight_reduces_on_display_sizes() {
    // Display-size styles (LargeTitle: 32 pt leading, 26 pt size) have
    // enough headroom above the 1.15× SF Pro floor that the 0.95
    // multiplier wins outright — Tight genuinely reduces. Body-scale
    // styles are now clamped at the 1.5× WCAG floor and therefore
    // *increase* under Tight; that contract is covered by
    // `leading_style_tight_clamps_body_to_wcag_floor`.
    let standard = TextStyle::LargeTitle.attrs();
    let tight = standard.with_leading(LeadingStyle::Tight);
    assert!(f32::from(tight.leading) < f32::from(standard.leading));
    let expected = f32::from(standard.leading) * 0.95;
    assert!((f32::from(tight.leading) - expected).abs() < LEADING_TOLERANCE);
}

#[test]
fn leading_style_tight_clamps_body_to_wcag_floor() {
    // Body-scale styles (size ≤ 15 pt) clamp Tight to `size × 1.5` so
    // running paragraphs meet WCAG 1.4.12 (Text Spacing). Body is
    // 13 pt × 1.5 = 19.5 pt, which is *above* the 16 pt standard
    // leading — Tight on body copy actually opens the line box to
    // the accessibility floor rather than compressing below it.
    let standard = TextStyle::Body.attrs();
    let tight = standard.with_leading(LeadingStyle::Tight);
    let size = f32::from(standard.size);
    let expected = size * 1.5;
    assert!((f32::from(tight.leading) - expected).abs() < LEADING_TOLERANCE);
    assert!(f32::from(tight.leading) >= f32::from(standard.leading));
}

#[test]
fn leading_style_tight_stays_above_practical_floor() {
    // Regression guard: a more aggressive Tight multiplier (e.g. the
    // original 0.85) drops Body's 16 pt leading to 13.6 pt against a
    // 13 pt body size — a 1.046× ratio where SF Pro's ascenders and
    // descenders start colliding. Tight must never land below 1.15×
    // the style's size for any HIG text style. Sweeping every
    // variant guards the clamp in `with_leading` against a future
    // style (or multiplier adjustment) that would push a smaller
    // style below the floor.
    for style in [
        TextStyle::LargeTitle,
        TextStyle::Title1,
        TextStyle::Title2,
        TextStyle::Title3,
        TextStyle::Headline,
        TextStyle::Body,
        TextStyle::Callout,
        TextStyle::Subheadline,
        TextStyle::Footnote,
        TextStyle::Caption1,
        TextStyle::Caption2,
    ] {
        let attrs = style.attrs();
        let tight = attrs.with_leading(LeadingStyle::Tight);
        let size = f32::from(attrs.size);
        let ratio = f32::from(tight.leading) / size;
        assert!(
            ratio >= 1.15 - LEADING_TOLERANCE,
            "{style:?}: Tight leading {} / size {size} = {ratio} fell below 1.15× — \
             too tight for SF Pro ascenders/descenders",
            f32::from(tight.leading),
        );
    }
}

#[test]
fn leading_style_loose_increases() {
    let standard = TextStyle::Body.attrs();
    let loose = standard.with_leading(LeadingStyle::Loose);
    assert!(f32::from(loose.leading) > f32::from(standard.leading));
    let expected = f32::from(standard.leading) * 1.15;
    assert!((f32::from(loose.leading) - expected).abs() < LEADING_TOLERANCE);
}

#[test]
fn leading_style_standard_unchanged() {
    let attrs = TextStyle::Body.attrs();
    let standard = attrs.with_leading(LeadingStyle::Standard);
    assert_eq!(attrs.leading, standard.leading);
}

#[test]
fn leading_style_proportional_across_all_text_styles() {
    // Regression coverage: a flat ±pt offset landed differently per style
    // (12.5% on Body's 16pt, 6.25% on LargeTitle's 32pt). The proportional
    // multiplier keeps the relative delta identical across every style;
    // this sweep catches any future regression that breaks that contract.
    //
    // Tight is `max(leading × 0.95, size × floor_ratio)` where
    // `floor_ratio` is 1.5 for body-scale styles (size ≤ 15 pt, per
    // WCAG 1.4.12) and 1.15 for display styles (SF Pro
    // ascender/descender floor).
    // Loose is a pure multiplier with no floor, so the assertion there
    // stays exact.
    for style in [
        TextStyle::LargeTitle,
        TextStyle::Title1,
        TextStyle::Title2,
        TextStyle::Title3,
        TextStyle::Headline,
        TextStyle::Body,
        TextStyle::Callout,
        TextStyle::Subheadline,
        TextStyle::Footnote,
        TextStyle::Caption1,
        TextStyle::Caption2,
    ] {
        let attrs = style.attrs();
        let base = f32::from(attrs.leading);
        let size = f32::from(attrs.size);
        let tight = f32::from(attrs.with_leading(LeadingStyle::Tight).leading);
        let loose = f32::from(attrs.with_leading(LeadingStyle::Loose).leading);
        let floor_ratio = if size <= 15.0 { 1.5 } else { 1.15 };
        let expected_tight = (base * 0.95).max(size * floor_ratio);
        assert!(
            (tight - expected_tight).abs() < LEADING_TOLERANCE,
            "{style:?}: tight leading {tight} is not max(0.95 × {base}, \
             {floor_ratio} × {size}) = {expected_tight}",
        );
        assert!(
            (loose - base * 1.15).abs() < LEADING_TOLERANCE,
            "{style:?}: loose leading {loose} is not 1.15 × {base}",
        );
    }
}

#[test]
fn label_level_resolve_all_variants() {
    // Resolution contract must hold for every built-in theme — the
    // dark / light / liquid-glass tokens share the same semantic
    // hierarchy, so [`LabelLevel::resolve`] must map each tier to the
    // matching theme accessor regardless of which palette is active.
    for theme in [
        TahoeTheme::dark(),
        TahoeTheme::light(),
        TahoeTheme::liquid_glass(),
    ] {
        assert_eq!(LabelLevel::Primary.resolve(&theme), theme.text);
        assert_eq!(LabelLevel::Secondary.resolve(&theme), theme.text_muted);
        assert_eq!(LabelLevel::Tertiary.resolve(&theme), theme.text_tertiary());
        assert_eq!(
            LabelLevel::Quaternary.resolve(&theme),
            theme.text_quaternary(),
        );
        assert_eq!(LabelLevel::Quinary.resolve(&theme), theme.text_quinary());
    }
}

#[test]
fn label_level_resolve_variants_are_distinct() {
    let theme = TahoeTheme::dark();
    let colors = [
        LabelLevel::Primary.resolve(&theme),
        LabelLevel::Secondary.resolve(&theme),
        LabelLevel::Tertiary.resolve(&theme),
        LabelLevel::Quaternary.resolve(&theme),
        LabelLevel::Quinary.resolve(&theme),
    ];
    // Each HIG tier resolves to a different color; if two collapse to
    // the same value, the hierarchy has broken.
    for (i, a) in colors.iter().enumerate() {
        for (j, b) in colors.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "variants {i} and {j} collapsed");
            }
        }
    }
}

#[test]
fn macos_tracking_body_size() {
    assert!((macos_tracking(13.0) - (-0.08)).abs() < f32::EPSILON);
}

#[test]
fn macos_tracking_large_sizes_approach_zero() {
    assert_eq!(macos_tracking(96.0), 0.0);
    assert_eq!(macos_tracking(100.0), 0.0);
}

#[test]
fn macos_tracking_extreme_sizes_do_not_overflow() {
    // Regression for #46: `u32::MAX as f32` rounds up to 2^32, so the previous
    // clamp allowed the `as u32` cast to overflow. Any finite input must
    // short-circuit safely to the `_ => 0.0` arm without panicking.
    assert_eq!(macos_tracking(f32::MAX), 0.0);
    assert_eq!(macos_tracking(u32::MAX as f32), 0.0);
    assert_eq!(macos_tracking(1.0e30), 0.0);
    assert_eq!(macos_tracking(f32::INFINITY), 0.0);
    assert_eq!(macos_tracking(f32::NAN), 0.0);
    assert_eq!(macos_tracking(-1.0e30), 0.0);
}

#[test]
fn font_design_families() {
    assert_eq!(FontDesign::Default.font_family(), ".AppleSystemUIFont");
    assert_eq!(FontDesign::Serif.font_family(), "New York");
    assert_eq!(
        FontDesign::Rounded.font_family(),
        ".AppleSystemUIFontRounded"
    );
    assert_eq!(FontDesign::Monospaced.font_family(), "SF Mono");
}

#[test]
fn text_style_with_design_sets_font_family() {
    let theme = TahoeTheme::dark();
    for (design, expected) in [
        (FontDesign::Default, ".AppleSystemUIFont"),
        (FontDesign::Serif, "New York"),
        (FontDesign::Rounded, ".AppleSystemUIFontRounded"),
        (FontDesign::Monospaced, "SF Mono"),
    ] {
        let mut el = div().text_style_with_design(TextStyle::Body, design, &theme);
        assert_eq!(
            Styled::text_style(&mut el).font_family,
            Some(SharedString::from(expected)),
            "design={design:?}"
        );

        let mut el_em =
            div().text_style_emphasized_with_design(TextStyle::Headline, design, &theme);
        assert_eq!(
            Styled::text_style(&mut el_em).font_family,
            Some(SharedString::from(expected)),
            "emphasized design={design:?}"
        );
    }
}

#[test]
fn text_style_preserves_font_family_cascade() {
    // `text_style` / `text_style_emphasized` must leave `font_family` alone so
    // callers that set it (on the element or a parent) are not clobbered.
    let theme = TahoeTheme::dark();

    let mut plain = div().text_style(TextStyle::Body, &theme);
    assert!(
        Styled::text_style(&mut plain).font_family.is_none(),
        "text_style must not set font_family"
    );

    let mut emphasized = div().text_style_emphasized(TextStyle::Headline, &theme);
    assert!(
        Styled::text_style(&mut emphasized).font_family.is_none(),
        "text_style_emphasized must not set font_family"
    );

    // Caller-chained `.font_family(...)` before `text_style(...)` must survive
    // — the pattern `/code/*` and `/markdown/code_block/*` currently use.
    let mut chained = div()
        .font_family("SF Mono")
        .text_style(TextStyle::Body, &theme);
    assert_eq!(
        Styled::text_style(&mut chained).font_family,
        Some(SharedString::from("SF Mono"))
    );
}

// ── relative_luminance hue-sector coverage ─────────────────────────────

#[test]
fn luminance_pure_red() {
    // Pure red (h=0, s=1, l=0.5) → luminance ~0.2126
    let lum = crate::foundations::color::relative_luminance(gpui::hsla(0.0, 1.0, 0.5, 1.0));
    assert!((lum - 0.2126).abs() < 0.01, "pure red luminance = {lum}");
}

#[test]
fn luminance_pure_green() {
    // Pure green (h=120°/360=0.333, s=1, l=0.5) → luminance ~0.7152
    let lum = crate::foundations::color::relative_luminance(gpui::hsla(0.333, 1.0, 0.5, 1.0));
    assert!((lum - 0.7152).abs() < 0.02, "pure green luminance = {lum}");
}

#[test]
fn luminance_pure_blue() {
    // Pure blue (h=240°/360=0.667, s=1, l=0.5) → luminance ~0.0722
    let lum = crate::foundations::color::relative_luminance(gpui::hsla(0.667, 1.0, 0.5, 1.0));
    assert!((lum - 0.0722).abs() < 0.01, "pure blue luminance = {lum}");
}

#[test]
fn luminance_mid_gray() {
    // Mid-gray (h=0, s=0, l=0.5) → luminance ~0.2140 (sRGB linearized 0.5)
    let lum = crate::foundations::color::relative_luminance(gpui::hsla(0.0, 0.0, 0.5, 1.0));
    assert!((lum - 0.2140).abs() < 0.01, "mid-gray luminance = {lum}");
}

#[test]
fn high_contrast_differs_from_standard() {
    let dark = TahoeTheme::dark();
    let dark_hc = TahoeTheme::dark_high_contrast();
    assert_ne!(
        dark.error, dark_hc.error,
        "HC error should differ from standard"
    );
    assert_ne!(
        dark.success, dark_hc.success,
        "HC success should differ from standard"
    );

    let light = TahoeTheme::light();
    let light_hc = TahoeTheme::light_high_contrast();
    assert_ne!(
        light.error, light_hc.error,
        "HC error should differ from standard"
    );
}

// ─── Phase 1 Theme Consolidation Tests ──────────────────────────────────

#[test]
fn text_equals_semantic_label_dark() {
    let theme = TahoeTheme::dark();
    assert_eq!(theme.text, theme.semantic.label);
}

#[test]
fn text_equals_semantic_label_light() {
    let theme = TahoeTheme::light();
    assert_eq!(theme.text, theme.semantic.label);
}

#[test]
fn text_muted_equals_semantic_secondary_label() {
    let dark = TahoeTheme::dark();
    assert_eq!(dark.text_muted, dark.semantic.secondary_label);
    let light = TahoeTheme::light();
    assert_eq!(light.text_muted, light.semantic.secondary_label);
}

#[test]
fn convenience_methods_delegate_to_semantic() {
    let theme = TahoeTheme::dark();
    assert_eq!(theme.text_tertiary(), theme.semantic.tertiary_label);
    assert_eq!(theme.text_quaternary(), theme.semantic.quaternary_label);
    assert_eq!(theme.text_quinary(), theme.semantic.quinary_label);
    assert_eq!(theme.placeholder_text(), theme.semantic.placeholder_text);
    assert_eq!(theme.system_fill(), theme.semantic.system_fill);
}

#[test]
fn liquid_glass_text_synced_with_semantic() {
    let theme = TahoeTheme::liquid_glass();
    assert_eq!(theme.text, theme.semantic.label);
    assert_eq!(theme.text_muted, theme.semantic.secondary_label);
}

#[test]
fn liquid_glass_light_text_synced_with_semantic() {
    let theme = TahoeTheme::liquid_glass_light();
    assert_eq!(theme.text, theme.semantic.label);
    assert_eq!(theme.text_muted, theme.semantic.secondary_label);
}

#[test]
fn liquid_glass_info_synced_with_semantic() {
    let dark = TahoeTheme::liquid_glass();
    assert_eq!(dark.info, dark.semantic.info);
    let light = TahoeTheme::liquid_glass_light();
    assert_eq!(light.info, light.semantic.info);
}

// ─── Phase 4: Clear Glass Variant Tests ─────────────────────────────────

#[test]
fn liquid_glass_clear_uses_clear_variant() {
    let theme = TahoeTheme::liquid_glass_clear();
    assert_eq!(theme.glass.variant, Glass::Clear);
}

#[test]
fn liquid_glass_clear_light_uses_clear_variant() {
    let theme = TahoeTheme::liquid_glass_clear_light();
    assert_eq!(theme.glass.variant, Glass::Clear);
}

#[test]
fn clear_fill_more_transparent_than_regular() {
    let theme = TahoeTheme::liquid_glass();
    let regular = theme.glass.fill(Glass::Regular);
    let clear = theme.glass.fill(Glass::Clear);
    assert!(
        clear.a < regular.a,
        "Clear should have lower alpha than Regular"
    );
}

// ─── Phase 5: Context-Aware Label Resolution Tests ──────────────────────

#[test]
fn resolve_label_opaque_returns_semantic() {
    use crate::foundations::materials::{SurfaceContext, resolve_label};
    let theme = TahoeTheme::dark();
    assert_eq!(
        resolve_label(&theme, SurfaceContext::Opaque, 0),
        theme.semantic.label
    );
    assert_eq!(
        resolve_label(&theme, SurfaceContext::Opaque, 1),
        theme.semantic.secondary_label
    );
}

#[test]
fn resolve_label_glass_dim_returns_dim_labels() {
    use crate::foundations::materials::{SurfaceContext, resolve_label};
    let theme = TahoeTheme::dark();
    assert_eq!(
        resolve_label(&theme, SurfaceContext::GlassDim, 0),
        theme.glass.labels_dim.primary
    );
}

#[test]
fn resolve_label_glass_bright_returns_bright_labels() {
    use crate::foundations::materials::{SurfaceContext, resolve_label};
    let theme = TahoeTheme::dark();
    assert_eq!(
        resolve_label(&theme, SurfaceContext::GlassBright, 0),
        theme.glass.labels_bright.primary
    );
}

// ─── Phase 6: Accent-Tinted Glass Tests ─────────────────────────────────

#[test]
fn accent_glass_tint_uses_accent_hue() {
    let theme = TahoeTheme::dark();
    // Per Figma: primary glass uses accent at full opacity.
    // The #000000 @20% glass tint is applied in apply_glass_chrome().
    assert!((theme.glass.accent_tint.bg.h - theme.accent.h).abs() < 0.01);
    assert!((theme.glass.accent_tint.bg.a - 1.0).abs() < 0.01);
}

// ─── Phase 7: GlassContainer Tests ──────────────────────────────────────

#[test]
fn glass_container_defaults() {
    use crate::foundations::materials::GlassContainer;
    let gc = GlassContainer::new("test");
    assert!(gc.spacing.is_none());
    assert!(gc.children.is_empty());
}

// ─── Phase 9: Morph State & Spring Easing Tests ─────────────────────────

#[test]
fn morph_state_lerp_at_zero() {
    use crate::foundations::motion::MorphState;
    let from = MorphState::new(0.0, 0.0, 100.0, 100.0, 10.0);
    let to = MorphState::new(50.0, 50.0, 200.0, 200.0, 20.0);
    let result = MorphState::lerp(&from, &to, 0.0);
    assert_eq!(result.x, 0.0);
    assert_eq!(result.width, 100.0);
    assert_eq!(result.corner_radius, 10.0);
}

#[test]
fn morph_state_lerp_at_half() {
    use crate::foundations::motion::MorphState;
    let from = MorphState::new(0.0, 0.0, 100.0, 100.0, 10.0);
    let to = MorphState::new(50.0, 50.0, 200.0, 200.0, 20.0);
    let result = MorphState::lerp(&from, &to, 0.5);
    assert_eq!(result.x, 25.0);
    assert_eq!(result.width, 150.0);
    assert_eq!(result.corner_radius, 15.0);
}

#[test]
fn morph_state_lerp_at_one() {
    use crate::foundations::motion::MorphState;
    let from = MorphState::new(0.0, 0.0, 100.0, 100.0, 10.0);
    let to = MorphState::new(50.0, 50.0, 200.0, 200.0, 20.0);
    let result = MorphState::lerp(&from, &to, 1.0);
    assert_eq!(result.x, 50.0);
    assert_eq!(result.width, 200.0);
    assert_eq!(result.corner_radius, 20.0);
}

#[test]
fn morph_state_lerp_clamped() {
    use crate::foundations::motion::MorphState;
    let from = MorphState::new(0.0, 0.0, 100.0, 100.0, 10.0);
    let to = MorphState::new(50.0, 50.0, 200.0, 200.0, 20.0);
    let below = MorphState::lerp(&from, &to, -0.5);
    assert_eq!(below.x, 0.0); // clamped to 0.0
    let above = MorphState::lerp(&from, &to, 1.5);
    assert_eq!(above.x, 50.0); // clamped to 1.0
}

#[test]
fn spring_easing_starts_at_zero() {
    use crate::foundations::motion::spring_easing;
    let easing = spring_easing(0.85, 0.35, 0.0);
    assert!((easing(0.0) - 0.0).abs() < 0.01);
}

#[test]
fn spring_easing_ends_at_one() {
    use crate::foundations::motion::spring_easing;
    let easing = spring_easing(0.85, 0.35, 0.0);
    assert!((easing(1.0) - 1.0).abs() < 0.01);
}

#[test]
fn spring_easing_monotonic_no_bounce() {
    use crate::foundations::motion::spring_easing;
    let easing = spring_easing(0.85, 0.35, 0.0);
    let mut prev = 0.0;
    for i in 0..=100 {
        let t = i as f32 / 100.0;
        let v = easing(t);
        assert!(
            v >= prev - 0.001,
            "Spring should be monotonic with no bounce: t={t}, v={v}, prev={prev}"
        );
        prev = v;
    }
}

// --- Phase 10: Blur & Lens Effect Tests ─────────────────────────────────

#[test]
fn elevation_shadows_length_resolves_per_tier() {
    // Resting / Elevated / Floating each resolve to the matching shadow
    // stack on `GlassStyle` — a single lookup per tier confirms the
    // `Elevation::shadows` dispatch.
    let theme = TahoeTheme::liquid_glass();
    assert_eq!(
        Elevation::Resting.shadows(&theme).len(),
        theme.glass.resting_shadows.len()
    );
    assert_eq!(
        Elevation::Elevated.shadows(&theme).len(),
        theme.glass.elevated_shadows.len()
    );
    assert_eq!(
        Elevation::Floating.shadows(&theme).len(),
        theme.glass.floating_shadows.len()
    );
}

// ─── HIG Typography v2: font_mono, font_scale, ios_attrs_emphasized ──

#[test]
fn font_mono_defaults_to_sf_mono() {
    // Per HIG: SF Mono is the system monospaced typeface on macOS 10.15+.
    // Fallbacks cover Linux, Windows, and macOS hosts without Xcode so code
    // text stays monospaced on every host (finding #29).
    let expected_fallbacks = ["Menlo", "Monaco", "Courier New", "monospace"];
    for theme in [
        TahoeTheme::dark(),
        TahoeTheme::light(),
        TahoeTheme::liquid_glass(),
        TahoeTheme::liquid_glass_light(),
    ] {
        assert_eq!(
            theme.font_mono.as_ref(),
            "SF Mono",
            "font_mono should be SF Mono, got {}",
            theme.font_mono
        );
        assert_eq!(
            theme.font_mono_fallbacks.fallback_list(),
            expected_fallbacks,
            "font_mono_fallbacks default list mismatch"
        );
    }
}

#[test]
fn with_font_mono_fallbacks_replaces_list() {
    let custom = FontFallbacks::from_fonts(vec!["JetBrains Mono".into(), "Menlo".into()]);
    let theme = TahoeTheme::dark().with_font_mono_fallbacks(custom);
    assert_eq!(
        theme.font_mono_fallbacks.fallback_list(),
        ["JetBrains Mono", "Menlo"],
        "builder must replace the default fallback list"
    );
    // Pin the builder → helper integration so `font_mono()` cannot drift
    // away from reading `font_mono_fallbacks`.
    assert_eq!(
        theme
            .font_mono()
            .fallbacks
            .as_ref()
            .expect("font_mono must forward the overridden list")
            .fallback_list(),
        ["JetBrains Mono", "Menlo"]
    );
}

#[test]
fn font_mono_wires_family_and_fallbacks() {
    let theme = TahoeTheme::dark();
    let font = theme.font_mono();
    assert_eq!(font.family.as_ref(), "SF Mono");
    let fallbacks = font
        .fallbacks
        .as_ref()
        .expect("font_mono must populate fallbacks so TextStyle::to_run forwards them");
    assert_eq!(
        fallbacks.fallback_list(),
        ["Menlo", "Monaco", "Courier New", "monospace"]
    );
}

#[test]
fn font_scale_factor_defaults_to_one() {
    for theme in [
        TahoeTheme::dark(),
        TahoeTheme::light(),
        TahoeTheme::liquid_glass(),
        TahoeTheme::liquid_glass_light(),
    ] {
        assert!(
            (theme.font_scale_factor - 1.0).abs() < f32::EPSILON,
            "font_scale_factor should default to 1.0, got {}",
            theme.font_scale_factor
        );
    }
}

#[test]
fn with_font_scale_factor_sets_positive_value() {
    let theme = TahoeTheme::dark().with_font_scale_factor(1.25);
    assert!((theme.font_scale_factor - 1.25).abs() < f32::EPSILON);
}

#[test]
fn with_font_scale_factor_rejects_invalid_values() {
    for bad in [0.0, -0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let theme = TahoeTheme::dark().with_font_scale_factor(bad);
        assert!(
            (theme.font_scale_factor - 1.0).abs() < f32::EPSILON,
            "invalid scale {bad} should clamp to 1.0, got {}",
            theme.font_scale_factor
        );
    }
}

#[test]
fn effective_font_scale_factor_returns_stored_value_when_valid() {
    for good in [0.5, 1.0, 1.25, 2.0] {
        let mut theme = TahoeTheme::dark();
        theme.font_scale_factor = good;
        assert!(
            (theme.effective_font_scale_factor() - good).abs() < f32::EPSILON,
            "valid scale {good} should pass through, got {}",
            theme.effective_font_scale_factor()
        );
    }
}

#[test]
fn effective_font_scale_factor_clamps_invalid_field_values() {
    // Direct pub-field assignment bypasses `with_font_scale_factor`'s guard,
    // so the reader must enforce the same contract to prevent invisible or
    // infinite text. Regression test for #66. `-0.0` pins the strict `> 0.0`
    // invariant — a naive refactor to `>= 0.0` would silently regress.
    for bad in [0.0, -0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut theme = TahoeTheme::dark();
        theme.font_scale_factor = bad;
        assert!(
            (theme.effective_font_scale_factor() - 1.0).abs() < f32::EPSILON,
            "invalid field value {bad} should read as 1.0, got {}",
            theme.effective_font_scale_factor()
        );
    }
}

#[test]
fn ios_attrs_emphasized_large_titles_are_bold() {
    for size in [
        DynamicTypeSize::XSmall,
        DynamicTypeSize::Large,
        DynamicTypeSize::AX5,
    ] {
        for style in [TextStyle::LargeTitle, TextStyle::Title1, TextStyle::Title2] {
            assert_eq!(
                style.ios_attrs_emphasized(size).weight,
                FontWeight::BOLD,
                "{:?} @ {:?} emphasized should be Bold",
                style,
                size
            );
        }
    }
}

#[test]
fn ios_attrs_emphasized_other_styles_are_semibold() {
    for size in [
        DynamicTypeSize::XSmall,
        DynamicTypeSize::Large,
        DynamicTypeSize::AX5,
    ] {
        for style in [
            TextStyle::Title3,
            TextStyle::Headline,
            TextStyle::Body,
            TextStyle::Callout,
            TextStyle::Subheadline,
            TextStyle::Footnote,
            TextStyle::Caption1,
            TextStyle::Caption2,
        ] {
            assert_eq!(
                style.ios_attrs_emphasized(size).weight,
                FontWeight::SEMIBOLD,
                "{:?} @ {:?} emphasized should be Semibold",
                style,
                size
            );
        }
    }
}

#[test]
fn ios_attrs_emphasized_preserves_size_and_leading() {
    for size in [
        DynamicTypeSize::XSmall,
        DynamicTypeSize::Large,
        DynamicTypeSize::XXLarge,
        DynamicTypeSize::AX3,
    ] {
        for style in [
            TextStyle::LargeTitle,
            TextStyle::Headline,
            TextStyle::Body,
            TextStyle::Caption1,
        ] {
            let base = style.ios_attrs(size);
            let emph = style.ios_attrs_emphasized(size);
            assert_eq!(base.size, emph.size);
            assert_eq!(base.leading, emph.leading);
        }
    }
}

#[test]
fn glass_effect_blur_no_refraction() {
    // The blur-only entry point shares its frost with the lens recipe
    // but skips refraction. Coverage for the (Glass, Elevation) matrix
    // lives alongside the internal recipes in
    // `foundations::materials::tests`; this smoke test just confirms the
    // entry point builds cleanly from a theme module perspective.
    use crate::foundations::materials::{Shape, glass_effect_blur};
    let theme = TahoeTheme::liquid_glass();
    let _: gpui::Div = glass_effect_blur(
        &theme,
        Glass::Regular,
        Shape::Default,
        Elevation::Elevated,
        None,
    );
}

#[gpui::test]
async fn active_theme_trait_resolves_to_registered_global(cx: &mut gpui::TestAppContext) {
    // Register a light theme; `cx.theme()` through the `ActiveTheme` trait
    // must return the same pointer as `cx.global::<TahoeTheme>()`. Guards
    // the additive trait in case a future refactor of the theme backing
    // store forgets to route through the global registry.
    cx.update(|cx| {
        cx.set_global(TahoeTheme::light());
        let via_trait = cx.theme() as *const TahoeTheme;
        let via_global = cx.global::<TahoeTheme>() as *const TahoeTheme;
        assert_eq!(
            via_trait, via_global,
            "cx.theme() and cx.global::<TahoeTheme>() must resolve to the same instance"
        );
        assert!(
            cx.theme().background.l > 0.8,
            "light theme should have a light background"
        );
    });
}

// Regression for https://github.com/df49b9cd/tahoe-gpui/issues/55:
// every public `TahoeTheme` constructor — including the four liquid-glass
// variants that build `SemanticColors` via struct literal instead of
// `SemanticColors::new()` — must keep `activity_ring_backdrop` pinned to
// the HIG "system dark gray" value. Catches value drift in any of the four
// construction sites even though the compiler already enforces field
// presence.
#[test]
fn activity_ring_backdrop_is_system_dark_gray_in_every_theme() {
    let themes: &[(&str, TahoeTheme)] = &[
        ("dark", TahoeTheme::dark()),
        ("light", TahoeTheme::light()),
        ("liquid_glass", TahoeTheme::liquid_glass()),
        ("liquid_glass_light", TahoeTheme::liquid_glass_light()),
        ("liquid_glass_clear", TahoeTheme::liquid_glass_clear()),
        (
            "liquid_glass_clear_light",
            TahoeTheme::liquid_glass_clear_light(),
        ),
    ];
    for (name, theme) in themes {
        let backdrop = theme.semantic.activity_ring_backdrop;
        assert!(
            (backdrop.l - 0.07).abs() < 1e-4,
            "{name}: activity_ring_backdrop lightness should be 0.07, got {}",
            backdrop.l
        );
        assert!(
            (backdrop.a - 1.0).abs() < f32::EPSILON,
            "{name}: activity_ring_backdrop must be opaque"
        );
    }
}
