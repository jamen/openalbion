struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec4<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.position = position;
    return result;
}

[[stage(fragment)]]
fn fs_main(vertex: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vertex.position;
}