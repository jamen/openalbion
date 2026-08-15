//! Where each character sits. A pen, a cell width, and nothing else.
//!
//! Monospace makes this arithmetic rather than a shaping problem (AGENTS.md §13): every
//! character advances one cell, so a column is `floor(x / cell_width)` and stays that way. When
//! proportional UI text arrives it will need real advances per glyph and this becomes the wrong
//! module — which is why it is small and separate rather than woven into the console.

use super::font::{Font, FontId, GlyphKey};

/// How many cells a tab advances to the next stop.
///
/// A convention, not a derivation — there is no oracle for this (§2 exempts §13 from rule 1).
/// Four because that is what the console's own output will be indented with.
pub const TAB_WIDTH: u32 = 4;

/// One character, positioned.
///
/// Carries the **pen** position — the left edge of the cell and the line's baseline — not the
/// bitmap's corner. Layout does not know a glyph's bitmap offsets without rasterizing it, and
/// making it rasterize would put the font behind every layout call for no gain. The caller
/// already holds the rasterized glyph, and adds its `left`/`top` to reach the bitmap.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PlacedGlyph {
    pub key: GlyphKey,
    pub pen_x: f32,
    pub baseline_y: f32,
}

/// Lay out one line, with `origin` at its **top-left** — not its baseline, which is a font
/// property the caller should not have to know.
///
/// Control characters do not draw and do not advance, except tab, which advances to the next
/// [`TAB_WIDTH`] stop measured from `origin`. Characters with no outline — space, and anything
/// the font does not map — are still returned: they advance the pen, and it is the caller's
/// rasterizer that discovers there is nothing to draw. Returning them keeps the column index of
/// every later character right, which a cursor depends on.
pub fn layout_line(
    font: &Font,
    px: u32,
    text: &str,
    origin: [f32; 2],
    out: &mut Vec<PlacedGlyph>,
) {
    let (cell_width, _) = font.cell(px);
    let baseline_y = origin[1] + font.ascent(px);

    let mut column: u32 = 0;
    for ch in text.chars() {
        match ch {
            '\t' => {
                column = (column / TAB_WIDTH + 1) * TAB_WIDTH;
                continue;
            }
            c if (c as u32) < 0x20 || c == '\u{7f}' => continue,
            _ => {}
        }

        out.push(PlacedGlyph {
            key: GlyphKey {
                font: FontId::Inconsolata,
                ch,
                px,
            },
            pen_x: origin[0] + column as f32 * cell_width,
            baseline_y,
        });
        column += 1;
    }
}

