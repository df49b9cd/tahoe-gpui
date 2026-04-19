//! Example: visual gallery of all icon variants with SVG rendering.

use std::time::Duration;

use tahoe_gpui::foundations::icons::{
    AnimatedIcon, AnimatedProviderIcon, EmbeddedIconAssets, GlassIconTile, GlassTileTint, Icon,
    IconAnimation, IconName,
};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{
    App, Bounds, Div, Entity, FontWeight, Window, WindowBackgroundAppearance, WindowBounds,
    WindowOptions, div, px, size,
};
use gpui_platform::application;

struct IconGallery {
    provider_icons: Vec<(Entity<AnimatedProviderIcon>, &'static str)>,
}

const CORE_UI: &[(IconName, &str)] = &[
    (IconName::ArrowDown, "ArrowDown"),
    (IconName::ArrowRight, "ArrowRight"),
    (IconName::Brain, "Brain"),
    (IconName::Check, "Check"),
    (IconName::ChevronDown, "ChevronDown"),
    (IconName::ChevronLeft, "ChevronLeft"),
    (IconName::ChevronRight, "ChevronRight"),
    (IconName::ChevronUp, "ChevronUp"),
    (IconName::Copy, "Copy"),
    (IconName::Download, "Download"),
    (IconName::Send, "Send"),
    (IconName::Square, "Square"),
    (IconName::X, "X"),
    (IconName::Loader, "Loader"),
    (IconName::Code, "Code"),
    (IconName::File, "File"),
    (IconName::Folder, "Folder"),
    (IconName::FolderOpen, "FolderOpen"),
    (IconName::Terminal, "Terminal"),
    (IconName::Play, "Play"),
    (IconName::Pause, "Pause"),
    (IconName::Mic, "Mic"),
    (IconName::Settings, "Settings"),
];

const PHASE2: &[(IconName, &str)] = &[
    (IconName::Bookmark, "Bookmark"),
    (IconName::Book, "Book"),
    (IconName::Search, "Search"),
    (IconName::Link, "Link"),
    (IconName::Globe, "Globe"),
    (IconName::Sparkle, "Sparkle"),
    (IconName::ListTodo, "ListTodo"),
    (IconName::CircleFilled, "CircleFilled"),
    (IconName::CircleOutline, "CircleOutline"),
    (IconName::AlertTriangle, "AlertTriangle"),
    (IconName::Image, "Image"),
    (IconName::Plus, "Plus"),
    (IconName::Minus, "Minus"),
];

const PHASE3: &[(IconName, &str)] = &[
    (IconName::Bug, "Bug"),
    (IconName::TestTube, "TestTube"),
    (IconName::GitCommit, "GitCommit"),
    (IconName::Package, "Package"),
    (IconName::Database, "Database"),
    (IconName::Key, "Key"),
    (IconName::Bot, "Bot"),
    (IconName::FileCode, "FileCode"),
    (IconName::Trash, "Trash"),
    (IconName::Eye, "Eye"),
    (IconName::EyeOff, "EyeOff"),
    (IconName::ExternalLink, "ExternalLink"),
];

const MESSAGES_WORKFLOW: &[(IconName, &str)] = &[
    (IconName::ChevronsUpDown, "ChevronsUpDown"),
    (IconName::ThumbsUp, "ThumbsUp"),
    (IconName::ThumbsDown, "ThumbsDown"),
    (IconName::RotateCcw, "RotateCcw"),
    (IconName::Share, "Share"),
    (IconName::Pencil, "Pencil"),
    (IconName::Volume2, "Volume2"),
    (IconName::VolumeX, "VolumeX"),
    (IconName::SkipBack, "SkipBack"),
    (IconName::SkipForward, "SkipForward"),
    (IconName::Lock, "Lock"),
    (IconName::Unlock, "Unlock"),
    (IconName::Maximize, "Maximize"),
    (IconName::Paperclip, "Paperclip"),
];

const DEV_TOOLS_IDE: &[(IconName, &str)] = &[
    (IconName::DevTab, "DevTab"),
    (IconName::DevSidebar, "DevSidebar"),
    (IconName::DevSplitView, "DevSplitView"),
    (IconName::DevSearch, "DevSearch"),
    (IconName::DevFindReplace, "DevFindReplace"),
    (IconName::DevMinimap, "DevMinimap"),
    (IconName::DevBreadcrumb, "DevBreadcrumb"),
    (IconName::DevSnippet, "DevSnippet"),
    (IconName::DevPalette, "DevPalette"),
    (IconName::DevExtension, "DevExtension"),
    (IconName::DevKeyboard, "DevKeyboard"),
    (IconName::DevDebug, "DevDebug"),
];

const DEV_TOOLS_AI: &[(IconName, &str)] = &[
    (IconName::Agent, "Agent"),
    (IconName::Prompt, "Prompt"),
    (IconName::Chain, "Chain"),
    (IconName::ToolUse, "ToolUse"),
    (IconName::Memory, "Memory"),
    (IconName::Context, "Context"),
    (IconName::Embedding, "Embedding"),
    (IconName::Rag, "Rag"),
    (IconName::Orchestrator, "Orchestrator"),
    (IconName::Model, "Model"),
    (IconName::Streaming, "Streaming"),
    (IconName::FunctionCall, "FunctionCall"),
    (IconName::Guardrail, "Guardrail"),
    (IconName::Token, "Token"),
    (IconName::FineTune, "FineTune"),
];

const DEV_TOOLS_DEVOPS: &[(IconName, &str)] = &[
    (IconName::Deploy, "Deploy"),
    (IconName::CiCd, "CiCd"),
    (IconName::Container, "Container"),
    (IconName::Pipeline, "Pipeline"),
    (IconName::Monitor, "Monitor"),
    (IconName::Logs, "Logs"),
    (IconName::Environment, "Environment"),
    (IconName::Secret, "Secret"),
    (IconName::Webhook, "Webhook"),
    (IconName::Api, "Api"),
    (IconName::Scale, "Scale"),
    (IconName::Rollback, "Rollback"),
    (IconName::Health, "Health"),
    (IconName::Queue, "Queue"),
    (IconName::Cache, "Cache"),
];

const GIT: &[(IconName, &str)] = &[
    (IconName::GitBranch, "GitBranch"),
    (IconName::GitMerge, "GitMerge"),
    (IconName::GitConflict, "GitConflict"),
    (IconName::GitPull, "GitPull"),
    (IconName::GitPush, "GitPush"),
    (IconName::GitCheckout, "GitCheckout"),
    (IconName::GitStash, "GitStash"),
    (IconName::GitTag, "GitTag"),
    (IconName::GitLog, "GitLog"),
    (IconName::GitRebase, "GitRebase"),
    (IconName::GitCompare, "GitCompare"),
    (IconName::GitInlineDiff, "GitInlineDiff"),
    (IconName::GitStaging, "GitStaging"),
    (IconName::GitPullRequest, "GitPullRequest"),
    (IconName::GitCodeReview, "GitCodeReview"),
    (IconName::GitFork, "GitFork"),
    (IconName::GitClone, "GitClone"),
    (IconName::GitRemote, "GitRemote"),
    (IconName::GitBlame, "GitBlame"),
    (IconName::GitStaged, "GitStaged"),
    (IconName::GitModified, "GitModified"),
    (IconName::GitUntracked, "GitUntracked"),
    (IconName::GitAdded, "GitAdded"),
    (IconName::GitDeleted, "GitDeleted"),
    (IconName::GitIgnored, "GitIgnored"),
    (IconName::GitAhead, "GitAhead"),
    (IconName::GitBehind, "GitBehind"),
    (IconName::GitClean, "GitClean"),
];

const LANGUAGES: &[(IconName, &str)] = &[
    (IconName::LangRust, "Rust"),
    (IconName::LangPython, "Python"),
    (IconName::LangJavaScript, "JavaScript"),
    (IconName::LangTypeScript, "TypeScript"),
    (IconName::LangGo, "Go"),
    (IconName::LangC, "C"),
    (IconName::LangCpp, "C++"),
    (IconName::LangBash, "Bash"),
    (IconName::LangJson, "JSON"),
    (IconName::LangToml, "TOML"),
    (IconName::LangHtml, "HTML"),
    (IconName::LangCss, "CSS"),
];

const PROVIDERS: &[(IconName, &str)] = &[
    (IconName::ProviderClaude, "Claude"),
    (IconName::ProviderGpt, "GPT"),
    (IconName::ProviderGemini, "Gemini"),
    (IconName::ProviderGrok, "Grok"),
    (IconName::ProviderLlama, "Llama"),
    (IconName::ProviderDeepSeek, "DeepSeek"),
    (IconName::ProviderMistral, "Mistral"),
    (IconName::ProviderGemma, "Gemma"),
    (IconName::ProviderPhi, "Phi"),
    (IconName::ProviderQwen, "Qwen"),
    (IconName::ProviderGlm, "GLM"),
    (IconName::ProviderMiniMax, "MiniMax"),
    (IconName::ProviderErnie, "Ernie"),
    (IconName::ProviderCohere, "Cohere"),
    (IconName::ProviderPerplexity, "Perplexity"),
    (IconName::ProviderNova, "Nova"),
    (IconName::ProviderCustom, "Custom"),
];

impl Render for IconGallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<TahoeTheme>();

