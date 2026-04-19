// Reference: Glass composite shader for GPUI upstream contribution.
// Combines blurred background with refraction, chromatic aberration,
// tint overlay, and Fresnel edge highlight (directional, light-aware).
//
// Field layout mirrors the Rust `LensEffect` so every Figma parameter
// (Refraction, Depth, Dispersion, Splay, Light angle, Light intensity)
// survives the Rust-to-shader boundary without being silently dropped.

struct GlassParams {
    bounds: vec4<f32>,           // x, y, width, height
    tint: vec4<f32>,             // RGBA tint color
    corner_radius: f32,
    refraction: f32,
    dispersion: f32,
    light_intensity: f32,
    // Depth of the glass surface (Figma: 0-100). Controls the parallax
    // envelope applied to refraction; larger values make the lens feel
    // thicker.
    depth: f32,
    // Edge splay distance in points (Figma: 0-100). Widens the Fresnel
    // highlight band so large panels get a softer, more diffuse edge.
    splay: f32,
    // Normalized direction of the light source (-1..1 in each axis).
    // Derived from Figma's `light_angle`: vec2(cos(a), sin(a)).
    light_dir: vec2<f32>,
}

@group(0) @binding(0) var blurred_tex: texture_2d<f32>;
@group(0) @binding(1) var blurred_sampler: sampler;
@group(0) @binding(2) var<uniform> glass: GlassParams;

// SDF for rounded rectangle (reuse GPUI's existing quad_sdf logic)
fn rounded_rect_sdf(pos: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let d = abs(pos) - half_size + vec2(radius);
    return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0) - radius;
}

@fragment
fn glass_composite(
    @location(0) uv: vec2<f32>,
    @builtin(position) frag_coord: vec4<f32>,
) -> @location(0) vec4<f32> {
    let center = glass.bounds.xy + glass.bounds.zw * 0.5;
    let half_size = glass.bounds.zw * 0.5;
    let local_pos = frag_coord.xy - center;

    // SDF mask for rounded rectangle
    let sdf = rounded_rect_sdf(local_pos, half_size, glass.corner_radius);
    if sdf > 0.0 {
        discard;
    }

    // Normalized distance from center (0 = center, 1 = edge)
    let normalized_dist = length(local_pos / half_size);

    // --- Refraction: parabolic UV distortion, scaled by depth ---
    // `depth` (Figma 0..100) modulates the refraction envelope so thicker
    // glass bends more. Scale by 0.01 to bring Figma units to [0,1].
    let depth_scale = clamp(glass.depth * 0.01, 0.0, 1.0);
    let distortion = (1.0 - normalized_dist * normalized_dist) * (1.0 + depth_scale);
    let direction = normalize(local_pos);
    let offset = distortion * direction * glass.refraction * 0.5;
    let refracted_uv = uv - offset / glass.bounds.zw;

    // --- Chromatic aberration ---
    let ca_shift = normalized_dist * glass.dispersion;
    let ca_dir = direction / glass.bounds.zw;
    var color: vec4<f32>;
    color.r = textureSample(blurred_tex, blurred_sampler, refracted_uv - ca_dir * ca_shift).r;
    color.g = textureSample(blurred_tex, blurred_sampler, refracted_uv).g;
    color.b = textureSample(blurred_tex, blurred_sampler, refracted_uv + ca_dir * ca_shift).b;
    color.a = 1.0;

    // --- Tint overlay ---
    color = mix(color, glass.tint, glass.tint.a);

    // --- Fresnel edge highlight (directional, splay-aware) ---
    // Isotropic Fresnel (distance to the rounded edge) scales down based on
    // the fragment's alignment with `light_dir`: the lit side of the glass
    // glows brighter than the shadowed side. `splay` widens the highlight
    // band in points so larger panels get a softer falloff.
    let edge_dist = abs(sdf);
    let splay_px = max(glass.splay, 1.0);
    let edge_band = max(glass.corner_radius * 0.5, splay_px);
    let edge_falloff = smoothstep(edge_band, 0.0, edge_dist);
    // Normalize the fragment's outward direction once; dot with light_dir
    // gives [-1,1] and we clamp to [0,1] so the shadowed side is dark rather
    // than negative. At the exact center `direction` has already been
    // normalized; fall back to light_dir to avoid NaN on near-zero vectors.
    let dir_ok = length(local_pos) > 0.001;
    let facing = select(1.0, clamp(dot(direction, glass.light_dir), 0.0, 1.0), dir_ok);
    let edge_glow = edge_falloff * facing * glass.light_intensity;
    color = color + vec4(edge_glow, edge_glow, edge_glow, 0.0);

    // --- Anti-alias at SDF boundary ---
    let aa = 1.0 - smoothstep(-1.0, 0.0, sdf);
    color.a *= aa;

    return color;
}
