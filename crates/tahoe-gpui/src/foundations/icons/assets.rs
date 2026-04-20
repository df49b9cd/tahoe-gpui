//! Embedded SVG icon assets.
//!
//! Provides an [`EmbeddedIconAssets`] implementation of [`gpui::AssetSource`]
//! that serves SVG icon data from compile-time `include_bytes!()` calls.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use gpui::SharedString;

/// Color role for multi-color icon layers.
///
/// Each layer of a multi-color icon is tinted with a semantic color
/// from the theme, identified by this role.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum IconColorRole {
    /// Default/muted strokes — maps to `theme.text_muted`.
    Muted,
    /// Success/green — maps to `theme.success`.
    Success,
    /// Info/blue — maps to `theme.info`.
    Info,
    /// Warning/amber — maps to `theme.warning`.
    Warning,
    /// Error/red — maps to `theme.error`.
    Error,
    /// AI/purple — maps to `theme.ai`.
    Ai,
}

/// Rendering strategy for an icon.
#[derive(Debug, Clone)]
pub(crate) enum RenderStrategy {
    /// Single monochrome SVG, tinted with the caller's color.
    Monochrome(&'static str),
    /// Multiple SVG layers, each tinted with a semantic theme color.
    /// The caller's color overrides the `Muted` layer.
    MultiColor(&'static [(&'static str, IconColorRole)]),
}

/// Embedded icon asset source for GPUI.
///
/// Register this with your GPUI application to enable SVG icon rendering:
/// ```ignore
/// application().with_assets(EmbeddedIconAssets).run(|cx| { ... });
/// ```
///
/// ## Stroke-width–qualified paths
///
/// Paths with a `__sw{value}` suffix (e.g. `icons/symbols/checkmark.svg__sw1.4`)
/// are resolved by loading the base SVG and replacing every `stroke-width="…"`
/// attribute with the requested value. The mutated bytes are cached so each
/// (path, stroke-width) pair is computed at most once.
pub struct EmbeddedIconAssets;

impl gpui::AssetSource for EmbeddedIconAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        // Fast path: no __sw suffix → static lookup, zero allocation.
        if let Some(bytes) = lookup_svg(path) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }

        // Slow path: stroke-width–qualified variant.
        if let Some((base_path, sw)) = parse_stroke_width_path(path) {
            // Check the variant cache first.
            if let Some(cached) = SW_VARIANTS
                .read()
                .expect("SW_VARIANTS lock poisoned")
                .get(path)
            {
                return Ok(Some(Cow::Owned(cached.clone())));
            }

            // Load original, mutate stroke-width, cache the result.
            if let Some(original) = lookup_svg(base_path) {
                let modified = replace_stroke_width(original, sw);
                SW_VARIANTS
                    .write()
                    .expect("SW_VARIANTS lock poisoned")
                    .insert(path.to_string(), modified);
                let cached = SW_VARIANTS
                    .read()
                    .expect("SW_VARIANTS lock poisoned")
                    .get(path)
                    .expect("just-inserted entry must exist")
                    .clone();
                return Ok(Some(Cow::Owned(cached)));
            }
        }

        Ok(None)
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(ICON_ENTRIES
            .iter()
            .filter(|(p, _)| p.starts_with(path))
            .map(|(p, _)| SharedString::from(*p))
            .collect())
    }
}

/// Stroke-width suffix separator embedded in qualified asset paths.
const SW_SUFFIX: &str = "__sw";

/// Parse a stroke-width–qualified path like `icons/checkmark.svg__sw1.4`.
///
/// Returns `Some((base_path, stroke_width))` on match, `None` otherwise.
pub(crate) fn parse_stroke_width_path(path: &str) -> Option<(&str, f32)> {
    let pos = path.rfind(SW_SUFFIX)?;
    let (base, sw_str) = (&path[..pos], &path[pos + SW_SUFFIX.len()..]);
    // Guard against false positives — the base must end with ".svg".
    if !base.ends_with(".svg") {
        return None;
    }
    let sw = sw_str.parse::<f32>().ok()?;
    Some((base, sw))
}

