//! Mermaid diagram rendering.
//!
//! Uses the pure-Rust [`mermaid_rs_renderer`] crate (same dependency as
//! Zed's markdown renderer — no external `mmdc` binary required) to
//! convert Mermaid source to SVG, then rasterizes the SVG via GPUI's
//! [`gpui::SvgRenderer::render_single_frame`] into a cached
//! [`RenderImage`]. The image is displayed inline via `img()`.
//!
//! Rasterization is synchronous on first paint; subsequent paints pull
//! from a module-local `Mutex<HashMap>` keyed by (content hash, dark
//! mode). Typical diagrams render in <100 ms, within a single frame
//! budget on a modern Mac. If rendering fails (invalid syntax, missing
//! fonts), the block falls back to the Mermaid source in a code block
//! with a Copy Mermaid button — the same UX as when the pure-Rust
//! renderer can't handle a directive.
//!
//! Closes #150 findings F24 (misleading success indicator) and F25
//! (hardcoded theme).

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

use crate::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use crate::markdown::code_block::CodeBlockView;
use gpui::prelude::*;
use gpui::{App, ClipboardItem, ImageSource, RenderImage, Window, div, img, px};

/// Cache key combining content hash and dark-mode flag so light/dark
/// appearances do not trample each other's rasterizations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct MermaidCacheKey {
    content_hash: u64,
    dark: bool,
}

type MermaidCache = HashMap<MermaidCacheKey, MermaidRender>;

/// Cached rasterization result. `Ok(image)` means a finished frame;
/// `Err` means the render failed (invalid source, font issues) — the
/// failure is memoized so we don't re-attempt on every frame.
#[derive(Clone)]
enum MermaidRender {
    Ok(Arc<RenderImage>),
    Err,
}

fn cache() -> &'static Mutex<MermaidCache> {
    static CACHE: OnceLock<Mutex<MermaidCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn hash_mermaid(code: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    code.hash(&mut hasher);
    hasher.finish()
}

/// Rasterize `code` to an `Arc<RenderImage>`, consulting and populating
/// the module-local cache. Returns `None` when the Mermaid renderer
/// rejects the source or the SVG cannot be rasterized; callers fall
/// back to displaying the source.
///
/// `scale_factor` controls the internal rasterization scale
/// ([`gpui::SvgRenderer`] multiplies by its own 2x smoothing factor on
/// top). `1.0` produces crisp output at natural size on Retina displays.
fn rasterize_mermaid(
    cx: &mut App,
    code: &str,
    dark: bool,
    scale_factor: f32,
) -> Option<Arc<RenderImage>> {
    let key = MermaidCacheKey {
        content_hash: hash_mermaid(code),
        dark,
    };

    if let Ok(guard) = cache().lock()
        && let Some(entry) = guard.get(&key)
    {
        return match entry {
            MermaidRender::Ok(image) => Some(image.clone()),
            MermaidRender::Err => None,
        };
    }

    let options = mermaid_render_options(dark);
    let svg = match mermaid_rs_renderer::render_with_options(code, options) {
        Ok(svg) => svg,
        Err(_) => {
            if let Ok(mut guard) = cache().lock() {
                guard.insert(key, MermaidRender::Err);
            }
            return None;
        }
    };

    let svg_renderer = cx.svg_renderer();
    let image = match svg_renderer.render_single_frame(svg.as_bytes(), scale_factor) {
        Ok(image) => image,
        Err(_) => {
            if let Ok(mut guard) = cache().lock() {
                guard.insert(key, MermaidRender::Err);
            }
            return None;
        }
    };

    if let Ok(mut guard) = cache().lock() {
        guard.insert(key, MermaidRender::Ok(image.clone()));
    }
    Some(image)
}

/// Build mermaid-rs render options appropriate for the current
/// appearance. Dark mode swaps in foreground colours that read on a
/// dark Liquid Glass surface; light mode uses the crate's default
/// "modern" theme.
fn mermaid_render_options(dark: bool) -> mermaid_rs_renderer::RenderOptions {
    let mut options = mermaid_rs_renderer::RenderOptions::modern();
    if dark {
        // The crate only ships light themes. Override the subset of
        // palette fields that otherwise render as black-on-dark. The
        // colour choices here track the semantic role of each token
        // rather than copying a full external palette.
        options.theme.background = "#1E1E1E".into();
        options.theme.primary_color = "#2F3440".into();
        options.theme.primary_text_color = "#E6E6E6".into();
        options.theme.primary_border_color = "#5C6B82".into();
        options.theme.secondary_color = "#3A4152".into();
        options.theme.tertiary_color = "#2F3440".into();
        options.theme.line_color = "#A8B0C0".into();
        options.theme.text_color = "#E6E6E6".into();
        options.theme.cluster_background = "#2A2E38".into();
        options.theme.cluster_border = "#5C6B82".into();
        options.theme.edge_label_background = "rgba(30, 30, 30, 0.85)".into();
    }
    options
}

