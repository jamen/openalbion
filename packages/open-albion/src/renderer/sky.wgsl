// Outer sky shader based on Fable's VSHADER_OUTER_SKY and PSHADER_OUTER_SKY
//
// The outer sky renders the background sky dome with:
// - Two sky textures that can be blended for time-of-day transitions
// - Dynamic gradient colors sampled from the lighting LUT based on time of day
// - The vertex elevation controls how much gradient shows vs texture

struct Uniforms {
    view_proj: mat4x4<f32>,
    time_of_day: f32,  // 0.0 to 24.0 hours
    sky_blend: f32,    // 0.0 to 1.0 blend between sky_texture_0 and sky_texture_1
    _padding: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// Two sky textures for blending
@group(1) @binding(0) var sky_texture_0: texture_2d<f32>;
@group(1) @binding(1) var sky_texture_1: texture_2d<f32>;
@group(1) @binding(2) var sky_sampler: sampler;

// Lighting lookup table
@group(2) @binding(0) var lighting_lut: texture_2d<f32>;
@group(2) @binding(1) var lut_sampler: sampler;

// Lighting LUT row indices (V = (row + 0.5) / 21.0)
const LUT_ROW_COUNT: f32 = 21.0;
const LUT_SKY_GRADIENT_TOP: f32 = 13.0;
const LUT_SKY_GRADIENT_TOP_ALPHA: f32 = 14.0;
const LUT_SKY_GRADIENT_BOTTOM: f32 = 15.0;
const LUT_SKY_GRADIENT_BOTTOM_ALPHA: f32 = 16.0;

fn sample_lut(row: f32) -> vec4<f32> {
    let u = uniforms.time_of_day / 24.0;
    let v = (row + 0.5) / LUT_ROW_COUNT;
    return textureSample(lighting_lut, lut_sampler, vec2<f32>(u, v));
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,  // Vertex color (unused now, kept for compatibility)
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) elevation: f32,  // 0.0 at horizon, 1.0 at zenith
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Transform position, push to far plane for sky rendering
    var clip_pos = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    clip_pos.z = clip_pos.w * 0.9999;
    out.clip_position = clip_pos;

    // Pass through UVs - flip V since texture is stored top-down
    out.uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);

    // Pass elevation (stored in UV.y, which is 0 at horizon, 1 at zenith)
    out.elevation = in.uv.y;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample both sky textures
    let tex_color_0 = textureSample(sky_texture_0, sky_sampler, in.uv);
    let tex_color_1 = textureSample(sky_texture_1, sky_sampler, in.uv);

    // Blend between the two sky textures based on sky_blend uniform
    let tex_color = mix(tex_color_0, tex_color_1, uniforms.sky_blend);

    // Sample gradient colors from lighting LUT based on current time
    let sky_top = sample_lut(LUT_SKY_GRADIENT_TOP);
    let sky_top_alpha = sample_lut(LUT_SKY_GRADIENT_TOP_ALPHA);
    let sky_bottom = sample_lut(LUT_SKY_GRADIENT_BOTTOM);
    let sky_bottom_alpha = sample_lut(LUT_SKY_GRADIENT_BOTTOM_ALPHA);

    // Interpolate gradient color based on elevation
    // elevation: 0.0 = horizon (bottom), 1.0 = zenith (top)
    let gradient_color = mix(sky_bottom.rgb, sky_top.rgb, in.elevation);

    // Alpha controls blend between texture and gradient
    // Higher alpha = more gradient color visible
    let gradient_alpha = mix(sky_bottom_alpha.r, sky_top_alpha.r, in.elevation);

    // Blend texture with gradient
    // When gradient_alpha is high, we see more of the gradient color
    // When gradient_alpha is low, we see more of the sky texture
    let final_rgb = mix(tex_color.rgb, gradient_color, gradient_alpha);

    return vec4<f32>(final_rgb, 1.0);
}
