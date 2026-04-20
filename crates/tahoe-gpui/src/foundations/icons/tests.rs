use super::{
    EmbeddedIconAssets, Icon, IconName, IconRenderMode, IconScale, IconStyle, RenderStrategy,
    hierarchical_opacity, weight_to_stroke_width,
};
use crate::foundations::surface_scope::GlassSurfaceGuard;
use crate::foundations::theme::TahoeTheme;
use core::prelude::v1::test;
use gpui::AssetSource;

/// Every known `IconName` variant. When adding a new variant to the enum,
/// you MUST also add it here -- the count assertion below will catch omissions.
const ALL_VARIANTS: &[IconName] = &[
    IconName::ArrowDown,
    IconName::ArrowRight,
    IconName::ArrowTriangleDown,
    IconName::ArrowTriangleRight,
    IconName::Brain,
    IconName::Check,
    IconName::ChevronDown,
    IconName::ChevronLeft,
    IconName::ChevronRight,
    IconName::ChevronUp,
    IconName::Copy,
    IconName::Download,
    IconName::Send,
    IconName::Square,
    IconName::X,
    IconName::Loader,
    IconName::ProgressSpinner,
    IconName::Code,
    IconName::File,
    IconName::Folder,
    IconName::FolderOpen,
    IconName::Terminal,
    IconName::Play,
    IconName::Pause,
    IconName::Mic,
    IconName::MicOff,
    IconName::StopFill,
    IconName::Phone,
    IconName::Video,
    IconName::Settings,
    IconName::Clock,
    IconName::Bookmark,
    IconName::Book,
    IconName::Search,
    IconName::Link,
    IconName::Globe,
    IconName::Sparkle,
    IconName::StarFill,
    IconName::Star,
    IconName::StarLeadingHalfFilled,
    IconName::ListTodo,
    IconName::CircleFilled,
    IconName::CircleOutline,
    IconName::AlertTriangle,
    IconName::Info,
    IconName::Image,
    IconName::Plus,
    IconName::Minus,
    IconName::Bug,
    IconName::TestTube,
    IconName::GitCommit,
    IconName::Package,
    IconName::Database,
    IconName::Key,
    IconName::Bot,
    IconName::FileCode,
    IconName::Trash,
    IconName::Eye,
    IconName::EyeOff,
    IconName::ExternalLink,
    IconName::ChevronsUpDown,
    IconName::ThumbsUp,
    IconName::ThumbsDown,
    IconName::RotateCcw,
    IconName::RotateCw,
    IconName::Share,
    IconName::Pencil,
    IconName::Volume2,
    IconName::VolumeX,
    IconName::SkipBack,
    IconName::SkipForward,
    IconName::Lock,
    IconName::Unlock,
    IconName::Maximize,
    IconName::Paperclip,
    IconName::XmarkCircleFill,
    IconName::Ellipsis,
    IconName::SidebarLeft,
    IconName::QuestionMark,
    IconName::DevTab,
    IconName::DevSidebar,
    IconName::DevSplitView,
    IconName::DevSearch,
    IconName::DevFindReplace,
    IconName::DevMinimap,
    IconName::DevBreadcrumb,
    IconName::DevSnippet,
    IconName::DevPalette,
    IconName::DevExtension,
    IconName::DevKeyboard,
    IconName::DevDebug,
    IconName::Agent,
    IconName::Prompt,
    IconName::Chain,
    IconName::ToolUse,
    IconName::Memory,
    IconName::Context,
    IconName::Embedding,
    IconName::Rag,
    IconName::Orchestrator,
    IconName::Model,
    IconName::Streaming,
    IconName::FunctionCall,
    IconName::Guardrail,
    IconName::Token,
    IconName::FineTune,
    IconName::Deploy,
    IconName::CiCd,
    IconName::Container,
    IconName::Pipeline,
    IconName::Monitor,
    IconName::Logs,
    IconName::Environment,
    IconName::Secret,
    IconName::Webhook,
    IconName::Api,
    IconName::Scale,
    IconName::Rollback,
    IconName::Health,
    IconName::Queue,
    IconName::Cache,
    IconName::GitBranch,
    IconName::GitMerge,
    IconName::GitConflict,
    IconName::GitPull,
    IconName::GitPush,
    IconName::GitCheckout,
    IconName::GitStash,
    IconName::GitTag,
    IconName::GitLog,
    IconName::GitRebase,
    IconName::GitCompare,
    IconName::GitInlineDiff,
    IconName::GitStaging,
    IconName::GitPullRequest,
    IconName::GitCodeReview,
    IconName::GitFork,
    IconName::GitClone,
    IconName::GitRemote,
    IconName::GitBlame,
    IconName::GitStaged,
    IconName::GitModified,
    IconName::GitUntracked,
    IconName::GitAdded,
    IconName::GitDeleted,
    IconName::GitIgnored,
    IconName::GitAhead,
    IconName::GitBehind,
    IconName::GitClean,
    IconName::LangRust,
    IconName::LangPython,
    IconName::LangJavaScript,
    IconName::LangTypeScript,
    IconName::LangGo,
    IconName::LangC,
    IconName::LangCpp,
    IconName::LangBash,
    IconName::LangJson,
    IconName::LangToml,
    IconName::LangHtml,
    IconName::LangCss,
    IconName::ProviderClaude,
    IconName::ProviderGpt,
    IconName::ProviderGemini,
    IconName::ProviderGrok,
    IconName::ProviderLlama,
    IconName::ProviderDeepSeek,
    IconName::ProviderMistral,
    IconName::ProviderGemma,
    IconName::ProviderPhi,
    IconName::ProviderQwen,
    IconName::ProviderGlm,
    IconName::ProviderMiniMax,
    IconName::ProviderErnie,
    IconName::ProviderCohere,
    IconName::ProviderPerplexity,
    IconName::ProviderNova,
    IconName::ProviderCustom,
];