/// Lay out several lines top to bottom, one cell height apart.
///
/// Lines are taken as given rather than split on `'\n'`: the console holds its scrollback as
/// lines already, and a layout that also split would have two notions of where a line ends.
pub fn layout_lines(
    font: &Font,
    px: u32,
    lines: impl IntoIterator<Item = impl AsRef<str>>,
    origin: [f32; 2],
) -> Vec<PlacedGlyph> {
    let (_, line_height) = font.cell(px);
    let mut out = Vec::new();
    for (row, line) in lines.into_iter().enumerate() {
        layout_line(
            font,
            px,
            line.as_ref(),
            [origin[0], origin[1] + row as f32 * line_height],
            &mut out,
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font() -> Font {
        Font::new().expect("the embedded font parses")
    }

    #[test]
    fn characters_advance_one_cell_each() {
        let font = font();
        let (cell_width, _) = font.cell(16);
        let mut out = Vec::new();
        layout_line(&font, 16, "abc", [10.0, 20.0], &mut out);

        assert_eq!(out.len(), 3);
        for (i, placed) in out.iter().enumerate() {
            assert_eq!(placed.pen_x, 10.0 + i as f32 * cell_width);
            // The baseline is one ascent below the line's top, on every glyph of the line.
            assert_eq!(placed.baseline_y, 20.0 + font.ascent(16));
        }
        assert_eq!(out[2].key.ch, 'c');
    }

    /// A space produces a `PlacedGlyph` even though it will rasterize to nothing. Dropping it
    /// here would still lay the line out correctly, but a cursor at column 3 of `"a b"` would
    /// have nothing to anchor to.
    #[test]
    fn spaces_are_placed_and_advance() {
        let font = font();
        let (cell_width, _) = font.cell(16);
        let mut out = Vec::new();
        layout_line(&font, 16, "a b", [0.0, 0.0], &mut out);

        assert_eq!(out.len(), 3);
        assert_eq!(out[1].key.ch, ' ');
        assert_eq!(out[2].pen_x, 2.0 * cell_width);
    }

    #[test]
    fn tabs_advance_to_the_next_stop_and_control_characters_do_not() {
        let font = font();
        let (cell_width, _) = font.cell(16);
        let mut out = Vec::new();
        layout_line(&font, 16, "ab\tc", [0.0, 0.0], &mut out);

        assert_eq!(out.len(), 3, "the tab itself is not drawn");
        assert_eq!(out[2].key.ch, 'c');
        assert_eq!(out[2].pen_x, TAB_WIDTH as f32 * cell_width);

        // A tab already on a stop still moves to the *next* one, like a terminal.
        let mut out = Vec::new();
        layout_line(&font, 16, "abcd\te", [0.0, 0.0], &mut out);
        assert_eq!(out[4].pen_x, 2.0 * TAB_WIDTH as f32 * cell_width);

        // Everything else below 0x20 is skipped without advancing.
        let mut out = Vec::new();
        layout_line(&font, 16, "a\r\n\u{7f}b", [0.0, 0.0], &mut out);
        assert_eq!(out.len(), 2);
        assert_eq!(out[1].pen_x, cell_width);
    }

    #[test]
    fn lines_stack_by_one_cell_height() {
        let font = font();
        let (_, line_height) = font.cell(16);
        let placed = layout_lines(&font, 16, ["a", "b", "c"], [0.0, 0.0]);

        assert_eq!(placed.len(), 3);
        for (row, glyph) in placed.iter().enumerate() {
            assert_eq!(
                glyph.baseline_y,
                row as f32 * line_height + font.ascent(16),
            );
            assert_eq!(glyph.pen_x, 0.0);
        }
    }

    /// An empty line still occupies a row, so blank scrollback entries do not close up.
    #[test]
    fn an_empty_line_still_takes_a_row() {
        let font = font();
        let (_, line_height) = font.cell(16);
        let placed = layout_lines(&font, 16, ["a", "", "c"], [0.0, 0.0]);

        assert_eq!(placed.len(), 2);
        assert_eq!(placed[1].key.ch, 'c');
        assert_eq!(placed[1].baseline_y, 2.0 * line_height + font.ascent(16));
    }

    /// Composite laid-out glyphs into a CPU bitmap exactly as the shader will place them:
    /// bitmap corner = pen + the glyph's own `left`/`top` offsets.
    fn composite(font: &Font, px: u32, text: &str, size: [usize; 2]) -> Vec<u8> {
        let mut canvas = vec![0u8; size[0] * size[1]];
        let mut placed = Vec::new();
        layout_line(font, px, text, [0.0, 0.0], &mut placed);

        for glyph in &placed {
            let Some(raster) = font.rasterize(glyph.key) else {
                continue;
            };
            let x0 = (glyph.pen_x + raster.left).round() as i64;
            let y0 = (glyph.baseline_y + raster.top).round() as i64;
            for y in 0..raster.image.height as i64 {
                for x in 0..raster.image.width as i64 {
                    let (cx, cy) = (x0 + x, y0 + y);
                    if cx < 0 || cy < 0 || cx >= size[0] as i64 || cy >= size[1] as i64 {
                        continue;
                    }
                    let src = raster.image.levels[0][(y * raster.image.width as i64 + x) as usize];
                    let dst = &mut canvas[cy as usize * size[0] + cx as usize];
                    *dst = (*dst).max(src);
                }
            }
        }
        canvas
    }

    /// The end-to-end CPU check: layout and rasterization compose into the picture they should.
    ///
    /// This is the test that would catch a flipped Y, a baseline off by an ascent, or `left`
    /// applied to the wrong axis — none of which the metric tests above can see, and all of
    /// which would otherwise be found by looking at the screen (§2 rule 4).
    #[test]
    fn glyphs_composite_onto_their_baseline() {
        let font = font();
        let px = 32;
        let (cell_width, _) = font.cell(px);
        let ascent = font.ascent(px);
        let (width, height) = (96usize, 48usize);
        let canvas = composite(&font, px, "Ag", [width, height]);

        let ink_in = |x: std::ops::Range<usize>, y: std::ops::Range<usize>| {
            y.flat_map(|yy| x.clone().map(move |xx| (xx, yy)))
                .any(|(xx, yy)| canvas[yy * width + xx] > 128)
        };

        let baseline = ascent.round() as usize;
        let col0 = 0..cell_width.round() as usize;
        let col1 = cell_width.round() as usize..(2.0 * cell_width).round() as usize;

        // Both letters drew, each inside its own cell.
        assert!(ink_in(col0.clone(), 0..baseline), "'A' has no ink above the baseline");
        assert!(ink_in(col1.clone(), 0..baseline), "'g' has no ink above the baseline");

        // 'A' sits on the baseline; 'g' hangs below it. That asymmetry is only right if the
        // baseline is where layout says it is.
        assert!(
            !ink_in(col0, baseline + 1..height),
            "'A' has a descender, so the baseline is wrong"
        );
        assert!(
            ink_in(col1, baseline + 1..height),
            "'g' has no descender, so the baseline is wrong"
        );
    }
}
