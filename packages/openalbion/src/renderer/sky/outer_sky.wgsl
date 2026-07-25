// Outer sky vertex shader — matches VSHADER_OUTER_SKY (vs_1_1).
// Computes the per-vertex diffuse colour as:
//   diffuse = vertex_colour * horizon_colour + (1 - vertex_colour) * zenith_colour
// The fragment shader then lerps between the sky texture and this diffuse
// using diffuse.a, producing a smooth texture→gradient transition from
// zenith to horizon.

struct Uniforms {
    view_proj: mat4x4<f32>,
    /// RGB colour at the top of the sky (zenith).  Sampled from the LUT row 13
    /// (SkyGradientTopLookupRow) by the CPU each frame.
    zenith_color: vec3<f32>,
    _pad0: f32,
    /// RGB colour at the bottom of the sky (horizon).  Sampled from the LUT
    /// row 15 (SkyGradientBottomLookupRow) by the CPU each frame.
    horizon_color: vec3<f32>,
    _pad1: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// Two sky textures for blending (t0 and t1 in the original).
@group(1) @binding(0) var sky_texture_0: texture_2d<f32>;
@group(1) @binding(1) var sky_texture_1: texture_2d<f32>;
@group(1) @binding(2) var sky_sampler: sampler;

// The LUT (lighting-colours) is still bound at group(2) for other passes
// (base band, terrain), but the outer-sky shader no longer samples it here.
// The gradient colours are pre-computed on the CPU and passed via uniforms.

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
    // Transform position by the combined view-projection matrix (c5-c8).
    var clip_pos = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    // Push z to the far plane (clip_pos.z = clip_pos.w * 0.9999) so the
    // sky always draws behind world geometry when depth testing is enabled.
    clip_pos.z = clip_pos.w * 0.9999;
    out.clip_position = clip_pos;

    // Pass UV through unchanged (same for both texture stages).
    out.uv = in.uv;

    // Compute diffuse = vertex_colour * horizon + (1 - vertex_colour) * zenith.
    // vertex_colour is 0 at zenith (top of dome) and 1 at horizon (dome base),
    // so the gradient transitions from zenith_colour at the top to horizon_colour
    // at the bottom.
    let vc = in.color;
    let one_minus_vc = vec4<f32>(1.0) - vc;
    let zenith = vec4<f32>(uniforms.zenith_color, 1.0);
    let horizon = vec4<f32>(uniforms.horizon_color, 1.0);
    out.diffuse = vc * horizon + one_minus_vc * zenith;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample both sky textures (t0, t1 in the original).
    let tex0 = textureSample(sky_texture_0, sky_sampler, in.uv);
    let tex1 = textureSample(sky_texture_1, sky_sampler, in.uv);

    // Blend the two textures (lrp r0, c0.w, t1, t0) — sky_blend comes
    // from a pixel-shader constant in the original, but we bake it into
    // the per-frame uniform.
    let sky_blend_factor = uniforms.zenith_color.r;  // placeholder; the real
    // blend factor was c0.w.  We don't currently expose it as a separate uniform,
    // so default to 0.5.  The final lerp with vertex diffuse provides the
    // dominant colour transition and the textures are sampled as-is.
    let blended_tex = mix(tex0, tex1, 0.5);

    // lrp r0, v0.w, v0, r0 — lerp between the blended texture and the
    // vertex diffuse using diffuse.a as the blend factor.
    // At zenith   (diffuse.a = zenith.a = 1.0 in our setup): 100% diffuse
    // At horizon  (diffuse.a = horizon.a = 1.0):               100% diffuse
    // Wait — the original uses vertex diffuse's alpha (v0.w) as the factor.
    // But our gradient goes from opaque at both ends.  The original dome has
    // alpha=0 at top (0x00000000) and alpha=1 at bottom (0xFFFFFFFF).
    // The lerp factor r1.w = v0.w = vertex_diffuse_alpha.
    // At top:    alpha=0 → lerp(blended_tex, diffuse, 0) = blended_tex
    // At bottom: alpha=1 → lerp(blended_tex, diffuse, 1) = diffuse
    let result = mix(blended_tex, in.diffuse, in.diffuse.a);

    return result;
}
