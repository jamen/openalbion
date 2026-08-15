// Local detail repeated meshes — a transcription of VSHADER_REPEATED_MESH and
// PSHADER_REPEATED_MESH. Register names are from CVertexShaderConstantLayout's repeated-mesh
// subclass (`fableengine/engine_vs_layout_repeated_mesh.cpp:60-91`), which gives this pass a
// layout of its own:
//
//   c19..c34  ObjectMatricies           (.Offset = 0x13, .Count = 0x10)
//   c35..c50  ObjectOffsets             (.Offset = 0x23, .Count = 0x10)
//   c51..c66  LightingResults           (.Offset = 0x33, .Count = 0x10)
//   c67..c82  MainLightLightingResults  (.Offset = 0x43, .Count = 0x10)
//   c83..c95  User                      (.Offset = 0x53, .Count = 0xd)
//
//   vs_1_1
//   dcl_texcoord0 v0                     ; v0 = object-space position
//   dcl_texcoord1 v1                     ; v1 = normal, declared and never read
//   dcl_texcoord2 v2                     ; v2 = uv
//   dcl_texcoord3 v3                     ; v3 = which of the 16 batched instances
//   mov a.x, v3.xyzw
//   mov r0.xyzw, c[a.xyzw + 19].xyzw     ; ObjectMatricies[i] = (cos*s, sin*s, 0, 0)
//   mov r1.xyzw, c[a.xyzw + 35].xyzw     ; ObjectOffsets[i]   = (x, y, z, scale)
//   mul r2.xyzw, r0.xyzw, c85.xyzw       ; c85 = User[2] = (1, -1, 1, ...) -> row 0
//   mul r3.xyzw, r0.yxzz, c0.yyyx        ; c0 = (0, 1, 2, 0.5) preset      -> row 1
//   dp4 r5.x, v0.xyzw, r2.xyzw           ; x' =  cos*s*x - sin*s*y
//   dp4 r5.y, v0.xyzw, r3.xyzw           ; y' =  sin*s*x + cos*s*y
//   mul r5.z, v0.z, r1.w                 ; z' =  scale*z
//   mov r5.w, c0.y
//   add r5.xyz, r5.xyzz, r1.xyzz         ; + the object's world position
//   mov oT0.xyzw, v2.xyzw
//   dp4 r0.x, r5.xyzw, c5.xyzw           ; c5..c8 = CombinedProjectionMatrix. World-space
//   dp4 r0.y, r5.xyzw, c6.xyzw           ;   geometry, like static meshes and unlike the
//   dp4 r0.z, r5.xyzw, c7.xyzw           ;   landscape, so no `-c4` (AGENTS.md §3.11)
//   dp4 r0.w, r5.xyzw, c8.xyzw
//   mov oPos.xyzw, r0.xyzw
//   mov oD0.w, c0.y                      ; alpha 1; the STIPPLE_ALPHA variant computes a fade
//   dp4 r1.x, r5.xyzw, c2.xyzw           ; c2 = FogTransform
//   mov r1.w, c0.y
//   min r1.x, r1.x, r1.w
//   mad oFog.xyzw, r1.x, -c18.w, r1.w    ; c18 = FogColour
//   mov oD0.xyz, c[a.xyzw + 51].xyzw     ; LightingResults[i] -- ONE colour per object
//
//   ps_1_1
//   tex t0.xyzw
//   mul_x2 r0.xyz, v0.xyzw, t0.xyzw
//   +mov r0.w, t0.w
//
// Two things this shader says that are worth reading off it directly:
//
// **Repeated meshes are lit once per object, from the landscape normal.** The vertex shader
// never touches v1. `ProcessLightingSW` (`engine_primitive_manager_repeated_meshes.cpp:1063`)
// calls `CalcSWLightingNoClip(pos, normal, lights)` per object into `LightingResults`, with
// the ground normal the generator recorded — which is why
// `CLocalDetailObjectCollectionType`'s constructor forces `LandscapeNormalLighting` on for
// this primitive type. That function (`fableengine/engine_lighting.cpp:2600`) computes
//
//   d    = -(DiffuseVector . normal)
//   col  = PrimitiveAmbientColour
//        + saturate(d)^2 * PrimitiveDiffuseColour
//        + max(-d, 0)    * PrimitiveBacklightColour
//
// which is `VSHADER_STATIC_DIRLIGHT`'s expression over the same four constants, not merely a
// similar one — so the static mesh pass, the landscape pass and this one light from one set
// of environment LUT rows (AGENTS.md step 2).
//
// DIVERGENCE, deliberate: we evaluate it here from a per-instance normal rather than on the
// CPU into `LightingResults`. Both inputs are per instance either way, so the result is
// identical; keeping it in the shader means the lighting constants live in exactly one place
// and step 2 changes all three passes at once.
// (The local point lights `CalcSWLightingNoClip` goes on to add are the 113 CTCPhysicsLight
// things, deferred with the rest of local lighting — AGENTS.md step 6.10.)
//
// **Only a Z rotation and a Z scale survive.** `BuildFromSourceMeshes`
// (`engine_local_detail_primitives.cpp:2799`) writes `ObjectMatricies[i]` as
// `(cos*Scale, sin*Scale, 0, 0)` and `ObjectOffsets[i]` as `(E41, E42, E43, Scale)` — so the
// tilt-to-slope basis `AddObjectsFromLayerElement` composed into the placement matrix is
// **dropped** for repeated meshes, even though many of them set `TiltToSlope` in their defs.
// The two zeroed components are where `SetupWindAnimation` writes the wind skew; wind is its
// own step, and they stay zero until then.
//
// DIVERGENCE, deliberate: the original packs 16 instances into vertex constants and picks one
// with the address register, because vs_1_1 has no instancing. We use a per-instance vertex
// buffer and one draw per mesh. The arithmetic above is transcribed unchanged; only the
// delivery of the constants differs, and the batch size of 16 has no observable effect to
// preserve.

