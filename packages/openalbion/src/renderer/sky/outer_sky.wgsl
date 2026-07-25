// Outer-sky shader — matches VSHADER_OUTER_SKY (vs_1_1) + PSHADER_OUTER_SKY (ps_1_1).
//
// TEMPORARY DEBUG MODE: outputs UV coordinates as colours so we can verify
// the dome geometry and UV mapping are correct.
//   Red   = U coordinate (should increase left-to-right across the dome)
//   Green = V coordinate (should increase top-to-bottom)
//   Blue  = 0
//
// Once UVs are verified, switch back to the texture+LUT rendering below.

struct Uniforms {
    view_proj: mat4x4<f32>,
    zenith_color: vec4<f32>,
    horizon_color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

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
    @location(1) diffuse: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var clip_pos = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    clip_pos.z = clip_pos.w * 0.9999;
    out.clip_position = clip_pos;
    out.uv = in.uv;

    let vc = in.color;
    out.diffuse = vc * uniforms.horizon_color + (vec4<f32>(1.0) - vc) * uniforms.zenith_color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // DEBUG: UV visualisation — red = U, green = V
    return vec4<f32>(in.uv.x, in.uv.y, 0.0, 1.0);
}
