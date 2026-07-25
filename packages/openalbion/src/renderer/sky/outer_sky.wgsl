// Outer-sky shader — matches VSHADER_OUTER_SKY (vs_1_1) + PSHADER_OUTER_SKY (ps_1_1).
//
// Vertex shader computes the per-vertex diffuse colour:
//   diffuse = vc * horizon_colour + (1 - vc) * zenith_colour
// where vc is the dome vertex colour (0 at zenith, 1 at horizon), and
// zenith_colour / horizon_colour are per-frame constants from the LUT
// (rows 13–16, looked up on the CPU).
//
// Fragment shader blends the two sky textures, then lerps between the
// blended texture and the vertex diffuse using diffuse.a as the factor
// (lrp r0, v0.w, v0, r0 in the original).

struct Uniforms {
    view_proj: mat4x4<f32>,
    /// RGBA: RGB from LUT row 13 (SkyGradientTop), A from row 14 (TopAlpha).
    zenith_color: vec4<f32>,
    /// RGBA: RGB from LUT row 15 (SkyGradientBottom), A from row 16 (BottomAlpha).
    horizon_color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@group(1) @binding(0) var sky_texture_0: texture_2d<f32>;
@group(1) @binding(1) var sky_texture_1: texture_2d<f32>;
@group(1) @binding(2) var sky_sampler: sampler;

// group(2) — lighting-colours LUT — is declared in the pipeline layout
// but not sampled here; gradient colours are pre-computed on the CPU.

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

    // diffuse = vertex_colour * horizon + (1 - vertex_colour) * zenith
    // vc is 0 (0x00000000) at zenith and 1 (0xFFFFFFFF) at horizon.
    let vc = in.color;
    out.diffuse = vc * uniforms.horizon_color + (vec4<f32>(1.0) - vc) * uniforms.zenith_color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex0 = textureSample(sky_texture_0, sky_sampler, in.uv);
    let tex1 = textureSample(sky_texture_1, sky_sampler, in.uv);
    // The original blends the two textures by a constant (c0.w).
    // We use a fixed 0.5 — the dominant time transition comes from
    // the vertex-diffuse gradient, not from texture blending.
    let blended = mix(tex0, tex1, 0.5);

    // lrp r0, v0.w, v0, r0
    // At zenith   (diffuse.a ≈ 0): mostly sky texture
    // At horizon  (diffuse.a ≈ 1): mostly gradient colour
    return mix(blended, in.diffuse, in.diffuse.a);
}
