//! Animated icon component.
//!
//! Wraps icon SVGs with GPUI-native animations including spin, pulse,
//! shake, bounce, and more. Uses `svg().with_transformation()` for
//! transform-based animations and `div().opacity()` for opacity effects.

use std::time::Duration;

use gpui::prelude::*;
use gpui::{
    Animation, AnimationExt, App, ElementId, Hsla, Pixels, Transformation, Window, div, point, px,
    radians, size as gpui_size, svg,
};

use super::{Icon, IconName, RenderStrategy};
use crate::foundations::theme::ActiveTheme;

/// Animation type for an icon.
#[derive(Debug, Clone)]
pub enum IconAnimation {
    /// Continuous rotation (e.g. loader spinner, Claude spokes, GPT knot).
    Spin { duration: Duration },
    /// Pulsing opacity (e.g. recording indicator, Gemini halves).
    Pulse { duration: Duration },
    /// Horizontal shake (e.g. error alert).
    Shake { duration: Duration },
    /// Heartbeat-style double scale pulse (e.g. health indicator).
    Heartbeat { duration: Duration },
    /// Twinkle scale+opacity (e.g. sparkle).
    Twinkle { duration: Duration },
    /// Gentle rotation oscillation ±angle (e.g. Phi tilt, Qwen tail swing).
    Rock { duration: Duration, degrees: f32 },
    /// Horizontal swim/bob translation (e.g. DeepSeek fish).
    Swim { duration: Duration },
    /// Gentle scale breathing (e.g. Custom LLM, Nova).
    Breath { duration: Duration },
    /// Scale + opacity pulse (e.g. Perplexity question mark).
    ScalePulse { duration: Duration },
    /// Vertical bounce oscillation (e.g. MiniMax up/down).
    BounceLoop { duration: Duration },
    // One-shot animations
    /// Vertical bounce (e.g. thumbs up feedback).
    Bounce,
    /// Progressive reveal (e.g. check mark confirmation).
    DrawOn,
    /// Fly out with fade (e.g. send action).
    FlyOut,
    /// Drop in from above (e.g. download complete).
    DropIn,
    /// Quick opacity flash (e.g. copy confirmation).
    Flash,

    // ── SF Symbols 6 / 7 named primitive approximations ─────────────────
    //
    // GPUI has no binding to `NSSymbolEffect`, so these variants mimic the
    // documented Apple SF Symbols animations using transforms and opacity.
    // Naming is kept in lock-step with `SymbolEffect` so that when GPUI
    // lands native SF Symbols support (or when the crate grows a macOS
    // FFI backend) the swap is a single-site change.
    /// SF Symbols 6 `.wiggle` — small translation-and-rotation wobble.
    /// Matches SwiftUI's `.wiggle(by:)` default. Approximation.
    Wiggle { duration: Duration },
    /// SF Symbols 6 `.breathe` — gentle scale oscillation ~1.0 → 1.04
    /// → 1.0. Matches SwiftUI's `.breathe` default. Approximation.
    Breathe { duration: Duration },
    /// SF Symbols 6 `.rotate` — continuous rotation; distinct from `Spin`
    /// only in that it uses the "rotate effect" easing cycle.
    /// Approximation.
    Rotate { duration: Duration },
    /// SF Symbols 6 `.variableColor` — opacity fill ramps up then wraps
    /// back to dim. On monochrome icons this drives the caller color;
    /// on multi-color icons it delegates to
    /// `IconRenderMode::VariableColor`. Approximation.
    VariableColor { duration: Duration },
    /// SF Symbols 6 `.replace` — cross-fade morph between two symbols
    /// (`from` → `to`). GPUI approximates as a dissolve; SF's path-morph
    /// (down-up / up-up / off-up) is unavailable without SymbolEffect.
    /// Approximation.
    Replace {
        from: IconName,
        to: IconName,
        duration: Duration,
    },
    /// SF Symbols 6 `.magic` / "Magic Replace" — cross-fade with
    /// simultaneous scale. Approximation.
    MagicReplace {
        from: IconName,
        to: IconName,
        duration: Duration,
    },
    /// SF Symbols 7 `.drawOff` — reverse of DrawOn: opacity fades out.
    /// Approximation of the real path-trace-out; GPUI cannot animate
    /// path length directly.
    DrawOff { duration: Duration },
}

