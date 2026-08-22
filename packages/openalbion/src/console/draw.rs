//! Turning a [`Layout`] into glyph instances — the only part of the console that has heard of
//! the renderer.
//!
//! [`super::Console::layout`] decided *where* everything goes with no GPU in sight; this decides
//! *which slot* each character is in, which is the one question only the renderer can answer.

use super::Console;
use renderer::{GlyphInstance, Renderer};

/// Lay the console out for the renderer's current size, register whatever glyphs are new, and
/// hand the whole overlay over as one batch.
///
/// **The order is §13.3's and it matters:** rasterize and register *every* new glyph first, then
/// draw. Each registration dirties the whole bindless array and `encode` rebuilds it once per
/// frame — so registering a glyph at a time would rebuild it once per character.
///
/// Slots are asked for every frame rather than cached. The registry is scene-scoped and a level
/// load invalidates every index handed out (§13.2a); a miss costs a re-register, not a
/// re-rasterize, because the bitmap is right here. The one exception is the panel, which points
/// at `solid_index` — the fallback slot, which no scene load touches.
pub fn draw(renderer: &mut Renderer<'_>, console: &Console) {
    let layout = console.layout(renderer.size());

    // The common case, and the one that has to stay exactly this cheap: a closed console with
    // the stats overlay off lays out nothing, and an empty batch restores the frame byte for
    // byte (§13.6).
    if layout.is_empty() {
        renderer.set_text(&[]);
        return;
    }

    let mut instances = Vec::with_capacity(layout.fills.len() + layout.glyphs.len());

    // Fills first: within a draw the instances blend in order, so the panel has to be laid down
    // before the text that sits on it.
    let solid = renderer.solid_index();
    for fill in &layout.fills {
        instances.push(GlyphInstance {
            rect: fill.rect,
            colour: fill.colour,
            texture_index: solid,
            _pad: [0; 3],
        });
    }

    for glyph in &layout.glyphs {
        // A space has no outline. Not an error — the pen advanced, there is nothing to draw.
        let Some(raster) = console.font.rasterize(glyph.placed.key) else {
            continue;
        };

        let key = glyph.placed.key.id();
        let index = match renderer.glyph_index(key) {
            Some(index) => index,
            None => match renderer.add_glyph(key, &raster.image) {
                Ok(index) => index,
                Err(error) => {
                    tracing::warn!("console: {:?} did not register: {error}", glyph.placed.key.ch);
                    continue;
                }
            },
        };

        instances.push(GlyphInstance {
            rect: [
                glyph.placed.pen_x + raster.left,
                glyph.placed.baseline_y + raster.top,
                raster.image.width as f32,
                raster.image.height as f32,
            ],
            colour: glyph.colour,
            texture_index: index,
            _pad: [0; 3],
        });
    }

    // An empty batch means "draw nothing", which is the state a closed console leaves the frame
    // in — and what keeps `--screenshot` byte-identical (§13.6).
    renderer.set_text(&instances);
}
