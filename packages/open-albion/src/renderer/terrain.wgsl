struct Uniforms {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) height: f32,
    @location(2) @interpolate(flat) theme_0: u32,
    @location(3) @interpolate(flat) theme_1: u32,
    @location(4) @interpolate(flat) theme_2: u32,
    @location(5) @interpolate(flat) blend_0: f32,
    @location(6) @interpolate(flat) blend_1: f32,
    @location(7) @interpolate(flat) cliff_u: f32,
    @location(8) @interpolate(flat) cliff_v: f32,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) theme_indices_in: vec4<u32>,
    @location(3) blend_in: vec4<u32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = u.view_proj * vec4<f32>(position, 1.0);
    out.normal = normal;
    out.height = position.y;

    out.theme_0 = theme_indices_in.x;
    out.theme_1 = theme_indices_in.y;
    out.theme_2 = theme_indices_in.z;

    out.blend_0 = f32(blend_in.x) / 255.0;
    out.blend_1 = f32(blend_in.y) / 255.0;
    out.cliff_u = f32(blend_in.z) / 255.0;
    out.cliff_v = f32(blend_in.w) / 255.0;

    return out;
}

fn theme_debug_color(idx: u32) -> vec3<f32> {
    let hue = f32(idx) * 0.382 + 0.15;
    let r = sin(hue * 6.283) * 0.5 + 0.5;
    let g = sin((hue + 0.333) * 6.283) * 0.5 + 0.5;
    let b = sin((hue + 0.667) * 6.283) * 0.5 + 0.5;
    return vec3<f32>(r, g, b);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.4, 1.0, 0.3));
    let n = normalize(in.normal);
    let diffuse = max(dot(n, light_dir), 0.0);
    let shade = 0.25 + diffuse * 0.75;

    let t0 = theme_debug_color(in.theme_0);
    let t1 = theme_debug_color(in.theme_1);
    let t2 = theme_debug_color(in.theme_2);

    let b0 = clamp(in.blend_0, 0.0, 1.0);
    let b1 = clamp(in.blend_1, 0.0, 1.0);

    // Serial blend: each blend weight mixes the next theme over the current base.
    var base = t0;
    base = mix(base, t1, b0);
    base = mix(base, t2, b1);

    // Slope-driven cliff lookup: steep areas mix toward a cliff colour.
    let slope_f = clamp((1.0 - in.normal.y) * 6.0 - 0.5, 0.0, 1.0);
    let cliff_col = theme_debug_color(in.theme_0 + in.theme_1 + 5u);
    base = mix(base, cliff_col, slope_f);

    return vec4<f32>(base * shade, 1.0);
}
