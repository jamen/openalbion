//! The embedded font, and rasterizing one glyph from it.

use ab_glyph::{Font as _, FontRef, PxScaleFont, ScaleFont as _};
use derive_more::{Display, Error};
use renderer::{ImageFormat, TextureImage};

/// Inconsolata Regular, embedded.
///
/// **Why an embedded font and not the game's own** (AGENTS.md §3.14, §13.1): `fonts.big` ships
/// pre-rasterized GDI atlases of Arial and Tahoma and **nothing monospace**, they are licensed
/// Microsoft faces, they are scoped to the install's language, and they would need a parser.
/// The console is the tool used to debug parsing, so it must not depend on parsing.
///
/// SIL Open Font License 1.1, with **no Reserved Font Name** declared — see
/// `assets/fonts/Inconsolata-OFL.txt`, which ships beside it as the licence requires.
const INCONSOLATA: &[u8] = include_bytes!("../../assets/fonts/Inconsolata-Regular.ttf");

/// Which font a glyph came from. One variant today; the enum exists so [`GlyphKey`] does not
/// have to change shape when a second one arrives.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontId {
    Inconsolata,
}

/// Identifies one rasterized glyph: a character at a size, in a font.
///
/// The size is in whole pixels rather than points because that is what the rasterizer is asked
/// for and what makes two requests the same bitmap. A DPI change therefore produces a *different
/// key*, which is what lets the cache survive one without special handling (§13.6).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GlyphKey {
    pub font: FontId,
    pub ch: char,
    pub px: u32,
}

impl GlyphKey {
    /// The opaque `u64` the renderer keys its bindless slot by.
    ///
    /// The renderer must not know what a font or a character is (§11.1), so it is handed an
    /// identifier rather than a key. Packed rather than hashed so it is injective — two glyphs
    /// cannot collide into one slot, which a hash could do and which would render as one
    /// character wearing another's shape.
    pub fn id(self) -> u64 {
        (self.font as u64) << 56 | (self.px as u64 & 0xff_ffff) << 32 | self.ch as u64
    }
}

/// One glyph's coverage bitmap and where it sits relative to the pen.
pub struct RasterizedGlyph {
    /// R8 coverage, one level, no mips. `R8` is already an `ImageFormat` the renderer accepts
    /// and already satisfies §12.2 fact 6's sample-type invariant, so no new upload path is
    /// needed.
    pub image: TextureImage,
    /// Offset from the pen position to the bitmap's top-left corner, in pixels. `left` is
    /// usually a small positive side bearing; `top` is negative for anything reaching above the
    /// baseline, which is nearly everything.
    pub left: f32,
    pub top: f32,
}

#[derive(Debug, Display, Error)]
pub enum FontError {
    #[display("the embedded font could not be parsed")]
    Invalid,
}

/// The embedded font, plus the metrics that make it a *monospace grid*.
pub struct Font {
    inner: FontRef<'static>,
}

impl Font {
    pub fn new() -> Result<Self, FontError> {
        FontRef::try_from_slice(INCONSOLATA)
            .map(|inner| Self { inner })
            .map_err(|_| FontError::Invalid)
    }

