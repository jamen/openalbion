// Transcription of VSHADER_OUTER_SKY (vs_1_1) + PSHADER_OUTER_SKY (ps_1_1).
// Source: ~/dl/Fable Shader Disassembly.md, SHADERS_SKY.
// Vertex registers per CEngineVSConstantLayoutBasic (engine_vs_layout_basic.cpp:109-182).
//
//   VSHADER_OUTER_SKY
//     dp4 oPos.{x,y,z,w}, v0, c5..c8   ; CombinedProjectionMatrix
//     mul r0, v1, c93                  ; v1 = vertex diffuse, c93 = gradient bottom
//     add r1, c0.y, -v1                ; c0.y = 1.0 (reserved preset)
//     mul r1, r1, c92                  ; c92 = gradient top
//     add oD0, r0, r1
//     mov oT0, v2
//     mov oT1, v2                      ; both stages sample the SAME uv
//
//   PSHADER_OUTER_SKY
//     tex t0                           ; sky texture 0
//     tex t1                           ; sky texture 1
//     mov_sat r0, c0                   ; c0.w = sky texture blend factor
//     mov_sat r1, v0                   ; r1.w = saturate(oD0.a)
//     lrp r0, r0.w, t1, t0             ; lerp(t0, t1, saturate(c0.w))
//     lrp r0, r1.w, v0, r0             ; lerp(that, diffuse, saturate(diffuse.a))
//
// D3D `lrp dst, s0, s1, s2` = s2 + s0*(s1 - s2) = lerp(s2, s1, s0).
// Note the pixel shader's c0 is a pixel constant, unrelated to the vertex c0 preset.

// Rust side: SkyUniforms in sky.rs. Trailing tail padding is implicit — the struct's
// align is 16 (mat4x4), so a trailing f32 rounds the size up to 112, matching Rust's
// explicit `_pad: [f32; 3]`. Declaring `_pad: vec3<f32>` here would instead align the
// pad to 16 and push the size to 128.
struct Uniforms {
    view_proj: mat4x4<f32>,
    gradient_top: vec4<f32>,      // c92
    gradient_bottom: vec4<f32>,   // c93
    texture_blend: f32,           // pixel shader c0.w
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// Per draw: which slots of the shared bindless array the two texture stages are in. Both are
// sampled through `clamp_sampler` — a sky texture is wrapped around the dome by its UVs, not
// tiled, so a UV outside 0..1 is out of range rather than another tile.
struct DrawConstants {
    texture0_index: u32,
    texture1_index: u32,
    _pad0: u32,
    _pad1: u32,
};
var<immediate> draw: DrawConstants;

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

    // dp4 oPos.{x,y,z,w}, v0, c5..c8
    var clip_pos = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    // The sky has no depth buffer of its own and is drawn before the world; pin it to
    // the far plane so later depth-tested passes always win.
    clip_pos.z = clip_pos.w * 0.9999;
    out.clip_position = clip_pos;

    // mov oT0, v2 / mov oT1, v2
    out.uv = in.uv;

    // mul r0, v1, c93 / add r1, c0.y, -v1 / mul r1, r1, c92 / add oD0, r0, r1
    let vc = in.color;
    out.diffuse = vc * uniforms.gradient_bottom
                + (vec4<f32>(1.0) - vc) * uniforms.gradient_top;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // tex t0 / tex t1
    let t0 = textureSample(bindless_textures[draw.texture0_index], clamp_sampler, in.uv);
    let t1 = textureSample(bindless_textures[draw.texture1_index], clamp_sampler, in.uv);

    // mov_sat r0, c0 / lrp r0, r0.w, t1, t0
    var color = mix(t0, t1, saturate(uniforms.texture_blend));

    // mov_sat r1, v0 / lrp r0, r1.w, v0, r0
    color = mix(color, in.diffuse, saturate(in.diffuse.a));

    return vec4<f32>(color.rgb, 1.0);
}