        div()
            .id("icon-gallery-scroll")
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .p(px(24.0))
            .gap(px(24.0))
            .overflow_y_scroll()
            // Title
            .child(
                div()
                    .text_style(TextStyle::Title1, theme)
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text)
                    .child("AI Elements \u{2014} Icon Gallery"),
            )
            .child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text_muted)
                    .child("189 SVG icons across 12 categories, rendered via EmbeddedIconAssets"),
            )
            // Static icon sections
            .child(icon_section("Core UI", CORE_UI, theme))
            .child(icon_section("Phase 2", PHASE2, theme))
            .child(icon_section("Phase 3", PHASE3, theme))
            .child(icon_section(
                "Messages & Workflow",
                MESSAGES_WORKFLOW,
                theme,
            ))
            .child(icon_section(
                "Dev Tools: IDE & Editor",
                DEV_TOOLS_IDE,
                theme,
            ))
            .child(icon_section("Dev Tools: AI & Agents", DEV_TOOLS_AI, theme))
            .child(icon_section("Dev Tools: DevOps", DEV_TOOLS_DEVOPS, theme))
            .child(icon_section("Git", GIT, theme))
            .child(icon_section("Programming Languages", LANGUAGES, theme))
            .child(icon_section("LLM Providers", PROVIDERS, theme))
            // Animated provider icons (canvas-drawn)
            .child({
                let mut grid = div().flex().flex_wrap().gap(px(12.0));
                for (entity, label) in &self.provider_icons {
                    let tile = div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(4.0))
                        .p(px(10.0))
                        .w(px(88.0))
                        .rounded(theme.radius_md)
                        .bg(theme.surface)
                        .child(div().size(px(32.0)).child(entity.clone()))
                        .child(
                            div()
                                .text_size(px(9.0))
                                .text_color(theme.text_muted)
                                .child((*label).to_string()),
                        );
                    grid = grid.child(tile);
                }
                section("LLM Providers (Canvas Animated)", theme).child(grid)
            })
            // Animated icons section
            .child(animated_section(theme))
            // Liquid Glass section
            .child(glass_section(theme))
    }
}