// Per frame. The same registers `model.wgsl` reads, by their names in the Lights layout
// (§3.8) — this pass's own layout renumbers the instance data, not the lighting.
struct Frame {
    // c5..c8
    view_proj: mat4x4<f32>,
    // c3 / c19 / c20 / c35 — see lighting.wgsl
    lighting: Lighting,
};

struct Material {
    // The alpha test the object type asks for, `AlphaRef / 255`. Unlike the static mesh
    // pass's cutoff this is read, not invented: `CLocalDetailObjectCollectionType`'s
    // constructor takes `AlphaRef` from the def and falls back to the ENGINE def's
    // `LocalDetailBooleanAlphaDefaultAlphaRef` or `DefaultPrimitiveAlphaRef`.
    alpha_cutoff: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
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
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    // c[a + 19] — (cos * scale, sin * scale, wind, wind)
    @location(3) rotation: vec4<f32>,
    // c[a + 35] — (x, y, z, scale)
    @location(4) offset: vec4<f32>,
    // The landscape normal under the object, which `LightingResults[i]` was computed from.
    @location(5) ground_normal: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    // mul r2, r0, c85 ; mul r3, r0.yxzz, c0.yyyx ; dp4 r5.x, v0, r2 ; dp4 r5.y, v0, r3
    //
    // c85's x/y/z are (1, -1, 1), written by `UploadShaderConstants`
    // (`engine_primitive_manager_repeated_meshes.cpp:1208`) as user constant 2. With the
    // wind components at zero the two dot products reduce to the rotation below; the `v0.z`
    // terms both read `r0.z`, so wind displaces x and y together rather than along a
    // per-object direction.
    let rotated = vec2<f32>(
        rotation.x * position.x - rotation.y * position.y,
        rotation.y * position.x + rotation.x * position.y,
    );

    // mul r5.z, v0.z, r1.w ; add r5.xyz, r5.xyz, r1.xyz
    let world = vec3<f32>(
        rotated.x + offset.x,
        rotated.y + offset.y,
        position.z * offset.w + offset.z,
    );

    // dp4 oPos, r5, c5..c8
    out.clip_position = frame.view_proj * vec4<f32>(world, 1.0);

    // mov oD0.xyz, c[a + 51] — CalcSWLightingNoClip's expression, over the ground normal.
    // The vertex normal is declared by the shader and never read, so it is discarded here.
    _ = normal;
    let n_dot_l = dot(ground_normal.xyz, -frame.lighting.light_dir.xyz);
    let lit = max(n_dot_l, 0.0);
    let back = min(n_dot_l, 0.0);
    out.light = frame.lighting.ambient.rgb + lit * lit * frame.lighting.diffuse.rgb - back * frame.lighting.backlight.rgb;

    // mov oT0, v2
    out.uv = uv;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // tex t0
    let base = textureSample(base_texture, base_sampler, in.uv);

    // The original does this with D3DRS_ALPHATESTENABLE and D3DRS_ALPHAREF around the draw,
    // not in the shader. Without it the alpha channel of a grass blade would draw as opaque
    // black, since the pass does not blend.
    if base.a < material.alpha_cutoff {
        discard;
    }

    // mul_x2 r0.xyz, v0, t0 ; +mov r0.w, t0.w
    return vec4<f32>(base.rgb * in.light * 2.0, base.a);
}
