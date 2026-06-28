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

struct Material {
    // Non-zero enables alpha testing (cutout): fragments below `alpha_cutoff` are discarded.
    alpha_test: u32,
    alpha_cutoff: f32,
    _pad0: f32,
    _pad1: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var model_texture: texture_2d<f32>;
@group(1) @binding(1) var model_sampler: sampler;
@group(1) @binding(2) var<uniform> material: Material;

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

    // Alpha-test (cutout) materials: drop fully-transparent texels instead of blending them.
    if material.alpha_test != 0u && tex_color.a < material.alpha_cutoff {
        discard;
    }

    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let n = normalize(in.normal);
    let diffuse = max(dot(n, light_dir), 0.0);
    let lighting = 0.3 + diffuse * 0.7;
    return vec4<f32>(tex_color.rgb * lighting, tex_color.a);
}