impl IconName {
    /// Returns the characteristic animation for LLM provider icons.
    ///
    /// These approximate the CSS animations from the design reference
    /// using GPUI's transform and opacity primitives.
    pub fn provider_animation(&self) -> Option<IconAnimation> {
        match self {
            IconName::ProviderClaude => Some(IconAnimation::Spin {
                duration: Duration::from_millis(3000),
            }),
            IconName::ProviderGpt => Some(IconAnimation::Spin {
                duration: Duration::from_millis(4000),
            }),
            IconName::ProviderGemini => Some(IconAnimation::Pulse {
                duration: Duration::from_millis(3000),
            }),
            IconName::ProviderGrok => Some(IconAnimation::Pulse {
                duration: Duration::from_millis(2500),
            }),
            IconName::ProviderLlama => Some(IconAnimation::Pulse {
                duration: Duration::from_millis(4000),
            }),
            IconName::ProviderDeepSeek => Some(IconAnimation::Swim {
                duration: Duration::from_millis(4000),
            }),
            IconName::ProviderMistral => Some(IconAnimation::Pulse {
                duration: Duration::from_millis(1500),
            }),
            IconName::ProviderGemma => Some(IconAnimation::Pulse {
                duration: Duration::from_millis(2500),
            }),
            IconName::ProviderPhi => Some(IconAnimation::Rock {
                duration: Duration::from_millis(3000),
                degrees: 8.0,
            }),
            IconName::ProviderQwen => Some(IconAnimation::Rock {
                duration: Duration::from_millis(1500),
                degrees: 8.0,
            }),
            IconName::ProviderGlm => Some(IconAnimation::Pulse {
                duration: Duration::from_millis(2500),
            }),
            IconName::ProviderMiniMax => Some(IconAnimation::BounceLoop {
                duration: Duration::from_millis(1800),
            }),
            IconName::ProviderErnie => Some(IconAnimation::Pulse {
                duration: Duration::from_millis(2500),
            }),
            IconName::ProviderCohere => Some(IconAnimation::Pulse {
                duration: Duration::from_millis(2500),
            }),
            IconName::ProviderPerplexity => Some(IconAnimation::ScalePulse {
                duration: Duration::from_millis(2000),
            }),
            IconName::ProviderNova => Some(IconAnimation::Breath {
                duration: Duration::from_millis(2000),
            }),
            IconName::ProviderCustom => Some(IconAnimation::Breath {
                duration: Duration::from_millis(4000),
            }),
            _ => None,
        }
    }
}

/// An animated icon component.
///
/// Uses `svg().with_transformation()` for transform-based animations
/// (Spin, Shake, Bounce, Heartbeat, Twinkle) and `div().opacity()` for
/// opacity-based animations (Pulse, DrawOn, FlyOut, DropIn, Flash).
///
/// # Example
/// ```ignore
/// AnimatedIcon::new("loader", IconName::Loader, IconAnimation::Spin {
///     duration: Duration::from_millis(1800),
/// })
/// ```
#[derive(IntoElement)]
pub struct AnimatedIcon {
    pub(crate) id: ElementId,
    pub(crate) name: IconName,
    pub(crate) animation: IconAnimation,
    pub(crate) size: Option<Pixels>,
    pub(crate) color: Option<Hsla>,
}

