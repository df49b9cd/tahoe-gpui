//! Icon name enumeration with Unicode fallback symbols and SVG strategies.

pub(crate) use super::assets::{IconColorRole, RenderStrategy};

/// Per-symbol layout behaviour for RTL rendering.
///
/// Finding 31 in the Zed cross-reference audit tracks this: HIG
/// §Right to Left distinguishes symbols that must be flipped
/// geometrically (`ChevronLeft`, arrows, progress indicators) from
/// symbols that ship with a localised Arabic / Hebrew variant
/// (`signature`, some rich-text glyphs) from symbols that stay upright
/// regardless of reading direction (`Clock`, `Camera`, numerals).
///
/// [`crate::foundations::right_to_left::icon_direction`] folds the
/// current `LayoutDirection` into this classification so callers get a
/// single answer (flip / swap-variant / leave alone) at render time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IconLayoutBehavior {
    /// Direction-free glyph — render identically in LTR and RTL.
    /// Default so any icon not enumerated below gets the safe behaviour.
    #[default]
    Neutral,
    /// Directional glyph — mirror horizontally in RTL layouts.
    /// Applies to arrows, chevrons, progress bars, send arrows, etc.
    Directional,
    /// Culture-specific glyph — swap in a localised Arabic / Hebrew
    /// asset when available. Falls back to the neutral glyph until
    /// the localised variant ships.
    Localized,
}

/// Known icon names for built-in icons.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum IconName {
    // ── Core UI ──────────────────────────────────────────────────────────────
    ArrowDown,
    ArrowRight,
    /// Filled right-pointing triangle — `arrowtriangle.right.fill` in SF
    /// Symbols 7. Per HIG Disclosure Controls, the disclosure indicator
    /// for a collapsed section is a filled triangle pointing in the reading
    /// direction (right in LTR). Distinct from [`IconName::ChevronRight`],
    /// which is a navigation affordance, not a disclosure indicator.
    ArrowTriangleRight,
    /// Filled down-pointing triangle — `arrowtriangle.down.fill` in SF
    /// Symbols 7. Disclosure indicator for an expanded section.
    ArrowTriangleDown,
    Brain,
    Check,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    ChevronUp,
    Copy,
    Download,
    Send,
    Square,
    X,
    Loader,
    /// macOS HIG 12-tick radial progress spinner (`progress.indicator` in
    /// SF Symbols 7). Distinct from [`IconName::Loader`] (Lucide single-arc
    /// clockwise-arrow): this is the native 12-tick stepped spinner used
    /// by `NSProgressIndicator.style = .spinning` and by `ActivityIndicator`
    /// on macOS. Opacity fades around the circle; rotate the whole symbol
    /// via [`super::AnimatedIcon`] + [`super::IconAnimation::Spin`] for the native look.
    ProgressSpinner,
    Code,
    File,
    Folder,
    FolderOpen,
    Terminal,
    Play,
    Pause,
    Mic,
    /// Muted microphone — `mic.slash` in SF Symbols 7. Used by voice
    /// components to distinguish "microphone unavailable" or
    /// "permission denied" states from the active mic glyph.
    MicOff,
    Phone,
    Video,
    Settings,
    /// Clock face used by time pickers and time-related affordances
    /// (`clock` in SF Symbols). Fallback is a non-emoji circle-quadrant
    /// glyph.
    Clock,
    // ── Phase 2 ─────────────────────────────────────────────────────────────
    Bookmark,
    Book,
    Search,
    Link,
    Globe,
    Sparkle,
    /// Filled star — `star.fill` in SF Symbols 7. Used for full-rating stars.
    StarFill,
    /// Outlined (empty) star — `star` in SF Symbols 7. Used for empty-rating stars.
    Star,
    /// Half-filled star (leading half filled) — `star.leadinghalf.filled` in
    /// SF Symbols 7. Used for half-rating stars; renders as a single glyph
    /// with the left half filled and the right half outlined.
    StarLeadingHalfFilled,
    ListTodo,
    CircleFilled,
    CircleOutline,
    AlertTriangle,
    Image,
    Plus,
    Minus,
    // ── Phase 3 ─────────────────────────────────────────────────────────────
    Bug,
    TestTube,
    GitCommit,
    Package,
    Database,
    Key,
    Bot,
    FileCode,
    Trash,
    Eye,
    EyeOff,
    ExternalLink,
    // ── Plan icons ──────────────────────────────────────────────────────────
    ChevronsUpDown,
    // ── Message actions ─────────────────────────────────────────────────────
    ThumbsUp,
    ThumbsDown,
    RotateCcw,
    /// Clockwise U-turn arrow — `arrow.uturn.forward` in SF Symbols 7.
    /// Used as the canonical macOS Redo icon, paired with [`IconName::RotateCcw`]
    /// (Undo) in content-editing toolbars per HIG Undo and redo.
    RotateCw,
    Share,
    Pencil,
    // ── Audio ───────────────────────────────────────────────────────────────
    Volume2,
    VolumeX,
    SkipBack,
    SkipForward,
    // ── Workflow ─────────────────────────────────────────────────────────────
    Lock,
    Unlock,
    Maximize,
    Paperclip,
    /// Filled circle-X used as the HIG-specified clear button for search
    /// fields (`xmark.circle.fill` in SF Symbols). Unlike [`IconName::X`]
    /// which is a bare glyph, this is the circled variant that Apple uses
    /// in the trailing clear affordance of every native search field.
    XmarkCircleFill,
    /// Horizontal three-dot ellipsis used for toolbar overflow / "more
    /// actions" buttons (`ellipsis` in SF Symbols). Collapses excess items
    /// into a trailing pulldown when a toolbar is too narrow.
    Ellipsis,
    /// Left-hand sidebar-toggle icon used as the canonical first toolbar
    /// item on macOS unified toolbars (`sidebar.left` in SF Symbols).
    SidebarLeft,

    // ── Dev Tools: IDE & Editor ─────────────────────────────────────────────
    DevTab,
    DevSidebar,
    DevSplitView,
    DevSearch,
    DevFindReplace,
    DevMinimap,
    DevBreadcrumb,
    DevSnippet,
    DevPalette,
    DevExtension,
    DevKeyboard,
    DevDebug,

    // ── Dev Tools: AI & Agents ──────────────────────────────────────────────
    Agent,
    Prompt,
    Chain,
    ToolUse,
    Memory,
    Context,
    Embedding,
    Rag,
    Orchestrator,
    Model,
    Streaming,
    FunctionCall,
    Guardrail,
    Token,
    FineTune,

    // ── Dev Tools: DevOps ───────────────────────────────────────────────────
    Deploy,
    CiCd,
    Container,
    Pipeline,
    Monitor,
    Logs,
    Environment,
    Secret,
    Webhook,
    Api,
    Scale,
    Rollback,
    Health,
    Queue,
    Cache,

    // ── Git ─────────────────────────────────────────────────────────────────
    GitBranch,
    GitMerge,
    GitConflict,
    GitPull,
    GitPush,
    GitCheckout,
    GitStash,
    GitTag,
    GitLog,
    GitRebase,
    GitCompare,
    GitInlineDiff,
    GitStaging,
    GitPullRequest,
    GitCodeReview,
    GitFork,
    GitClone,
    GitRemote,
    GitBlame,
    GitStaged,
    GitModified,
    GitUntracked,
    GitAdded,
    GitDeleted,
    GitIgnored,
    GitAhead,
    GitBehind,
    GitClean,

    // ── Programming Languages ───────────────────────────────────────────────
    LangRust,
    LangPython,
    LangJavaScript,
    LangTypeScript,
    LangGo,
    LangC,
    LangCpp,
    LangBash,
    LangJson,
    LangToml,
    LangHtml,
    LangCss,

    // ── LLM Providers ───────────────────────────────────────────────────────
    ProviderClaude,
    ProviderGpt,
    ProviderGemini,
    ProviderGrok,
    ProviderLlama,
    ProviderDeepSeek,
    ProviderMistral,
    ProviderGemma,
    ProviderPhi,
    ProviderQwen,
    ProviderGlm,
    ProviderMiniMax,
    ProviderErnie,
    ProviderCohere,
    ProviderPerplexity,
    ProviderNova,
    ProviderCustom,
}