/// Replace every `stroke-width="…"` attribute in the SVG bytes with `sw`.
///
/// SVG assets use the consistent format `stroke-width="X.XX"`, so a simple
/// scan-and-replace suffices. Fill-based icons (no `stroke-width` attribute)
/// are returned unchanged — correct because weight has no visual effect on them.
pub(crate) fn replace_stroke_width(svg_bytes: &[u8], sw: f32) -> Vec<u8> {
    let svg = std::str::from_utf8(svg_bytes).expect("SVG assets are valid UTF-8");
    let new_attr = format!("stroke-width=\"{sw}\"");

    let needle = "stroke-width=\"";
    let mut result = String::with_capacity(svg.len());
    let mut pos = 0;

    while pos < svg.len() {
        if let Some(offset) = svg[pos..].find(needle) {
            let abs = pos + offset;
            let value_start = abs + needle.len();
            // Find the closing quote of the attribute value.
            if let Some(end_quote) = svg[value_start..].find('"') {
                result.push_str(&svg[pos..abs]);
                result.push_str(&new_attr);
                pos = value_start + end_quote + 1;
                continue;
            }
        }
        result.push_str(&svg[pos..]);
        break;
    }

    result.into_bytes()
}

/// O(1) lookup from icon path to embedded SVG bytes, built lazily on first use.
///
/// GPUI caches decoded SVGs after first load, so this map is only consulted
/// on cold paths. Using a `HashMap` keeps cold-path cost constant as the
/// icon set grows.
static ICON_INDEX: LazyLock<HashMap<&'static str, &'static [u8]>> =
    LazyLock::new(|| ICON_ENTRIES.iter().copied().collect());

/// Cache for stroke-width–modified SVG variants, keyed by the full qualified
/// path (e.g. `icons/symbols/checkmark.svg__sw1.4`). Each (path, stroke-width)
/// pair is computed once and reused for the process lifetime.
static SW_VARIANTS: LazyLock<RwLock<HashMap<String, Vec<u8>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

fn lookup_svg(path: &str) -> Option<&'static [u8]> {
    ICON_INDEX.get(path).copied()
}

