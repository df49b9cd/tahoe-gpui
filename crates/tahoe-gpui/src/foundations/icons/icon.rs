//! Icon component with SVG rendering and Unicode fallback.

use crate::foundations::surface_scope;
use crate::foundations::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::{App, FontWeight, Hsla, Pixels, Window, div};

use super::assets::RenderStrategy;
use super::layers;
use super::names::IconName;
use crate::foundations::typography::TextStyle;

/// Visual style for icon rendering.
///
/// This enum is a tahoe-gpui *approximation* of Apple's vibrancy effect on
/// Liquid Glass surfaces — it is not a HIG concept. Per HIG
/// §Foundations / Materials (`docs/hig/foundations.md:1045`), icons placed
/// on a Liquid Glass surface inherit vibrancy automatically. Since GPUI
/// cannot composite `NSVisualEffectView` vibrancy onto an SVG icon layer,
/// this crate approximates the visual by swapping color tokens (glass uses
/// `theme.glass.icon_*`; standard uses `theme.text_muted`) and stroke
/// widths (glass 1.5pt, standard 1.2pt).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum IconStyle {
    /// Resolve against the ambient surface scope: `LiquidGlass` when the
    /// icon sits inside a [`super::GlassSurfaceScope`] (set by the
    /// `*_scoped` helpers in `foundations::materials` and by glass-aware
    /// components such as [`super::GlassIconTile`]), otherwise `Standard`.
    #[default]
    Auto,
    /// Standard flat icons (stroke-width 1.2, muted text color).
    Standard,
    /// Liquid Glass vibrancy approximation (stroke-width 1.5, glass pastel
    /// tokens `theme.glass.icon_*`). Use on Liquid Glass surfaces only.
    LiquidGlass,
}

impl IconStyle {
    /// Resolve `Auto` against the ambient surface scope; pass-through for
    /// explicit styles.
    ///
    /// Consults [`surface_scope::is_on_glass_surface`], which reflects
    /// whether an ancestor element declared itself a Liquid Glass surface
    /// via [`super::GlassSurfaceScope`]. Resolution no longer depends on
    /// the active theme — vibrancy is a surface concern, not a theme one.
    pub fn resolve(self) -> IconStyle {
        match self {
            IconStyle::Auto => {
                if surface_scope::is_on_glass_surface() {
                    IconStyle::LiquidGlass
                } else {
                    IconStyle::Standard
                }
            }
            other => other,
        }
    }
}

/// Scale multiplier for icon rendering.
///
/// Per HIG SF Symbols scale is defined relative to the cap height
/// of the adjacent text (not a fixed pixel grid). `multiplier()` returns
/// the relative factor; `size_for_text_style()` resolves an absolute
/// pixel size against a specific [`TextStyle`], which is what `Icon`
/// calls at render time when the user supplies a text style via
/// [`Icon::match_text_style`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum IconScale {
    /// ~0.75× cap height — for dense UIs / badges.
    Small,
    /// Matches adjacent text cap height (default).
    #[default]
    Medium,
    /// ~1.25× cap height — for emphasis / oversized toolbar icons.
    Large,
}

impl IconScale {
    /// Returns the multiplier applied to the cap height of adjacent text.
    pub fn multiplier(self) -> f32 {
        match self {
            Self::Small => 0.75,
            Self::Medium => 1.0,
            Self::Large => 1.25,
        }
    }

    /// Resolve an absolute pixel size for this scale against a given
    /// text style's cap height.
    ///
    /// SF Pro's cap height is ~70 % of its point size. This ratio is used
    /// to derive the symbol's pixel size so a `Medium` icon next to
    /// `TextStyle::Body` (13 pt) renders at 13×0.70 = ~9.1 pt cap
    /// equivalent, matching how SF Symbols scales next to body text.
    pub fn size_for_text_style(self, text_style: TextStyle) -> gpui::Pixels {
        let attrs = text_style.attrs();
        // HIG: SF Symbols are sized by the typesetter to match text; the
        // practical output is `point_size × cap_ratio × scale_multiplier`
        // with SF Pro's cap ratio ≈ 0.70.
        const SF_PRO_CAP_HEIGHT_RATIO: f32 = 0.70;
        let pt = f32::from(attrs.size);
        // The cap-height-scaled icon should still visually match the text
        // point size at Medium scale, so we render at the text's point
        // size times the scale multiplier. The cap ratio above is
        // documented for hosts that want to lay out glyphs on a strict
        // cap baseline.
        let _ = SF_PRO_CAP_HEIGHT_RATIO;
        gpui::px(pt * self.multiplier())
    }
}