/// A Mermaid diagram block.
///
/// Renders the diagram as an inline image when the pure-Rust Mermaid
/// renderer succeeds. On failure — syntax errors, or directives the
/// Rust renderer does not yet support — falls back to the original
/// Mermaid source in a code block with a Copy Mermaid button.
#[derive(IntoElement)]
pub struct MermaidBlock {
    code: String,
}

impl MermaidBlock {
    pub fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }
}

impl RenderOnce for MermaidBlock {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let dark = cx.theme().appearance.is_dark();
        let render_image = rasterize_mermaid(cx, &self.code, dark, 1.0);
        let theme = cx.theme();
        let code_for_copy = self.code.clone();

        match render_image {
            Some(render_image) => div()
                .flex()
                .flex_col()
                .bg(theme.surface)
                .rounded(theme.radius_lg)
                .border_1()
                .border_color(theme.border)
                .overflow_hidden()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(theme.spacing_md)
                        .py(theme.spacing_xs)
                        .border_b_1()
                        .border_color(theme.border)
                        .child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text_muted)
                                .child("Mermaid diagram"),
                        )
                        .child(
                            Button::new("copy-mermaid")
                                .label("Copy Mermaid")
                                .icon(Icon::new(IconName::Copy))
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm)
                                .on_click(move |_, _window, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(
                                        code_for_copy.clone(),
                                    ));
                                }),
                        ),
                )
                .child(
                    div()
                        .w_full()
                        .p(theme.spacing_md)
                        .flex()
                        .justify_center()
                        .child(img(ImageSource::Render(render_image)).max_w_full()),
                )
                .into_any_element(),
            None => fallback_source_view(self.code, theme).into_any_element(),
        }
    }
}

/// Source-code fallback for diagrams the Rust renderer rejects.
/// Matches the previous "no SVG preview" UX: users still get the raw
/// Mermaid source plus a Copy button so they can render elsewhere.
fn fallback_source_view(
    code: String,
    theme: &crate::foundations::theme::TahoeTheme,
) -> impl IntoElement {
    let code_for_copy = code.clone();
    div()
        .flex()
        .flex_col()
        .bg(theme.surface)
        .rounded(theme.radius_lg)
        .border_1()
        .border_color(theme.border)
        .overflow_hidden()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(theme.spacing_md)
                .py(theme.spacing_xs)
                .border_b_1()
                .border_color(theme.border)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(theme.spacing_xs)
                        .child(
                            Icon::new(IconName::AlertTriangle)
                                .size(px(12.0))
                                .color(theme.text_muted),
                        )
                        .child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text_muted)
                                .child("Mermaid source (unsupported diagram syntax)"),
                        ),
                )
                .child(
                    Button::new("copy-mermaid")
                        .label("Copy Mermaid")
                        .icon(Icon::new(IconName::Copy))
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .on_click(move |_, _window, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(code_for_copy.clone()));
                        }),
                ),
        )
        .child(
            CodeBlockView::new(code)
                .language(Some("mermaid".into()))
                .show_header(false),
        )
}

#[cfg(test)]
mod tests {
    use super::{MermaidCacheKey, hash_mermaid, mermaid_render_options};
    use core::prelude::v1::test;

    #[test]
    fn hash_mermaid_is_deterministic() {
        assert_eq!(
            hash_mermaid("flowchart LR; A-->B"),
            hash_mermaid("flowchart LR; A-->B")
        );
    }

    #[test]
    fn hash_mermaid_distinguishes_content() {
        assert_ne!(
            hash_mermaid("flowchart LR; A-->B"),
            hash_mermaid("flowchart LR; A-->C")
        );
    }

    #[test]
    fn cache_keys_distinguish_dark_and_light() {
        let h = hash_mermaid("x");
        let light = MermaidCacheKey {
            content_hash: h,
            dark: false,
        };
        let dark = MermaidCacheKey {
            content_hash: h,
            dark: true,
        };
        assert_ne!(light, dark);
    }

    #[test]
    fn dark_options_override_foreground_tokens() {
        let light = mermaid_render_options(false);
        let dark = mermaid_render_options(true);
        assert_eq!(light.theme.background, "#FFFFFF");
        assert_ne!(light.theme.background, dark.theme.background);
        assert_ne!(light.theme.text_color, dark.theme.text_color);
        assert_ne!(
            light.theme.primary_text_color,
            dark.theme.primary_text_color
        );
    }

    #[test]
    fn mermaid_render_succeeds_for_simple_flowchart() {
        // Sanity check: mermaid-rs-renderer should handle basic LR
        // flowcharts. We don't exercise the GPUI rasterizer here (no
        // window), just the SVG generation step.
        let svg = mermaid_rs_renderer::render("flowchart LR; A-->B-->C");
        assert!(svg.is_ok(), "mermaid render failed: {:?}", svg.err());
        let svg = svg.unwrap();
        assert!(svg.contains("<svg"), "expected SVG output, got: {svg:?}");
    }
}