#[test]
fn all_variants_count_matches() {
    assert_eq!(
        ALL_VARIANTS.len(),
        178,
        "New IconName variants must be added to ALL_VARIANTS above",
    );
}

#[test]
fn all_symbols_non_empty() {
    for v in ALL_VARIANTS {
        let sym = v.symbol();
        assert!(!sym.is_empty(), "{v:?}.symbol() returned an empty string",);
    }
}

/// HIG: do not use emoji in icons that appear in system-controlled surfaces
/// (toolbars, menus, context menus). Emoji are rendered with their own color
/// font and resist `text_color` tinting. Fallbacks must stay outside the
/// pictographic planes U+1F000..=U+1FAFF (Emoticons, Misc Symbols and
/// Pictographs, Transport, Supplemental Symbols and Pictographs) plus the
/// Regional Indicator block U+1F1E6..=U+1F1FF.
#[test]
fn fallback_symbols_contain_no_emoji_codepoints() {
    for v in ALL_VARIANTS {
        for c in v.symbol().chars() {
            let cp = c as u32;
            assert!(
                !(0x1F000..=0x1FAFF).contains(&cp),
                "{v:?}.symbol() returned codepoint U+{cp:04X} in the emoji plane; HIG prohibits emoji in system-surface icons",
            );
        }
    }
}

/// Every `system_name()` mapping must point at a bundled asset that loads
/// to non-empty bytes — otherwise hosts that call `Icon::new(name)` will
/// silently fall back to the Unicode placeholder for a glyph that should
/// have rendered natively.
#[test]
fn system_name_mapping_invariants_hold() {
    let assets = EmbeddedIconAssets;
    let mut with_system_name = 0;

    for v in ALL_VARIANTS {
        if v.system_name().is_some() {
            with_system_name += 1;
            let path =
                v.bundled_asset_path().unwrap_or_else(|| {
                    panic!(
                        "{v:?}: system_name() returned Some but bundled_asset_path() returned None",
                    );
                });
            assert!(
                path.starts_with("icons/symbols/") && path.ends_with(".svg"),
                "{v:?}: bundled_asset_path {path:?} should live under icons/symbols/",
            );
            assert!(
                assets.load(path).unwrap().is_some_and(|b| !b.is_empty()),
                "{v:?}: bundled asset {path:?} is not embedded in ICON_ENTRIES or is empty",
            );
        }
    }

    // Guard against a future refactor silently losing the system_name
    // mapping for the canonical UI-action set.
    assert!(
        with_system_name >= 60,
        "expected >= 60 variants mapped to a system_name, found {with_system_name}",
    );
}

