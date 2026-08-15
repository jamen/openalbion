// Static meshes — a transcription of VSHADER_STATIC_DIRLIGHT and PSHADER_TEXTURE_DIFFUSE.
// Register names are from CVertexShaderConstantLayout's Lights subclass (AGENTS.md §3.8).
//
//   vs_1_1
//   dcl_texcoord0 v0                     ; v0 = object-space position
//   dcl_texcoord1 v1                     ; v1 = object-space normal
//   dcl_texcoord2 v2                     ; v2 = uv
//   mov r1.xyzw, v0.xyzw
//   dp4 r0.x, r1.xyzw, c5.xyzw           ; c5..c8 = CombinedProjectionMatrix
//   dp4 r0.y, r1.xyzw, c6.xyzw           ;   = Projection x View x World, so the object
//   dp4 r0.z, r1.xyzw, c7.xyzw           ;   matrix is already folded in and there is no
//   dp4 r0.w, r1.xyzw, c8.xyzw           ;   `-c4` here (CShaderRenderManager:2960-3090)
//   mov oPos.xyzw, r0.xyzw
//   dp4 r2.x, r1.xyzw, c2.xyzw           ; c2 = FogTransform
//   mov r2.w, c0.y                       ; c0.y = 1.0 preset
//   min r2.x, r2.x, r2.w
//   mad oFog.xyzw, r2.x, -c18.w, r2.w    ; c18 = FogColour
//   mov r2.xyzw, v1.xyzw
//   dp3 r4.xyzw, r2.xyzw, -c19.xyzw      ; c19 = light[0] direction (LightArray.Offset = 0x13)
//   max r4.x, r4.x, c0.x                 ; c0.x = 0.0 preset
//   min r4.y, r4.y, c0.x
//   mul r4.x, r4.x, r4.x                 ; SQUARED n.l
//   mul r3.xyzw, r4.x, c20.xyzw          ; c20 = light[0] colour
//   mad r3.xyzw, -r4.y, c35.xyzw, r3.xyzw; c35 = LightGlobals -- the backlight
//   mov oD0.w, c0.y                      ; vertex alpha is a constant 1
//   add oD0.xyz, r3.xyzw, c3.xyzw        ; c3 = Ambient
//   mov oT0.xyzw, v2.xyzw
//
//   ps_1_1
//   tex t0.xyzw                          ; t0 = the material's base texture
//   mul r0.xyzw, v0.xyzw, c0.xyzw        ; c0 = the per-object colour (tint x fade alpha)
//   mul r0.w, t0.xyzw, r0.xyzw
//   mul_x2 r0.xyz, t0.xyzw, r0.xyzw
//
// This is the same lighting the landscape foreground runs -- `Ambient +
// saturate(n.l)^2 * Diffuse + max(-n.l, 0) * Backlight`, doubled in the pixel shader --
// over the same four constants, so both subsystems light from one set of environment LUT
// rows (AGENTS.md step 2). See `terrain.wgsl`.
//
// The fog instructions (oFog from c2/c18) are omitted: nothing sets up fog yet.
//
// DIVERGENCE, deliberate: the original evaluates `dp3 v1, -c19` in **object** space, which
// means it pre-transforms the light direction into each object's frame. We keep one
// world-space light shared with the landscape and rotate the *normal* into world space
// instead. For the orthonormal object matrices `CalcObjectMatrix` produces the two are
// exactly equal -- `(R n) . l == n . (R^T l)` -- and this way the light constant is the
// same byte for every pass.

// Per frame. Names are the register's name in the layout, not ours.
struct Frame {
    // c5..c8
    view_proj: mat4x4<f32>,
    // c3 / c19 / c20 / c35 — see lighting.wgsl
    lighting: Lighting,
};

struct Material {
    // Non-zero enables alpha testing (cutout): fragments below `alpha_cutoff` are discarded.
    alpha_test: u32,
    alpha_cutoff: f32,
    _pad0: f32,
    _pad1: f32,
};

@group(0) @binding(0) var<uniform> frame: Frame;

@group(1) @binding(0) var base_texture: texture_2d<f32>;
@group(1) @binding(1) var base_sampler: sampler;
@group(1) @binding(2) var<uniform> material: Material;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // oD0.xyz
    @location(0) light: vec3<f32>,
    // oT0
    @location(1) uv: vec2<f32>,
    // the pixel shader's c0
    @location(2) colour: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    // The object matrix, per instance: `CalcObjectMatrix`'s CMatrix3x4 as four columns.
    @location(3) object_0: vec4<f32>,
    @location(4) object_1: vec4<f32>,
    @location(5) object_2: vec4<f32>,
    @location(6) object_3: vec4<f32>,
    @location(7) colour: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    let object = mat4x4<f32>(object_0, object_1, object_2, object_3);

    // dp4 oPos, v0, c5..c8 -- the object matrix is the World half of c5..c8.
    out.clip_position = frame.view_proj * (object * vec4<f32>(position, 1.0));

    // dp3 r4, v1, -c19 -- see the DIVERGENCE note: the normal is rotated instead of the
    // light. `CalcObjectMatrix` scales uniformly, so normalising is enough to undo it.
    let world_normal = normalize(mat3x3<f32>(
        object_0.xyz,
        object_1.xyz,
        object_2.xyz,
    ) * normal);

    // max r4.x, r4.x, 0 ; min r4.y, r4.y, 0 ; mul r4.x, r4.x, r4.x
    let n_dot_l = dot(world_normal, -frame.lighting.light_dir.xyz);
    let lit = max(n_dot_l, 0.0);
    let back = min(n_dot_l, 0.0);

    // mul r3, r4.x*r4.x, c20 ; mad r3, -r4.y, c35, r3 ; add oD0.xyz, r3, c3
    out.light = lit * lit * frame.lighting.diffuse.rgb - back * frame.lighting.backlight.rgb + frame.lighting.ambient.rgb;

    // mov oT0, v2
    out.uv = uv;
    out.colour = colour;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // tex t0
    let base = textureSample(base_texture, base_sampler, in.uv);

    // Alpha-test (cutout) materials discard rather than blend. The original does this with
    // D3DRS_ALPHATESTENABLE around the same draw, not in the shader.
    if material.alpha_test != 0u && base.a < material.alpha_cutoff {
        discard;
    }

    // mul r0, v0, c0 -- oD0.w is a constant 1, so the alpha term is c0.w alone.
    let modulated = vec4<f32>(in.light, 1.0) * in.colour;

    // mul_x2 r0.xyz, t0, r0
    let colour = base.rgb * modulated.rgb * 2.0;
    // mul r0.w, t0, r0
    let alpha = base.a * modulated.a;

    return vec4<f32>(colour, alpha);
}
