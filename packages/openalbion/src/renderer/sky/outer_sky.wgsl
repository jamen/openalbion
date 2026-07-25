// Outer sky shader based on Fable's VSHADER_OUTER_SKY and PSHADER_OUTER_SKY
//
// Renders the background sky dome with time-of-day texture blending and
// lighting-colours LUT tinting.

struct Uniforms {
    view_proj: mat4x4<f32>,
    time_of_day: f32,
    sky_blend: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@group(1) @binding(0) var sky_texture_0: texture_2d<f32>;
@group(1) @binding(1) var sky_texture_1: texture_2d<f32>;
@group(1) @binding(2) var sky_sampler: sampler;

@group(2) @binding(0) var lighting_lut: texture_2d<f32>;
@group(2) @binding(1) var lut_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) vertex_color: vec4<f32>,
    @location(2) height_fraction: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    var clip_pos = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    clip_pos.z = clip_pos.w * 0.9999;
    out.clip_position = clip_pos;

    out.uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);
    out.vertex_color = in.color;

    // Height fraction: 0 at zenith (Y=7000), 1 at horizon (Y=-500).
    out.height_fraction = (7000.0 - in.position.y) / 7500.0;

    return out;
}

fn sample_lut_row(row: f32) -> vec4<f32> {
    let time_u = uniforms.time_of_day / 24.0;
    let row_v = (row + 0.5) / 21.0;
    return textureSample(lighting_lut, lut_sampler, vec2<f32>(time_u, row_v));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color_0 = textureSample(sky_texture_0, sky_sampler, in.uv);
    let tex_color_1 = textureSample(sky_texture_1, sky_sampler, in.uv);
    let base_sky = mix(tex_color_0, tex_color_1, uniforms.sky_blend);

    let grad_top = sample_lut_row(13.0);
    let grad_top_a = sample_lut_row(14.0);
    let grad_bottom = sample_lut_row(15.0);
    let grad_bottom_a = sample_lut_row(16.0);

    let h = clamp(in.height_fraction, 0.0, 1.0);
    let gradient_color = mix(grad_top.rgb, grad_bottom.rgb, h);
    let gradient_alpha = mix(grad_top_a.r, grad_bottom_a.r, h);

    // Blend the sky texture with the LUT gradient — use the gradient to tint,
    // not the vertex alpha (which is 0 at zenith, causing the black clear colour
    // to bleed through). Output fully opaque so the clear colour never shows.
    let tinted = base_sky.rgb * gradient_color;
    let alpha = max(gradient_alpha, 1.0);

    return vec4<f32>(tinted, alpha);
}
