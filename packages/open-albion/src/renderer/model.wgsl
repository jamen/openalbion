struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
}

struct Uniforms {
    view_proj: mat4x4<f32>,
    model_scale: f32,
    model_pos: vec3<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var model_texture: texture_2d<f32>;
@group(1) @binding(1) var model_sampler: sampler;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal_in: vec3<f32>,
    @location(2) uv_in: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = uniforms.view_proj * vec4<f32>(position * uniforms.model_scale + uniforms.model_pos, 1.0);
    out.uv = uv_in;
    out.normal = normal_in;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(model_texture, model_sampler, in.uv);
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let n = normalize(in.normal);
    let diffuse = max(dot(n, light_dir), 0.0);
    let ambient = 0.3;
    let lighting = ambient + diffuse * 0.7;
    return vec4<f32>(tex_color.rgb * lighting, tex_color.a);
}