/// Rendering mode for SF Symbols / multi-layer icons.
///
/// Maps to HIG's four canonical SF Symbols rendering modes plus a
/// fifth Gradient mode introduced in SF Symbols 7 (see issue #139 finding
/// #17). Each mode picks a different strategy when the icon has more than
/// one semantic layer; monochrome icons render identically under all
/// modes.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum IconRenderMode {
    /// Single color for all layers — toolbar / navigation default.
    #[default]
    Monochrome,
    /// Single color with opacity-graded layers: primary=1.0, secondary=0.50,
    /// tertiary=0.25. Communicates depth without changing hue.
    Hierarchical,
    /// Two or three explicit colors, one per layer. `palette` holds the
    /// color for each layer index. If the array is shorter than the icon's
    /// layer count, later layers inherit the last supplied color.
    Palette {
        /// Per-layer color palette. Index 0 is the primary layer.
        palette: &'static [Hsla],
    },
    /// Intrinsic, symbol-defined colors (e.g. `trash.slash` renders red
    /// regardless of caller color). This crate implements Multicolor as
    /// "use the per-layer semantic role colors from the theme" — the
    /// closest analog since GPUI can't read SF Symbols' intrinsic palette.
    MultiColor,
    /// Variable color: a single fill whose opacity reflects `progress`
    /// (0.0 … 1.0). Used to communicate strength/capacity on symbols like
    /// `speaker.wave.3`, `wifi`, or `battery.*` per SF Symbols 6.
    /// HIG: "Use variable color to communicate change — don't use it to
    /// communicate depth."
    VariableColor {
        /// 0.0 = dim, 1.0 = full. Clamped at render time.
        progress: f32,
    },
    /// SF Symbols 7 gradient render: a smooth linear gradient from
    /// `source` at the top-left to a derived darker stop at the
    /// bottom-right. When `source` is `None`, the icon's caller color is
    /// used as the gradient's source.
    Gradient {
        /// Optional gradient source color. `None` → use caller color.
        source: Option<Hsla>,
    },
}

/// An icon component with SVG rendering and Unicode fallback.
///
/// When [`super::EmbeddedIconAssets`] is registered as the app's asset source,
/// icons render as GPU-accelerated SVGs. Otherwise, they fall back to
/// Unicode symbol placeholders.
///
/// # Example
/// ```ignore
/// Icon::new(IconName::Check).size(px(14.0)).color(theme.success)
/// ```
#[derive(IntoElement)]
pub struct Icon {
    pub(crate) name: IconName,
    pub(crate) size: Option<Pixels>,
    pub(crate) color: Option<Hsla>,
    pub(crate) style: IconStyle,
    pub(crate) scale: Option<IconScale>,
    pub(crate) render_mode: Option<IconRenderMode>,
    pub(crate) weight: Option<FontWeight>,
    /// Adjacent text style — when set, `size` is derived from the text
    /// style's cap height via [`IconScale::size_for_text_style`] instead
    /// of the theme's fixed `icon_size`. Matches SF Symbols' "match
    /// surrounding text" behavior.
    pub(crate) match_text_style: Option<TextStyle>,
    /// Optional vertical baseline offset in points. Used by optical
    /// alignment utilities (see [`Icon::align_baseline`]).
    pub(crate) baseline_offset: Option<Pixels>,
    /// When true, icons with `IconLayoutBehavior::Directional` are
    /// horizontally mirrored under RTL themes. Defaults to true so
    /// arrow / chevron glyphs in toolbars and nav bars follow the
    /// reading direction automatically.
    pub(crate) follow_layout_direction: bool,
}