fn icon_section(title: &str, icons: &[(IconName, &str)], theme: &TahoeTheme) -> Div {
    let mut grid = div().flex().flex_wrap().gap(px(6.0));
    for &(name, label) in icons {
        grid = grid.child(icon_tile(name, label, theme));
    }

    section(title, theme).child(grid)
}

fn icon_tile(name: IconName, label: &str, theme: &TahoeTheme) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .p(px(8.0))
        .w(px(80.0))
        .rounded(theme.radius_md)
        .hover(|s| s.bg(theme.hover))
        .child(Icon::new(name).size(px(24.0)))
        .child(
            div()
                .text_size(px(9.0))
                .text_color(theme.text_muted)
                .text_ellipsis()
                .max_w(px(72.0))
                .child(label.to_string()),
        )
}

fn animated_section(theme: &TahoeTheme) -> Div {
    let ms = |ms: u64| Duration::from_millis(ms);

    section("Animated Icons", theme).child(
        div()
            .flex()
            .flex_wrap()
            .gap(px(12.0))
            .child(anim_tile(
                "spin",
                IconName::Loader,
                "Spin",
                IconAnimation::Spin { duration: ms(1800) },
                theme,
            ))
            .child(anim_tile(
                "pulse",
                IconName::Brain,
                "Pulse",
                IconAnimation::Pulse { duration: ms(2000) },
                theme,
            ))
            .child(anim_tile(
                "shake",
                IconName::AlertTriangle,
                "Shake",
                IconAnimation::Shake { duration: ms(2000) },
                theme,
            ))
            .child(anim_tile(
                "heartbeat",
                IconName::Health,
                "Heartbeat",
                IconAnimation::Heartbeat { duration: ms(1400) },
                theme,
            ))
            .child(anim_tile(
                "twinkle",
                IconName::Sparkle,
                "Twinkle",
                IconAnimation::Twinkle { duration: ms(3000) },
                theme,
            ))
            .child(anim_tile(
                "draw-on",
                IconName::Check,
                "DrawOn",
                IconAnimation::DrawOn,
                theme,
            ))
            .child(anim_tile(
                "fly-out",
                IconName::Send,
                "FlyOut",
                IconAnimation::FlyOut,
                theme,
            ))
            .child(anim_tile(
                "flash",
                IconName::Copy,
                "Flash",
                IconAnimation::Flash,
                theme,
            ))
            .child(anim_tile(
                "drop-in",
                IconName::Download,
                "DropIn",
                IconAnimation::DropIn,
                theme,
            ))
            .child(anim_tile(
                "bounce",
                IconName::ThumbsUp,
                "Bounce",
                IconAnimation::Bounce,
                theme,
            )),
    )
}

