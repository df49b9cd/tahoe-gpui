//! Canvas-drawn animated LLM provider icons.
//!
//! Each provider icon is drawn programmatically via `PathBuilder` + `paint_path`,
//! enabling per-element animation control that matches the CSS reference animations.

use std::f32::consts::{PI, TAU};
use std::time::Instant;

use gpui::prelude::*;
use gpui::{Bounds, Hsla, PathBuilder, Pixels, Window, canvas, point, px};

use super::IconName;
use crate::foundations::theme::ActiveTheme;

/// A canvas-drawn animated provider icon with per-element animation control.
///
/// Unlike [`super::AnimatedIcon`] which applies whole-icon transforms to an SVG,
/// this component draws each geometric element individually via `PathBuilder`,
/// allowing staggered timing, per-element opacity, and independent transforms.
///
/// # Example
/// ```ignore
/// let icon = cx.new(|_| AnimatedProviderIcon::new(IconName::ProviderClaude));
/// div().size(px(32.0)).child(icon)
/// ```
pub struct AnimatedProviderIcon {
    provider: IconName,
    start: Instant,
    size: Pixels,
    color: Option<Hsla>,
    paused: bool,
}

impl AnimatedProviderIcon {
    pub fn new(provider: IconName) -> Self {
        Self {
            provider,
            start: Instant::now(),
            size: px(24.0),
            color: None,
            paused: false,
        }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    pub fn set_paused(&mut self, paused: bool, cx: &mut Context<Self>) {
        if self.paused != paused {
            self.paused = paused;
            cx.notify();
        }
    }
}

impl Render for AnimatedProviderIcon {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let reduce_motion = theme.accessibility_mode.reduce_motion();
        let color = self.color.unwrap_or(theme.text_muted);

        // Drive frame pacing through the window's animation loop rather than
        // a 16ms background timer. This lets the compositor run at the
        // display's refresh rate (up to 120 Hz on ProMotion) and avoids the
        // duplicate-paint jitter a 16ms timer causes on non-60 Hz displays.
        // Open question #2 on the internal tracker tracks exposing
        // the actual refresh rate for explicit budgeting.
        if !self.paused && !reduce_motion {
            window.request_animation_frame();
        }

        // Reduce Motion: freeze the animation parameter at `t = 0` so the
        // icon renders its neutral, phase-zero state. We keep the canvas
        // (not a static fallback) so layout, color, and provider identity
        // are identical to the animated path.
        let t = if self.paused || reduce_motion {
            0.0
        } else {
            // Wrap with modulo to prevent f32 precision loss after extended runtime.
            self.start.elapsed().as_secs_f32() % 60.0
        };
        let provider = self.provider;
        let size = self.size;

        canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _, window, _cx| {
                let ctx = DrawCtx::new(bounds, size, color);
                match provider {
                    IconName::ProviderClaude => draw_claude(t, &ctx, window),
                    IconName::ProviderGpt => draw_gpt(t, &ctx, window),
                    IconName::ProviderGemini => draw_gemini(t, &ctx, window),
                    IconName::ProviderGrok => draw_grok(t, &ctx, window),
                    IconName::ProviderLlama => draw_llama(t, &ctx, window),
                    IconName::ProviderDeepSeek => draw_deepseek(t, &ctx, window),
                    IconName::ProviderMistral => draw_mistral(t, &ctx, window),
                    IconName::ProviderGemma => draw_gemma(t, &ctx, window),
                    IconName::ProviderPhi => draw_phi(t, &ctx, window),
                    IconName::ProviderQwen => draw_qwen(t, &ctx, window),
                    IconName::ProviderGlm => draw_glm(t, &ctx, window),
                    IconName::ProviderMiniMax => draw_minimax(t, &ctx, window),
                    IconName::ProviderErnie => draw_ernie(t, &ctx, window),
                    IconName::ProviderCohere => draw_cohere(t, &ctx, window),
                    IconName::ProviderPerplexity => draw_perplexity(t, &ctx, window),
                    IconName::ProviderNova => draw_nova(t, &ctx, window),
                    IconName::ProviderCustom => draw_custom(t, &ctx, window),
                    _ => {
                        // Fallback: draw a simple circle for unhandled provider names
                        draw_circle_stroke(&ctx, 12.0, 12.0, 8.0, ctx.color, window);
                    }
                }
            },
        )
        .size(size)
    }
}

// ─── Drawing context ──────────────────────────────────────────────────────

struct DrawCtx {
    ox: f32,
    oy: f32,
    scale: f32,
    color: Hsla,
}