#[test]
fn all_render_strategy_paths_resolve() {
    let assets = EmbeddedIconAssets;

    for v in ALL_VARIANTS {
        let strategy = match v.render_strategy() {
            Some(s) => s,
            None => continue,
        };
        match strategy {
            RenderStrategy::Monochrome(path) => {
                assert!(
                    assets.load(path).unwrap().is_some(),
                    "{v:?}: Monochrome path {path:?} did not resolve",
                );
            }
            RenderStrategy::MultiColor(layers) => {
                for (layer_path, _role) in layers {
                    assert!(
                        assets.load(layer_path).unwrap().is_some(),
                        "{v:?}: MultiColor layer path {layer_path:?} did not resolve",
                    );
                }
            }
        }
    }
}

/// All bundled paths must live under `icons/symbols/` (generic UI) or one
/// of the domain-specific roots `icons/{git,dev-tools,languages,providers}/`.
/// Standard and Liquid Glass themes share the same asset set; the glass
/// appearance is produced by tinting layers, not by swapping SVGs.
#[test]
fn all_paths_live_under_known_roots() {
    let assets = EmbeddedIconAssets;
    const ROOTS: &[&str] = &[
        "icons/symbols/",
        "icons/git/",
        "icons/dev-tools/",
        "icons/languages/",
        "icons/providers/",
    ];
    let starts_with_root = |p: &str| -> bool { ROOTS.iter().any(|root| p.starts_with(root)) };

    for variant in ALL_VARIANTS {
        if let Some(strategy) = variant.render_strategy() {
            match strategy {
                RenderStrategy::Monochrome(path) => {
                    assert!(
                        starts_with_root(path),
                        "{variant:?}: path {path:?} not under a known root {ROOTS:?}",
                    );
                    assert!(
                        assets.load(path).unwrap().is_some(),
                        "{variant:?}: path {path:?} did not resolve",
                    );
                }
                RenderStrategy::MultiColor(layers) => {
                    for (layer_path, _role) in layers.iter() {
                        assert!(
                            starts_with_root(layer_path),
                            "{variant:?}: layer {layer_path:?} not under a known root {ROOTS:?}",
                        );
                        assert!(
                            assets.load(layer_path).unwrap().is_some(),
                            "{variant:?}: layer {layer_path:?} did not resolve",
                        );
                    }
                }
            }
        }
    }
}

// ── IconScale Tests ──────────────────────────────────────────────────────

#[test]
fn icon_scale_default_is_medium() {
    let scale = IconScale::default();
    assert_eq!(scale, IconScale::Medium);
}

#[test]
fn icon_scale_small_multiplier() {
    assert!((IconScale::Small.multiplier() - 0.75).abs() < f32::EPSILON);
}

