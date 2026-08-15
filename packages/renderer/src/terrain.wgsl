// Landscape foreground — a transcription of VSHADER_LANDSCAPE_FOREGROUND and
// PSHADER_LANDSCAPE_FOREGROUND. Register names are from CVertexShaderConstantLayout's
// Lights subclass (AGENTS.md §3.8); `User` starts at c38, so the pass's own constants
// c40/c41/c42 are user 2/3/4.
//
//   vs_1_1
//   dcl_texcoord0 v0                     ; v0.xy = X, Y      (world cell coords)
//   dcl_texcoord1 v1                     ; v1.x  = Z         (height)
//   dcl_texcoord2 v2                     ; v2    = normal
//   dcl_texcoord3 v3                     ; v3    = (Blend, CliffU, CliffV, -) as D3DCOLOR
//   mov r0.xy, v0.xyzw
//   mov r0.z, v1.x
//   mov r0.w, c0.y                       ; c0.y = 1.0 preset
//   add r1.xyzw, r0.xyzw, -c4.xyzw       ; c4 = CameraPos -- geometry is camera-relative
//   dp4 oPos.x, r1.xyzw, c5.xyzw         ; c5..c8 = CombinedProjectionMatrix
//   dp4 oPos.y, r1.xyzw, c6.xyzw
//   dp4 oPos.z, r1.xyzw, c7.xyzw
//   dp4 oPos.w, r1.xyzw, c8.xyzw
//   add r2.xyzw, r0.xyzz, -c4.xyzz
//   dp3 r2.xyzw, r2.xyzw, r2.xyzw
//   rsq r2.xyzw, r2.w
//   rcp r2.xyzw, r2.w                    ; r2 = distance to camera, in every component
//   dp3 r4.xyzw, v2.xyzw, -c19.xyzw      ; c19 = light[0] direction
//   max r4.x, r4.x, c0.x                 ; c0.x = 0.0 preset
//   min r4.y, r4.y, c0.x
//   mul r4.x, r4.x, r4.x                 ; SQUARED n.l
//   mul r3.xyzw, r4.x, c20.xyzw          ; c20 = light[0] colour
//   mad r3.xyzw, -r4.y, c35.xyzw, r3.xyzw; c35 = backlight
//   add r3.xyzw, r3.xyzw, c3.xyzw        ; c3 = Ambient
//   mov oT0.xy, v3.yzzz                  ; blend table lookup = (CliffU, CliffV)
//   mov oT0.zw, c0.y
//   dp3 r5.xyzw, r2.xyzw, c42.xyzw       ; c42 = ForegroundFadeTransform
//   add r4.xyzw, r5.xyzw, c42.w
//   min r4.xyzw, r4.xyzw, c0.y
//   mul oD0.w, r4.xyzw, v3.x             ; alpha = fade * Blend
//   mov oD0.xyz, r3.xyzw
//   dp4 r5.x, r0.xyzw, c40.xyzw          ; c40/c41 = PositionToTextureUVTransformU/V
//   dp4 r5.y, r0.xyzw, c41.xyzw          ;   for this layer's mapping direction
//   mov oT1.xyzw, r5.xyzw
//
//   ps_1_1
//   tex t0.xyzw                          ; t0 = ForegroundBlendTables[MappingDirection]
//   tex t1.xyzw                          ; t1 = the layer's ground texture
//   mul_x2_sat r0.xyz, t1.xyzw, v0.xyzw
//   mul_sat r0.w, t0.w, v0.w
//
// The fog instructions (oFog from c2/c18) are omitted: nothing sets up fog yet.
//
// The layers are preceded by VSHADER_LANDSCAPE_FOREGROUND_BLACKOUT_PASS over the same
// meshes, which is the transform above followed by:
//
//   mov oD0.xyzw, c0.x                   ; c0.x = 0.0 preset -- solid black
//
// and then composited ADDITIVELY rather than source-alpha-over. That pairing is what makes
// the layer alphas correct: they sum to exactly 1 at every point (the theme weights are
// renormalised to 255, and the five mapping directions partition unity), so
// `sum(alpha_i * colour_i)` over black is a true weighted average that covers completely.
// Alpha-over would instead give `1 - prod(1 - alpha_i)`, which leaks the background
// anywhere no single layer is at full strength -- visible as sky bleeding through the
// seams between themes.

