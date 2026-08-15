// The four lighting registers every world pass reads (AGENTS.md §3.8, §12.7).
//
// Prepended to terrain.wgsl, model.wgsl and local_detail.wgsl at shader-module creation, so
// this declaration and `LightingUniforms` in lighting.rs are the only two places the layout
// exists — and the terrain, the static meshes and the repeated meshes cannot drift apart on
// what the original treats as one set of constants.

struct Lighting {
    // c3 — Ambient
    ambient: vec4<f32>,
    // c19 — light[0] direction (the directional light; it has no attenuation)
    light_dir: vec4<f32>,
    // c20 — light[0] colour
    diffuse: vec4<f32>,
    // c35 — LightGlobals, the backlight
    backlight: vec4<f32>,
};
