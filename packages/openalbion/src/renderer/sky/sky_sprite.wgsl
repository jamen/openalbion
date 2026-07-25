// Sky sprite shader — matches VSHADER_SKY_SPRITE (vs_1_1)
// and PSHADER_SKY_SPRITE (ps_1_1).
// Renders a textured billboard quad (sun or moon) with the vertex
// colour providing the tint.

struct Uniforms {
    view_proj: mat4x4<f32>,
    zenith_color: vec4<f32>,
    horizon_color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(sprite_texture, sprite_sampler, in.uv);
    return tex_color * in.color;
}