// Per frame. Names are the register's name in the layout, not ours.
struct Frame {
    // c5..c8
    view_proj: mat4x4<f32>,
    // c4
    camera_pos: vec4<f32>,
    // c3 / c19 / c20 / c35 — see lighting.wgsl
    lighting: Lighting,
    // c42
    fade_transform: vec4<f32>,
};

// Per draw — one layer's mapping direction, as the projection it implies.
struct Draw {
    // c40
    uv_transform_u: vec4<f32>,
    // c41
    uv_transform_v: vec4<f32>,
};

@group(0) @binding(0) var<uniform> frame: Frame;

@group(1) @binding(0) var<uniform> draw: Draw;
// t1 — the layer's ground texture, wrapped.
@group(1) @binding(1) var ground_texture: texture_2d<f32>;
@group(1) @binding(2) var ground_sampler: sampler;
// t0 — the mapping direction's blend table, clamped: it is a lookup, not a tile.
@group(1) @binding(3) var blend_table: texture_2d<f32>;
@group(1) @binding(4) var blend_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // oD0.xyz
    @location(0) light: vec3<f32>,
    // oD0.w
    @location(1) alpha: f32,
    // oT0
    @location(2) cliff_uv: vec2<f32>,
    // oT1
    @location(3) ground_uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) blend: f32,
    @location(3) cliff_uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    // mov r0.xy, v0 / mov r0.z, v1.x / mov r0.w, c0.y
    let world = vec4<f32>(position, 1.0);

    // add r1, r0, -c4 ; dp4 oPos, r1, c5..c8
    out.clip_position = frame.view_proj * vec4<f32>(position - frame.camera_pos.xyz, 1.0);

    // add r2, r0.xyzz, -c4.xyzz ; dp3 ; rsq ; rcp — the distance, splatted
    let distance = length(position - frame.camera_pos.xyz);

    // dp3 r4, v2, -c19 ; max r4.x, r4.x, 0 ; min r4.y, r4.y, 0 ; mul r4.x, r4.x, r4.x
    let n_dot_l = dot(normal, -frame.lighting.light_dir.xyz);
    let lit = max(n_dot_l, 0.0);
    let back = min(n_dot_l, 0.0);

    // mul r3, r4.x*r4.x, c20 ; mad r3, -r4.y, c35, r3 ; add r3, r3, c3
    out.light = lit * lit * frame.lighting.diffuse.rgb - back * frame.lighting.backlight.rgb + frame.lighting.ambient.rgb;

    // dp3 r5, r2, c42 ; add r4, r5, c42.w ; min r4, c0.y
    let fade = min(
        dot(vec3<f32>(distance, distance, distance), frame.fade_transform.xyz)
            + frame.fade_transform.w,
        1.0,
    );

    // mul oD0.w, r4, v3.x
    out.alpha = fade * blend;

    // mov oT0.xy, v3.yzzz
    out.cliff_uv = cliff_uv;

    // dp4 r5.x, r0, c40 ; dp4 r5.y, r0, c41 ; mov oT1, r5
    out.ground_uv = vec2<f32>(dot(world, draw.uv_transform_u), dot(world, draw.uv_transform_v));

    return out;
}

// VSHADER_LANDSCAPE_FOREGROUND_BLACKOUT_PASS — the same transform, emitting `c0.x`.
@vertex
fn vs_blackout(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) blend: f32,
    @location(3) cliff_uv: vec2<f32>,
) -> @builtin(position) vec4<f32> {
    return frame.view_proj * vec4<f32>(position - frame.camera_pos.xyz, 1.0);
}

@fragment
fn fs_blackout() -> @location(0) vec4<f32> {
    // mov oD0.xyzw, c0.x
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // tex t0 — the original's table is A8 and the shader reads t0.w; ours is R8.
    let mask = textureSample(blend_table, blend_sampler, in.cliff_uv).r;
    // tex t1
    let ground = textureSample(ground_texture, ground_sampler, in.ground_uv);

    // mul_x2_sat r0.xyz, t1, v0
    let colour = saturate(ground.rgb * in.light * 2.0);
    // mul_sat r0.w, t0.w, v0.w
    let alpha = saturate(mask * in.alpha);

    return vec4<f32>(colour, alpha);
}
