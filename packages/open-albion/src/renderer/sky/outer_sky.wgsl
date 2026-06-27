// Outer sky shader based on Fable's VSHADER_OUTER_SKY and PSHADER_OUTER_SKY
//
// Renders the background sky dome with time-of-day texture blending.

struct Uniforms {
    view_proj: mat4x4<f32>,
    time_of_day: f32,
    sky_blend: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// Two sky textures for blending
@group(1) @binding(0) var sky_texture_0: texture_2d<f32>;
@group(1) @binding(1) var sky_texture_1: texture_2d<f32>;
@group(1) @binding(2) var sky_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    var clip_pos = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    clip_pos.z = clip_pos.w * 0.9999;
    out.clip_position = clip_pos;

    out.uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color_0 = textureSample(sky_texture_0, sky_sampler, in.uv);
    let tex_color_1 = textureSample(sky_texture_1, sky_sampler, in.uv);
    let final_color = mix(tex_color_0, tex_color_1, uniforms.sky_blend);
    return vec4<f32>(final_color.rgb, 1.0);
}
