// Outer sky shader based on Fable's VSHADER_OUTER_SKY and PSHADER_OUTER_SKY
//
// The outer sky renders the background sky dome with:
// - A sky texture (or blend of two textures for time-of-day transitions)
// - Vertex color gradient from horizon to zenith
// - The vertex alpha controls how much the gradient color shows vs the texture

struct Uniforms {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@group(1) @binding(0) var sky_texture: texture_2d<f32>;
@group(1) @binding(1) var sky_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,  // Vertex color for horizon gradient
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) vertex_color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Transform position, push to far plane for sky rendering
    var clip_pos = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    clip_pos.z = clip_pos.w * 0.9999;
    out.clip_position = clip_pos;

    // Pass through UVs - flip V since texture is stored top-down
    out.uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);

    // Pass vertex color to fragment shader
    // In Fable, this would be computed by lerping between two colors (c92, c93)
    // based on the input vertex color. We pre-compute this in the mesh.
    out.vertex_color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample sky texture
    let tex_color = textureSample(sky_texture, sky_sampler, in.uv);

    // Fable's PSHADER_OUTER_SKY does:
    //   lrp r0, c0.w, t1, t0      // blend between two sky textures (time of day)
    //   lrp r0, v0.w, v0, r0      // blend texture with vertex color based on alpha
    //
    // For now we only have one texture, so skip the first lerp.
    // The second lerp blends the texture color with the vertex gradient color,
    // controlled by vertex alpha (high alpha = more gradient, low alpha = more texture)

    let gradient_color = in.vertex_color.rgb;
    let blend_factor = in.vertex_color.a;

    // Lerp: result = tex_color * (1 - blend) + gradient_color * blend
    let final_rgb = mix(tex_color.rgb, gradient_color, blend_factor);

    return vec4<f32>(final_rgb, 1.0);
}
