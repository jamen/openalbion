[[block]]
struct Locals {
    transform: mat4x4<f32>;
};
[[group(0), binding(0)]]
var locals: Locals;

struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] normal: vec3<f32>,
    [[location(2)]] tex_coord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    // out.tex_coord = vec2<f32>((tex_coord.x + 512.0) / 1024.0, (tex_coord.y + 512.0) / 1024.0);
    // out.tex_coord = vec2<f32>((tex_coord.x + 32.0) / 64.0, (tex_coord.y + 32.0) / 64.0);
    // out.tex_coord = vec2<f32>(tex_coord.x / 32.0, tex_coord.y / 32.0);
    out.tex_coord = tex_coord;
    out.position = locals.transform * vec4<f32>(position, 1.0);
    return out;
}

[[group(1), binding(0)]] var base_color: texture_2d<f32>;
[[group(1), binding(1)]] var base_color_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let color = textureSample(base_color, base_color_sampler, in.tex_coord).xyz;
    return vec4<f32>(color, 1.0);
}

[[stage(vertex)]]
fn vs_wire(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] normal: vec3<f32>,
    [[location(2)]] tex_coord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = tex_coord;
    out.position = locals.transform * vec4<f32>(position, 1.0);
    return out;
}

[[stage(fragment)]]
fn fs_wire(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}