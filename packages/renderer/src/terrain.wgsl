// Landscape — PLACEHOLDER.
//
// The original renders the landscape as a background pass plus N alpha-blended
// per-theme layer passes (VSHADER/PSHADER_LANDSCAPE_FOREGROUND); the layer texture
// supplies the alpha mask and the colour comes from a planar-projected composited
// surface texture. Lighting is
//     Ambient + saturate(n·l)^2 * Diffuse + max(-n·l, 0) * Backlight
// with all three colours fetched from the environment colour LUT.
// See AGENTS.md §3.4; the real implementation is step 5.
//
// Until then this draws untextured flat-lit geometry so the mesh itself stays
// inspectable. It deliberately does NOT sample the theme textures: the previous
// single-pass 3-way blend, the slope-driven cliff mix and the hardcoded light
// direction were all inventions with no counterpart in the original.

struct Uniforms {
    view_proj: mat4x4<f32>,
    texture_scale: f32,
    _pad: vec3<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = u.view_proj * vec4<f32>(position, 1.0);
    out.normal = normal;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // UNVERIFIED: placeholder light. The real direction and the ambient/diffuse/
    // backlight colours come from the environment LUT (rows 1/0/3) via the shader
    // constant registers — see AGENTS.md §3.4, §3.8. Logged in AGENTS.md §9.
    let light_dir = normalize(vec3<f32>(0.4, 0.3, 1.0));
    let n = normalize(in.normal);
    let shade = 0.25 + max(dot(n, light_dir), 0.0) * 0.75;
    return vec4<f32>(vec3<f32>(shade), 1.0);
}