    /// `px` is an **em size** — the conventional meaning of "a 16px font", and the size
    /// AGENTS.md §13.1's metrics are quoted at.
    ///
    /// **`ab_glyph`'s `PxScale` is not that**, and the difference is silent: its scale is the
    /// *ascent-to-descent pixel height*, so its scale factor is `scale / height_unscaled`
    /// (`ab_glyph-0.2.32/src/scale.rs:88`), where the em interpretation divides by
    /// `units_per_em`. Handing it 16.0 directly gives Inconsolata an advance of 7.626 px rather
    /// than the 8.0 that `500/1000 em` predicts — a 5% error that reads as "the font is a bit
    /// narrow", not as a bug. So the conversion happens here, once, and every metric below is
    /// in em-relative pixels.
    fn scaled(&self, px: u32) -> PxScaleFont<&FontRef<'static>> {
        let units_per_em = self.inner.units_per_em().unwrap_or(1000.0);
        let scale = px as f32 * self.inner.height_unscaled() / units_per_em;
        self.inner.as_scaled(scale)
    }

    /// The cell every character occupies at `px`: `(advance, line height)`, unrounded.
    ///
    /// **This is only a cell because the font is monospace**, which is checked rather than
    /// assumed — see `advance_is_uniform_across_ascii` in the tests. Inconsolata's advance is
    /// exactly `500/1000 em` for every glyph in `0x20..=0x7E`, so the cell is `0.5 px` wide and
    /// `1.049 px` tall.
    pub fn cell(&self, px: u32) -> (f32, f32) {
        let scaled = self.scaled(px);
        (
            scaled.h_advance(self.inner.glyph_id('M')),
            scaled.height() + scaled.line_gap(),
        )
    }

    /// Distance from the top of a line's cell down to its baseline, in pixels.
    pub fn ascent(&self, px: u32) -> f32 {
        self.scaled(px).ascent()
    }

    /// Rasterize one glyph, or `None` when it has no outline.
    ///
    /// A space has no outline, and neither does an unmapped character — both are `None`, and
    /// both are correct: the pen still advances a full cell, there is simply nothing to draw.
    /// That is also why the caller must not treat `None` as an error and substitute a box.
    pub fn rasterize(&self, key: GlyphKey) -> Option<RasterizedGlyph> {
        // Through `scaled` so the outline is rasterized at the same scale the metrics are
        // measured at — see its doc comment for why passing `px` straight in is wrong.
        let scaled = self.scaled(key.px);
        let glyph = self.inner.glyph_id(key.ch).with_scale(scaled.scale());
        let outlined = self.inner.outline_glyph(glyph)?;

        let bounds = outlined.px_bounds();
        let width = bounds.width().ceil() as u32;
        let height = bounds.height().ceil() as u32;
        if width == 0 || height == 0 {
            return None;
        }

        // `draw` hands back coverage in `0.0..=1.0` over the bitmap's own coordinates, so this
        // writes straight into the destination with no intermediate buffer.
        let mut coverage = vec![0u8; (width * height) as usize];
        outlined.draw(|x, y, c| {
            if x < width && y < height {
                coverage[(y * width + x) as usize] = (c.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        });

        Some(RasterizedGlyph {
            image: TextureImage::single(width, height, ImageFormat::R8, coverage),
            left: bounds.min.x,
            top: bounds.min.y,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The claim the whole layout rests on (AGENTS.md §13.1): Inconsolata is monospace, and
    /// every printable ASCII glyph advances the same. Pinned as a test because "it is a
    /// monospace font" is exactly the kind of assumption §2 exists to stop being an assumption.
    #[test]
    fn advance_is_uniform_across_ascii() {
        let font = Font::new().expect("the embedded font parses");
        let scaled = font.inner.as_scaled(16.0);
        let advances: Vec<f32> = (0x20u8..=0x7e)
            .map(|c| scaled.h_advance(font.inner.glyph_id(c as char)))
            .collect();
        let first = advances[0];
        for (i, advance) in advances.iter().enumerate() {
            assert!(
                (advance - first).abs() < 1e-4,
                "{:?} advances {advance}, not {first}",
                (0x20 + i) as u8 as char,
            );
        }
    }

    /// Read out of the file 2026-08-15 and recorded in §13.1: 1000 upem, `advance = 500`,
    /// ascent 859, descent -190, line gap 0. These pin the *font asset*, so a silent swap for a
    /// different Inconsolata cut fails here rather than by looking slightly wrong on screen.
    #[test]
    fn embedded_font_has_the_metrics_13_1_recorded() {
        let font = Font::new().expect("the embedded font parses");
        assert_eq!(font.inner.units_per_em(), Some(1000.0));

        // At 16px: advance 500/1000 * 16 = 8, line height (859 + 190 + 0)/1000 * 16 = 16.784.
        let (advance, line_height) = font.cell(16);
        assert!((advance - 8.0).abs() < 1e-3, "advance {advance}");
        assert!((line_height - 16.784).abs() < 1e-3, "line height {line_height}");
        assert!((font.ascent(16) - 13.744).abs() < 1e-3, "ascent {}", font.ascent(16));
    }

    #[test]
    fn a_space_has_no_outline_and_a_letter_does() {
        let font = Font::new().expect("the embedded font parses");
        let key = |ch| GlyphKey {
            font: FontId::Inconsolata,
            ch,
            px: 16,
        };

        assert!(font.rasterize(key(' ')).is_none());

        let m = font.rasterize(key('M')).expect("'M' has an outline");
        assert_eq!(m.image.format, ImageFormat::R8);
        assert!(m.image.is_complete());
        // It fits the 8 x 17 cell §13.1 predicted, and reaches above the baseline.
        assert!(m.image.width <= 8, "width {}", m.image.width);
        assert!(m.image.height <= 17, "height {}", m.image.height);
        assert!(m.top < 0.0, "top {}", m.top);
        assert!(
            m.image.levels[0].iter().any(|&c| c > 128),
            "'M' rasterized to nothing"
        );
    }

    /// Two glyphs must never share a slot. Packing rather than hashing is what guarantees it
    /// (see `GlyphKey::id`), and this is the check that keeps the packing injective.
    #[test]
    fn glyph_ids_are_distinct_across_chars_and_sizes() {
        let mut seen = std::collections::HashSet::new();
        for px in [8u32, 16, 17, 64, 0xff_ffff] {
            for ch in ('\u{0}'..='\u{ff}').chain(['€', '𝄞']) {
                let key = GlyphKey {
                    font: FontId::Inconsolata,
                    ch,
                    px,
                };
                assert!(seen.insert(key.id()), "{key:?} collided");
            }
        }
    }
}