impl Icon {
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            size: None,
            color: None,
            style: IconStyle::Auto,
            scale: None,
            render_mode: None,
            weight: None,
            match_text_style: None,
            baseline_offset: None,
            follow_layout_direction: true,
        }
    }

    /// Opt out of the automatic RTL mirror for directional glyphs.
    ///
    /// By default, [`Icon`] consults [`IconName::layout_behavior()`] in
    /// its render path: directional glyphs (`ChevronRight`, arrows,
    /// `Send`) flip horizontally when the theme is RTL. Pass `false`
    /// to keep the glyph upright regardless of reading direction —
    /// use for glyphs that read the same in both orientations (e.g.
    /// an arrow indicating physical down-motion rather than
    /// forward-in-reading-order). Finding 23 in
    /// the Zed cross-reference audit tracks this wiring.
    pub fn follow_layout_direction(mut self, follow: bool) -> Self {
        self.follow_layout_direction = follow;
        self
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = Some(size);
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    pub fn style(mut self, style: IconStyle) -> Self {
        self.style = style;
        self
    }

    pub fn scale(mut self, scale: IconScale) -> Self {
        self.scale = Some(scale);
        self
    }

    pub fn render_mode(mut self, render_mode: IconRenderMode) -> Self {
        self.render_mode = Some(render_mode);
        self
    }

    /// Set the icon stroke weight.
    ///
    /// # Pending GPUI support
    ///
    /// GPUI's `svg()` element does not currently expose a per-instance
    /// stroke-width setter, so this builder stores the value but does
    /// not affect visual rendering today — the weight baked into the
    /// SVG asset wins. The weight is still carried through the render
    /// pipeline so that once GPUI lands stroke-width, every caller of
    /// `weight(...)` picks up the behaviour without a code change, and
    /// so host apps on the native AppKit backend can map the weight
    /// onto `NSImage.symbolConfiguration(weight:)` in the meantime.
    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Configure the icon to track the cap height of an adjacent text
    /// style. Overrides any explicit `size()` unless `size()` is called
    /// afterwards. HIG: "symbols match the weight of adjacent text when
    /// using the system font."
    pub fn match_text_style(mut self, text_style: TextStyle) -> Self {
        self.match_text_style = Some(text_style);
        self
    }

    /// Apply a vertical baseline offset in points. Positive moves the
    /// icon down, negative moves it up. Used to optically center the
    /// glyph on the adjacent text baseline; see
    /// [`optical_baseline_offset`] for the HIG-derived default offsets.
    pub fn align_baseline(mut self, offset: Pixels) -> Self {
        self.baseline_offset = Some(offset);
        self
    }

    /// Whether this icon will be horizontally mirrored under the given
    /// theme. True iff the caller has not opted out via
    /// [`Icon::follow_layout_direction`], the theme reports an RTL layout,
    /// and the symbol's [`IconName::layout_behavior`] is `Directional`.
    /// Matches the predicate used by the render path — callers that need
    /// to mirror surrounding geometry (e.g. custom drop-shadow offsets)
    /// can consult this instead of re-deriving the logic.
    pub fn would_flip_horizontally(&self, theme: &crate::foundations::theme::TahoeTheme) -> bool {
        self.follow_layout_direction
            && theme.is_rtl()
            && matches!(
                self.name.layout_behavior(),
                super::IconLayoutBehavior::Directional
            )
    }

    /// Resolved stroke width in points for this icon.
    ///
    /// - Explicit weight via `Icon::weight()` takes priority.
    /// - Otherwise uses the style default: 1.2 pt for Standard, 1.5 pt
    ///   for Liquid Glass.
    ///
    /// Intended for consumers that need to introspect the computed
    /// stroke width (tests, custom SVG renderers). Production
    /// `Icon::render` does not yet forward this into GPUI's svg element
    /// because upstream does not expose per-SVG stroke-width, but the
    /// value is authoritative and will be wired through when that API
    /// lands.
    pub fn resolved_stroke_width(&self) -> f32 {
        stroke_width_for(self.weight, self.style.resolve())
    }

    /// Convenience: turn this icon into a continuously rotating
    /// [`super::AnimatedIcon`] at `turns_per_second` revolutions/sec.
    ///
    /// Mirrors Zed's `Icon::with_rotate_animation(turns_per_second)` shorthand
    /// (see `crates/ui/src/components/button/button.rs`). Without this, every
    /// caller has to construct `AnimatedIcon::new(id, name,
    /// IconAnimation::Spin { duration: … })` manually and convert
    /// `turns_per_second` into a duration by hand.
    ///
    /// The returned element carries the current icon's `size` and `color` so
    /// the builder chain on `Icon` is preserved end-to-end. Other icon
    /// modifiers (`style`, `scale`, `render_mode`, `weight`,
    /// `match_text_style`, `baseline_offset`) do not apply — `AnimatedIcon`
    /// reads `IconName` directly and relies on the icon's default rendering
    /// strategy. Callers needing those knobs must still use `AnimatedIcon`
    /// explicitly.
    ///
    /// ```ignore
    /// Icon::new(IconName::LoadCircle)
    ///     .size(px(16.0))
    ///     .with_rotate_animation("spinner", 2.0)
    /// ```
    pub fn with_rotate_animation(
        self,
        id: impl Into<gpui::ElementId>,
        turns_per_second: f32,
    ) -> super::AnimatedIcon {
        use std::time::Duration;
        // Guard against 0 or negative tps: fall back to one rotation per
        // second so `Animation` never sees a zero-duration divisor (GPUI
        // divides by the duration when interpolating `delta`).
        let tps = if turns_per_second > 0.0 {
            turns_per_second
        } else {
            1.0
        };
        let duration = Duration::from_secs_f32(1.0 / tps);

        let mut anim =
            super::AnimatedIcon::new(id, self.name, super::IconAnimation::Spin { duration });
        if let Some(size) = self.size {
            anim = anim.size(size);
        }
        if let Some(color) = self.color {
            anim = anim.color(color);
        }
        anim
    }
}

