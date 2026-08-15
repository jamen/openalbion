// text.wgsl — screen-space glyph quads.
//
// **This is not a transcription.** Every other `.wgsl` in this project carries the original's
// disassembly in its header and cites it per line (AGENTS.md §6.7); this one has no original,
// because Fable's engine drew its text through `CStaticFontBank::Render` and a fixed-function
// D3D9 path we are deliberately not reproducing (§13.5, §3.14). The absence of an asm block
// below is therefore a statement, not an omission.
//
// The bindless declarations — `bindless_textures`, `repeat_sampler`, `clamp_sampler` — are
// prepended by `BindlessTextures::wgsl_prelude` (§12.4), so group 1 here is the same group
// every other pass binds.

struct FrameUniforms {
    // (width, height) of the target in physical pixels; `.zw` is padding.
    viewport: vec4<f32>,
}

@group(0) @binding(0) var<uniform> frame: FrameUniforms;

struct GlyphInstance {
    // (x, y, width, height) of the glyph's bitmap in pixels, origin top-left, y down.
    @location(0) rect: vec4<f32>,
    @location(1) colour: vec4<f32>,
    // Slot in the bindless array. This varies *per instance* within one draw, which makes it
    // the first non-uniform index in the renderer (§13.4). No annotation is needed: naga
    // detects it and emits the `NonUniform` decoration itself.
    @location(2) texture_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) colour: vec4<f32>,
    // `flat` because a slot index must not be interpolated between a quad's corners.
    @location(2) @interpolate(flat) texture_index: u32,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    instance: GlyphInstance,
) -> VertexOutput {
    // Four vertices as a triangle strip, with no vertex or index buffer: bit 0 of the index is
    // the corner's x and bit 1 is its y, which walks (0,0) (1,0) (0,1) (1,1) — exactly a
    // strip's order. A glyph is a quad and never anything else, so geometry it does not have
    // is geometry that cannot be wrong.
    let corner = vec2<f32>(
        f32(vertex_index & 1u),
        f32((vertex_index >> 1u) & 1u),
    );

    let pixel = instance.rect.xy + corner * instance.rect.zw;
    // Pixels to clip space. Y flips: pixels run down from the top left, clip space runs up.
    let ndc = vec2<f32>(
        pixel.x / frame.viewport.x * 2.0 - 1.0,
        1.0 - pixel.y / frame.viewport.y * 2.0,
    );

    var out: VertexOutput;
    // Z is 0 and there is no depth attachment: this pass draws after the resolve, over a
    // finished frame, in the order the instances were given.
    out.clip_position = vec4<f32>(ndc, 0.0, 1.0);
    out.uv = corner;
    out.colour = instance.colour;
    out.texture_index = instance.texture_index;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // The glyph texture is R8 coverage, one level, its own texture per glyph — so the whole
    // 0..1 UV range is this glyph and `clamp_sampler` (linear, clamp, no mip) is the right one.
    let coverage = textureSample(
        bindless_textures[in.texture_index],
        clamp_sampler,
        in.uv,
    ).r;

    // Coverage modulates alpha, not colour: text is one colour with a soft edge, and
    // multiplying RGB instead would darken the edge toward black rather than fade it.
    return vec4<f32>(in.colour.rgb, in.colour.a * coverage);
}