#[test]
fn icon_scale_medium_multiplier() {
    assert!((IconScale::Medium.multiplier() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn icon_scale_large_multiplier() {
    assert!((IconScale::Large.multiplier() - 1.25).abs() < f32::EPSILON);
}

// ── IconRenderMode Tests ─────────────────────────────────────────────────

#[test]
fn icon_render_mode_default_is_monochrome() {
    let mode = IconRenderMode::default();
    assert_eq!(mode, IconRenderMode::Monochrome);
}

#[test]
fn hierarchical_opacity_primary_is_full() {
    assert!((hierarchical_opacity(0) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn hierarchical_opacity_secondary_is_half() {
    assert!((hierarchical_opacity(1) - 0.50).abs() < f32::EPSILON);
}

#[test]
fn hierarchical_opacity_tertiary_is_quarter() {
    assert!((hierarchical_opacity(2) - 0.25).abs() < f32::EPSILON);
    assert!((hierarchical_opacity(5) - 0.25).abs() < f32::EPSILON);
}

// ── Weight-to-Stroke Tests ───────────────────────────────────────────────

#[test]
fn weight_to_stroke_width_thin() {
    use gpui::FontWeight;
    assert!((weight_to_stroke_width(FontWeight::THIN) - 0.8).abs() < f32::EPSILON);
}

#[test]
fn weight_to_stroke_width_light() {
    use gpui::FontWeight;
    assert!((weight_to_stroke_width(FontWeight::LIGHT) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn weight_to_stroke_width_normal() {
    use gpui::FontWeight;
    assert!((weight_to_stroke_width(FontWeight::NORMAL) - 1.2).abs() < f32::EPSILON);
}

#[test]
fn weight_to_stroke_width_medium() {
    use gpui::FontWeight;
    assert!((weight_to_stroke_width(FontWeight::MEDIUM) - 1.4).abs() < f32::EPSILON);
}

#[test]
fn weight_to_stroke_width_semibold() {
    use gpui::FontWeight;
    assert!((weight_to_stroke_width(FontWeight::SEMIBOLD) - 1.5).abs() < f32::EPSILON);
}

#[test]
fn weight_to_stroke_width_bold() {
    use gpui::FontWeight;
    assert!((weight_to_stroke_width(FontWeight::BOLD) - 1.8).abs() < f32::EPSILON);
}

#[test]
fn weight_bold_step_bumps_normal_to_medium() {
    use crate::foundations::theme::bold_step;
    use gpui::FontWeight;
    assert_eq!(bold_step(FontWeight::NORMAL), FontWeight::MEDIUM);
}

#[test]
fn weight_bold_step_bumps_semibold_to_bold() {
    use crate::foundations::theme::bold_step;
    use gpui::FontWeight;
    assert_eq!(bold_step(FontWeight::SEMIBOLD), FontWeight::BOLD);
}

// ── Builder Method Tests ─────────────────────────────────────────────────

#[test]
fn icon_builder_scale_sets_field() {
    let icon = Icon::new(IconName::Check).scale(IconScale::Large);
    assert_eq!(icon.scale, Some(IconScale::Large));
}

#[test]
fn icon_builder_render_mode_sets_field() {
    let icon = Icon::new(IconName::Check).render_mode(IconRenderMode::Hierarchical);
    assert_eq!(icon.render_mode, Some(IconRenderMode::Hierarchical));
}

#[test]
fn icon_builder_weight_sets_field() {
    use gpui::FontWeight;
    let icon = Icon::new(IconName::Check).weight(FontWeight::BOLD);
    assert_eq!(icon.weight, Some(FontWeight::BOLD));
}

#[test]
fn icon_with_rotate_animation_encodes_turns_per_second() {
    use super::IconAnimation;
    use gpui::px;
    use std::time::Duration;

    let anim = Icon::new(IconName::Loader)
        .size(px(16.0))
        .with_rotate_animation("spin", 2.0);
    // 2 turns per second → 500 ms per revolution.
    match anim.animation {
        IconAnimation::Spin { duration } => {
            assert_eq!(duration, Duration::from_millis(500));
        }
        other => panic!("expected Spin, got {:?}", other),
    }
}

#[test]
fn icon_with_rotate_animation_guards_nonpositive_turns() {
    use super::IconAnimation;
    use std::time::Duration;

    // Zero tps → fallback to 1 tps (1000 ms period), never a divide-by-zero.
    let anim = Icon::new(IconName::Loader).with_rotate_animation("spin", 0.0);
    match anim.animation {
        IconAnimation::Spin { duration } => {
            assert_eq!(duration, Duration::from_millis(1000));
        }
        other => panic!("expected Spin, got {:?}", other),
    }

    // Negative tps also falls back to 1 tps.
    let anim = Icon::new(IconName::Loader).with_rotate_animation("spin", -3.0);
    match anim.animation {
        IconAnimation::Spin { duration } => {
            assert_eq!(duration, Duration::from_millis(1000));
        }
        other => panic!("expected Spin, got {:?}", other),
    }
}

// ─── IconStyle::Auto surface-scope resolution (issue #13) ─────────────────

/// Outside a [`GlassSurfaceGuard`], `IconStyle::Auto` resolves to
/// `Standard`. This is the core fix for issue #13 — previously `Auto`
/// always returned `LiquidGlass`, which mis-colored icons under plain
/// dark/light themes.
#[test]
fn icon_style_auto_resolves_to_standard_when_not_scoped() {
    // Theme doesn't enter the decision; iterate it anyway to lock the
    // invariant "vibrancy is a surface concern, not a theme one."
    let _ = TahoeTheme::dark();
    let _ = TahoeTheme::light();
    let _ = TahoeTheme::liquid_glass();
    assert_eq!(IconStyle::Auto.resolve(), IconStyle::Standard);
}

/// Inside a [`GlassSurfaceGuard`], `IconStyle::Auto` resolves to
/// `LiquidGlass` — this is how glass-surface components (GlassIconTile,
/// glass Button variants) flag "my subtree sits on Liquid Glass;
/// descendants should adopt vibrancy." Surface scope, not theme, drives
/// the choice.
#[test]
fn icon_style_auto_resolves_to_glass_when_scoped() {
    let _g = GlassSurfaceGuard::enter();
    assert_eq!(IconStyle::Auto.resolve(), IconStyle::LiquidGlass);
}

/// Dropping the guard must return the resolution to `Standard` so scope
/// state doesn't leak across unrelated render subtrees.
#[test]
fn glass_surface_guard_is_raii() {
    {
        let _g = GlassSurfaceGuard::enter();
        assert_eq!(IconStyle::Auto.resolve(), IconStyle::LiquidGlass);
    }
    assert_eq!(IconStyle::Auto.resolve(), IconStyle::Standard);
}

/// Nested guards use a depth counter: inner scope exiting does not end the
/// outer scope. Required because a glass Button variant may be rendered
/// inside a glass panel, and leaving the button must not strip the panel's
/// scope from sibling elements.
#[test]
fn nested_glass_surface_guards_retain_scope() {
    let _outer = GlassSurfaceGuard::enter();
    {
        let _inner = GlassSurfaceGuard::enter();
        assert_eq!(IconStyle::Auto.resolve(), IconStyle::LiquidGlass);
    }
    // Inner dropped; outer still active.
    assert_eq!(IconStyle::Auto.resolve(), IconStyle::LiquidGlass);
}

/// Explicit `IconStyle::Standard` and `IconStyle::LiquidGlass` are
/// caller overrides; the surface scope must not flip them.
#[test]
fn explicit_icon_styles_pass_through_regardless_of_scope() {
    let _g = GlassSurfaceGuard::enter();
    assert_eq!(IconStyle::Standard.resolve(), IconStyle::Standard);
    assert_eq!(IconStyle::LiquidGlass.resolve(), IconStyle::LiquidGlass);
}

/// `Icon::resolved_stroke_width` reads through `resolve()` and therefore
/// through the surface scope — locks the observable stroke-width effect
/// of the fix from icon_gallery's point of view.
#[test]
fn resolved_stroke_width_tracks_surface_scope() {
    let icon = Icon::new(IconName::Check);
    assert!(
        (icon.resolved_stroke_width() - 1.2).abs() < f32::EPSILON,
        "outside a glass scope the stroke width should be 1.2pt"
    );
    let _g = GlassSurfaceGuard::enter();
    let icon = Icon::new(IconName::Check);
    assert!(
        (icon.resolved_stroke_width() - 1.5).abs() < f32::EPSILON,
        "inside a glass scope the stroke width should be 1.5pt"
    );
}

// ─── RTL flip predicate ────────────────────────────────────────────────────

fn rtl_theme() -> TahoeTheme {
    use crate::foundations::layout::LayoutDirection;
    let mut theme = TahoeTheme::dark();
    theme.layout_direction = LayoutDirection::RightToLeft;
    theme
}

#[test]
fn directional_icons_flip_under_rtl_theme() {
    let theme = rtl_theme();
    // All explicitly directional symbols must mirror in RTL.
    for &name in &[
        IconName::ArrowRight,
        IconName::ArrowTriangleRight,
        IconName::ChevronLeft,
        IconName::ChevronRight,
        IconName::Send,
    ] {
        assert!(
            Icon::new(name).would_flip_horizontally(&theme),
            "{name:?} should flip in RTL"
        );
    }
}

#[test]
fn neutral_icons_never_flip() {
    let rtl = rtl_theme();
    let ltr = TahoeTheme::dark();
    for &name in &[
        IconName::ArrowDown,
        IconName::ChevronDown,
        IconName::ChevronUp,
        IconName::Clock,
        IconName::Search,
    ] {
        assert!(
            !Icon::new(name).would_flip_horizontally(&rtl),
            "{name:?} is neutral and must not flip even in RTL"
        );
        assert!(
            !Icon::new(name).would_flip_horizontally(&ltr),
            "{name:?} must never flip in LTR"
        );
    }
}

#[test]
fn ltr_theme_never_flips_any_icon() {
    let theme = TahoeTheme::dark();
    for &name in &[
        IconName::ArrowRight,
        IconName::ChevronRight,
        IconName::ChevronLeft,
        IconName::Send,
    ] {
        assert!(
            !Icon::new(name).would_flip_horizontally(&theme),
            "{name:?} must not flip under LTR even though it's classified Directional"
        );
    }
}

#[test]
fn follow_layout_direction_false_opts_out_of_flip() {
    let theme = rtl_theme();
    let icon = Icon::new(IconName::ChevronRight).follow_layout_direction(false);
    assert!(
        !icon.would_flip_horizontally(&theme),
        "opt-out via follow_layout_direction(false) must suppress the flip"
    );
}