fn anim_tile(
    id: &'static str,
    name: IconName,
    label: &str,
    animation: IconAnimation,
    theme: &TahoeTheme,
) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .p(px(10.0))
        .w(px(88.0))
        .rounded(theme.radius_md)
        .bg(theme.surface)
        .child(AnimatedIcon::new(id, name, animation).size(px(24.0)))
        .child(
            div()
                .text_size(px(9.0))
                .text_color(theme.text_muted)
                .child(label.to_string()),
        )
}

fn section(title: &str, theme: &TahoeTheme) -> Div {
    div().flex().flex_col().gap(px(8.0)).child(
        div()
            .text_style(TextStyle::Subheadline, theme)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.text_muted)
            .child(title.to_string()),
    )
}

fn glass_section(theme: &TahoeTheme) -> Div {
    let glass_icons: &[(IconName, &str, Option<GlassTileTint>)] = &[
        // Core UI (no tint)
        (IconName::Check, "Check", None),
        (IconName::Copy, "Copy", None),
        (IconName::Send, "Send", None),
        (IconName::Download, "Download", None),
        (IconName::Search, "Search", None),
        (IconName::Settings, "Settings", None),
        (IconName::Terminal, "Terminal", None),
        (IconName::Sparkle, "Sparkle", None),
        // Git (green tint)
        (IconName::GitBranch, "Branch", Some(GlassTileTint::Green)),
        (IconName::GitMerge, "Merge", Some(GlassTileTint::Green)),
        (IconName::GitPullRequest, "PR", Some(GlassTileTint::Green)),
        (IconName::GitStaged, "Staged", Some(GlassTileTint::Green)),
        // Dev Tools (blue tint)
        (IconName::DevSearch, "Search", Some(GlassTileTint::Blue)),
        (IconName::Api, "API", Some(GlassTileTint::Blue)),
        (IconName::Container, "Container", Some(GlassTileTint::Blue)),
        (IconName::Streaming, "Streaming", Some(GlassTileTint::Blue)),
        // AI (purple tint)
        (IconName::Agent, "Agent", Some(GlassTileTint::Purple)),
        (IconName::Model, "Model", Some(GlassTileTint::Purple)),
        (
            IconName::Orchestrator,
            "Orchestrator",
            Some(GlassTileTint::Purple),
        ),
        (
            IconName::ProviderClaude,
            "Claude",
            Some(GlassTileTint::Purple),
        ),
        // Languages (amber tint)
        (IconName::LangRust, "Rust", Some(GlassTileTint::Amber)),
        (IconName::LangPython, "Python", Some(GlassTileTint::Amber)),
        (
            IconName::LangTypeScript,
            "TypeScript",
            Some(GlassTileTint::Amber),
        ),
        (IconName::LangGo, "Go", Some(GlassTileTint::Amber)),
    ];

    let mut grid = div().flex().flex_wrap().gap(px(10.0));
    for &(name, label, tint) in glass_icons {
        let mut tile = GlassIconTile::new(name).icon_size(px(32.0)).label(label);
        if let Some(t) = tint {
            tile = tile.tint(t);
        }
        grid = grid.child(div().w(px(90.0)).child(tile));
    }

    section("Liquid Glass", theme)
        .child(
            div()
                .text_size(px(10.0))
                .text_color(theme.text_muted)
                .mb(px(8.0))
                .child("Apple Liquid Glass design language \u{2014} bolder strokes, bright pastels, frosted tiles"),
        )
        .child(
            div()
                .rounded(px(20.0))
                .bg(theme.surface)
                .p(px(16.0))
                .child(grid),
        )
}

fn main() {
    application()
        .with_assets(EmbeddedIconAssets)
        .run(|cx: &mut App| {
            cx.set_global(TahoeTheme::dark());

            let bounds = Bounds::centered(None, size(px(1000.), px(900.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: WindowBackgroundAppearance::Blurred,
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|cx| {
                        let provider_icons: Vec<_> = PROVIDERS
                            .iter()
                            .map(|&(name, label)| {
                                let entity =
                                    cx.new(|_| AnimatedProviderIcon::new(name).size(px(32.0)));
                                (entity, label)
                            })
                            .collect();
                        IconGallery { provider_icons }
                    })
                },
            )
            .unwrap();
            cx.activate(true);
        });
}
