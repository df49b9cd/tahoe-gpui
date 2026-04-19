// Reference: Dual Kawase blur for GPUI upstream contribution.
// Algorithm: Marius Bjorge, ARM, SIGGRAPH 2015.
// Used by: KDE KWin, picom, Blender.
//
// This file is NOT compiled -- it serves as the implementation reference
// for the GPUI upstream PR that adds BlurRect rendering support.
//
// Color space: GPUI's default framebuffer format is `rgba8unorm` (sRGB-
// encoded). Averaging raw sRGB samples darkens high-contrast edges
// (Jensen 2001), so each downsample/upsample step linearizes samples
// before accumulating and re-encodes the result back to sRGB. If the
// input texture view is declared as `rgba8unorm-srgb`, the GPU performs
// this conversion in hardware and the `linearize` / `encode_srgb`
// helpers can be stripped.

struct BlurParams {
    viewport_size: vec2<f32>,
    offset_multiplier: f32,
    _pad: f32,
}

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> params: BlurParams;

// --- sRGB <-> linear-light helpers ---
// Gamma 2.2 is the canonical "fast" approximation of the full sRGB curve
// (within ~1% across [0,1]) and matches what Metal/DX uses when a
// texture view is declared as the non-_SRGB variant. Prefer the full
// piecewise function if precision matters for your target platform.

fn linearize(c: vec3<f32>) -> vec3<f32> {
    return pow(c, vec3<f32>(2.2));
}

fn encode_srgb(c: vec3<f32>) -> vec3<f32> {
    return pow(c, vec3<f32>(1.0 / 2.2));
}

fn sample_linear(uv: vec2<f32>) -> vec4<f32> {
    let s = textureSample(input_tex, input_sampler, uv);
    return vec4(linearize(s.rgb), s.a);
}

// --- Downsample pass ---
// Renders to a texture at half the resolution of the input.
// 5 texture samples per pixel, exploiting bilinear filtering.

@fragment
fn downsample(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let o = 0.5 / params.viewport_size * params.offset_multiplier;

    var color = sample_linear(uv) * 4.0;
    color += sample_linear(uv + vec2(-o.x, -o.y));
    color += sample_linear(uv + vec2( o.x, -o.y));
    color += sample_linear(uv + vec2(-o.x,  o.y));
    color += sample_linear(uv + vec2( o.x,  o.y));

    let averaged = color / 8.0;
    return vec4(encode_srgb(averaged.rgb), averaged.a);
}

// --- Upsample pass ---
// Renders to a texture at double the resolution of the input.
// 8 texture samples per pixel (4 diagonal + 4 cardinal).

@fragment
fn upsample(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let o = 0.5 / params.viewport_size * params.offset_multiplier;

    var color = sample_linear(uv + vec2(-o.x,  o.y)) * 2.0;
    color += sample_linear(uv + vec2( o.x,  o.y)) * 2.0;
    color += sample_linear(uv + vec2(-o.x, -o.y)) * 2.0;
    color += sample_linear(uv + vec2( o.x, -o.y)) * 2.0;
    color += sample_linear(uv + vec2(-o.x * 2.0, 0.0));
    color += sample_linear(uv + vec2( o.x * 2.0, 0.0));
    color += sample_linear(uv + vec2(0.0, -o.y * 2.0));
    color += sample_linear(uv + vec2(0.0,  o.y * 2.0));

    let averaged = color / 12.0;
    return vec4(encode_srgb(averaged.rgb), averaged.a);
}