impl AnimatedIcon {
    pub fn new(id: impl Into<ElementId>, name: IconName, animation: IconAnimation) -> Self {
        Self {
            id: id.into(),
            name,
            animation,
            size: None,
            color: None,
        }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = Some(size);
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

impl RenderOnce for AnimatedIcon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let size = self.size.unwrap_or(theme.icon_size);
        let color = self.color.unwrap_or(theme.text_muted);

        // HIG Motion: "Respect the Reduce Motion accessibility setting.
        // Replace large, dramatic transitions with subtle cross-fades or
        // omit them entirely." When the user has Reduce Motion enabled
        // we render the static icon — matching the behaviour of the
        // existing `ActivityIndicator` which already swaps Spin → Pulse.
        //
        // For `Replace` / `MagicReplace` the reduced-motion fallback
        // renders the *target* symbol so the UI still reflects the
        // post-transition state; for all other animations the current
        // icon is what the user sees.
        if theme.accessibility_mode.reduce_motion() {
            let static_name = match self.animation {
                IconAnimation::Replace { to, .. } | IconAnimation::MagicReplace { to, .. } => to,
                _ => self.name,
            };
            return div()
                .id(self.id)
                .size(size)
                .child(Icon::new(static_name).size(size).color(color))
                .into_any_element();
        }

        // Unique animation ID for fallback branches (where self.id goes to the div).
        let anim_id = self.id.clone();

        // Try to get a monochrome SVG path for transform-based animations.
        // Note: Multi-color icons (Git, DevTools, AI Agents, DevOps — ~80+ icons)
        // always take the fallback branch because stacked SVG layers cannot be
        // individually transformed as a single unit. These icons receive opacity-only
        // animations instead of transforms. For per-element animation control, use
        // `AnimatedProviderIcon` instead.
        let svg_path: Option<&'static str> = self.name.render_strategy().and_then(|s| match s {
            RenderStrategy::Monochrome(p) => Some(p),
            _ => None,
        });

        match self.animation {
            // ── Transform-based animations (use svg() directly) ──────────
            IconAnimation::Spin { duration } => {
                if let Some(path) = svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color)
                        .with_animation(
                            self.id,
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                el.with_transformation(Transformation::rotate(radians(
                                    delta * std::f32::consts::TAU,
                                )))
                            },
                        )
                        .into_any_element()
                } else {
                    // Fallback: opacity pulse for non-monochrome icons
                    div()
                        .id(self.id)
                        .size(size)
                        .child(Icon::new(self.name).size(size).color(color))
                        .with_animation(
                            anim_id.clone(),
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let opacity =
                                    0.3 + 0.7 * ((delta * std::f32::consts::TAU).cos() * 0.5 + 0.5);
                                el.opacity(opacity)
                            },
                        )
                        .into_any_element()
                }
            }
            IconAnimation::Shake { duration } => {
                if let Some(path) = svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color)
                        .with_animation(
                            self.id,
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                if delta < 0.3 {
                                    let t = delta / 0.3;
                                    let offset =
                                        (t * std::f32::consts::PI * 6.0).sin() * (1.0 - t) * 2.0;
                                    el.with_transformation(Transformation::translate(point(
                                        px(offset),
                                        px(0.0),
                                    )))
                                } else {
                                    el
                                }
                            },
                        )
                        .into_any_element()
                } else {
                    div()
                        .id(self.id)
                        .size(size)
                        .child(Icon::new(self.name).size(size).color(color))
                        .with_animation(
                            anim_id.clone(),
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                if delta < 0.3 {
                                    let t = delta / 0.3;
                                    let opacity =
                                        0.5 + 0.5 * (t * std::f32::consts::PI * 4.0).cos();
                                    el.opacity(opacity)
                                } else {
                                    el
                                }
                            },
                        )
                        .into_any_element()
                }
            }
            IconAnimation::Bounce => {
                if let Some(path) = svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color)
                        .with_animation(
                            self.id,
                            Animation::new(Duration::from_millis(500)),
                            move |el, delta| {
                                let offset = if delta < 0.4 {
                                    -3.0 * (delta / 0.4)
                                } else if delta < 0.6 {
                                    -3.0 + 1.5 * ((delta - 0.4) / 0.2)
                                } else {
                                    -1.5 * (1.0 - (delta - 0.6) / 0.4)
                                };
                                el.with_transformation(Transformation::translate(point(
                                    px(0.0),
                                    px(offset),
                                )))
                            },
                        )
                        .into_any_element()
                } else {
                    div()
                        .id(self.id)
                        .size(size)
                        .child(Icon::new(self.name).size(size).color(color))
                        .with_animation(
                            anim_id.clone(),
                            Animation::new(Duration::from_millis(500)),
                            move |el, delta| {
                                let opacity = if delta < 0.3 {
                                    0.6
                                } else {
                                    0.6 + 0.4 * ((delta - 0.3) / 0.7)
                                };
                                el.opacity(opacity)
                            },
                        )
                        .into_any_element()
                }
            }
            IconAnimation::Heartbeat { duration } => {
                if let Some(path) = svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color)
                        .with_animation(
                            self.id,
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let scale = if delta < 0.14 {
                                    1.0 + 0.15 * (delta / 0.14)
                                } else if delta < 0.28 {
                                    1.15 - 0.15 * ((delta - 0.14) / 0.14)
                                } else if delta < 0.42 {
                                    1.0 + 0.1 * ((delta - 0.28) / 0.14)
                                } else if delta < 0.56 {
                                    1.1 - 0.1 * ((delta - 0.42) / 0.14)
                                } else {
                                    1.0
                                };
                                el.with_transformation(Transformation::scale(gpui_size(
                                    scale, scale,
                                )))
                            },
                        )
                        .into_any_element()
                } else {
                    div()
                        .id(self.id)
                        .size(size)
                        .child(Icon::new(self.name).size(size).color(color))
                        .with_animation(
                            anim_id.clone(),
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let opacity = if delta < 0.14 {
                                    0.7 + 0.3 * (delta / 0.14)
                                } else if delta < 0.28 {
                                    1.0 - 0.3 * ((delta - 0.14) / 0.14)
                                } else if delta < 0.42 {
                                    0.7 + 0.2 * ((delta - 0.28) / 0.14)
                                } else if delta < 0.56 {
                                    0.9 - 0.2 * ((delta - 0.42) / 0.14)
                                } else {
                                    0.7
                                };
                                el.opacity(opacity)
                            },
                        )
                        .into_any_element()
                }
            }
            IconAnimation::Twinkle { duration } => {
                if let Some(path) = svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color)
                        .with_animation(
                            self.id,
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let t = (delta * std::f32::consts::PI).sin();
                                let scale = 0.8 + 0.2 * t;
                                el.opacity(0.4 + 0.6 * t).with_transformation(
                                    Transformation::scale(gpui_size(scale, scale)),
                                )
                            },
                        )
                        .into_any_element()
                } else {
                    div()
                        .id(self.id)
                        .size(size)
                        .child(Icon::new(self.name).size(size).color(color))
                        .with_animation(
                            anim_id.clone(),
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let t = (delta * std::f32::consts::PI).sin();
                                el.opacity(0.4 + 0.6 * t)
                            },
                        )
                        .into_any_element()
                }
            }

            // ── Provider-specific transform animations ──────────────────
            IconAnimation::Rock { duration, degrees } => {
                if let Some(path) = svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color)
                        .with_animation(
                            self.id,
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let angle =
                                    degrees.to_radians() * (delta * std::f32::consts::TAU).sin();
                                el.with_transformation(Transformation::rotate(radians(angle)))
                            },
                        )
                        .into_any_element()
                } else {
                    div()
                        .id(self.id)
                        .size(size)
                        .child(Icon::new(self.name).size(size).color(color))
                        .with_animation(
                            anim_id.clone(),
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let t = (delta * std::f32::consts::TAU).sin();
                                el.opacity(0.7 + 0.3 * t.abs())
                            },
                        )
                        .into_any_element()
                }
            }
            IconAnimation::Swim { duration } => {
                if let Some(path) = svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color)
                        .with_animation(
                            self.id,
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let x = 1.5 * (delta * std::f32::consts::TAU).sin();
                                let y = -0.8 * (delta * std::f32::consts::TAU * 2.0).sin();
                                el.with_transformation(Transformation::translate(point(
                                    px(x),
                                    px(y),
                                )))
                            },
                        )
                        .into_any_element()
                } else {
                    div()
                        .id(self.id)
                        .size(size)
                        .child(Icon::new(self.name).size(size).color(color))
                        .with_animation(
                            anim_id.clone(),
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let t = (delta * std::f32::consts::PI).sin();
                                el.opacity(0.6 + 0.4 * t)
                            },
                        )
                        .into_any_element()
                }
            }
            IconAnimation::Breath { duration } => {
                if let Some(path) = svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color)
                        .with_animation(
                            self.id,
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let t = (delta * std::f32::consts::PI).sin();
                                let scale = 1.0 + 0.06 * t;
                                el.with_transformation(Transformation::scale(gpui_size(
                                    scale, scale,
                                )))
                            },
                        )
                        .into_any_element()
                } else {
                    div()
                        .id(self.id)
                        .size(size)
                        .child(Icon::new(self.name).size(size).color(color))
                        .with_animation(
                            anim_id.clone(),
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let t = (delta * std::f32::consts::PI).sin();
                                el.opacity(0.85 + 0.15 * t)
                            },
                        )
                        .into_any_element()
                }
            }
            IconAnimation::ScalePulse { duration } => {
                if let Some(path) = svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color)
                        .with_animation(
                            self.id,
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let t = (delta * std::f32::consts::PI).sin();
                                let scale = 1.0 + 0.12 * t;
                                el.opacity(0.85 + 0.15 * t).with_transformation(
                                    Transformation::scale(gpui_size(scale, scale)),
                                )
                            },
                        )
                        .into_any_element()
                } else {
                    div()
                        .id(self.id)
                        .size(size)
                        .child(Icon::new(self.name).size(size).color(color))
                        .with_animation(
                            anim_id.clone(),
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let t = (delta * std::f32::consts::PI).sin();
                                el.opacity(0.85 + 0.15 * t)
                            },
                        )
                        .into_any_element()
                }
            }
            IconAnimation::BounceLoop { duration } => {
                if let Some(path) = svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color)
                        .with_animation(
                            self.id,
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let offset = -2.0 * (delta * std::f32::consts::TAU).sin();
                                el.with_transformation(Transformation::translate(point(
                                    px(0.0),
                                    px(offset),
                                )))
                            },
                        )
                        .into_any_element()
                } else {
                    div()
                        .id(self.id)
                        .size(size)
                        .child(Icon::new(self.name).size(size).color(color))
                        .with_animation(
                            anim_id.clone(),
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let t = (delta * std::f32::consts::PI).sin();
                                el.opacity(0.6 + 0.4 * t)
                            },
                        )
                        .into_any_element()
                }
            }

            // ── Opacity-based animations (use Icon wrapper) ──────────────
            IconAnimation::Pulse { duration } => div()
                .id(anim_id.clone())
                .size(size)
                .child(Icon::new(self.name).size(size).color(color))
                .with_animation(
                    anim_id.clone(),
                    Animation::new(duration).repeat(),
                    move |el, delta| {
                        let t = (delta * std::f32::consts::PI).sin();
                        el.opacity(0.4 + 0.6 * t)
                    },
                )
                .into_any_element(),
            IconAnimation::DrawOn => div()
                .id(self.id)
                .size(size)
                .child(Icon::new(self.name).size(size).color(color))
                .with_animation(
                    anim_id.clone(),
                    Animation::new(Duration::from_millis(600)),
                    move |el, delta| el.opacity(delta),
                )
                .into_any_element(),
            IconAnimation::FlyOut => {
                if let Some(path) = svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color)
                        .with_animation(
                            self.id,
                            Animation::new(Duration::from_millis(500)),
                            move |el, delta| {
                                let offset = delta * 6.0;
                                el.opacity(1.0 - delta).with_transformation(
                                    Transformation::translate(point(px(offset), px(-offset))),
                                )
                            },
                        )
                        .into_any_element()
                } else {
                    div()
                        .id(self.id)
                        .size(size)
                        .child(Icon::new(self.name).size(size).color(color))
                        .with_animation(
                            anim_id.clone(),
                            Animation::new(Duration::from_millis(500)),
                            move |el, delta| el.opacity(1.0 - delta),
                        )
                        .into_any_element()
                }
            }
            IconAnimation::DropIn => {
                if let Some(path) = svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color)
                        .with_animation(
                            self.id,
                            Animation::new(Duration::from_millis(400)),
                            move |el, delta| {
                                let offset = -4.0 * (1.0 - delta);
                                el.opacity(delta)
                                    .with_transformation(Transformation::translate(point(
                                        px(0.0),
                                        px(offset),
                                    )))
                            },
                        )
                        .into_any_element()
                } else {
                    div()
                        .id(self.id)
                        .size(size)
                        .child(Icon::new(self.name).size(size).color(color))
                        .with_animation(
                            anim_id.clone(),
                            Animation::new(Duration::from_millis(400)),
                            move |el, delta| el.opacity(delta),
                        )
                        .into_any_element()
                }
            }
            IconAnimation::Flash => div()
                .id(self.id)
                .size(size)
                .child(Icon::new(self.name).size(size).color(color))
                .with_animation(
                    anim_id.clone(),
                    Animation::new(Duration::from_millis(300)),
                    move |el, delta| {
                        let opacity = if delta < 0.5 {
                            1.0 - 0.7 * (delta / 0.5)
                        } else {
                            0.3 + 0.7 * ((delta - 0.5) / 0.5)
                        };
                        el.opacity(opacity)
                    },
                )
                .into_any_element(),

            // ── SF Symbols 6 / 7 named primitives (approximations) ───────
            IconAnimation::Wiggle { duration } => {
                if let Some(path) = svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color)
                        .with_animation(
                            self.id,
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let phase = delta * std::f32::consts::TAU;
                                let dx = (phase * 2.0).sin() * 1.2;
                                let angle = (phase * 2.0).cos() * 4.0f32.to_radians();
                                el.with_transformation(
                                    Transformation::translate(point(px(dx), px(0.0)))
                                        .with_rotation(radians(angle)),
                                )
                            },
                        )
                        .into_any_element()
                } else {
                    div()
                        .id(self.id)
                        .size(size)
                        .child(Icon::new(self.name).size(size).color(color))
                        .into_any_element()
                }
            }
            IconAnimation::Breathe { duration } => {
                if let Some(path) = svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color)
                        .with_animation(
                            self.id,
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                let t = (delta * std::f32::consts::PI).sin();
                                let s = 1.0 + 0.04 * t;
                                el.with_transformation(Transformation::scale(gpui_size(s, s)))
                            },
                        )
                        .into_any_element()
                } else {
                    div()
                        .id(self.id)
                        .size(size)
                        .child(Icon::new(self.name).size(size).color(color))
                        .into_any_element()
                }
            }
            IconAnimation::Rotate { duration } => {
                if let Some(path) = svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color)
                        .with_animation(
                            self.id,
                            Animation::new(duration).repeat(),
                            move |el, delta| {
                                el.with_transformation(Transformation::rotate(radians(
                                    delta * std::f32::consts::TAU,
                                )))
                            },
                        )
                        .into_any_element()
                } else {
                    div()
                        .id(self.id)
                        .size(size)
                        .child(Icon::new(self.name).size(size).color(color))
                        .into_any_element()
                }
            }
            IconAnimation::VariableColor { duration } => {
                // Opacity ramp that wraps back to dim — approximation of
                // SF Symbols' incremental-layer fill.
                div()
                    .id(anim_id.clone())
                    .size(size)
                    .child(Icon::new(self.name).size(size).color(color))
                    .with_animation(
                        anim_id.clone(),
                        Animation::new(duration).repeat(),
                        move |el, delta| {
                            // 0 → 1 linear ramp with a 10 % hold at each
                            // end so the symbol doesn't strobe.
                            let t = if delta < 0.1 {
                                0.35
                            } else if delta > 0.9 {
                                1.0
                            } else {
                                0.35 + 0.65 * ((delta - 0.1) / 0.8)
                            };
                            el.opacity(t)
                        },
                    )
                    .into_any_element()
            }
            IconAnimation::Replace { from, to, duration } => {
                // Cross-fade between two icons stacked in a relative
                // container. At delta=0 only `from` is visible; at
                // delta=1 only `to` is visible.
                let color_from = color;
                let color_to = color;
                div()
                    .id(self.id)
                    .size(size)
                    .relative()
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .size(size)
                            .child(Icon::new(from).size(size).color(color_from))
                            .with_animation(
                                anim_id.clone(),
                                Animation::new(duration),
                                move |el, delta| el.opacity(1.0 - delta),
                            ),
                    )
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .size(size)
                            .child(Icon::new(to).size(size).color(color_to)),
                    )
                    .into_any_element()
            }
            IconAnimation::MagicReplace { from, to, duration } => {
                // Cross-fade with a subtle scale on the incoming symbol.
                // The scale has to land on an `svg()` element because GPUI's
                // Transformation::scale only applies to svg/img; we use the
                // target icon's monochrome path when available and fall
                // back to an opacity-only cross-fade otherwise.
                let color_from = color;
                let color_to = color;
                let to_svg_path: Option<&'static str> =
                    to.render_strategy().and_then(|s| match s {
                        RenderStrategy::Monochrome(p) => Some(p),
                        _ => None,
                    });
                let incoming = if let Some(path) = to_svg_path {
                    svg()
                        .path(path)
                        .size(size)
                        .text_color(color_to)
                        .with_animation(
                            ElementId::Integer(1),
                            Animation::new(duration),
                            move |el, delta| {
                                let s = 0.92 + 0.08 * delta;
                                el.opacity(delta)
                                    .with_transformation(Transformation::scale(gpui_size(s, s)))
                            },
                        )
                        .into_any_element()
                } else {
                    div()
                        .id(ElementId::Integer(1))
                        .size(size)
                        .child(Icon::new(to).size(size).color(color_to))
                        .with_animation(
                            ElementId::Integer(2),
                            Animation::new(duration),
                            move |el, delta| el.opacity(delta),
                        )
                        .into_any_element()
                };
                div()
                    .id(self.id)
                    .size(size)
                    .relative()
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .size(size)
                            .child(Icon::new(from).size(size).color(color_from))
                            .with_animation(
                                anim_id.clone(),
                                Animation::new(duration),
                                move |el, delta| el.opacity(1.0 - delta),
                            ),
                    )
                    .child(div().absolute().top_0().left_0().size(size).child(incoming))
                    .into_any_element()
            }
            IconAnimation::DrawOff { duration } => div()
                .id(self.id)
                .size(size)
                .child(Icon::new(self.name).size(size).color(color))
                .with_animation(
                    anim_id.clone(),
                    Animation::new(duration),
                    move |el, delta| el.opacity(1.0 - delta),
                )
                .into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::foundations::icons::IconName;
    use core::prelude::v1::test;

    #[test]
    fn all_providers_have_animation() {
        let providers = [
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
        for p in &providers {
            assert!(
                p.provider_animation().is_some(),
                "{p:?} should have an animation"
            );
        }
    }

    #[test]
    fn non_providers_have_no_animation() {
        assert!(IconName::Check.provider_animation().is_none());
        assert!(IconName::ArrowDown.provider_animation().is_none());
    }
}