impl IconName {
    /// Returns a non-emoji Unicode symbol for this icon (visible
    /// placeholder/fallback shown when [`super::EmbeddedIconAssets`] is not
    /// registered).
    ///
    /// Fallbacks are intentionally restricted to geometric / arrow /
    /// punctuation codepoints — never emoji presentation characters. HIG
    /// prohibits emoji in icons that appear in system-controlled surfaces
    /// (toolbars, menus, context menus) because emoji glyphs render at a
    /// different baseline and resist `text_color` tinting. Geometric
    /// fallbacks participate in `text_color` tinting and align with
    /// surrounding text.
    #[allow(unreachable_patterns)] // _ => arm for #[non_exhaustive] forward compat
    pub fn symbol(&self) -> &'static str {
        match self {
            // Core UI
            IconName::ArrowDown => "\u{2193}",
            IconName::ArrowRight => "\u{2192}",
            // BLACK RIGHT-POINTING TRIANGLE / DOWN-POINTING TRIANGLE — the
            // HIG disclosure glyphs. Unicode already renders these as filled
            // triangles so no emoji plane concerns.
            IconName::ArrowTriangleRight => "\u{25B6}",
            IconName::ArrowTriangleDown => "\u{25BC}",
            IconName::Brain => "\u{2217}", // asterisk operator — non-emoji placeholder
            IconName::Check => "\u{2713}",
            IconName::ChevronDown => "\u{25BE}",
            IconName::ChevronLeft => "\u{25C2}",
            IconName::ChevronRight => "\u{25B8}",
            IconName::ChevronUp => "\u{25B4}",
            IconName::Copy => "\u{2398}",
            IconName::Download => "\u{21E9}",
            IconName::Send => "\u{27A4}",
            IconName::Square => "\u{25A0}",
            IconName::X => "\u{2715}",
            IconName::Loader => "\u{21BB}",
            // ☸ U+2638 WHEEL OF DHARMA — 12-spoke wheel, closest BMP glyph
            // for the 12-tick HIG progress spinner fallback.
            IconName::ProgressSpinner => "\u{2638}",
            IconName::Code => "\u{2039}\u{203A}",
            IconName::File => "\u{25A1}", // white square — generic file
            IconName::Folder => "\u{25B7}", // white right-pointing triangle (folder)
            IconName::FolderOpen => "\u{25B9}", // small triangle
            IconName::Terminal => "\u{276F}",
            IconName::Play => "\u{25B6}",
            IconName::Pause => "\u{23F8}",
            IconName::Mic => "\u{25CF}", // black circle — mic indicator
            // ⊘ U+2298 CIRCLED DIVISION SLASH — non-emoji "mic disabled" stand-in.
            IconName::MicOff => "\u{2298}",
            IconName::Phone => "\u{260E}", // black telephone (non-emoji BMP)
            IconName::Video => "\u{25B7}", // triangle
            IconName::Settings => "\u{2699}",
            // U+25F4 WHITE CIRCLE WITH UPPER RIGHT QUADRANT — non-emoji,
            // BMP, reads as a clock-face quadrant.
            IconName::Clock => "\u{25F4}",
            // Phase 2
            IconName::Bookmark => "\u{2691}", // flag
            IconName::Book => "B", // ASCII book stand-in — non-emoji, renders with text color
            IconName::Search => "\u{26B2}", // neuter — circle-on-line glyph (non-emoji)
            IconName::Link => "\u{221E}", // infinity — non-emoji link stand-in
            IconName::Globe => "\u{25EF}", // large circle
            IconName::Sparkle => "\u{2728}",
            // ★ U+2605 BLACK STAR (BMP, non-emoji) — participates in text_color tinting.
            IconName::StarFill => "\u{2605}",
            // ☆ U+2606 WHITE STAR (BMP, non-emoji).
            IconName::Star => "\u{2606}",
            // ◐ U+25D0 CIRCLE WITH LEFT HALF BLACK — geometric half-fill stand-in.
            IconName::StarLeadingHalfFilled => "\u{25D0}",
            IconName::ListTodo => "\u{2611}",
            IconName::CircleFilled => "\u{25CF}",
            IconName::CircleOutline => "\u{25CB}",
            IconName::AlertTriangle => "\u{26A0}",
            IconName::Image => "\u{25A2}", // square with rounded corners
            IconName::Plus => "\u{002B}",
            IconName::Minus => "\u{2212}",
            // Phase 3
            IconName::Bug => "\u{25CB}",      // non-emoji bug stand-in
            IconName::TestTube => "\u{25AE}", // black vertical rectangle
            IconName::GitCommit => "\u{25C9}",
            IconName::Package => "\u{25A3}", // white square with black small square
            IconName::Database => "\u{25CB}", // circle stack fallback
            IconName::Key => "\u{26B7}",     // chiron (non-emoji, vaguely key-like)
            IconName::Bot => "\u{2609}",     // sun (non-emoji agent stand-in)
            IconName::FileCode => "\u{2263}", // strictly equivalent to
            IconName::Trash => "\u{2327}",   // x in a rectangle
            IconName::Eye => "\u{25CE}",     // bull's eye
            IconName::EyeOff => "\u{2205}",  // empty set
            IconName::ExternalLink => "\u{2197}",
            // Plan
            IconName::ChevronsUpDown => "\u{21C5}",
            // Message actions
            IconName::ThumbsUp => "\u{2191}", // up arrow (non-emoji)
            IconName::ThumbsDown => "\u{2193}", // down arrow
            IconName::RotateCcw => "\u{21BA}",
            IconName::RotateCw => "\u{21BB}",
            IconName::Share => "\u{2B06}",
            IconName::Pencil => "\u{270F}",
            // Audio
            IconName::Volume2 => "\u{2669}", // quarter note
            IconName::VolumeX => "\u{266D}", // flat (music flat sign) — "muted"
            IconName::SkipBack => "\u{23EE}",
            IconName::SkipForward => "\u{23ED}",
            // Workflow
            IconName::Lock => "\u{2302}",   // house — generic container
            IconName::Unlock => "\u{2300}", // diameter sign
            IconName::Maximize => "\u{2922}",
            IconName::Paperclip => "\u{27BC}",
            IconName::XmarkCircleFill => "\u{24E7}", // circled latin letter x — filled-circle stand-in
            IconName::Ellipsis => "\u{22EF}",        // midline horizontal ellipsis
            IconName::SidebarLeft => "\u{25E7}", // square with left half black — sidebar stand-in

            // Dev Tools
            IconName::DevTab => "\u{2B1C}",
            IconName::DevSidebar => "\u{25E7}",
            IconName::DevSplitView => "\u{25EB}",
            IconName::DevSearch => "\u{26B2}", // neuter — non-emoji search stand-in
            IconName::DevFindReplace => "\u{21C4}",
            IconName::DevMinimap => "\u{25A3}",
            IconName::DevBreadcrumb => "\u{2192}",
            IconName::DevSnippet => "\u{2702}",
            IconName::DevPalette => "\u{276F}",
            IconName::DevExtension => "\u{229E}", // squared plus — extension stand-in
            IconName::DevKeyboard => "\u{2328}",
            IconName::DevDebug => "\u{25CB}", // circle — non-emoji bug

            // AI & Agents
            IconName::Agent => "\u{2609}",   // sun (non-emoji agent)
            IconName::Prompt => "\u{201C}",  // left double quote
            IconName::Chain => "\u{221E}",   // infinity — non-emoji link
            IconName::ToolUse => "\u{2692}", // hammer and pick (BMP, non-emoji)
            IconName::Memory => "\u{25A3}",  // square with smaller square (db-like)
            IconName::Context => "\u{2630}", // trigram (line set)
            IconName::Embedding => "\u{2022}",
            IconName::Rag => "\u{26B2}", // neuter — non-emoji search stand-in
            IconName::Orchestrator => "\u{2609}",
            IconName::Model => "\u{2B21}",
            IconName::Streaming => "\u{223F}",
            IconName::FunctionCall => "\u{2192}",
            IconName::Guardrail => "\u{25F3}", // white square with rounded corners
            IconName::Token => "\u{25A0}",
            IconName::FineTune => "\u{2699}",

            // DevOps
            IconName::Deploy => "\u{21E9}",
            IconName::CiCd => "\u{21BB}",
            IconName::Container => "\u{25A3}", // square with smaller square
            IconName::Pipeline => "\u{2192}",
            IconName::Monitor => "\u{25AD}", // white horizontal rectangle (screen)
            IconName::Logs => "\u{2630}",    // trigram (lines)
            IconName::Environment => "\u{25B3}",
            IconName::Secret => "\u{26B7}", // chiron — generic key-like
            IconName::Webhook => "\u{21AA}",
            IconName::Api => "\u{2039}\u{203A}",
            IconName::Scale => "\u{2197}",
            IconName::Rollback => "\u{21BA}",
            IconName::Health => "\u{2764}",
            IconName::Queue => "\u{25A0}",
            IconName::Cache => "\u{26A1}",

            // Git
            IconName::GitBranch => "\u{2387}",
            IconName::GitMerge => "\u{2934}",
            IconName::GitConflict => "\u{26A1}",
            IconName::GitPull => "\u{21E9}",
            IconName::GitPush => "\u{21E7}",
            IconName::GitCheckout => "\u{21B3}",
            IconName::GitStash => "\u{25A3}",
            IconName::GitTag => "\u{2691}", // flag (BMP, non-emoji)
            IconName::GitLog => "\u{2630}",
            IconName::GitRebase => "\u{21C5}",
            IconName::GitCompare => "\u{2194}",
            IconName::GitInlineDiff => "\u{2261}",
            IconName::GitStaging => "\u{21E7}",
            IconName::GitPullRequest => "\u{26B2}", // non-emoji search stand-in
            IconName::GitCodeReview => "\u{26B2}",
            IconName::GitFork => "\u{2442}",
            IconName::GitClone => "\u{2398}",
            IconName::GitRemote => "\u{2601}",
            IconName::GitBlame => "\u{25CE}", // bull's eye (non-emoji)
            IconName::GitStaged => "\u{2713}",
            IconName::GitModified => "\u{25CF}",
            IconName::GitUntracked => "\u{003F}",
            IconName::GitAdded => "\u{002B}",
            IconName::GitDeleted => "\u{2212}",
            IconName::GitIgnored => "\u{20E0}",
            IconName::GitAhead => "\u{25B2}",
            IconName::GitBehind => "\u{25BC}",
            IconName::GitClean => "\u{25CE}",

            // Programming Languages
            IconName::LangRust => "R",
            IconName::LangPython => "Py",
            IconName::LangJavaScript => "JS",
            IconName::LangTypeScript => "TS",
            IconName::LangGo => "Go",
            IconName::LangC => "C",
            IconName::LangCpp => "C++",
            IconName::LangBash => "$",
            IconName::LangJson => "{ }",
            IconName::LangToml => "\u{2699}",
            IconName::LangHtml => "\u{2039}\u{203A}",
            IconName::LangCss => "{ }",

            // LLM Providers
            IconName::ProviderClaude => "\u{2726}",
            IconName::ProviderGpt => "\u{2B21}",
            IconName::ProviderGemini => "\u{2727}",
            IconName::ProviderGrok => "\u{2716}",
            IconName::ProviderLlama => "L",
            IconName::ProviderDeepSeek => "D",
            IconName::ProviderMistral => "\u{2501}",
            IconName::ProviderGemma => "\u{2B22}",
            IconName::ProviderPhi => "\u{03A6}",
            IconName::ProviderQwen => "Q",
            IconName::ProviderGlm => "\u{25B3}",
            IconName::ProviderMiniMax => "\u{21C5}",
            IconName::ProviderErnie => "\u{223F}",
            IconName::ProviderCohere => "\u{25B3}",
            IconName::ProviderPerplexity => "\u{003F}",
            IconName::ProviderNova => "\u{2B50}",
            IconName::ProviderCustom => "\u{2699}",
            _ => "\u{25CB}", // generic fallback for future variants
        }
    }

    /// Returns the canonical SF Symbols 7 identifier for this icon, if one
    /// exists in Apple's library.
    ///
    /// UI actions (cut/copy/paste, navigation chevrons, play/pause, etc.)
    /// map to a real SF Symbol identifier that can be used with
    /// `NSImage(systemSymbolName:)` on macOS or exported from SF Symbols.app.
    /// Provider-specific icons (Claude, GPT, Gemini, …), version-control
    /// iconography (Git*), and programming language marks (Lang*) do not
    /// have SF Symbol equivalents and therefore return `None`.
    ///
    /// The name is dot-notation per Apple's convention — callers pass it
    /// straight to the host's system-symbol API.
    #[allow(unreachable_patterns)]
    pub fn system_name(&self) -> Option<&'static str> {
        match self {
            // Core UI
            IconName::ArrowDown => Some("arrow.down"),
            IconName::ArrowRight => Some("arrow.right"),
            IconName::ArrowTriangleRight => Some("arrowtriangle.right.fill"),
            IconName::ArrowTriangleDown => Some("arrowtriangle.down.fill"),
            IconName::Brain => Some("brain"),
            IconName::Check => Some("checkmark"),
            IconName::ChevronDown => Some("chevron.down"),
            IconName::ChevronLeft => Some("chevron.left"),
            IconName::ChevronRight => Some("chevron.right"),
            IconName::ChevronUp => Some("chevron.up"),
            IconName::Copy => Some("document.on.document"),
            IconName::Download => Some("arrow.down.circle"),
            IconName::Send => Some("paperplane"),
            IconName::Square => Some("square"),
            IconName::X => Some("xmark"),
            IconName::Loader => Some("arrow.clockwise"),
            IconName::ProgressSpinner => Some("progress.indicator"),
            IconName::Code => Some("chevron.left.forwardslash.chevron.right"),
            IconName::File => Some("document"),
            IconName::Folder => Some("folder"),
            IconName::FolderOpen => Some("folder.fill"),
            IconName::Terminal => Some("apple.terminal"),
            IconName::Play => Some("play.fill"),
            IconName::Pause => Some("pause.fill"),
            IconName::Mic => Some("microphone"),
            IconName::MicOff => Some("microphone.slash"),
            IconName::Phone => Some("phone"),
            IconName::Video => Some("film"),
            IconName::Settings => Some("gear"),
            // Phase 2
            IconName::Bookmark => Some("bookmark"),
            IconName::Book => Some("book"),
            IconName::Search => Some("magnifyingglass"),
            IconName::Link => Some("link"),
            IconName::Globe => Some("globe"),
            IconName::Sparkle => Some("sparkles"),
            IconName::StarFill => Some("star.fill"),
            IconName::Star => Some("star"),
            IconName::StarLeadingHalfFilled => Some("star.leadinghalf.filled"),
            IconName::ListTodo => Some("checklist"),
            IconName::CircleFilled => Some("circle.fill"),
            IconName::CircleOutline => Some("circle"),
            IconName::AlertTriangle => Some("exclamationmark.triangle"),
            IconName::Image => Some("photo"),
            IconName::Plus => Some("plus"),
            IconName::Minus => Some("minus"),
            // Phase 3
            IconName::Bug => Some("ant"),
            IconName::TestTube => Some("testtube.2"),
            IconName::Package => Some("shippingbox"),
            IconName::Database => Some("cylinder"),
            IconName::Key => Some("key"),
            IconName::Trash => Some("trash"),
            IconName::Eye => Some("eye"),
            IconName::EyeOff => Some("eye.slash"),
            IconName::ExternalLink => Some("arrow.up.right.square"),
            // Plan
            IconName::ChevronsUpDown => Some("chevron.up.chevron.down"),
            // Message actions
            IconName::ThumbsUp => Some("hand.thumbsup"),
            IconName::ThumbsDown => Some("hand.thumbsdown"),
            IconName::RotateCcw => Some("arrow.uturn.backward"),
            IconName::RotateCw => Some("arrow.uturn.forward"),
            IconName::Share => Some("square.and.arrow.up"),
            IconName::Pencil => Some("pencil"),
            // Audio
            IconName::Volume2 => Some("speaker.wave.2"),
            IconName::VolumeX => Some("speaker.slash"),
            IconName::SkipBack => Some("backward.end"),
            IconName::SkipForward => Some("forward.end"),
            // Workflow
            IconName::Lock => Some("lock"),
            IconName::Unlock => Some("lock.open"),
            IconName::Maximize => Some("arrow.up.left.and.arrow.down.right"),
            IconName::Paperclip => Some("paperclip"),
            // No SF Symbol equivalent — dev-tools/AI-agents/git/lang/provider
            // iconography is intentionally custom and stays custom to keep
            // domain-specific marks consistent (see HIG Icons & issue #139).
            _ => None,
        }
    }

    /// Internal: the embedded `icons/sf/<name>.svg` asset path for this
    /// icon if the mapping in [`Self::system_name`] covers it.
    ///
    /// Must stay in lockstep with `system_name()` — the invariant that
    /// every `system_name()` entry has a corresponding extracted SVG in
    /// `assets/icons/sf/` and a matching `ICON_ENTRIES` registration is
    /// exercised by the tests.
    #[allow(unreachable_patterns)]
    pub(crate) fn sf_asset_path(&self) -> Option<&'static str> {
        match self {
            IconName::ArrowDown => Some("icons/sf/arrow.down.svg"),
            IconName::ArrowRight => Some("icons/sf/arrow.right.svg"),
            IconName::ArrowTriangleRight => Some("icons/sf/arrowtriangle.right.fill.svg"),
            IconName::ArrowTriangleDown => Some("icons/sf/arrowtriangle.down.fill.svg"),
            IconName::Brain => Some("icons/sf/brain.svg"),
            IconName::Check => Some("icons/sf/checkmark.svg"),
            IconName::ChevronDown => Some("icons/sf/chevron.down.svg"),
            IconName::ChevronLeft => Some("icons/sf/chevron.left.svg"),
            IconName::ChevronRight => Some("icons/sf/chevron.right.svg"),
            IconName::ChevronUp => Some("icons/sf/chevron.up.svg"),
            IconName::Copy => Some("icons/sf/document.on.document.svg"),
            IconName::Download => Some("icons/sf/arrow.down.circle.svg"),
            IconName::Send => Some("icons/sf/paperplane.svg"),
            IconName::Square => Some("icons/sf/square.svg"),
            IconName::X => Some("icons/sf/xmark.svg"),
            IconName::Loader => Some("icons/sf/arrow.clockwise.svg"),
            IconName::ProgressSpinner => Some("icons/sf/progress.indicator.svg"),
            IconName::Code => Some("icons/sf/chevron.left.forwardslash.chevron.right.svg"),
            IconName::File => Some("icons/sf/document.svg"),
            IconName::Folder => Some("icons/sf/folder.svg"),
            IconName::FolderOpen => Some("icons/sf/folder.fill.svg"),
            IconName::Terminal => Some("icons/sf/apple.terminal.svg"),
            IconName::Play => Some("icons/sf/play.fill.svg"),
            IconName::Pause => Some("icons/sf/pause.fill.svg"),
            IconName::Mic => Some("icons/sf/microphone.svg"),
            IconName::MicOff => Some("icons/sf/microphone.slash.svg"),
            IconName::Phone => Some("icons/sf/phone.svg"),
            IconName::Video => Some("icons/sf/film.svg"),
            IconName::Settings => Some("icons/sf/gear.svg"),
            IconName::Bookmark => Some("icons/sf/bookmark.svg"),
            IconName::Book => Some("icons/sf/book.svg"),
            IconName::Search => Some("icons/sf/magnifyingglass.svg"),
            IconName::Link => Some("icons/sf/link.svg"),
            IconName::Globe => Some("icons/sf/globe.svg"),
            IconName::Sparkle => Some("icons/sf/sparkles.svg"),
            IconName::StarFill => Some("icons/sf/star.fill.svg"),
            IconName::Star => Some("icons/sf/star.svg"),
            IconName::StarLeadingHalfFilled => Some("icons/sf/star.leadinghalf.filled.svg"),
            IconName::ListTodo => Some("icons/sf/checklist.svg"),
            IconName::CircleFilled => Some("icons/sf/circle.fill.svg"),
            IconName::CircleOutline => Some("icons/sf/circle.svg"),
            IconName::AlertTriangle => Some("icons/sf/exclamationmark.triangle.svg"),
            IconName::Image => Some("icons/sf/photo.svg"),
            IconName::Plus => Some("icons/sf/plus.svg"),
            IconName::Minus => Some("icons/sf/minus.svg"),
            IconName::Bug => Some("icons/sf/ant.svg"),
            IconName::TestTube => Some("icons/sf/testtube.2.svg"),
            IconName::Package => Some("icons/sf/shippingbox.svg"),
            IconName::Database => Some("icons/sf/cylinder.svg"),
            IconName::Key => Some("icons/sf/key.svg"),
            IconName::Trash => Some("icons/sf/trash.svg"),
            IconName::Eye => Some("icons/sf/eye.svg"),
            IconName::EyeOff => Some("icons/sf/eye.slash.svg"),
            IconName::ExternalLink => Some("icons/sf/arrow.up.right.square.svg"),
            IconName::ChevronsUpDown => Some("icons/sf/chevron.up.chevron.down.svg"),
            IconName::ThumbsUp => Some("icons/sf/hand.thumbsup.svg"),
            IconName::ThumbsDown => Some("icons/sf/hand.thumbsdown.svg"),
            IconName::RotateCcw => Some("icons/sf/arrow.uturn.backward.svg"),
            IconName::RotateCw => Some("icons/sf/arrow.uturn.forward.svg"),
            IconName::Share => Some("icons/sf/square.and.arrow.up.svg"),
            IconName::Pencil => Some("icons/sf/pencil.svg"),
            IconName::Volume2 => Some("icons/sf/speaker.wave.2.svg"),
            IconName::VolumeX => Some("icons/sf/speaker.slash.svg"),
            IconName::SkipBack => Some("icons/sf/backward.end.svg"),
            IconName::SkipForward => Some("icons/sf/forward.end.svg"),
            IconName::Lock => Some("icons/sf/lock.svg"),
            IconName::Unlock => Some("icons/sf/lock.open.svg"),
            IconName::Maximize => Some("icons/sf/arrow.up.left.and.arrow.down.right.svg"),
            IconName::Paperclip => Some("icons/sf/paperclip.svg"),
            _ => None,
        }
    }

    /// Returns the SVG rendering strategy for this icon, if available.
    ///
    /// Returns `None` for icons whose SVG assets haven't been added yet,
    /// causing [`super::Icon`] to fall back to Unicode symbol rendering.
    ///
    /// UI action icons (chevrons, trash, gear, …) now resolve to canonical
    /// SF Symbols 7 glyphs embedded at `icons/sf/<name>.svg`. Provider, git,
    /// dev-tools, and language icons keep the crate's custom Lucide-derived
    /// SVGs because Apple's library contains no equivalents. [`Self::system_name`]
    /// mirrors this split.
    ///
    /// **Sync points** -- when adding a new `IconName` variant, update all of:
    /// 1. `render_strategy()` match arms (standard SVG paths)
    /// 2. `render_strategy_glass()` match arms (glass SVG paths)
    /// 3. `ICON_ENTRIES` in `assets.rs` (embedded asset registration)
    /// 4. `ALL_VARIANTS` in the test module (count guard will fail if missed)
    #[allow(unreachable_patterns)] // _ => arm for #[non_exhaustive] forward compat
    pub(crate) fn render_strategy(&self) -> Option<RenderStrategy> {
        // Route every icon with a canonical SF Symbol identifier to the
        // extracted SF Symbols 7 asset. Falls through to the Lucide path
        // for domain-specific icons (providers, git, dev-tools, langs)
        // that have no SF-Symbol equivalent.
        if let Some(path) = self.sf_asset_path() {
            return Some(RenderStrategy::Monochrome(path));
        }
        match self {
            IconName::ArrowDown => Some(RenderStrategy::Monochrome("icons/ai-sdk/arrow-down.svg")),
            IconName::ArrowRight => {
                Some(RenderStrategy::Monochrome("icons/ai-sdk/arrow-right.svg"))
            }
            IconName::Brain => Some(RenderStrategy::Monochrome("icons/ai-sdk/brain.svg")),
            IconName::Check => Some(RenderStrategy::Monochrome("icons/ai-sdk/check.svg")),
            IconName::ChevronDown => {
                Some(RenderStrategy::Monochrome("icons/ai-sdk/chevron-down.svg"))
            }
            IconName::ChevronLeft => {
                Some(RenderStrategy::Monochrome("icons/ai-sdk/chevron-left.svg"))
            }
            IconName::ChevronRight => {
                Some(RenderStrategy::Monochrome("icons/ai-sdk/chevron-right.svg"))
            }
            IconName::ChevronUp => Some(RenderStrategy::Monochrome("icons/ai-sdk/chevron-up.svg")),
            IconName::Copy => Some(RenderStrategy::Monochrome("icons/ai-sdk/copy.svg")),
            IconName::Download => Some(RenderStrategy::Monochrome("icons/ai-sdk/download.svg")),
            IconName::Send => Some(RenderStrategy::Monochrome("icons/ai-sdk/send.svg")),
            IconName::Square => Some(RenderStrategy::Monochrome("icons/ai-sdk/square.svg")),
            IconName::X => Some(RenderStrategy::Monochrome("icons/ai-sdk/x.svg")),
            IconName::Loader => Some(RenderStrategy::Monochrome("icons/ai-sdk/loader.svg")),
            IconName::Code => Some(RenderStrategy::Monochrome("icons/ai-sdk/code.svg")),
            IconName::File => Some(RenderStrategy::Monochrome("icons/ai-sdk/file.svg")),
            IconName::Folder => Some(RenderStrategy::Monochrome("icons/ai-sdk/folder.svg")),
            IconName::FolderOpen => {
                Some(RenderStrategy::Monochrome("icons/ai-sdk/folder-open.svg"))
            }
            IconName::Terminal => Some(RenderStrategy::Monochrome("icons/ai-sdk/terminal.svg")),
            IconName::Play => Some(RenderStrategy::Monochrome("icons/ai-sdk/play.svg")),
            IconName::Pause => Some(RenderStrategy::Monochrome("icons/ai-sdk/pause.svg")),
            IconName::Mic => Some(RenderStrategy::Monochrome("icons/ai-sdk/mic.svg")),
            IconName::MicOff => Some(RenderStrategy::Monochrome("icons/ai-sdk/mic-off.svg")),
            IconName::Phone => Some(RenderStrategy::Monochrome("icons/ai-sdk/phone.svg")),
            IconName::Video => Some(RenderStrategy::Monochrome("icons/ai-sdk/video.svg")),
            IconName::Settings => Some(RenderStrategy::Monochrome("icons/ai-sdk/settings.svg")),
            IconName::Bookmark => Some(RenderStrategy::Monochrome("icons/ai-sdk/bookmark.svg")),
            IconName::Book => Some(RenderStrategy::Monochrome("icons/ai-sdk/book.svg")),
            IconName::Search => Some(RenderStrategy::Monochrome("icons/ai-sdk/search.svg")),
            IconName::Link => Some(RenderStrategy::Monochrome("icons/ai-sdk/link.svg")),
            IconName::Globe => Some(RenderStrategy::Monochrome("icons/ai-sdk/globe.svg")),
            IconName::Sparkle => Some(RenderStrategy::Monochrome("icons/ai-sdk/sparkle.svg")),
            IconName::ListTodo => Some(RenderStrategy::Monochrome("icons/ai-sdk/list-todo.svg")),
            IconName::CircleFilled => {
                Some(RenderStrategy::Monochrome("icons/ai-sdk/circle-filled.svg"))
            }
            IconName::CircleOutline => Some(RenderStrategy::Monochrome(
                "icons/ai-sdk/circle-outline.svg",
            )),
            IconName::AlertTriangle => Some(RenderStrategy::Monochrome(
                "icons/ai-sdk/alert-triangle.svg",
            )),
            IconName::Image => Some(RenderStrategy::Monochrome("icons/ai-sdk/image.svg")),
            IconName::Plus => Some(RenderStrategy::Monochrome("icons/ai-sdk/plus.svg")),
            IconName::Minus => Some(RenderStrategy::Monochrome("icons/ai-sdk/minus.svg")),
            IconName::Bug => Some(RenderStrategy::Monochrome("icons/ai-sdk/bug.svg")),
            IconName::TestTube => Some(RenderStrategy::Monochrome("icons/ai-sdk/test-tube.svg")),
            IconName::GitCommit => Some(RenderStrategy::Monochrome("icons/ai-sdk/git-commit.svg")),
            IconName::Package => Some(RenderStrategy::Monochrome("icons/ai-sdk/package.svg")),
            IconName::Database => Some(RenderStrategy::Monochrome("icons/ai-sdk/database.svg")),
            IconName::Key => Some(RenderStrategy::Monochrome("icons/ai-sdk/key.svg")),
            IconName::Bot => Some(RenderStrategy::Monochrome("icons/ai-sdk/bot.svg")),
            IconName::FileCode => Some(RenderStrategy::Monochrome("icons/ai-sdk/file-code.svg")),
            IconName::Trash => Some(RenderStrategy::Monochrome("icons/ai-sdk/trash.svg")),
            IconName::Eye => Some(RenderStrategy::Monochrome("icons/ai-sdk/eye.svg")),
            IconName::EyeOff => Some(RenderStrategy::Monochrome("icons/ai-sdk/eye-off.svg")),
            IconName::ExternalLink => {
                Some(RenderStrategy::Monochrome("icons/ai-sdk/external-link.svg"))
            }
            IconName::ChevronsUpDown => Some(RenderStrategy::Monochrome(
                "icons/ai-sdk/chevrons-up-down.svg",
            )),
            IconName::ThumbsUp => Some(RenderStrategy::Monochrome("icons/ai-sdk/thumbs-up.svg")),
            IconName::ThumbsDown => {
                Some(RenderStrategy::Monochrome("icons/ai-sdk/thumbs-down.svg"))
            }
            IconName::RotateCcw => Some(RenderStrategy::Monochrome("icons/ai-sdk/rotate-ccw.svg")),
            IconName::Share => Some(RenderStrategy::Monochrome("icons/ai-sdk/share.svg")),
            IconName::Pencil => Some(RenderStrategy::Monochrome("icons/ai-sdk/pencil.svg")),
            IconName::Volume2 => Some(RenderStrategy::Monochrome("icons/ai-sdk/volume2.svg")),
            IconName::VolumeX => Some(RenderStrategy::Monochrome("icons/ai-sdk/volume-x.svg")),
            IconName::SkipBack => Some(RenderStrategy::Monochrome("icons/ai-sdk/skip-back.svg")),
            IconName::SkipForward => {
                Some(RenderStrategy::Monochrome("icons/ai-sdk/skip-forward.svg"))
            }
            IconName::Lock => Some(RenderStrategy::Monochrome("icons/ai-sdk/lock.svg")),
            IconName::Unlock => Some(RenderStrategy::Monochrome("icons/ai-sdk/unlock.svg")),
            IconName::Maximize => Some(RenderStrategy::Monochrome("icons/ai-sdk/maximize.svg")),
            IconName::Paperclip => Some(RenderStrategy::Monochrome("icons/ai-sdk/paperclip.svg")),
            // Languages
            IconName::LangRust => Some(RenderStrategy::Monochrome("icons/languages/rust.svg")),
            IconName::LangPython => Some(RenderStrategy::Monochrome("icons/languages/python.svg")),
            IconName::LangJavaScript => {
                Some(RenderStrategy::Monochrome("icons/languages/javascript.svg"))
            }
            IconName::LangTypeScript => {
                Some(RenderStrategy::Monochrome("icons/languages/typescript.svg"))
            }
            IconName::LangGo => Some(RenderStrategy::Monochrome("icons/languages/go.svg")),
            IconName::LangC => Some(RenderStrategy::Monochrome("icons/languages/c.svg")),
            IconName::LangCpp => Some(RenderStrategy::Monochrome("icons/languages/cpp.svg")),
            IconName::LangBash => Some(RenderStrategy::Monochrome("icons/languages/bash.svg")),
            IconName::LangJson => Some(RenderStrategy::Monochrome("icons/languages/json.svg")),
            IconName::LangToml => Some(RenderStrategy::Monochrome("icons/languages/toml.svg")),
            IconName::LangHtml => Some(RenderStrategy::Monochrome("icons/languages/html.svg")),
            IconName::LangCss => Some(RenderStrategy::Monochrome("icons/languages/css.svg")),
            // Providers
            IconName::ProviderClaude => {
                Some(RenderStrategy::Monochrome("icons/providers/claude.svg"))
            }
            IconName::ProviderGpt => Some(RenderStrategy::Monochrome("icons/providers/gpt.svg")),
            IconName::ProviderGemini => {
                Some(RenderStrategy::Monochrome("icons/providers/gemini.svg"))
            }
            IconName::ProviderGrok => Some(RenderStrategy::Monochrome("icons/providers/grok.svg")),
            IconName::ProviderLlama => {
                Some(RenderStrategy::Monochrome("icons/providers/llama.svg"))
            }
            IconName::ProviderDeepSeek => {
                Some(RenderStrategy::Monochrome("icons/providers/deepseek.svg"))
            }
            IconName::ProviderMistral => {
                Some(RenderStrategy::Monochrome("icons/providers/mistral.svg"))
            }
            IconName::ProviderGemma => {
                Some(RenderStrategy::Monochrome("icons/providers/gemma.svg"))
            }
            IconName::ProviderPhi => Some(RenderStrategy::Monochrome("icons/providers/phi.svg")),
            IconName::ProviderQwen => Some(RenderStrategy::Monochrome("icons/providers/qwen.svg")),
            IconName::ProviderGlm => Some(RenderStrategy::Monochrome("icons/providers/glm.svg")),
            IconName::ProviderMiniMax => {
                Some(RenderStrategy::Monochrome("icons/providers/minimax.svg"))
            }
            IconName::ProviderErnie => {
                Some(RenderStrategy::Monochrome("icons/providers/ernie.svg"))
            }
            IconName::ProviderCohere => {
                Some(RenderStrategy::Monochrome("icons/providers/cohere.svg"))
            }
            IconName::ProviderPerplexity => {
                Some(RenderStrategy::Monochrome("icons/providers/perplexity.svg"))
            }
            IconName::ProviderNova => Some(RenderStrategy::Monochrome("icons/providers/nova.svg")),
            IconName::ProviderCustom => {
                Some(RenderStrategy::Monochrome("icons/providers/custom.svg"))
            }
            // Git (multi-color layers)
            IconName::GitBranch => Some(RenderStrategy::MultiColor(&[(
                "icons/git/branch_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitMerge => Some(RenderStrategy::MultiColor(&[(
                "icons/git/merge_success.svg",
                IconColorRole::Success,
            )])),
            IconName::GitConflict => Some(RenderStrategy::MultiColor(&[(
                "icons/git/conflict_warning.svg",
                IconColorRole::Warning,
            )])),
            IconName::GitPull => Some(RenderStrategy::MultiColor(&[(
                "icons/git/pull_info.svg",
                IconColorRole::Info,
            )])),
            IconName::GitPush => Some(RenderStrategy::MultiColor(&[(
                "icons/git/push_success.svg",
                IconColorRole::Success,
            )])),
            IconName::GitCheckout => Some(RenderStrategy::MultiColor(&[(
                "icons/git/checkout_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitStash => Some(RenderStrategy::MultiColor(&[(
                "icons/git/stash_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitTag => Some(RenderStrategy::MultiColor(&[(
                "icons/git/tag_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitLog => Some(RenderStrategy::MultiColor(&[(
                "icons/git/log_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitRebase => Some(RenderStrategy::MultiColor(&[
                ("icons/git/rebase_muted.svg", IconColorRole::Muted),
                ("icons/git/rebase_ai.svg", IconColorRole::Ai),
            ])),
            IconName::GitCompare => Some(RenderStrategy::MultiColor(&[
                ("icons/git/compare_muted.svg", IconColorRole::Muted),
                ("icons/git/compare_error.svg", IconColorRole::Error),
                ("icons/git/compare_success.svg", IconColorRole::Success),
            ])),
            IconName::GitInlineDiff => Some(RenderStrategy::MultiColor(&[
                ("icons/git/inline-diff_muted.svg", IconColorRole::Muted),
                ("icons/git/inline-diff_error.svg", IconColorRole::Error),
                ("icons/git/inline-diff_success.svg", IconColorRole::Success),
            ])),
            IconName::GitStaging => Some(RenderStrategy::MultiColor(&[
                ("icons/git/staging_muted.svg", IconColorRole::Muted),
                ("icons/git/staging_success.svg", IconColorRole::Success),
            ])),
            IconName::GitPullRequest => Some(RenderStrategy::MultiColor(&[(
                "icons/git/pull-request_success.svg",
                IconColorRole::Success,
            )])),
            IconName::GitCodeReview => Some(RenderStrategy::MultiColor(&[
                ("icons/git/code-review_muted.svg", IconColorRole::Muted),
                ("icons/git/code-review_info.svg", IconColorRole::Info),
            ])),
            IconName::GitFork => Some(RenderStrategy::MultiColor(&[(
                "icons/git/fork_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitClone => Some(RenderStrategy::MultiColor(&[(
                "icons/git/clone_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitRemote => Some(RenderStrategy::MultiColor(&[(
                "icons/git/remote_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitBlame => Some(RenderStrategy::MultiColor(&[
                ("icons/git/blame_muted.svg", IconColorRole::Muted),
                ("icons/git/blame_warning.svg", IconColorRole::Warning),
            ])),
            IconName::GitStaged => Some(RenderStrategy::MultiColor(&[(
                "icons/git/staged_success.svg",
                IconColorRole::Success,
            )])),
            IconName::GitModified => Some(RenderStrategy::MultiColor(&[(
                "icons/git/modified_warning.svg",
                IconColorRole::Warning,
            )])),
            IconName::GitUntracked => Some(RenderStrategy::MultiColor(&[(
                "icons/git/untracked_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitAdded => Some(RenderStrategy::MultiColor(&[(
                "icons/git/added_success.svg",
                IconColorRole::Success,
            )])),
            IconName::GitDeleted => Some(RenderStrategy::MultiColor(&[(
                "icons/git/deleted_error.svg",
                IconColorRole::Error,
            )])),
            IconName::GitIgnored => Some(RenderStrategy::MultiColor(&[(
                "icons/git/ignored_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitAhead => Some(RenderStrategy::MultiColor(&[(
                "icons/git/ahead_info.svg",
                IconColorRole::Info,
            )])),
            IconName::GitBehind => Some(RenderStrategy::MultiColor(&[(
                "icons/git/behind_warning.svg",
                IconColorRole::Warning,
            )])),
            IconName::GitClean => Some(RenderStrategy::MultiColor(&[(
                "icons/git/clean_success.svg",
                IconColorRole::Success,
            )])),
            // Dev Tools (multi-color layers)
            IconName::DevTab => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/tab_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::DevSidebar => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/sidebar_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::DevSplitView => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/split-view_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::DevSearch => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/search_info.svg",
                IconColorRole::Info,
            )])),
            IconName::DevFindReplace => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/find-replace_info.svg",
                IconColorRole::Info,
            )])),
            IconName::DevMinimap => Some(RenderStrategy::MultiColor(&[
                ("icons/dev-tools/minimap_muted.svg", IconColorRole::Muted),
                ("icons/dev-tools/minimap_info.svg", IconColorRole::Info),
            ])),
            IconName::DevBreadcrumb => Some(RenderStrategy::MultiColor(&[
                ("icons/dev-tools/breadcrumb_muted.svg", IconColorRole::Muted),
                ("icons/dev-tools/breadcrumb_info.svg", IconColorRole::Info),
            ])),
            IconName::DevSnippet => Some(RenderStrategy::MultiColor(&[
                ("icons/dev-tools/snippet_muted.svg", IconColorRole::Muted),
                (
                    "icons/dev-tools/snippet_warning.svg",
                    IconColorRole::Warning,
                ),
            ])),
            IconName::DevPalette => Some(RenderStrategy::MultiColor(&[
                ("icons/dev-tools/palette_muted.svg", IconColorRole::Muted),
                ("icons/dev-tools/palette_info.svg", IconColorRole::Info),
            ])),
            IconName::DevExtension => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/extension_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::DevKeyboard => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/keyboard_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::DevDebug => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/debug_error.svg",
                IconColorRole::Error,
            )])),
            // AI & Agents
            IconName::Agent => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/agent_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::Prompt => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/prompt_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::Chain => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/chain_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::ToolUse => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/tool-use_warning.svg",
                IconColorRole::Warning,
            )])),
            IconName::Memory => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/memory_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::Context => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/context_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::Embedding => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/embedding_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::Rag => Some(RenderStrategy::MultiColor(&[
                ("icons/dev-tools/rag_muted.svg", IconColorRole::Muted),
                ("icons/dev-tools/rag_info.svg", IconColorRole::Info),
            ])),
            IconName::Orchestrator => Some(RenderStrategy::MultiColor(&[
                (
                    "icons/dev-tools/orchestrator_muted.svg",
                    IconColorRole::Muted,
                ),
                ("icons/dev-tools/orchestrator_ai.svg", IconColorRole::Ai),
            ])),
            IconName::Model => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/model_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::Streaming => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/streaming_info.svg",
                IconColorRole::Info,
            )])),
            IconName::FunctionCall => Some(RenderStrategy::MultiColor(&[
                (
                    "icons/dev-tools/function-call_muted.svg",
                    IconColorRole::Muted,
                ),
                (
                    "icons/dev-tools/function-call_warning.svg",
                    IconColorRole::Warning,
                ),
            ])),
            IconName::Guardrail => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/guardrail_success.svg",
                IconColorRole::Success,
            )])),
            IconName::Token => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/token_warning.svg",
                IconColorRole::Warning,
            )])),
            IconName::FineTune => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/fine-tune_warning.svg",
                IconColorRole::Warning,
            )])),
            // DevOps
            IconName::Deploy => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/deploy_success.svg",
                IconColorRole::Success,
            )])),
            IconName::CiCd => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/ci-cd_success.svg",
                IconColorRole::Success,
            )])),
            IconName::Container => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/container_info.svg",
                IconColorRole::Info,
            )])),
            IconName::Pipeline => Some(RenderStrategy::MultiColor(&[
                ("icons/dev-tools/pipeline_muted.svg", IconColorRole::Muted),
                (
                    "icons/dev-tools/pipeline_success.svg",
                    IconColorRole::Success,
                ),
                (
                    "icons/dev-tools/pipeline_warning.svg",
                    IconColorRole::Warning,
                ),
            ])),
            IconName::Monitor => Some(RenderStrategy::MultiColor(&[
                ("icons/dev-tools/monitor_muted.svg", IconColorRole::Muted),
                (
                    "icons/dev-tools/monitor_success.svg",
                    IconColorRole::Success,
                ),
            ])),
            IconName::Logs => Some(RenderStrategy::MultiColor(&[
                ("icons/dev-tools/logs_muted.svg", IconColorRole::Muted),
                ("icons/dev-tools/logs_warning.svg", IconColorRole::Warning),
            ])),
            IconName::Environment => Some(RenderStrategy::MultiColor(&[
                (
                    "icons/dev-tools/environment_muted.svg",
                    IconColorRole::Muted,
                ),
                ("icons/dev-tools/environment_info.svg", IconColorRole::Info),
            ])),
            IconName::Secret => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/secret_warning.svg",
                IconColorRole::Warning,
            )])),
            IconName::Webhook => Some(RenderStrategy::MultiColor(&[
                ("icons/dev-tools/webhook_muted.svg", IconColorRole::Muted),
                ("icons/dev-tools/webhook_info.svg", IconColorRole::Info),
            ])),
            IconName::Api => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/api_info.svg",
                IconColorRole::Info,
            )])),
            IconName::Scale => Some(RenderStrategy::MultiColor(&[
                ("icons/dev-tools/scale_muted.svg", IconColorRole::Muted),
                ("icons/dev-tools/scale_success.svg", IconColorRole::Success),
            ])),
            IconName::Rollback => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/rollback_error.svg",
                IconColorRole::Error,
            )])),
            IconName::Health => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/health_error.svg",
                IconColorRole::Error,
            )])),
            IconName::Queue => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/queue_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::Cache => Some(RenderStrategy::MultiColor(&[(
                "icons/dev-tools/cache_warning.svg",
                IconColorRole::Warning,
            )])),
            _ => None,
        }
    }

    /// Returns the Liquid Glass SVG rendering strategy (stroke-width 1.5).
    ///
    /// For icons that resolve to a canonical SF Symbol, both the standard
    /// and glass strategies point at the same fill-based SF asset — SF
    /// Symbols are filled glyphs, not strokes, so `stroke-width` does not
    /// apply; Liquid Glass's brighter color palette is applied instead.
    ///
    /// For domain-specific icons (providers, git, languages, dev-tools),
    /// uses glass variant SVGs from `assets/icons-glass/` with thicker
    /// strokes (1.5 vs 1.2).
    #[allow(unreachable_patterns)] // _ => arm for #[non_exhaustive] forward compat
    pub(crate) fn render_strategy_glass(&self) -> Option<RenderStrategy> {
        // SF Symbols are fill-based and look identical across standard and
        // glass themes — the only thing that changes is the tint color.
        if let Some(path) = self.sf_asset_path() {
            return Some(RenderStrategy::Monochrome(path));
        }
        match self {
            // ── Core UI ──────────────────────────────────────────────────
            IconName::ArrowDown => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/arrow-down.svg",
            )),
            IconName::ArrowRight => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/arrow-right.svg",
            )),
            IconName::Brain => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/brain.svg")),
            IconName::Check => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/check.svg")),
            IconName::ChevronDown => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/chevron-down.svg",
            )),
            IconName::ChevronLeft => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/chevron-left.svg",
            )),
            IconName::ChevronRight => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/chevron-right.svg",
            )),
            IconName::ChevronUp => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/chevron-up.svg",
            )),
            IconName::Copy => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/copy.svg")),
            IconName::Download => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/download.svg",
            )),
            IconName::Send => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/send.svg")),
            IconName::Square => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/square.svg")),
            IconName::X => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/x.svg")),
            IconName::Loader => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/loader.svg")),
            IconName::ProgressSpinner => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/progress-spinner.svg",
            )),
            IconName::Code => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/code.svg")),
            IconName::File => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/file.svg")),
            IconName::Folder => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/folder.svg")),
            IconName::FolderOpen => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/folder-open.svg",
            )),
            IconName::Terminal => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/terminal.svg",
            )),
            IconName::Play => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/play.svg")),
            IconName::Pause => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/pause.svg")),
            IconName::Mic => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/mic.svg")),
            IconName::MicOff => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/mic-off.svg")),
            IconName::Phone => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/phone.svg")),
            IconName::Video => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/video.svg")),
            IconName::Settings => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/settings.svg",
            )),
            // ── Phase 2 ──────────────────────────────────────────────────
            IconName::Bookmark => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/bookmark.svg",
            )),
            IconName::Book => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/book.svg")),
            IconName::Search => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/search.svg")),
            IconName::Link => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/link.svg")),
            IconName::Globe => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/globe.svg")),
            IconName::Sparkle => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/sparkle.svg")),
            IconName::ListTodo => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/list-todo.svg",
            )),
            IconName::CircleFilled => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/circle-filled.svg",
            )),
            IconName::CircleOutline => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/circle-outline.svg",
            )),
            IconName::AlertTriangle => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/alert-triangle.svg",
            )),
            IconName::Image => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/image.svg")),
            IconName::Plus => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/plus.svg")),
            IconName::Minus => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/minus.svg")),
            // ── Phase 3 ──────────────────────────────────────────────────
            IconName::Bug => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/bug.svg")),
            IconName::TestTube => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/test-tube.svg",
            )),
            IconName::GitCommit => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/git-commit.svg",
            )),
            IconName::Package => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/package.svg")),
            IconName::Database => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/database.svg",
            )),
            IconName::Key => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/key.svg")),
            IconName::Bot => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/bot.svg")),
            IconName::FileCode => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/file-code.svg",
            )),
            IconName::Trash => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/trash.svg")),
            IconName::Eye => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/eye.svg")),
            IconName::EyeOff => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/eye-off.svg")),
            IconName::ExternalLink => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/external-link.svg",
            )),
            // ── Plan / Message actions / Audio / Workflow ────────────────
            IconName::ChevronsUpDown => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/chevrons-up-down.svg",
            )),
            IconName::ThumbsUp => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/thumbs-up.svg",
            )),
            IconName::ThumbsDown => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/thumbs-down.svg",
            )),
            IconName::RotateCcw => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/rotate-ccw.svg",
            )),
            IconName::Share => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/share.svg")),
            IconName::Pencil => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/pencil.svg")),
            IconName::Volume2 => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/volume2.svg")),
            IconName::VolumeX => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/volume-x.svg",
            )),
            IconName::SkipBack => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/skip-back.svg",
            )),
            IconName::SkipForward => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/skip-forward.svg",
            )),
            IconName::Lock => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/lock.svg")),
            IconName::Unlock => Some(RenderStrategy::Monochrome("icons-glass/ai-sdk/unlock.svg")),
            IconName::Maximize => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/maximize.svg",
            )),
            IconName::Paperclip => Some(RenderStrategy::Monochrome(
                "icons-glass/ai-sdk/paperclip.svg",
            )),
            // ── Languages ────────────────────────────────────────────────
            IconName::LangRust => {
                Some(RenderStrategy::Monochrome("icons-glass/languages/rust.svg"))
            }
            IconName::LangPython => Some(RenderStrategy::Monochrome(
                "icons-glass/languages/python.svg",
            )),
            IconName::LangJavaScript => Some(RenderStrategy::Monochrome(
                "icons-glass/languages/javascript.svg",
            )),
            IconName::LangTypeScript => Some(RenderStrategy::Monochrome(
                "icons-glass/languages/typescript.svg",
            )),
            IconName::LangGo => Some(RenderStrategy::Monochrome("icons-glass/languages/go.svg")),
            IconName::LangC => Some(RenderStrategy::Monochrome("icons-glass/languages/c.svg")),
            IconName::LangCpp => Some(RenderStrategy::Monochrome("icons-glass/languages/cpp.svg")),
            IconName::LangBash => {
                Some(RenderStrategy::Monochrome("icons-glass/languages/bash.svg"))
            }
            IconName::LangJson => {
                Some(RenderStrategy::Monochrome("icons-glass/languages/json.svg"))
            }
            IconName::LangToml => {
                Some(RenderStrategy::Monochrome("icons-glass/languages/toml.svg"))
            }
            IconName::LangHtml => {
                Some(RenderStrategy::Monochrome("icons-glass/languages/html.svg"))
            }
            IconName::LangCss => Some(RenderStrategy::Monochrome("icons-glass/languages/css.svg")),
            // ── Providers ────────────────────────────────────────────────
            IconName::ProviderClaude => Some(RenderStrategy::Monochrome(
                "icons-glass/providers/claude.svg",
            )),
            IconName::ProviderGpt => {
                Some(RenderStrategy::Monochrome("icons-glass/providers/gpt.svg"))
            }
            IconName::ProviderGemini => Some(RenderStrategy::Monochrome(
                "icons-glass/providers/gemini.svg",
            )),
            IconName::ProviderGrok => {
                Some(RenderStrategy::Monochrome("icons-glass/providers/grok.svg"))
            }
            IconName::ProviderLlama => Some(RenderStrategy::Monochrome(
                "icons-glass/providers/llama.svg",
            )),
            IconName::ProviderDeepSeek => Some(RenderStrategy::Monochrome(
                "icons-glass/providers/deepseek.svg",
            )),
            IconName::ProviderMistral => Some(RenderStrategy::Monochrome(
                "icons-glass/providers/mistral.svg",
            )),
            IconName::ProviderGemma => Some(RenderStrategy::Monochrome(
                "icons-glass/providers/gemma.svg",
            )),
            IconName::ProviderPhi => {
                Some(RenderStrategy::Monochrome("icons-glass/providers/phi.svg"))
            }
            IconName::ProviderQwen => {
                Some(RenderStrategy::Monochrome("icons-glass/providers/qwen.svg"))
            }
            IconName::ProviderGlm => {
                Some(RenderStrategy::Monochrome("icons-glass/providers/glm.svg"))
            }
            IconName::ProviderMiniMax => Some(RenderStrategy::Monochrome(
                "icons-glass/providers/minimax.svg",
            )),
            IconName::ProviderErnie => Some(RenderStrategy::Monochrome(
                "icons-glass/providers/ernie.svg",
            )),
            IconName::ProviderCohere => Some(RenderStrategy::Monochrome(
                "icons-glass/providers/cohere.svg",
            )),
            IconName::ProviderPerplexity => Some(RenderStrategy::Monochrome(
                "icons-glass/providers/perplexity.svg",
            )),
            IconName::ProviderNova => {
                Some(RenderStrategy::Monochrome("icons-glass/providers/nova.svg"))
            }
            IconName::ProviderCustom => Some(RenderStrategy::Monochrome(
                "icons-glass/providers/custom.svg",
            )),
            // ── Git (multi-color layers) ─────────────────────────────────
            IconName::GitBranch => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/branch_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitMerge => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/merge_success.svg",
                IconColorRole::Success,
            )])),
            IconName::GitConflict => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/conflict_warning.svg",
                IconColorRole::Warning,
            )])),
            IconName::GitPull => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/pull_info.svg",
                IconColorRole::Info,
            )])),
            IconName::GitPush => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/push_success.svg",
                IconColorRole::Success,
            )])),
            IconName::GitCheckout => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/checkout_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitStash => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/stash_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitTag => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/tag_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitLog => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/log_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitRebase => Some(RenderStrategy::MultiColor(&[
                ("icons-glass/git/rebase_muted.svg", IconColorRole::Muted),
                ("icons-glass/git/rebase_ai.svg", IconColorRole::Ai),
            ])),
            IconName::GitCompare => Some(RenderStrategy::MultiColor(&[
                ("icons-glass/git/compare_muted.svg", IconColorRole::Muted),
                ("icons-glass/git/compare_error.svg", IconColorRole::Error),
                (
                    "icons-glass/git/compare_success.svg",
                    IconColorRole::Success,
                ),
            ])),
            IconName::GitInlineDiff => Some(RenderStrategy::MultiColor(&[
                (
                    "icons-glass/git/inline-diff_muted.svg",
                    IconColorRole::Muted,
                ),
                (
                    "icons-glass/git/inline-diff_error.svg",
                    IconColorRole::Error,
                ),
                (
                    "icons-glass/git/inline-diff_success.svg",
                    IconColorRole::Success,
                ),
            ])),
            IconName::GitStaging => Some(RenderStrategy::MultiColor(&[
                ("icons-glass/git/staging_muted.svg", IconColorRole::Muted),
                (
                    "icons-glass/git/staging_success.svg",
                    IconColorRole::Success,
                ),
            ])),
            IconName::GitPullRequest => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/pull-request_success.svg",
                IconColorRole::Success,
            )])),
            IconName::GitCodeReview => Some(RenderStrategy::MultiColor(&[
                (
                    "icons-glass/git/code-review_muted.svg",
                    IconColorRole::Muted,
                ),
                ("icons-glass/git/code-review_info.svg", IconColorRole::Info),
            ])),
            IconName::GitFork => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/fork_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitClone => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/clone_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitRemote => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/remote_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitBlame => Some(RenderStrategy::MultiColor(&[
                ("icons-glass/git/blame_muted.svg", IconColorRole::Muted),
                ("icons-glass/git/blame_warning.svg", IconColorRole::Warning),
            ])),
            IconName::GitStaged => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/staged_success.svg",
                IconColorRole::Success,
            )])),
            IconName::GitModified => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/modified_warning.svg",
                IconColorRole::Warning,
            )])),
            IconName::GitUntracked => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/untracked_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitAdded => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/added_success.svg",
                IconColorRole::Success,
            )])),
            IconName::GitDeleted => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/deleted_error.svg",
                IconColorRole::Error,
            )])),
            IconName::GitIgnored => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/ignored_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::GitAhead => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/ahead_info.svg",
                IconColorRole::Info,
            )])),
            IconName::GitBehind => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/behind_warning.svg",
                IconColorRole::Warning,
            )])),
            IconName::GitClean => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/git/clean_success.svg",
                IconColorRole::Success,
            )])),
            // ── Dev Tools: IDE & Editor ──────────────────────────────────
            IconName::DevTab => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/tab_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::DevSidebar => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/sidebar_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::DevSplitView => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/split-view_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::DevSearch => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/search_info.svg",
                IconColorRole::Info,
            )])),
            IconName::DevFindReplace => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/find-replace_info.svg",
                IconColorRole::Info,
            )])),
            IconName::DevMinimap => Some(RenderStrategy::MultiColor(&[
                (
                    "icons-glass/dev-tools/minimap_muted.svg",
                    IconColorRole::Muted,
                ),
                (
                    "icons-glass/dev-tools/minimap_info.svg",
                    IconColorRole::Info,
                ),
            ])),
            IconName::DevBreadcrumb => Some(RenderStrategy::MultiColor(&[
                (
                    "icons-glass/dev-tools/breadcrumb_muted.svg",
                    IconColorRole::Muted,
                ),
                (
                    "icons-glass/dev-tools/breadcrumb_info.svg",
                    IconColorRole::Info,
                ),
            ])),
            IconName::DevSnippet => Some(RenderStrategy::MultiColor(&[
                (
                    "icons-glass/dev-tools/snippet_muted.svg",
                    IconColorRole::Muted,
                ),
                (
                    "icons-glass/dev-tools/snippet_warning.svg",
                    IconColorRole::Warning,
                ),
            ])),
            IconName::DevPalette => Some(RenderStrategy::MultiColor(&[
                (
                    "icons-glass/dev-tools/palette_muted.svg",
                    IconColorRole::Muted,
                ),
                (
                    "icons-glass/dev-tools/palette_info.svg",
                    IconColorRole::Info,
                ),
            ])),
            IconName::DevExtension => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/extension_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::DevKeyboard => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/keyboard_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::DevDebug => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/debug_error.svg",
                IconColorRole::Error,
            )])),
            // ── Dev Tools: AI & Agents ───────────────────────────────────
            IconName::Agent => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/agent_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::Prompt => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/prompt_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::Chain => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/chain_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::ToolUse => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/tool-use_warning.svg",
                IconColorRole::Warning,
            )])),
            IconName::Memory => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/memory_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::Context => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/context_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::Embedding => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/embedding_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::Rag => Some(RenderStrategy::MultiColor(&[
                ("icons-glass/dev-tools/rag_muted.svg", IconColorRole::Muted),
                ("icons-glass/dev-tools/rag_info.svg", IconColorRole::Info),
            ])),
            IconName::Orchestrator => Some(RenderStrategy::MultiColor(&[
                (
                    "icons-glass/dev-tools/orchestrator_muted.svg",
                    IconColorRole::Muted,
                ),
                (
                    "icons-glass/dev-tools/orchestrator_ai.svg",
                    IconColorRole::Ai,
                ),
            ])),
            IconName::Model => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/model_ai.svg",
                IconColorRole::Ai,
            )])),
            IconName::Streaming => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/streaming_info.svg",
                IconColorRole::Info,
            )])),
            IconName::FunctionCall => Some(RenderStrategy::MultiColor(&[
                (
                    "icons-glass/dev-tools/function-call_muted.svg",
                    IconColorRole::Muted,
                ),
                (
                    "icons-glass/dev-tools/function-call_warning.svg",
                    IconColorRole::Warning,
                ),
            ])),
            IconName::Guardrail => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/guardrail_success.svg",
                IconColorRole::Success,
            )])),
            IconName::Token => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/token_warning.svg",
                IconColorRole::Warning,
            )])),
            IconName::FineTune => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/fine-tune_warning.svg",
                IconColorRole::Warning,
            )])),
            // ── Dev Tools: DevOps ────────────────────────────────────────
            IconName::Deploy => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/deploy_success.svg",
                IconColorRole::Success,
            )])),
            IconName::CiCd => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/ci-cd_success.svg",
                IconColorRole::Success,
            )])),
            IconName::Container => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/container_info.svg",
                IconColorRole::Info,
            )])),
            IconName::Pipeline => Some(RenderStrategy::MultiColor(&[
                (
                    "icons-glass/dev-tools/pipeline_muted.svg",
                    IconColorRole::Muted,
                ),
                (
                    "icons-glass/dev-tools/pipeline_success.svg",
                    IconColorRole::Success,
                ),
                (
                    "icons-glass/dev-tools/pipeline_warning.svg",
                    IconColorRole::Warning,
                ),
            ])),
            IconName::Monitor => Some(RenderStrategy::MultiColor(&[
                (
                    "icons-glass/dev-tools/monitor_muted.svg",
                    IconColorRole::Muted,
                ),
                (
                    "icons-glass/dev-tools/monitor_success.svg",
                    IconColorRole::Success,
                ),
            ])),
            IconName::Logs => Some(RenderStrategy::MultiColor(&[
                ("icons-glass/dev-tools/logs_muted.svg", IconColorRole::Muted),
                (
                    "icons-glass/dev-tools/logs_warning.svg",
                    IconColorRole::Warning,
                ),
            ])),
            IconName::Environment => Some(RenderStrategy::MultiColor(&[
                (
                    "icons-glass/dev-tools/environment_muted.svg",
                    IconColorRole::Muted,
                ),
                (
                    "icons-glass/dev-tools/environment_info.svg",
                    IconColorRole::Info,
                ),
            ])),
            IconName::Secret => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/secret_warning.svg",
                IconColorRole::Warning,
            )])),
            IconName::Webhook => Some(RenderStrategy::MultiColor(&[
                (
                    "icons-glass/dev-tools/webhook_muted.svg",
                    IconColorRole::Muted,
                ),
                (
                    "icons-glass/dev-tools/webhook_info.svg",
                    IconColorRole::Info,
                ),
            ])),
            IconName::Api => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/api_info.svg",
                IconColorRole::Info,
            )])),
            IconName::Scale => Some(RenderStrategy::MultiColor(&[
                (
                    "icons-glass/dev-tools/scale_muted.svg",
                    IconColorRole::Muted,
                ),
                (
                    "icons-glass/dev-tools/scale_success.svg",
                    IconColorRole::Success,
                ),
            ])),
            IconName::Rollback => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/rollback_error.svg",
                IconColorRole::Error,
            )])),
            IconName::Health => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/health_error.svg",
                IconColorRole::Error,
            )])),
            IconName::Queue => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/queue_muted.svg",
                IconColorRole::Muted,
            )])),
            IconName::Cache => Some(RenderStrategy::MultiColor(&[(
                "icons-glass/dev-tools/cache_warning.svg",
                IconColorRole::Warning,
            )])),
            _ => None,
        }
    }

    /// Classification of this symbol under RTL layouts.
    ///
    /// See [`IconLayoutBehavior`]. The default is `Neutral`; only
    /// explicitly directional symbols (arrows, chevrons, progress
    /// glyphs) return `Directional`, and only culture-specific glyphs
    /// (signature, some rich-text glyphs when they ship) return
    /// `Localized`.
    pub fn layout_behavior(self) -> IconLayoutBehavior {
        match self {
            // Directional arrows & chevrons — geometrically mirrored in RTL.
            IconName::ArrowRight
            | IconName::ArrowTriangleRight
            | IconName::ChevronLeft
            | IconName::ChevronRight
            | IconName::Send => IconLayoutBehavior::Directional,
            // Vertical arrows, up/down chevrons, and everything else stays
            // upright regardless of reading direction. Add symbols here as
            // new directional glyphs are introduced.
            _ => IconLayoutBehavior::Neutral,
        }
    }
}