/// All embedded SVG icon data.
///
/// Symbol icons live under `icons/symbols/` — an original, SF-Symbols-aligned
/// glyph set authored for this crate (Apache-2.0). Domain-specific icons
/// (programming languages, LLM providers, git, dev-tools) keep their own
/// folders with both standard and Liquid Glass variants.
static ICON_ENTRIES: &[(&str, &[u8])] = &[
    (
        "icons/symbols/ant.svg",
        include_bytes!("../../../assets/icons/symbols/ant.svg"),
    ),
    (
        "icons/symbols/arrow-clockwise.svg",
        include_bytes!("../../../assets/icons/symbols/arrow-clockwise.svg"),
    ),
    (
        "icons/symbols/arrow-down-circle.svg",
        include_bytes!("../../../assets/icons/symbols/arrow-down-circle.svg"),
    ),
    (
        "icons/symbols/arrow-down.svg",
        include_bytes!("../../../assets/icons/symbols/arrow-down.svg"),
    ),
    (
        "icons/symbols/arrow-right.svg",
        include_bytes!("../../../assets/icons/symbols/arrow-right.svg"),
    ),
    (
        "icons/symbols/arrow-up-left-down-right.svg",
        include_bytes!("../../../assets/icons/symbols/arrow-up-left-down-right.svg"),
    ),
    (
        "icons/symbols/arrow-up-right-square.svg",
        include_bytes!("../../../assets/icons/symbols/arrow-up-right-square.svg"),
    ),
    (
        "icons/symbols/arrow-uturn-backward.svg",
        include_bytes!("../../../assets/icons/symbols/arrow-uturn-backward.svg"),
    ),
    (
        "icons/symbols/arrow-uturn-forward.svg",
        include_bytes!("../../../assets/icons/symbols/arrow-uturn-forward.svg"),
    ),
    (
        "icons/symbols/arrowtriangle-down-fill.svg",
        include_bytes!("../../../assets/icons/symbols/arrowtriangle-down-fill.svg"),
    ),
    (
        "icons/symbols/arrowtriangle-right-fill.svg",
        include_bytes!("../../../assets/icons/symbols/arrowtriangle-right-fill.svg"),
    ),
    (
        "icons/symbols/backward-end.svg",
        include_bytes!("../../../assets/icons/symbols/backward-end.svg"),
    ),
    (
        "icons/symbols/book.svg",
        include_bytes!("../../../assets/icons/symbols/book.svg"),
    ),
    (
        "icons/symbols/bookmark.svg",
        include_bytes!("../../../assets/icons/symbols/bookmark.svg"),
    ),
    (
        "icons/symbols/bot.svg",
        include_bytes!("../../../assets/icons/symbols/bot.svg"),
    ),
    (
        "icons/symbols/brain.svg",
        include_bytes!("../../../assets/icons/symbols/brain.svg"),
    ),
    (
        "icons/symbols/checklist.svg",
        include_bytes!("../../../assets/icons/symbols/checklist.svg"),
    ),
    (
        "icons/symbols/checkmark.svg",
        include_bytes!("../../../assets/icons/symbols/checkmark.svg"),
    ),
    (
        "icons/symbols/chevron-down.svg",
        include_bytes!("../../../assets/icons/symbols/chevron-down.svg"),
    ),
    (
        "icons/symbols/chevron-left-fwdslash-chevron-right.svg",
        include_bytes!("../../../assets/icons/symbols/chevron-left-fwdslash-chevron-right.svg"),
    ),
    (
        "icons/symbols/chevron-left.svg",
        include_bytes!("../../../assets/icons/symbols/chevron-left.svg"),
    ),
    (
        "icons/symbols/chevron-right.svg",
        include_bytes!("../../../assets/icons/symbols/chevron-right.svg"),
    ),
    (
        "icons/symbols/chevron-up-chevron-down.svg",
        include_bytes!("../../../assets/icons/symbols/chevron-up-chevron-down.svg"),
    ),
    (
        "icons/symbols/chevron-up.svg",
        include_bytes!("../../../assets/icons/symbols/chevron-up.svg"),
    ),
    (
        "icons/symbols/circle-fill.svg",
        include_bytes!("../../../assets/icons/symbols/circle-fill.svg"),
    ),
    (
        "icons/symbols/circle.svg",
        include_bytes!("../../../assets/icons/symbols/circle.svg"),
    ),
    (
        "icons/symbols/clock.svg",
        include_bytes!("../../../assets/icons/symbols/clock.svg"),
    ),
    (
        "icons/symbols/cylinder.svg",
        include_bytes!("../../../assets/icons/symbols/cylinder.svg"),
    ),
    (
        "icons/symbols/document-on-document.svg",
        include_bytes!("../../../assets/icons/symbols/document-on-document.svg"),
    ),
    (
        "icons/symbols/document.svg",
        include_bytes!("../../../assets/icons/symbols/document.svg"),
    ),
    (
        "icons/symbols/ellipsis.svg",
        include_bytes!("../../../assets/icons/symbols/ellipsis.svg"),
    ),
    (
        "icons/symbols/exclamationmark-triangle.svg",
        include_bytes!("../../../assets/icons/symbols/exclamationmark-triangle.svg"),
    ),
    (
        "icons/symbols/info-circle.svg",
        include_bytes!("../../../assets/icons/symbols/info-circle.svg"),
    ),
    (
        "icons/symbols/file-code.svg",
        include_bytes!("../../../assets/icons/symbols/file-code.svg"),
    ),
    (
        "icons/symbols/eye-slash.svg",
        include_bytes!("../../../assets/icons/symbols/eye-slash.svg"),
    ),
    (
        "icons/symbols/eye.svg",
        include_bytes!("../../../assets/icons/symbols/eye.svg"),
    ),
    (
        "icons/symbols/film.svg",
        include_bytes!("../../../assets/icons/symbols/film.svg"),
    ),
    (
        "icons/symbols/folder-fill.svg",
        include_bytes!("../../../assets/icons/symbols/folder-fill.svg"),
    ),
    (
        "icons/symbols/folder.svg",
        include_bytes!("../../../assets/icons/symbols/folder.svg"),
    ),
    (
        "icons/symbols/forward-end.svg",
        include_bytes!("../../../assets/icons/symbols/forward-end.svg"),
    ),
    (
        "icons/symbols/gear.svg",
        include_bytes!("../../../assets/icons/symbols/gear.svg"),
    ),
    (
        "icons/symbols/git-commit.svg",
        include_bytes!("../../../assets/icons/symbols/git-commit.svg"),
    ),
    (
        "icons/symbols/globe.svg",
        include_bytes!("../../../assets/icons/symbols/globe.svg"),
    ),
    (
        "icons/symbols/hand-thumbsdown.svg",
        include_bytes!("../../../assets/icons/symbols/hand-thumbsdown.svg"),
    ),
    (
        "icons/symbols/hand-thumbsup.svg",
        include_bytes!("../../../assets/icons/symbols/hand-thumbsup.svg"),
    ),
    (
        "icons/symbols/key.svg",
        include_bytes!("../../../assets/icons/symbols/key.svg"),
    ),
    (
        "icons/symbols/link.svg",
        include_bytes!("../../../assets/icons/symbols/link.svg"),
    ),
    (
        "icons/symbols/lock-open.svg",
        include_bytes!("../../../assets/icons/symbols/lock-open.svg"),
    ),
    (
        "icons/symbols/lock.svg",
        include_bytes!("../../../assets/icons/symbols/lock.svg"),
    ),
    (
        "icons/symbols/magnifyingglass.svg",
        include_bytes!("../../../assets/icons/symbols/magnifyingglass.svg"),
    ),
    (
        "icons/symbols/microphone-slash.svg",
        include_bytes!("../../../assets/icons/symbols/microphone-slash.svg"),
    ),
    (
        "icons/symbols/microphone.svg",
        include_bytes!("../../../assets/icons/symbols/microphone.svg"),
    ),
    (
        "icons/symbols/minus.svg",
        include_bytes!("../../../assets/icons/symbols/minus.svg"),
    ),
    (
        "icons/symbols/paperclip.svg",
        include_bytes!("../../../assets/icons/symbols/paperclip.svg"),
    ),
    (
        "icons/symbols/paperplane.svg",
        include_bytes!("../../../assets/icons/symbols/paperplane.svg"),
    ),
    (
        "icons/symbols/pause-fill.svg",
        include_bytes!("../../../assets/icons/symbols/pause-fill.svg"),
    ),
    (
        "icons/symbols/pencil.svg",
        include_bytes!("../../../assets/icons/symbols/pencil.svg"),
    ),
    (
        "icons/symbols/phone.svg",
        include_bytes!("../../../assets/icons/symbols/phone.svg"),
    ),
    (
        "icons/symbols/photo.svg",
        include_bytes!("../../../assets/icons/symbols/photo.svg"),
    ),
    (
        "icons/symbols/play-fill.svg",
        include_bytes!("../../../assets/icons/symbols/play-fill.svg"),
    ),
    (
        "icons/symbols/plus.svg",
        include_bytes!("../../../assets/icons/symbols/plus.svg"),
    ),
    (
        "icons/symbols/progress-indicator.svg",
        include_bytes!("../../../assets/icons/symbols/progress-indicator.svg"),
    ),
    (
        "icons/symbols/questionmark-circle.svg",
        include_bytes!("../../../assets/icons/symbols/questionmark-circle.svg"),
    ),
    (
        "icons/symbols/shippingbox.svg",
        include_bytes!("../../../assets/icons/symbols/shippingbox.svg"),
    ),
    (
        "icons/symbols/sidebar-left.svg",
        include_bytes!("../../../assets/icons/symbols/sidebar-left.svg"),
    ),
    (
        "icons/symbols/sparkles.svg",
        include_bytes!("../../../assets/icons/symbols/sparkles.svg"),
    ),
    (
        "icons/symbols/speaker-slash.svg",
        include_bytes!("../../../assets/icons/symbols/speaker-slash.svg"),
    ),
    (
        "icons/symbols/speaker-wave-2.svg",
        include_bytes!("../../../assets/icons/symbols/speaker-wave-2.svg"),
    ),
    (
        "icons/symbols/square-and-arrow-up.svg",
        include_bytes!("../../../assets/icons/symbols/square-and-arrow-up.svg"),
    ),
    (
        "icons/symbols/square.svg",
        include_bytes!("../../../assets/icons/symbols/square.svg"),
    ),
    (
        "icons/symbols/stop-fill.svg",
        include_bytes!("../../../assets/icons/symbols/stop-fill.svg"),
    ),
    (
        "icons/symbols/star-fill.svg",
        include_bytes!("../../../assets/icons/symbols/star-fill.svg"),
    ),
    (
        "icons/symbols/star-leadinghalf-filled.svg",
        include_bytes!("../../../assets/icons/symbols/star-leadinghalf-filled.svg"),
    ),
    (
        "icons/symbols/star.svg",
        include_bytes!("../../../assets/icons/symbols/star.svg"),
    ),
    (
        "icons/symbols/terminal.svg",
        include_bytes!("../../../assets/icons/symbols/terminal.svg"),
    ),
    (
        "icons/symbols/testtube-2.svg",
        include_bytes!("../../../assets/icons/symbols/testtube-2.svg"),
    ),
    (
        "icons/symbols/trash.svg",
        include_bytes!("../../../assets/icons/symbols/trash.svg"),
    ),
    (
        "icons/symbols/xmark-circle-fill.svg",
        include_bytes!("../../../assets/icons/symbols/xmark-circle-fill.svg"),
    ),
    (
        "icons/symbols/xmark.svg",
        include_bytes!("../../../assets/icons/symbols/xmark.svg"),
    ),
    (
        "icons/languages/bash.svg",
        include_bytes!("../../../assets/icons/languages/bash.svg"),
    ),
    (
        "icons/languages/c.svg",
        include_bytes!("../../../assets/icons/languages/c.svg"),
    ),
    (
        "icons/languages/cpp.svg",
        include_bytes!("../../../assets/icons/languages/cpp.svg"),
    ),
    (
        "icons/languages/css.svg",
        include_bytes!("../../../assets/icons/languages/css.svg"),
    ),
    (
        "icons/languages/go.svg",
        include_bytes!("../../../assets/icons/languages/go.svg"),
    ),
    (
        "icons/languages/html.svg",
        include_bytes!("../../../assets/icons/languages/html.svg"),
    ),
    (
        "icons/languages/javascript.svg",
        include_bytes!("../../../assets/icons/languages/javascript.svg"),
    ),
    (
        "icons/languages/json.svg",
        include_bytes!("../../../assets/icons/languages/json.svg"),
    ),
    (
        "icons/languages/python.svg",
        include_bytes!("../../../assets/icons/languages/python.svg"),
    ),
    (
        "icons/languages/rust.svg",
        include_bytes!("../../../assets/icons/languages/rust.svg"),
    ),
    (
        "icons/languages/toml.svg",
        include_bytes!("../../../assets/icons/languages/toml.svg"),
    ),
    (
        "icons/languages/typescript.svg",
        include_bytes!("../../../assets/icons/languages/typescript.svg"),
    ),
    (
        "icons/providers/claude.svg",
        include_bytes!("../../../assets/icons/providers/claude.svg"),
    ),
    (
        "icons/providers/cohere.svg",
        include_bytes!("../../../assets/icons/providers/cohere.svg"),
    ),
    (
        "icons/providers/custom.svg",
        include_bytes!("../../../assets/icons/providers/custom.svg"),
    ),
    (
        "icons/providers/deepseek.svg",
        include_bytes!("../../../assets/icons/providers/deepseek.svg"),
    ),
    (
        "icons/providers/ernie.svg",
        include_bytes!("../../../assets/icons/providers/ernie.svg"),
    ),
    (
        "icons/providers/gemini.svg",
        include_bytes!("../../../assets/icons/providers/gemini.svg"),
    ),
    (
        "icons/providers/gemma.svg",
        include_bytes!("../../../assets/icons/providers/gemma.svg"),
    ),
    (
        "icons/providers/glm.svg",
        include_bytes!("../../../assets/icons/providers/glm.svg"),
    ),
    (
        "icons/providers/gpt.svg",
        include_bytes!("../../../assets/icons/providers/gpt.svg"),
    ),
    (
        "icons/providers/grok.svg",
        include_bytes!("../../../assets/icons/providers/grok.svg"),
    ),
    (
        "icons/providers/llama.svg",
        include_bytes!("../../../assets/icons/providers/llama.svg"),
    ),
    (
        "icons/providers/minimax.svg",
        include_bytes!("../../../assets/icons/providers/minimax.svg"),
    ),
    (
        "icons/providers/mistral.svg",
        include_bytes!("../../../assets/icons/providers/mistral.svg"),
    ),
    (
        "icons/providers/nova.svg",
        include_bytes!("../../../assets/icons/providers/nova.svg"),
    ),
    (
        "icons/providers/perplexity.svg",
        include_bytes!("../../../assets/icons/providers/perplexity.svg"),
    ),
    (
        "icons/providers/phi.svg",
        include_bytes!("../../../assets/icons/providers/phi.svg"),
    ),
    (
        "icons/providers/qwen.svg",
        include_bytes!("../../../assets/icons/providers/qwen.svg"),
    ),
    (
        "icons/git/added_success.svg",
        include_bytes!("../../../assets/icons/git/added_success.svg"),
    ),
    (
        "icons/git/ahead_info.svg",
        include_bytes!("../../../assets/icons/git/ahead_info.svg"),
    ),
    (
        "icons/git/behind_warning.svg",
        include_bytes!("../../../assets/icons/git/behind_warning.svg"),
    ),
    (
        "icons/git/blame_muted.svg",
        include_bytes!("../../../assets/icons/git/blame_muted.svg"),
    ),
    (
        "icons/git/blame_warning.svg",
        include_bytes!("../../../assets/icons/git/blame_warning.svg"),
    ),
    (
        "icons/git/branch_muted.svg",
        include_bytes!("../../../assets/icons/git/branch_muted.svg"),
    ),
    (
        "icons/git/checkout_muted.svg",
        include_bytes!("../../../assets/icons/git/checkout_muted.svg"),
    ),
    (
        "icons/git/clean_success.svg",
        include_bytes!("../../../assets/icons/git/clean_success.svg"),
    ),
    (
        "icons/git/clone_muted.svg",
        include_bytes!("../../../assets/icons/git/clone_muted.svg"),
    ),
    (
        "icons/git/code-review_info.svg",
        include_bytes!("../../../assets/icons/git/code-review_info.svg"),
    ),
    (
        "icons/git/code-review_muted.svg",
        include_bytes!("../../../assets/icons/git/code-review_muted.svg"),
    ),
    (
        "icons/git/compare_error.svg",
        include_bytes!("../../../assets/icons/git/compare_error.svg"),
    ),
    (
        "icons/git/compare_muted.svg",
        include_bytes!("../../../assets/icons/git/compare_muted.svg"),
    ),
    (
        "icons/git/compare_success.svg",
        include_bytes!("../../../assets/icons/git/compare_success.svg"),
    ),
    (
        "icons/git/conflict_warning.svg",
        include_bytes!("../../../assets/icons/git/conflict_warning.svg"),
    ),
    (
        "icons/git/deleted_error.svg",
        include_bytes!("../../../assets/icons/git/deleted_error.svg"),
    ),
    (
        "icons/git/fork_muted.svg",
        include_bytes!("../../../assets/icons/git/fork_muted.svg"),
    ),
    (
        "icons/git/ignored_muted.svg",
        include_bytes!("../../../assets/icons/git/ignored_muted.svg"),
    ),
    (
        "icons/git/inline-diff_error.svg",
        include_bytes!("../../../assets/icons/git/inline-diff_error.svg"),
    ),
    (
        "icons/git/inline-diff_muted.svg",
        include_bytes!("../../../assets/icons/git/inline-diff_muted.svg"),
    ),
    (
        "icons/git/inline-diff_success.svg",
        include_bytes!("../../../assets/icons/git/inline-diff_success.svg"),
    ),
    (
        "icons/git/log_muted.svg",
        include_bytes!("../../../assets/icons/git/log_muted.svg"),
    ),
    (
        "icons/git/merge_success.svg",
        include_bytes!("../../../assets/icons/git/merge_success.svg"),
    ),
    (
        "icons/git/modified_warning.svg",
        include_bytes!("../../../assets/icons/git/modified_warning.svg"),
    ),
    (
        "icons/git/pull-request_success.svg",
        include_bytes!("../../../assets/icons/git/pull-request_success.svg"),
    ),
    (
        "icons/git/pull_info.svg",
        include_bytes!("../../../assets/icons/git/pull_info.svg"),
    ),
    (
        "icons/git/push_success.svg",
        include_bytes!("../../../assets/icons/git/push_success.svg"),
    ),
    (
        "icons/git/rebase_ai.svg",
        include_bytes!("../../../assets/icons/git/rebase_ai.svg"),
    ),
    (
        "icons/git/rebase_muted.svg",
        include_bytes!("../../../assets/icons/git/rebase_muted.svg"),
    ),
    (
        "icons/git/remote_muted.svg",
        include_bytes!("../../../assets/icons/git/remote_muted.svg"),
    ),
    (
        "icons/git/staged_success.svg",
        include_bytes!("../../../assets/icons/git/staged_success.svg"),
    ),
    (
        "icons/git/staging_muted.svg",
        include_bytes!("../../../assets/icons/git/staging_muted.svg"),
    ),
    (
        "icons/git/staging_success.svg",
        include_bytes!("../../../assets/icons/git/staging_success.svg"),
    ),
    (
        "icons/git/stash_muted.svg",
        include_bytes!("../../../assets/icons/git/stash_muted.svg"),
    ),
    (
        "icons/git/tag_muted.svg",
        include_bytes!("../../../assets/icons/git/tag_muted.svg"),
    ),
    (
        "icons/git/untracked_muted.svg",
        include_bytes!("../../../assets/icons/git/untracked_muted.svg"),
    ),
    (
        "icons/dev-tools/agent_ai.svg",
        include_bytes!("../../../assets/icons/dev-tools/agent_ai.svg"),
    ),
    (
        "icons/dev-tools/api_info.svg",
        include_bytes!("../../../assets/icons/dev-tools/api_info.svg"),
    ),
    (
        "icons/dev-tools/breadcrumb_info.svg",
        include_bytes!("../../../assets/icons/dev-tools/breadcrumb_info.svg"),
    ),
    (
        "icons/dev-tools/breadcrumb_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/breadcrumb_muted.svg"),
    ),
    (
        "icons/dev-tools/cache_warning.svg",
        include_bytes!("../../../assets/icons/dev-tools/cache_warning.svg"),
    ),
    (
        "icons/dev-tools/chain_ai.svg",
        include_bytes!("../../../assets/icons/dev-tools/chain_ai.svg"),
    ),
    (
        "icons/dev-tools/ci-cd_success.svg",
        include_bytes!("../../../assets/icons/dev-tools/ci-cd_success.svg"),
    ),
    (
        "icons/dev-tools/container_info.svg",
        include_bytes!("../../../assets/icons/dev-tools/container_info.svg"),
    ),
    (
        "icons/dev-tools/context_ai.svg",
        include_bytes!("../../../assets/icons/dev-tools/context_ai.svg"),
    ),
    (
        "icons/dev-tools/debug_error.svg",
        include_bytes!("../../../assets/icons/dev-tools/debug_error.svg"),
    ),
    (
        "icons/dev-tools/deploy_success.svg",
        include_bytes!("../../../assets/icons/dev-tools/deploy_success.svg"),
    ),
    (
        "icons/dev-tools/embedding_ai.svg",
        include_bytes!("../../../assets/icons/dev-tools/embedding_ai.svg"),
    ),
    (
        "icons/dev-tools/environment_info.svg",
        include_bytes!("../../../assets/icons/dev-tools/environment_info.svg"),
    ),
    (
        "icons/dev-tools/environment_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/environment_muted.svg"),
    ),
    (
        "icons/dev-tools/extension_ai.svg",
        include_bytes!("../../../assets/icons/dev-tools/extension_ai.svg"),
    ),
    (
        "icons/dev-tools/find-replace_info.svg",
        include_bytes!("../../../assets/icons/dev-tools/find-replace_info.svg"),
    ),
    (
        "icons/dev-tools/fine-tune_warning.svg",
        include_bytes!("../../../assets/icons/dev-tools/fine-tune_warning.svg"),
    ),
    (
        "icons/dev-tools/function-call_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/function-call_muted.svg"),
    ),
    (
        "icons/dev-tools/function-call_warning.svg",
        include_bytes!("../../../assets/icons/dev-tools/function-call_warning.svg"),
    ),
    (
        "icons/dev-tools/guardrail_success.svg",
        include_bytes!("../../../assets/icons/dev-tools/guardrail_success.svg"),
    ),
    (
        "icons/dev-tools/health_error.svg",
        include_bytes!("../../../assets/icons/dev-tools/health_error.svg"),
    ),
    (
        "icons/dev-tools/keyboard_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/keyboard_muted.svg"),
    ),
    (
        "icons/dev-tools/logs_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/logs_muted.svg"),
    ),
    (
        "icons/dev-tools/logs_warning.svg",
        include_bytes!("../../../assets/icons/dev-tools/logs_warning.svg"),
    ),
    (
        "icons/dev-tools/memory_ai.svg",
        include_bytes!("../../../assets/icons/dev-tools/memory_ai.svg"),
    ),
    (
        "icons/dev-tools/minimap_info.svg",
        include_bytes!("../../../assets/icons/dev-tools/minimap_info.svg"),
    ),
    (
        "icons/dev-tools/minimap_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/minimap_muted.svg"),
    ),
    (
        "icons/dev-tools/model_ai.svg",
        include_bytes!("../../../assets/icons/dev-tools/model_ai.svg"),
    ),
    (
        "icons/dev-tools/monitor_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/monitor_muted.svg"),
    ),
    (
        "icons/dev-tools/monitor_success.svg",
        include_bytes!("../../../assets/icons/dev-tools/monitor_success.svg"),
    ),
    (
        "icons/dev-tools/orchestrator_ai.svg",
        include_bytes!("../../../assets/icons/dev-tools/orchestrator_ai.svg"),
    ),
    (
        "icons/dev-tools/orchestrator_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/orchestrator_muted.svg"),
    ),
    (
        "icons/dev-tools/palette_info.svg",
        include_bytes!("../../../assets/icons/dev-tools/palette_info.svg"),
    ),
    (
        "icons/dev-tools/palette_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/palette_muted.svg"),
    ),
    (
        "icons/dev-tools/pipeline_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/pipeline_muted.svg"),
    ),
    (
        "icons/dev-tools/pipeline_success.svg",
        include_bytes!("../../../assets/icons/dev-tools/pipeline_success.svg"),
    ),
    (
        "icons/dev-tools/pipeline_warning.svg",
        include_bytes!("../../../assets/icons/dev-tools/pipeline_warning.svg"),
    ),
    (
        "icons/dev-tools/prompt_ai.svg",
        include_bytes!("../../../assets/icons/dev-tools/prompt_ai.svg"),
    ),
    (
        "icons/dev-tools/queue_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/queue_muted.svg"),
    ),
    (
        "icons/dev-tools/rag_info.svg",
        include_bytes!("../../../assets/icons/dev-tools/rag_info.svg"),
    ),
    (
        "icons/dev-tools/rag_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/rag_muted.svg"),
    ),
    (
        "icons/dev-tools/rollback_error.svg",
        include_bytes!("../../../assets/icons/dev-tools/rollback_error.svg"),
    ),
    (
        "icons/dev-tools/scale_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/scale_muted.svg"),
    ),
    (
        "icons/dev-tools/scale_success.svg",
        include_bytes!("../../../assets/icons/dev-tools/scale_success.svg"),
    ),
    (
        "icons/dev-tools/search_info.svg",
        include_bytes!("../../../assets/icons/dev-tools/search_info.svg"),
    ),
    (
        "icons/dev-tools/secret_warning.svg",
        include_bytes!("../../../assets/icons/dev-tools/secret_warning.svg"),
    ),
    (
        "icons/dev-tools/sidebar_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/sidebar_muted.svg"),
    ),
    (
        "icons/dev-tools/snippet_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/snippet_muted.svg"),
    ),
    (
        "icons/dev-tools/snippet_warning.svg",
        include_bytes!("../../../assets/icons/dev-tools/snippet_warning.svg"),
    ),
    (
        "icons/dev-tools/split-view_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/split-view_muted.svg"),
    ),
    (
        "icons/dev-tools/streaming_info.svg",
        include_bytes!("../../../assets/icons/dev-tools/streaming_info.svg"),
    ),
    (
        "icons/dev-tools/tab_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/tab_muted.svg"),
    ),
    (
        "icons/dev-tools/token_warning.svg",
        include_bytes!("../../../assets/icons/dev-tools/token_warning.svg"),
    ),
    (
        "icons/dev-tools/tool-use_warning.svg",
        include_bytes!("../../../assets/icons/dev-tools/tool-use_warning.svg"),
    ),
    (
        "icons/dev-tools/webhook_info.svg",
        include_bytes!("../../../assets/icons/dev-tools/webhook_info.svg"),
    ),
    (
        "icons/dev-tools/webhook_muted.svg",
        include_bytes!("../../../assets/icons/dev-tools/webhook_muted.svg"),
    ),
];