/// Compute the stroke width for an already-resolved `IconStyle`, honoring
/// an explicit weight override when set. Shared between
/// `Icon::resolved_stroke_width` (introspection) and `Icon::render`
/// (actual paint pipeline) so both paths stay in lockstep without a
/// double surface-scope lookup per frame.
fn stroke_width_for(weight: Option<FontWeight>, resolved_style: IconStyle) -> f32 {
    if let Some(w) = weight {
        return super::weight_to_stroke_width(w);
    }
    match resolved_style {
        IconStyle::LiquidGlass => 1.5,
        _ => 1.2,
    }
}

/// Hierarchical opacity for a given layer index within a multi-layer icon.
/// primary (0) = 1.0, secondary (1) = 0.50, tertiary (2+) = 0.25.
pub(crate) fn hierarchical_opacity(layer_index: usize) -> f32 {
    match layer_index {
        0 => 1.0,
        1 => 0.50,
        _ => 0.25,
    }
}

/// HIG-derived optical baseline offset for icons adjacent to a given text
/// style, in points.
///
/// the HIG custom-icon guidance says "optically center" an icon on
/// the adjacent text's baseline — not the mathematical center of the
/// icon's bounding box. Because SF Symbols ship with their vertical
/// alignment pre-tuned, the offset is typically small: the returned
/// value is the vertical nudge applied by `Icon::render` when the
/// caller invokes [`Icon::match_text_style`] without an explicit
/// `align_baseline`.
///
/// Values are derived from SF Pro's cap-height vs. x-height ratio
/// (cap height ≈ 0.70 × point size, x-height ≈ 0.52 × point size), with
/// the symbol centered on the x-height midpoint so a Body-sized
/// trash icon sits visually where the lowercase letters do rather than
/// floating above them.
pub fn optical_baseline_offset(text_style: TextStyle) -> Pixels {
    let pt = f32::from(text_style.attrs().size);
    // The midpoint of the x-height is at ~0.26 × point size above the
    // baseline. The icon's geometric center is at ~0.5 × icon_size above
    // the baseline. The difference per SF Pro layout is ~0.10 × point
    // size — so a Body-sized icon gets nudged down by ~1.3 pt.
    gpui::px(pt * 0.10)
}

impl RenderOnce for Icon {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let resolved_style = self.style.resolve();
        let is_glass = resolved_style == IconStyle::LiquidGlass;
        // Default color selection. Under `Reduce Transparency` the glass
        // chrome becomes opaque; the icon tokens (`glass.icon_*` pastels)
        // are tuned for translucent surfaces, so fall back to
        // `theme.text_muted` in that mode to keep contrast predictable on
        // the opaque fallback fill.
        let reduce_transparency = theme.accessibility_mode.reduce_transparency();
        let color = self.color.unwrap_or(if is_glass && !reduce_transparency {
            theme.glass.icon_text
        } else {
            theme.text_muted
        });

        // RTL mirror for directional glyphs. Stays off unless the caller
        // opted in (default true) *and* the theme reports an RTL layout
        // *and* the symbol itself declares `IconLayoutBehavior::Directional`.
        // Localised variants are out of scope for the bare `Icon`; callers
        // who need those can inspect `IconName::layout_behavior()` themselves.
        let should_flip_directional = self.would_flip_horizontally(theme);

        let scale = self.scale.unwrap_or_default();
        // Source of truth for pixel size, in priority order:
        //   1. explicit .size()
        //   2. .match_text_style(ts) → cap-height-relative
        //   3. theme.icon_size × scale multiplier
        let size = if let Some(explicit) = self.size {
            gpui::px(f32::from(explicit) * scale.multiplier())
        } else if let Some(ts) = self.match_text_style {
            scale.size_for_text_style(ts)
        } else {
            gpui::px(f32::from(theme.icon_size) * scale.multiplier())
        };

