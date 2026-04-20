use super::{
    AccessibilityMode, ActiveTheme, DynamicTypeSize, FontDesign, GlassSize, GlassTintColor,
    GlassVariant, LeadingStyle, TahoeTheme, TextStyle, bold_step, contrast_ratio, macos_tracking,
    meets_contrast,
};
use crate::foundations::color::{AccentColor, Appearance};
use core::prelude::v1::test;
use gpui::{FontWeight, hsla};

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
    // Per HIG macOS Tahoe, glass is always present.
    let _dark = TahoeTheme::dark().glass;
    let _light = TahoeTheme::light().glass;
    let _lg = TahoeTheme::liquid_glass().glass;
    // All three have GlassStyle -- no Option unwrap needed.
    assert!(f32::from(_dark.small_radius) > 0.0);
    assert!(f32::from(_light.small_radius) > 0.0);
    assert!(f32::from(_lg.small_radius) > 0.0);
}

#[test]
fn glass_bg_is_semi_transparent() {
    // Dark small_bg is fully opaque (Figma-accurate composite of 3 fills),
    // but medium and large use container opacity < 1.0 — the glass achieves
    // translucency through the window blur, not necessarily the fill alpha.
    let glass = TahoeTheme::liquid_glass().glass;
    assert!(
        glass.small_bg.a > 0.0,
        "small_bg should have non-zero alpha"
    );
    assert!(
        glass.medium_bg.a > 0.0 && glass.medium_bg.a < 1.0,
        "medium_bg should be semi-transparent, got {}",
        glass.medium_bg.a
    );
    assert!(
        glass.large_bg.a > 0.0 && glass.large_bg.a < 1.0,
        "large_bg should be semi-transparent, got {}",
        glass.large_bg.a
    );
}

#[test]
fn glass_hover_more_opaque_than_base() {
    // Hover is an additive overlay on the glass surface — it should have
    // substantial alpha so the hover state is visible. With Figma-accurate
    // dark fills the base containers already have high opacity, so we verify
    // hover_bg exceeds the clear fills (the thinnest glass variant).
    let glass = TahoeTheme::liquid_glass().glass;
    assert!(
        glass.hover_bg.a > 0.0,
        "hover_bg should have positive alpha"
    );
    assert!(
        glass.hover_bg.a > glass.clear_small_bg.a,
        "hover_bg alpha ({}) should exceed clear_small_bg alpha ({})",
        glass.hover_bg.a,
        glass.clear_small_bg.a
    );
}