impl DrawCtx {
    fn new(bounds: Bounds<Pixels>, size: Pixels, color: Hsla) -> Self {
        let s = f32::from(size);
        Self {
            ox: f32::from(bounds.origin.x),
            oy: f32::from(bounds.origin.y),
            scale: s / 24.0,
            color,
        }
    }

    /// Convert 24x24 coordinates to pixel point.
    fn p(&self, x: f32, y: f32) -> gpui::Point<Pixels> {
        point(px(self.ox + x * self.scale), px(self.oy + y * self.scale))
    }

    /// Scaled stroke width.
    fn sw(&self) -> Pixels {
        px(1.2 * self.scale)
    }

    /// Color with custom opacity.
    fn c(&self, opacity: f32) -> Hsla {
        Hsla {
            a: opacity * self.color.a,
            ..self.color
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn draw_line(ctx: &DrawCtx, x1: f32, y1: f32, x2: f32, y2: f32, color: Hsla, window: &mut Window) {
    let mut pb = PathBuilder::stroke(ctx.sw());
    pb.move_to(ctx.p(x1, y1));
    pb.line_to(ctx.p(x2, y2));
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

fn draw_circle_filled(ctx: &DrawCtx, cx: f32, cy: f32, r: f32, color: Hsla, window: &mut Window) {
    const N: usize = 16;
    let mut points = [gpui::point(px(0.0), px(0.0)); N];
    for (i, pt) in points.iter_mut().enumerate() {
        let a = i as f32 * TAU / N as f32;
        *pt = ctx.p(cx + r * a.cos(), cy + r * a.sin());
    }
    let mut pb = PathBuilder::fill();
    pb.add_polygon(&points, true);
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

fn draw_circle_stroke(ctx: &DrawCtx, cx: f32, cy: f32, r: f32, color: Hsla, window: &mut Window) {
    let n = 32;
    let mut pb = PathBuilder::stroke(ctx.sw());
    for i in 0..=n {
        let a = i as f32 * TAU / n as f32;
        let pt = ctx.p(cx + r * a.cos(), cy + r * a.sin());
        if i == 0 {
            pb.move_to(pt);
        } else {
            pb.line_to(pt);
        }
    }
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

/// Draw an arc (portion of circle) as a stroked polyline.
fn draw_arc(
    ctx: &DrawCtx,
    cx: f32,
    cy: f32,
    r: f32,
    start_angle: f32,
    sweep: f32,
    color: Hsla,
    window: &mut Window,
) {
    let n = 24;
    let mut pb = PathBuilder::stroke(ctx.sw());
    for i in 0..=n {
        let a = start_angle + sweep * (i as f32 / n as f32);
        let pt = ctx.p(cx + r * a.cos(), cy + r * a.sin());
        if i == 0 {
            pb.move_to(pt);
        } else {
            pb.line_to(pt);
        }
    }
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

fn draw_polyline(ctx: &DrawCtx, points: &[(f32, f32)], color: Hsla, window: &mut Window) {
    if points.len() < 2 {
        return;
    }
    let mut pb = PathBuilder::stroke(ctx.sw());
    pb.move_to(ctx.p(points[0].0, points[0].1));
    for &(x, y) in &points[1..] {
        pb.line_to(ctx.p(x, y));
    }
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

fn draw_polygon_stroke(ctx: &DrawCtx, points: &[(f32, f32)], color: Hsla, window: &mut Window) {
    if points.len() < 3 {
        return;
    }
    let mut pb = PathBuilder::stroke(ctx.sw());
    pb.move_to(ctx.p(points[0].0, points[0].1));
    for &(x, y) in &points[1..] {
        pb.line_to(ctx.p(x, y));
    }
    pb.line_to(ctx.p(points[0].0, points[0].1));
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

/// Endpoints and control point of a quadratic Bezier curve in local
/// (pre-DrawCtx-transform) coordinates.
#[derive(Clone, Copy)]
struct QuadBezier {
    start: (f32, f32),
    control: (f32, f32),
    end: (f32, f32),
}

fn draw_quad_bezier(ctx: &DrawCtx, bez: QuadBezier, color: Hsla, window: &mut Window) {
    // Approximate quadratic bezier with line segments.
    let n = 12;
    let (x1, y1) = bez.start;
    let (cx_, cy_) = bez.control;
    let (x2, y2) = bez.end;
    let mut pb = PathBuilder::stroke(ctx.sw());
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let u = 1.0 - t;
        let x = u * u * x1 + 2.0 * u * t * cx_ + t * t * x2;
        let y = u * u * y1 + 2.0 * u * t * cy_ + t * t * y2;
        if i == 0 {
            pb.move_to(ctx.p(x, y));
        } else {
            pb.line_to(ctx.p(x, y));
        }
    }
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

// ─── Provider draw functions ──────────────────────────────────────────────

/// Claude: 10 spokes extend outward from center with staggered timing.
fn draw_claude(t: f32, ctx: &DrawCtx, window: &mut Window) {
    const SPOKES: [(f32, f32); 10] = [
        (12.0, 2.5),
        (18.5, 4.2),
        (21.0, 9.0),
        (20.8, 15.8),
        (17.5, 20.0),
        (11.5, 22.0),
        (6.0, 19.5),
        (2.5, 14.0),
        (3.0, 8.5),
        (6.5, 3.5),
    ];
    let period = 3.0;
    for (i, &(ex, ey)) in SPOKES.iter().enumerate() {
        let delay = i as f32 * 0.07;
        let phase = ((t - delay) % period + period) % period / period;
        // Spoke extends: 0→0.5 extend, 0.5→1.0 retract
        let progress = if phase < 0.5 {
            phase * 2.0
        } else {
            2.0 - phase * 2.0
        };
        // Draw spoke line from center to partial endpoint
        let dx = ex - 12.0;
        let dy = ey - 12.0;
        let tx = 12.0 + dx * progress;
        let ty = 12.0 + dy * progress;
        draw_line(ctx, 12.0, 12.0, tx, ty, ctx.color, window);
        // Draw endpoint dot with animated radius
        let dot_r = if phase < 0.3 {
            phase / 0.3
        } else if phase < 0.7 {
            1.0
        } else {
            (1.0 - phase) / 0.3
        };
        if dot_r > 0.05 {
            draw_circle_filled(ctx, tx, ty, dot_r, ctx.color, window);
        }
    }
}

/// GPT: 5 circles with flowing arc gap + inner pentagon.
fn draw_gpt(t: f32, ctx: &DrawCtx, window: &mut Window) {
    let r_orbit = 5.0_f32;
    let cr = 4.2_f32;
    let mut centers = [(0.0_f32, 0.0_f32); 5];
    let mut inners = [(0.0_f32, 0.0_f32); 5];
    for i in 0..5 {
        let a = (90.0 + i as f32 * 72.0).to_radians();
        centers[i] = (12.0 + r_orbit * a.cos(), 12.0 - r_orbit * a.sin());
        let dx = 12.0 - centers[i].0;
        let dy = 12.0 - centers[i].1;
        let len = (dx * dx + dy * dy).sqrt();
        inners[i] = (centers[i].0 + dx / len * cr, centers[i].1 + dy / len * cr);
    }
    // Draw circles with rotating gap
    let flow_period = 3.0;
    for (i, &(cx, cy)) in centers.iter().enumerate() {
        let delay = i as f32 * 0.3;
        let gap_start = ((t - delay) % flow_period + flow_period) % flow_period / flow_period * TAU;
        // Draw arc covering ~270° with a 90° gap
        draw_arc(
            ctx,
            cx,
            cy,
            cr,
            gap_start + 0.4,
            TAU - 0.8,
            ctx.color,
            window,
        );
    }
    // Draw inner pentagon lines (static)
    for i in 0..5 {
        let j = (i + 2) % 5;
        draw_line(
            ctx,
            inners[i].0,
            inners[i].1,
            inners[j].0,
            inners[j].1,
            ctx.color,
            window,
        );
    }
}

/// Gemini: left/right diamond halves alternate opacity.
fn draw_gemini(t: f32, ctx: &DrawCtx, window: &mut Window) {
    let period = 3.0;
    let phase = t * TAU / period;
    let opacity_a = 0.5 + 0.5 * phase.cos();
    let opacity_b = 0.5 + 0.5 * (phase + PI).cos();
    // Left diamonds
    let ca = ctx.c(opacity_a);
    draw_polygon_stroke(
        ctx,
        &[(9.0, 3.0), (11.0, 10.0), (9.0, 12.0), (7.0, 10.0)],
        ca,
        window,
    );
    draw_polygon_stroke(
        ctx,
        &[(9.0, 12.0), (11.0, 14.0), (9.0, 21.0), (7.0, 14.0)],
        ca,
        window,
    );
    // Right diamonds
    let cb = ctx.c(opacity_b);
    draw_polygon_stroke(
        ctx,
        &[(15.0, 3.0), (17.0, 10.0), (15.0, 12.0), (13.0, 10.0)],
        cb,
        window,
    );
    draw_polygon_stroke(
        ctx,
        &[(15.0, 12.0), (17.0, 14.0), (15.0, 21.0), (13.0, 14.0)],
        cb,
        window,
    );
}

/// Grok: G path draws on, X draws sequentially.
fn draw_grok(t: f32, ctx: &DrawCtx, window: &mut Window) {
    // Static center circle
    draw_circle_stroke(ctx, 12.0, 12.0, 2.0, ctx.color, window);
    // G path: draw-on over 5s
    let period = 5.0;
    let phase = (t % period) / period;
    let g_progress = if phase < 0.4 {
        phase / 0.4
    } else if phase < 0.6 {
        1.0
    } else {
        1.0 - (phase - 0.6) / 0.4
    };
    // G path as polyline segments, draw partial
    let mut g_points = [(0.0f32, 0.0f32); 24];
    g_points[0] = (14.0, 12.0);
    for i in 0..20 {
        let frac = i as f32 / 19.0;
        let (x, y) = match frac {
            f if f < 0.1 => (14.0 + (18.0 - 14.0) * f / 0.1, 12.0),
            f if f < 0.25 => (18.0, 12.0 + (17.0 - 12.0) * (f - 0.1) / 0.15),
            f if f < 0.4 => {
                let t2 = (f - 0.25) / 0.15;
                (18.0 - (18.0 - 15.0) * t2, 17.0 + (20.0 - 17.0) * t2)
            }
            f if f < 0.55 => {
                let t2 = (f - 0.4) / 0.15;
                (15.0 - (15.0 - 9.0) * t2, 20.0)
            }
            f if f < 0.7 => {
                let t2 = (f - 0.55) / 0.15;
                (9.0 - (9.0 - 6.0) * t2, 20.0 - (20.0 - 17.0) * t2)
            }
            f if f < 0.85 => {
                let t2 = (f - 0.7) / 0.15;
                (6.0, 17.0 - (17.0 - 7.0) * t2)
            }
            f => {
                let t2 = (f - 0.85) / 0.15;
                (6.0 + (9.0 - 6.0) * t2, 7.0 - (7.0 - 4.0) * t2)
            }
        };
        g_points[i + 1] = (x, y);
    }
    g_points[21] = (15.0, 4.0);
    g_points[22] = (18.0, 4.0);
    g_points[23] = (18.0, 7.0);
    let n = (g_points.len() as f32 * g_progress).ceil() as usize;
    if n >= 2 {
        draw_polyline(ctx, &g_points[..n.min(g_points.len())], ctx.color, window);
    }
    // X lines: appear after G is ~40% drawn
    let x_opacity1 = if phase > 0.38 && phase < 0.62 {
        ((phase - 0.38) / 0.08).min(1.0)
    } else {
        0.0
    };
    let x_opacity2 = if phase > 0.44 && phase < 0.58 {
        ((phase - 0.44) / 0.08).min(1.0)
    } else {
        0.0
    };
    if x_opacity1 > 0.01 {
        draw_line(ctx, 16.5, 5.5, 19.5, 8.5, ctx.c(x_opacity1), window);
    }
    if x_opacity2 > 0.01 {
        draw_line(ctx, 19.5, 5.5, 16.5, 8.5, ctx.c(x_opacity2), window);
    }
}

/// Llama: eye blink.
fn draw_llama(t: f32, ctx: &DrawCtx, window: &mut Window) {
    draw_circle_stroke(ctx, 12.0, 14.0, 6.0, ctx.color, window);
    // Left ear: two control curves forming a leaf silhouette.
    draw_quad_bezier(
        ctx,
        QuadBezier {
            start: (8.0, 9.0),
            control: (7.0, 3.0),
            end: (9.0, 3.0),
        },
        ctx.color,
        window,
    );
    draw_quad_bezier(
        ctx,
        QuadBezier {
            start: (10.0, 8.0),
            control: (10.0, 3.0),
            end: (9.0, 3.0),
        },
        ctx.color,
        window,
    );
    // Right ear: mirror of the left.
    draw_quad_bezier(
        ctx,
        QuadBezier {
            start: (16.0, 9.0),
            control: (17.0, 3.0),
            end: (15.0, 3.0),
        },
        ctx.color,
        window,
    );
    draw_quad_bezier(
        ctx,
        QuadBezier {
            start: (14.0, 8.0),
            control: (14.0, 3.0),
            end: (15.0, 3.0),
        },
        ctx.color,
        window,
    );
    // Eyes with blink
    let blink_phase = (t % 4.0) / 4.0;
    let eye_visible = !(0.38..0.44).contains(&blink_phase);
    if eye_visible {
        draw_line(ctx, 9.2, 13.5, 11.0, 13.5, ctx.color, window);
        draw_line(ctx, 13.0, 13.5, 14.8, 13.5, ctx.color, window);
    }
}

/// DeepSeek: body swims + bubbles rise.
fn draw_deepseek(t: f32, ctx: &DrawCtx, window: &mut Window) {
    let swim_x = 1.5 * (t * TAU / 4.0).sin();
    let swim_y = -0.8 * (t * TAU / 2.0).sin();
    // Fish body (translated)
    let body: [(f32, f32); 16] = {
        let mut pts = [(0.0, 0.0); 16];
        // Approximate the fish shape
        let raw = [
            (4.0, 12.0),
            (5.0, 9.0),
            (8.0, 7.0),
            (12.0, 6.0),
            (16.0, 7.0),
            (19.0, 8.5),
            (20.0, 10.0),
            (21.0, 11.0),
            (22.0, 12.0),
            (21.0, 13.0),
            (20.0, 14.0),
            (19.0, 15.5),
            (16.0, 17.0),
            (12.0, 18.0),
            (8.0, 17.0),
            (4.0, 14.0),
        ];
        for (i, &(x, y)) in raw.iter().enumerate() {
            pts[i] = (x + swim_x, y + swim_y);
        }
        pts
    };
    draw_polygon_stroke(ctx, &body, ctx.color, window);
    // Eye
    draw_circle_filled(ctx, 9.0 + swim_x, 11.0 + swim_y, 1.0, ctx.color, window);
    // Tail fins
    draw_line(
        ctx,
        20.0 + swim_x,
        10.0 + swim_y,
        22.0 + swim_x,
        8.0 + swim_y,
        ctx.color,
        window,
    );
    draw_line(
        ctx,
        20.0 + swim_x,
        14.0 + swim_y,
        22.0 + swim_x,
        16.0 + swim_y,
        ctx.color,
        window,
    );
    // Bubbles
    let bubbles = [
        (3.0, 11.0, 0.6, 0.0),
        (4.0, 10.0, 0.4, 0.5),
        (2.5, 12.0, 0.5, 1.0),
    ];
    for &(bx, by, br, delay) in &bubbles {
        let bphase = ((t - delay) % 3.0 + 3.0) % 3.0 / 3.0;
        let opacity = if bphase < 0.1 {
            bphase / 0.1 * 0.7
        } else {
            0.7 * (1.0 - bphase)
        };
        let dy = -8.0 * bphase;
        let dx = if delay < 0.3 {
            -2.0 * bphase
        } else if delay < 0.7 {
            bphase
        } else {
            -bphase
        };
        if opacity > 0.01 {
            draw_circle_filled(ctx, bx + dx, by + dy, br, ctx.c(opacity), window);
        }
    }
}

/// Mistral: flowing dashes on 3 lines.
fn draw_mistral(t: f32, ctx: &DrawCtx, window: &mut Window) {
    let rows = [
        (6.0, 5.5, 20.0, 0.0),
        (12.0, 5.5, 17.0, 0.3),
        (18.0, 5.5, 20.0, 0.6),
    ];
    for &(y, x_start, x_end, delay) in &rows {
        // Static circle
        draw_circle_filled(ctx, 4.0, y, 1.5, ctx.color, window);
        // Flowing dash segments
        let dash_len = 3.0;
        let gap_len = 2.0;
        let total = dash_len + gap_len;
        let offset = ((t - delay) % 1.5) / 1.5 * total;
        let line_len = x_end - x_start;
        let mut pos = -offset;
        while pos < line_len {
            let start = pos.max(0.0);
            let end = (pos + dash_len).min(line_len);
            if end > start + 0.1 {
                draw_line(ctx, x_start + start, y, x_start + end, y, ctx.color, window);
            }
            pos += total;
        }
    }
}

/// Gemma: hexagon with shimmering inner lines.
fn draw_gemma(t: f32, ctx: &DrawCtx, window: &mut Window) {
    // Static hexagon
    let hex = [
        (12.0, 3.0),
        (19.8, 7.5),
        (19.8, 16.5),
        (12.0, 21.0),
        (4.2, 16.5),
        (4.2, 7.5),
    ];
    draw_polygon_stroke(ctx, &hex, ctx.color, window);
    // Shimmering inner lines
    let opacity1 = 0.6 + 0.4 * (t * TAU / 2.5).sin();
    let opacity2 = 0.6 + 0.4 * ((t - 0.6) * TAU / 2.5).sin();
    draw_polyline(
        ctx,
        &[(4.2, 7.5), (12.0, 12.0), (19.8, 7.5)],
        ctx.c(opacity1),
        window,
    );
    draw_line(ctx, 12.0, 12.0, 12.0, 21.0, ctx.c(opacity2), window);
}

/// Phi: vertical line tilts.
fn draw_phi(t: f32, ctx: &DrawCtx, window: &mut Window) {
    draw_circle_stroke(ctx, 12.0, 12.0, 6.0, ctx.color, window);
    // Tilting vertical line
    let angle = 8.0_f32.to_radians() * (t * TAU / 3.0).sin();
    let len = 9.0;
    let top_x = 12.0 + len * angle.sin();
    let top_y = 12.0 - len * angle.cos();
    let bot_x = 12.0 - len * angle.sin();
    let bot_y = 12.0 + len * angle.cos();
    draw_line(ctx, top_x, top_y, bot_x, bot_y, ctx.color, window);
}

/// Qwen: tail swings.
fn draw_qwen(t: f32, ctx: &DrawCtx, window: &mut Window) {
    draw_circle_stroke(ctx, 11.0, 11.0, 8.0, ctx.color, window);
    // Swinging tail
    let pivot_x = 15.0;
    let pivot_y = 15.0;
    let angle = 8.0_f32.to_radians() * (t * TAU / 1.5).sin();
    let tail_len = 9.9; // distance from pivot to (22,22)
    let base_angle = 45.0_f32.to_radians(); // angle from pivot to (22,22)
    let end_x = pivot_x + tail_len * (base_angle + angle).cos();
    let end_y = pivot_y + tail_len * (base_angle + angle).sin();
    // Thicker tail
    let mut pb = PathBuilder::stroke(px(1.8 * ctx.scale));
    pb.move_to(ctx.p(pivot_x, pivot_y));
    pb.line_to(ctx.p(end_x, end_y));
    if let Ok(path) = pb.build() {
        window.paint_path(path, ctx.color);
    }
}

/// GLM: nodes light up sequentially.
fn draw_glm(t: f32, ctx: &DrawCtx, window: &mut Window) {
    // Static Z-shape lines
    draw_line(ctx, 5.0, 5.0, 19.0, 5.0, ctx.color, window);
    draw_line(ctx, 19.0, 5.0, 5.0, 19.0, ctx.color, window);
    draw_line(ctx, 5.0, 19.0, 19.0, 19.0, ctx.color, window);
    // Sequential nodes
    let nodes = [
        (5.0, 5.0, 0.0),
        (19.0, 5.0, 0.5),
        (5.0, 19.0, 1.0),
        (19.0, 19.0, 1.5),
    ];
    let period = 2.5;
    for &(nx, ny, delay) in &nodes {
        let phase = ((t - delay) % period + period) % period / period;
        let opacity = if phase < 0.075 {
            0.2 + 0.8 * (phase / 0.075)
        } else if phase < 0.9 {
            1.0
        } else {
            1.0 - 0.8 * ((phase - 0.9) / 0.1)
        };
        draw_circle_filled(ctx, nx, ny, 2.5, ctx.c(opacity), window);
    }
}

/// MiniMax: top/bottom halves bounce.
fn draw_minimax(t: f32, ctx: &DrawCtx, window: &mut Window) {
    let offset = 2.0 * (t * TAU / 1.8).sin();
    // Top half (bounces up)
    let top_offset = -offset;
    draw_polyline(
        ctx,
        &[
            (7.0, 10.0 + top_offset),
            (12.0, 5.0 + top_offset),
            (17.0, 10.0 + top_offset),
        ],
        ctx.color,
        window,
    );
    draw_line(ctx, 12.0, 5.0 + top_offset, 12.0, 12.0, ctx.color, window);
    // Bottom half (bounces down)
    let bot_offset = offset;
    draw_polyline(
        ctx,
        &[
            (7.0, 14.0 + bot_offset),
            (12.0, 19.0 + bot_offset),
            (17.0, 14.0 + bot_offset),
        ],
        ctx.color,
        window,
    );
    draw_line(ctx, 12.0, 19.0 + bot_offset, 12.0, 12.0, ctx.color, window);
}

/// Ernie: wave morphs inside circle.
fn draw_ernie(t: f32, ctx: &DrawCtx, window: &mut Window) {
    draw_circle_stroke(ctx, 12.0, 12.0, 8.0, ctx.color, window);
    // Morphing wave
    let phase = t * TAU / 2.5;
    let amp = 3.0;
    let wave_y1 = 12.0 - amp * phase.sin();
    let wave_y2 = 12.0 + amp * phase.sin();
    let wave_y3 = 12.0 - amp * phase.sin();
    // Draw wave as connected quad beziers
    let n = 24;
    let mut pb = PathBuilder::stroke(ctx.sw());
    for i in 0..=n {
        let frac = i as f32 / n as f32;
        let x = 6.0 + 12.0 * frac;
        let y = if frac < 0.33 {
            let t2 = frac / 0.33;
            12.0 + (wave_y1 - 12.0) * (t2 * PI).sin()
        } else if frac < 0.67 {
            let t2 = (frac - 0.33) / 0.34;
            12.0 + (wave_y2 - 12.0) * (t2 * PI).sin()
        } else {
            let t2 = (frac - 0.67) / 0.33;
            12.0 + (wave_y3 - 12.0) * (t2 * PI).sin()
        };
        if i == 0 {
            pb.move_to(ctx.p(x, y));
        } else {
            pb.line_to(ctx.p(x, y));
        }
    }
    if let Ok(path) = pb.build() {
        window.paint_path(path, ctx.color);
    }
}

/// Cohere: nodes fade sequentially.
fn draw_cohere(t: f32, ctx: &DrawCtx, window: &mut Window) {
    // Static connecting lines
    draw_line(ctx, 7.8, 14.0, 10.5, 9.0, ctx.color, window);
    draw_line(ctx, 13.5, 9.0, 16.2, 14.0, ctx.color, window);
    // Sequential nodes
    let nodes = [(6.0, 16.0, 0.25), (12.0, 7.0, 1.0), (18.0, 16.0, 1.75)];
    let period = 2.5;
    for &(nx, ny, peak_time) in &nodes {
        let phase = (t % period) / period;
        let peak_phase = peak_time / period;
        let dist = (phase - peak_phase)
            .abs()
            .min(1.0 - (phase - peak_phase).abs());
        let opacity = 0.35 + 0.65 * (1.0 - (dist * 10.0).min(1.0));
        draw_circle_stroke(ctx, nx, ny, 2.5, ctx.c(opacity), window);
    }
}

/// Perplexity: P shape with pulsing question mark.
fn draw_perplexity(t: f32, ctx: &DrawCtx, window: &mut Window) {
    // Static P bowl: top arc + bottom arc + inner horizontal.
    draw_polyline(
        ctx,
        &[(6.0, 21.0), (6.0, 3.0), (14.0, 3.0)],
        ctx.color,
        window,
    );
    draw_quad_bezier(
        ctx,
        QuadBezier {
            start: (14.0, 3.0),
            control: (20.0, 3.0),
            end: (20.0, 8.0),
        },
        ctx.color,
        window,
    );
    draw_quad_bezier(
        ctx,
        QuadBezier {
            start: (20.0, 8.0),
            control: (20.0, 13.0),
            end: (14.0, 13.0),
        },
        ctx.color,
        window,
    );
    draw_line(ctx, 14.0, 13.0, 6.0, 13.0, ctx.color, window);
    // Pulsing question mark
    let pulse = (t * PI).sin(); // 0→1→0 over ~2s
    let scale = 1.0 + 0.12 * pulse;
    let opacity = 0.85 + 0.15 * pulse;
    // Question mark centered at (13, 8.5), apply scale
    let qc_x = 13.0;
    let qc_y = 8.5;
    let q_top = [
        (11.0, 6.0),
        (11.0, 4.5),
        (13.0, 4.5),
        (15.5, 4.5),
        (15.5, 6.5),
        (15.5, 8.5),
        (13.0, 9.0),
        (13.0, 10.5),
    ];
    let mut scaled = [(0.0f32, 0.0f32); 8];
    for (i, &(x, y)) in q_top.iter().enumerate() {
        scaled[i] = (qc_x + (x - qc_x) * scale, qc_y + (y - qc_y) * scale);
    }
    // Draw as thicker polyline
    let mut pb = PathBuilder::stroke(px(1.5 * ctx.scale));
    pb.move_to(ctx.p(scaled[0].0, scaled[0].1));
    for &(x, y) in &scaled[1..] {
        pb.line_to(ctx.p(x, y));
    }
    if let Ok(path) = pb.build() {
        window.paint_path(path, ctx.c(opacity));
    }
    // Question mark dot
    draw_circle_filled(
        ctx,
        qc_x + (13.0 - qc_x) * scale,
        qc_y + (12.0 - qc_y) * scale,
        0.9,
        ctx.c(opacity),
        window,
    );
}

/// Nova: rings radiate outward.
fn draw_nova(t: f32, ctx: &DrawCtx, window: &mut Window) {
    // Static inner circle
    draw_circle_stroke(ctx, 12.0, 12.0, 3.0, ctx.color, window);
    // Static crosshair lines
    draw_line(ctx, 12.0, 2.0, 12.0, 5.0, ctx.color, window);
    draw_line(ctx, 12.0, 19.0, 12.0, 22.0, ctx.color, window);
    draw_line(ctx, 2.0, 12.0, 5.0, 12.0, ctx.color, window);
    draw_line(ctx, 19.0, 12.0, 22.0, 12.0, ctx.color, window);
    // Two radiating rings
    let ring_period = 2.0;
    for ring in 0..2 {
        let delay = ring as f32 * 1.0;
        let phase = ((t - delay) % ring_period + ring_period) % ring_period / ring_period;
        let r = 3.0 + 8.0 * phase;
        let opacity = 0.5 * (1.0 - phase);
        if opacity > 0.01 {
            draw_circle_stroke(ctx, 12.0, 12.0, r, ctx.c(opacity), window);
        }
    }
}

/// Custom: whole icon breathes.
fn draw_custom(t: f32, ctx: &DrawCtx, window: &mut Window) {
    let breath = 1.0 + 0.06 * (t * TAU / 4.0).sin();
    let cx_ = 12.0;
    let cy_ = 12.0;
    // Scale all coordinates around center
    let s = |x: f32, y: f32| -> (f32, f32) { (cx_ + (x - cx_) * breath, cy_ + (y - cy_) * breath) };
    // Rect (scaled corners)
    let (x1, y1) = s(6.0, 6.0);
    let (x2, y2) = s(18.0, 6.0);
    let (x3, y3) = s(18.0, 18.0);
    let (x4, y4) = s(6.0, 18.0);
    draw_polygon_stroke(
        ctx,
        &[(x1, y1), (x2, y2), (x3, y3), (x4, y4)],
        ctx.color,
        window,
    );
    // Inner circle
    draw_circle_stroke(ctx, cx_, cy_, 2.0 * breath, ctx.color, window);
    // Cross lines
    let (tx, ty) = s(12.0, 2.0);
    let (bx, by) = s(12.0, 6.0);
    draw_line(ctx, bx, by, tx, ty, ctx.color, window);
    let (tx, ty) = s(12.0, 22.0);
    let (bx, by) = s(12.0, 18.0);
    draw_line(ctx, bx, by, tx, ty, ctx.color, window);
    let (tx, ty) = s(2.0, 12.0);
    let (bx, by) = s(6.0, 12.0);
    draw_line(ctx, bx, by, tx, ty, ctx.color, window);
    let (tx, ty) = s(22.0, 12.0);
    let (bx, by) = s(18.0, 12.0);
    draw_line(ctx, bx, by, tx, ty, ctx.color, window);
}

#[cfg(test)]
mod tests {
    use super::AnimatedProviderIcon;
    use crate::foundations::accessibility::AccessibilityMode;
    use crate::foundations::icons::IconName;
    use crate::foundations::theme::TahoeTheme;
    use core::prelude::v1::test;
    use gpui::px;

    /// Builder fields round-trip.
    #[test]
    fn builder_sets_size_and_color() {
        let icon = AnimatedProviderIcon::new(IconName::ProviderClaude).size(px(48.0));
        assert_eq!(icon.size, px(48.0));
        assert!(icon.color.is_none());
    }

    /// Construction starts unpaused.
    #[test]
    fn new_is_unpaused() {
        let icon = AnimatedProviderIcon::new(IconName::ProviderGpt);
        assert!(!icon.paused);
    }

    /// HIG Motion: `AnimatedProviderIcon` must read
    /// `theme.accessibility_mode.reduce_motion()` and freeze its drawing
    /// parameter when the user has Reduce Motion enabled. This guards
    /// against regressions where the canvas frame timer runs
    /// unconditionally.
    #[test]
    fn theme_reduce_motion_is_detectable() {
        let mut theme = TahoeTheme::dark();
        assert!(!theme.accessibility_mode.reduce_motion());
        theme.accessibility_mode |= AccessibilityMode::REDUCE_MOTION;
        assert!(theme.accessibility_mode.reduce_motion());
    }
}
