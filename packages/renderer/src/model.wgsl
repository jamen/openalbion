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

// Per draw, in immediate storage — wgpu's push constants. This replaces a whole bind group
// per material: a texture, a sampler and a two-field uniform buffer allocated for every
// material of every mesh (AGENTS.md §12.5). A WGSL module may declare at most one of these.
struct DrawConstants {
    // Which slot of `bindless_textures` this material's base map is in. Constant for the
    // whole draw, so the index is dynamically uniform and needs no non-uniform indexing.
    texture_index: u32,
    // Non-zero enables alpha testing (cutout): fragments below `alpha_cutoff` are discarded.
    alpha_test: u32,
    alpha_cutoff: f32,
    _pad: u32,
};

@group(0) @binding(0) var<uniform> frame: Frame;
var<immediate> draw: DrawConstants;

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

// ---------------------------------------------------------------------------------------------
// Skinned meshes — a transcription of VSHADER_PALSKIN_DIRLIGHT_FOG's blend, on top of the
// static shader above. The disassembly (`SHADERS_PALSKIN`, and AGENTS.md §3.15):
//
//   mul r2.xyzw, v1.zyxw, c1.xyzw        ; v1 = blend indices, c1 = (256,256,256,256) preset
//   mov r3.xyzw, v2.zyxw                 ; v2 = blend weights
//   mov a.x, r2.x
//   mul r4.xyzw, r3.x, c[a.xyzw + 38]    ; \ three registers per bone: a CMatrix3x4's rows.
//   mul r5.xyzw, r3.x, c[a.xyzw + 39]    ;  > BoneMatrices starts at c38.
//   mul r6.xyzw, r3.x, c[a.xyzw + 40]    ; /
//   mov a.x, r2.y                        ; then `mad`ded twice more, for bones 1 and 2
//   ...
//   dp4 r0.x, v0, r4                     ; position through the blended matrix
//   dp3 r1.x, v3, r4                     ; normal through the same
//
// From `dp4 r2.x, r0, c5` onward it is character-for-character `vs_main` above, so the
// lighting below is the same expression over the same constants.
//
// DIVERGENCE, deliberate, and the same shape as `local_detail.rs`'s: vs_1_1 indexes a constant
// bank with the address register because it has no other way to do it. We read a storage buffer
// instead, indexed per instance. The arithmetic is unchanged; only the storage is. The blend
// itself is exact — three bones, weights as stored.
@group(2) @binding(0) var<storage, read> bone_matrices: array<mat4x4<f32>>;

struct SkinnedFrame {
    // How many bones each instance's palette holds, so an instance's block can be addressed.
    bones_per_instance: u32,
};
@group(2) @binding(1) var<uniform> skin: SkinnedFrame;

@vertex
fn vs_skinned(
    @builtin(instance_index) instance: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    // Bone indices, already resolved through the block's palette on the CPU — so this is a
    // direct index into the mesh's bones rather than `Groups[]` slot times three.
    @location(3) bones: vec4<u32>,
    @location(4) weights: vec4<f32>,
    @location(5) object_0: vec4<f32>,
    @location(6) object_1: vec4<f32>,
    @location(7) object_2: vec4<f32>,
    @location(8) object_3: vec4<f32>,
    @location(9) colour: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    let object = mat4x4<f32>(object_0, object_1, object_2, object_3);
    let base = instance * skin.bones_per_instance;

    // mul r4, r3.x, c[a.x + 38] ... mad r4, r3.y, c[a.y + 38], r4 ... — three bones, weighted.
    // `bones_per_vertex` is 3 on every shipped animated block (AGENTS.md §3.15), so the fourth
    // slot is always zero-weighted and is not summed.
    var skinned = mat4x4<f32>(
        vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0),
    );
    for (var k = 0u; k < 3u; k = k + 1u) {
        let w = weights[k];
        if w > 0.0 {
            let m = bone_matrices[base + bones[k]];
            skinned[0] = skinned[0] + m[0] * w;
            skinned[1] = skinned[1] + m[1] * w;
            skinned[2] = skinned[2] + m[2] * w;
            skinned[3] = skinned[3] + m[3] * w;
        }
    }

    // dp4 r0.x/y/z, v0, r4/r5/r6 — position through the blend, then the object matrix and
    // c5..c8 exactly as the static path.
    let posed = skinned * vec4<f32>(position, 1.0);
    out.clip_position = frame.view_proj * (object * vec4<f32>(posed.xyz, 1.0));

    // dp3 r1.x/y/z, v3, r4/r5/r6 — the normal through the same blend, then into world space.
    let posed_normal = mat3x3<f32>(skinned[0].xyz, skinned[1].xyz, skinned[2].xyz) * normal;
    let world_normal = normalize(mat3x3<f32>(
        object_0.xyz,
        object_1.xyz,
        object_2.xyz,
    ) * posed_normal);

    let n_dot_l = dot(world_normal, -frame.lighting.light_dir.xyz);
    let lit = max(n_dot_l, 0.0);
    let back = min(n_dot_l, 0.0);
    out.light = lit * lit * frame.lighting.diffuse.rgb - back * frame.lighting.backlight.rgb + frame.lighting.ambient.rgb;

    out.uv = uv;
    out.colour = colour;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // tex t0 -- the material's base map, out of the shared bindless array. `repeat_sampler`
    // is D3D9's default WRAP addressing: 501 of 1500 meshes sampled out of graphics.big carry
    // UVs outside 0..1, so clamping is visibly wrong for a third of the mesh library.
    let base = textureSample(
        bindless_textures[draw.texture_index],
        repeat_sampler,
        in.uv,
    );

    // Alpha-test (cutout) materials discard rather than blend. The original does this with
    // D3DRS_ALPHATESTENABLE around the same draw, not in the shader.
    if draw.alpha_test != 0u && base.a < draw.alpha_cutoff {
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