#[test]
fn glass_radius_larger_than_standard_dark() {
    let glass = TahoeTheme::liquid_glass().glass;
    let dark = TahoeTheme::dark();
    assert!(glass.small_radius > dark.radius_lg);
    assert!((f32::from(glass.medium_radius) - 34.0).abs() < f32::EPSILON);
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
fn small_shadow_count() {
    let glass = TahoeTheme::liquid_glass().glass;
    // Per Figma: single drop shadow for all sizes
    assert_eq!(glass.small_shadows.len(), 1);
}

#[test]
fn large_shadow_count() {
    let glass = TahoeTheme::liquid_glass().glass;
    // Per Figma: single drop shadow (X:0 Y:8 Blur:40 Spread:0 #000000@12%)
    assert_eq!(glass.large_shadows.len(), 1);
}

#[test]
fn medium_shadow_count() {
    let glass = TahoeTheme::liquid_glass().glass;
    assert_eq!(glass.medium_shadows.len(), 1);
}

#[test]
fn shadows_scale_by_size() {
    let glass = TahoeTheme::liquid_glass().glass;
    let small_blur = f32::from(glass.small_shadows[0].blur_radius);
    let medium_blur = f32::from(glass.medium_shadows[0].blur_radius);
    let large_blur = f32::from(glass.large_shadows[0].blur_radius);
    assert!(
        small_blur < medium_blur,
        "small blur ({small_blur}) should be less than medium ({medium_blur})"
    );
    assert!(
        medium_blur < large_blur,
        "medium blur ({medium_blur}) should be less than large ({large_blur})"
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
fn with_accent_color_propagates_to_derived_tokens() {
    // Switching the accent updates accent / ring / focus_ring / text_on_accent
    // and the glass accent tint, so a host that detects a runtime accent
    // change can rebuild without losing the rest of the theme.
    let base = TahoeTheme::dark();
    let purple = base.clone().with_accent_color(AccentColor::Purple);
    assert_ne!(purple.accent, base.accent);
    assert_eq!(purple.accent, purple.palette.purple);
    assert_eq!(purple.ring, purple.accent);
    assert_eq!(purple.focus_ring_color, purple.accent);
    assert_eq!(purple.glass.accent_tint.bg, purple.accent);
    assert_eq!(purple.accent_color, AccentColor::Purple);
    // Non-accent fields stay put.
    assert_eq!(purple.background, base.background);
    assert_eq!(purple.text, base.text);
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
    assert_eq!(glass.variant, GlassVariant::Regular);
}

#[test]
fn glass_clear_fills_semi_transparent() {
    let glass = TahoeTheme::liquid_glass().glass;
    for fill in [
        glass.clear_small_bg,
        glass.clear_medium_bg,
        glass.clear_large_bg,
    ] {
        assert!(fill.a > 0.0 && fill.a < 0.3);
    }
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
fn glass_bg_respects_variant() {
    let theme = TahoeTheme::liquid_glass();
    let glass = theme.glass;
    // Regular variant returns regular fills
    let regular = glass.bg(GlassSize::Small);
    assert_eq!(regular, glass.small_bg);
    // Clear fills are different from regular
    let clear = glass.clear_fill(GlassSize::Small);
    assert_ne!(regular, clear);
}

#[test]
fn glass_regular_and_clear_accessors() {
    let glass = TahoeTheme::liquid_glass().glass;
    assert_eq!(glass.regular_bg(GlassSize::Small), glass.small_bg);
    assert_eq!(glass.regular_bg(GlassSize::Medium), glass.medium_bg);
    assert_eq!(glass.regular_bg(GlassSize::Large), glass.large_bg);
    assert_eq!(glass.clear_fill(GlassSize::Small), glass.clear_small_bg);
    assert_eq!(glass.clear_fill(GlassSize::Medium), glass.clear_medium_bg);
    assert_eq!(glass.clear_fill(GlassSize::Large), glass.clear_large_bg);
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
    // Glass is always present -- just verify it has valid fields.
    assert!(f32::from(TahoeTheme::liquid_glass_light().glass.small_radius) > 0.0);
}

#[test]
fn liquid_glass_light_background_is_light() {
    let theme = TahoeTheme::liquid_glass_light();
    assert!(theme.background.l > 0.5);
}

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
fn liquid_glass_light_fills_are_white_tinted() {
    let glass = TahoeTheme::liquid_glass_light().glass;
    // Light glass uses semi-transparent white, not black
    assert!(glass.small_bg.l > 0.5);
    assert!(glass.medium_bg.l > 0.5);
    assert!(glass.large_bg.l > 0.5);
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
fn liquid_glass_light_geometry_matches_dark() {
    let dark_glass = TahoeTheme::liquid_glass().glass;
    let light_glass = TahoeTheme::liquid_glass_light().glass;
    assert_eq!(dark_glass.small_radius, light_glass.small_radius);
    assert_eq!(dark_glass.medium_radius, light_glass.medium_radius);
    assert_eq!(dark_glass.large_radius, light_glass.large_radius);
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
    // Dark small_bg is fully opaque (Figma composite), so compare the
    // reduced-transparency fallback against medium_bg which uses container
    // opacity — the fallback should be more opaque to serve its purpose.
    let glass = TahoeTheme::liquid_glass().glass;
    assert!(
        glass.accessibility.reduced_transparency_bg.a > glass.medium_bg.a,
        "reduced_transparency_bg alpha ({}) should exceed medium_bg alpha ({})",
        glass.accessibility.reduced_transparency_bg.a,
        glass.medium_bg.a
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

// ─── HIG Alignment: GlassVariant::Identity Tests ────────────────────────

#[test]
fn glass_identity_returns_transparent() {
    let mut glass = TahoeTheme::liquid_glass().glass;
    glass.variant = GlassVariant::Identity;
    let bg = glass.bg(GlassSize::Small);
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
    // With response=0.35, duration should be ~1400ms
    assert!(
        dur > 500 && dur < 3000,
        "spring_duration_ms should be reasonable, got {}",
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

#[test]
fn leading_style_tight_reduces() {
    let standard = TextStyle::Body.attrs();
    let tight = standard.with_leading(LeadingStyle::Tight);
    assert!(f32::from(tight.leading) < f32::from(standard.leading));
    assert!((f32::from(standard.leading) - f32::from(tight.leading) - 2.0).abs() < f32::EPSILON);
}

#[test]
fn leading_style_loose_increases() {
    let standard = TextStyle::Body.attrs();
    let loose = standard.with_leading(LeadingStyle::Loose);
    assert!(f32::from(loose.leading) > f32::from(standard.leading));
    assert!((f32::from(loose.leading) - f32::from(standard.leading) - 2.0).abs() < f32::EPSILON);
}

#[test]
fn leading_style_standard_unchanged() {
    let attrs = TextStyle::Body.attrs();
    let standard = attrs.with_leading(LeadingStyle::Standard);
    assert_eq!(attrs.leading, standard.leading);
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
fn font_design_families() {
    assert_eq!(FontDesign::Default.font_family(), ".AppleSystemUIFont");
    assert_eq!(FontDesign::Serif.font_family(), "New York");
    assert_eq!(
        FontDesign::Rounded.font_family(),
        ".AppleSystemUIFontRounded"
    );
    assert_eq!(FontDesign::Monospaced.font_family(), "SF Mono");
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

// ─── Phase 4: Clear Glass Variant Tests ─────────────────────────────────

#[test]
fn liquid_glass_clear_uses_clear_variant() {
    let theme = TahoeTheme::liquid_glass_clear();
    assert_eq!(theme.glass.variant, GlassVariant::Clear);
}

#[test]
fn liquid_glass_clear_light_uses_clear_variant() {
    let theme = TahoeTheme::liquid_glass_clear_light();
    assert_eq!(theme.glass.variant, GlassVariant::Clear);
}

#[test]
fn clear_fills_more_transparent_than_regular() {
    let theme = TahoeTheme::liquid_glass();
    let regular = theme.glass.regular_bg(GlassSize::Small);
    let clear = theme.glass.clear_fill(GlassSize::Small);
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
fn blur_effect_for_glass_size() {
    use crate::foundations::materials::BlurEffect;
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
    use crate::foundations::materials::LensEffect;
    let theme = TahoeTheme::dark();
    let lens = LensEffect::liquid_glass(GlassSize::Medium, &theme);
    assert_eq!(lens.refraction, 1.0);
    assert_eq!(lens.dispersion, 0.0);
    assert!(lens.light_intensity > 0.0);
    assert_eq!(lens.blur.radius, 12.0);
}

// ─── HIG Typography v2: font_mono, font_scale, ios_attrs_emphasized ──

#[test]
fn font_mono_defaults_to_sf_mono() {
    // Per HIG: SF Mono is the system monospaced typeface on macOS 10.15+.
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
    }
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
    for bad in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let theme = TahoeTheme::dark().with_font_scale_factor(bad);
        assert!(
            (theme.font_scale_factor - 1.0).abs() < f32::EPSILON,
            "invalid scale {bad} should clamp to 1.0, got {}",
            theme.font_scale_factor
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
fn lens_effect_blur_only_no_refraction() {
    use crate::foundations::materials::LensEffect;
    let theme = TahoeTheme::dark();
    let lens = LensEffect::blur_only(GlassSize::Small, &theme);
    assert_eq!(lens.refraction, 0.0);
    assert_eq!(lens.dispersion, 0.0);
    assert_eq!(lens.light_intensity, 0.0);
    assert!(lens.blur.radius > 0.0);
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
