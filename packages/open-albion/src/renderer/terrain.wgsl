struct Uniforms {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) height: f32,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = u.view_proj * vec4<f32>(position, 1.0);
    out.normal = normal;
    out.height = position.y;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.4, 1.0, 0.3));
    let n = normalize(in.normal);
    let diffuse = max(dot(n, light_dir), 0.0);
    let shade = 0.3 + diffuse * 0.7;

    // Placeholder terrain tint, lightened with height so relief is legible.
    let low = vec3<f32>(0.25, 0.35, 0.20);
    let high = vec3<f32>(0.55, 0.52, 0.42);
    let t = clamp(in.height / 80.0, 0.0, 1.0);
    let base = mix(low, high, t);

    return vec4<f32>(base * shade, 1.0);
}