        // Optical baseline offset: user-supplied explicit offset wins;
        // otherwise when paired with a text style, derive the HIG-default
        // nudge so the glyph sits on the adjacent text's baseline.
        let baseline_offset = self
            .baseline_offset
            .or_else(|| self.match_text_style.map(optical_baseline_offset));

        // Compute stroke width. GPUI does not yet support per-SVG
        // stroke-width, so the value is consumed only by the icon's own
        // introspection API today; the computation is cheap and kept
        // in the render pipeline so it goes live the moment GPUI lands
        // the feature. Uses the already-resolved style so the
        // surface-scope thread-local read happens exactly once per render.
        let _stroke_width = stroke_width_for(self.weight, resolved_style);

        let render_mode = self.render_mode.unwrap_or_default();

        // Standard and Liquid Glass themes share the same asset set; the
        // glass appearance comes from layer tinting below, not a separate
        // SVG. See `assets/icons/NOTICE.md`.
        let strategy = self.name.render_strategy();

        if let Some(strategy) = strategy {
            let element: gpui::AnyElement = match strategy {
                RenderStrategy::Monochrome(path) => layers::render_monochrome(
                    path,
                    size,
                    color,
                    render_mode,
                    should_flip_directional,
                )
                .into_any_element(),
                RenderStrategy::MultiColor(layer_list) => match render_mode {
                    IconRenderMode::Hierarchical if layer_list.len() > 1 => {
                        layers::render_multi_color_layers_hierarchical(
                            layer_list,
                            size,
                            self.color,
                            is_glass,
                            should_flip_directional,
                            window,
                            cx,
                        )
                        .into_any_element()
                    }
                    IconRenderMode::Palette { palette } => {
                        layers::render_multi_color_layers_palette(
                            layer_list,
                            size,
                            palette,
                            should_flip_directional,
                            window,
                            cx,
                        )
                        .into_any_element()
                    }
                    IconRenderMode::VariableColor { progress } => {
                        layers::render_multi_color_layers_variable(
                            layer_list,
                            size,
                            self.color,
                            progress,
                            is_glass,
                            should_flip_directional,
                            window,
                            cx,
                        )
                        .into_any_element()
                    }
                    IconRenderMode::Gradient { source } => {
                        // Fill-based gradient is applied to a single layer;
                        // for multi-layer icons we render the primary layer
                        // with the gradient and other layers with the
                        // semantic palette behind it.
                        layers::render_multi_color_layers_gradient(
                            layer_list,
                            size,
                            source.or(self.color),
                            color,
                            is_glass,
                            should_flip_directional,
                            window,
                            cx,
                        )
                        .into_any_element()
                    }
                    // MultiColor + Monochrome + Hierarchical-with-one-layer
                    // all fall through to the semantic-role rendering which
                    // is the closest analog to SF Symbols' intrinsic
                    // palette.
                    _ => {
                        if is_glass {
                            layers::render_multi_color_layers_glass(
                                layer_list,
                                size,
                                self.color,
                                should_flip_directional,
                                window,
                                cx,
                            )
                            .into_any_element()
                        } else {
                            layers::render_multi_color_layers(
                                layer_list,
                                size,
                                self.color,
                                should_flip_directional,
                                window,
                                cx,
                            )
                            .into_any_element()
                        }
                    }
                },
            };
            // RTL mirror for directional glyphs is applied at each inner
            // `svg()` via `Transformation::scale(size(-1, 1))`. GPUI mirrors
            // around `bounds.center()` (see `gpui::elements::svg::Svg::paint`),
            // so stacking multiple flipped layers produces the same visual as
            // flipping the stack as a unit. `follow_layout_direction(false)`
            // opts out.
            return apply_baseline(element, baseline_offset);
        }

        // Fallback: Unicode symbol. Unicode glyphs are already rendered
        // by the system's text engine which applies its own bidi
        // handling, so the `should_flip_directional` signal is
        // intentionally ignored in this branch.
        let _ = should_flip_directional;
        apply_baseline(
            div()
                .size(size)
                .text_color(color)
                .text_size(size)
                .flex()
                .items_center()
                .justify_center()
                .child(self.name.symbol())
                .into_any_element(),
            baseline_offset,
        )
    }
}

/// Wrap an element in a baseline-shifting `div` if `offset` is Some.
/// A `None` offset is the common case and returns the element as-is to
/// avoid gratuitous extra DOM nodes.
fn apply_baseline(element: gpui::AnyElement, offset: Option<Pixels>) -> gpui::AnyElement {
    match offset {
        Some(dy) => div().mt(dy).child(element).into_any_element(),
        None => element,
    }
}
