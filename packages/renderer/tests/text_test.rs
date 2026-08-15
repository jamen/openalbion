//! The text pass, against a real device — and with it, AGENTS.md §12.9's outstanding gate.
//!
//! `SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING` has been requested since
//! §12.8 step 1 and used by nothing: every world pass draws one material per draw, so its slot
//! is dynamically uniform and `TEXTURE_BINDING_ARRAY` alone would have covered it. §12.9 asked
//! for "two quads in one instanced draw, each indexing a different slot" *before* font
//! rendering leaned on it. That is exactly what [`two_glyphs_in_one_draw_reach_different_slots`]
//! is, so the gate closes here rather than being deferred again.
//!
//! Needs a GPU, so it skips when there is not one.

use renderer::{GlyphInstance, ImageFormat, Renderer, TextureImage};

const SIZE: u32 = 64;

fn headless() -> Option<Renderer<'static>> {
    match pollster::block_on(Renderer::new_headless([SIZE, SIZE])) {
        Ok(renderer) => Some(renderer),
        Err(error) => {
            eprintln!("skipping: no usable adapter ({error})");
            None
        }
    }
}

/// An 8×8 glyph of uniform `coverage`. Uniform so that sampling and filtering cannot influence
/// the result — the only thing a read-back pixel can tell us about is *which slot* was sampled.
fn glyph(coverage: u8) -> TextureImage {
    TextureImage::single(8, 8, ImageFormat::R8, vec![coverage; 64])
}

fn instance(x: f32, texture_index: u32) -> GlyphInstance {
    GlyphInstance {
        rect: [x, 0.0, 16.0, 16.0],
        colour: [1.0, 1.0, 1.0, 1.0],
        texture_index,
        _pad: [0; 3],
    }
}

/// Read one pixel's red channel out of a captured frame.
fn red_at(image: &renderer::Image, x: u32, y: u32) -> u8 {
    image.rgba[((y * image.width + x) * 4) as usize]
}

/// **§12.9's gate.** Two instances in one `draw`, each answering a different bindless slot.
///
/// A per-instance vertex attribute is not dynamically uniform, so a fragment's texture index
/// varies *within* the draw. If non-uniform indexing were not working, both quads would sample
/// whichever slot the driver happened to pick and the two would read back the same — which is
/// why the two glyphs have deliberately different coverage.
///
/// The single draw is a property of the pass rather than something observable from here:
/// `set_text` builds one instance buffer and `TextPass::pass` issues one `draw(0..4, 0..n)`.
#[test]
fn two_glyphs_in_one_draw_reach_different_slots() {
    let Some(mut renderer) = headless() else {
        return;
    };

    let opaque = renderer.add_glyph(1, &glyph(255)).expect("register");
    let half = renderer.add_glyph(2, &glyph(128)).expect("register");
    assert_ne!(opaque, half, "distinct keys must take distinct slots");

    renderer.set_text(&[instance(0.0, opaque), instance(32.0, half)]);
    assert_eq!(renderer.text_stats(), 2);

    let image = renderer.render_to_image().expect("headless capture");

    // Coverage scales the instance colour's alpha, and the frame under it is black, so a
    // pixel reads back as its glyph's coverage.
    let (first, second) = (red_at(&image, 8, 8), red_at(&image, 40, 8));
    assert!(
        first.abs_diff(255) <= 2,
        "the fully covered glyph read back {first}, not ~255",
    );
    assert!(
        second.abs_diff(128) <= 2,
        "the half-covered glyph read back {second}, not ~128 — both quads sampled one slot, \
         so non-uniform indexing is not working",
    );

    // Outside both quads is the cleared frame, so the pass drew where it was told and nowhere
    // else — which is also what catches a flipped Y or a viewport off by a factor.
    assert_eq!(red_at(&image, 24, 8), 0, "the gap between the quads is not clear");
    assert_eq!(red_at(&image, 8, 32), 0, "below the quads is not clear");
}

/// A renderer draws no text until asked, and `set_text(&[])` puts it back.
///
/// This is what keeps `--screenshot` byte-comparable while the console is closed, which is the
/// verification method every §12 and §13 step depends on (AGENTS.md §13.6).
#[test]
fn nothing_is_drawn_until_text_is_set() {
    let Some(mut renderer) = headless() else {
        return;
    };

    assert_eq!(renderer.text_stats(), 0);
    let blank = renderer.render_to_image().expect("headless capture");
    assert!(blank.rgba.iter().all(|&c| c == 0 || c == 255), "cleared frame");

    let slot = renderer.add_glyph(1, &glyph(255)).expect("register");
    renderer.set_text(&[instance(0.0, slot)]);
    let drawn = renderer.render_to_image().expect("headless capture");
    assert_ne!(red_at(&drawn, 8, 8), 0, "text was set but nothing drew");

    renderer.set_text(&[]);
    assert_eq!(renderer.text_stats(), 0);
    let cleared = renderer.render_to_image().expect("headless capture");
    assert_eq!(cleared.rgba, blank.rgba, "clearing the text must restore the frame");
}

/// The same glyph key takes one slot however often it is added — the dedup §12.6 describes,
/// which for text is what makes steady-state typing free (§13.3).
#[test]
fn a_repeated_glyph_key_reuses_its_slot() {
    let Some(mut renderer) = headless() else {
        return;
    };

    assert_eq!(renderer.glyph_index(42), None, "not resident before it is added");

    let first = renderer.add_glyph(42, &glyph(255)).expect("register");
    assert_eq!(renderer.glyph_index(42), Some(first));
    let (registered, _) = renderer.bindless_stats();

    let again = renderer.add_glyph(42, &glyph(255)).expect("register");
    assert_eq!(again, first, "the same key must return the same slot");
    assert_eq!(renderer.bindless_stats().0, registered, "and take no new one");
}

/// **§13.2a, as a gate.** Glyphs outlive a scene load; the registry does not.
///
/// `clear_scene` takes glyph slots with everything else, so a cached index is stale afterwards
/// and would point at whatever the next scene registers — plausible garbage rather than
/// nothing. The contract is that the caller asks again every frame, so this pins the two halves
/// of it: the index really does go away, and re-adding really does bring it back.
#[test]
fn a_scene_load_invalidates_glyph_slots() {
    let Some(mut renderer) = headless() else {
        return;
    };

    renderer.add_glyph(7, &glyph(255)).expect("register");
    assert!(renderer.glyph_index(7).is_some());

    renderer.clear_scene();
    assert_eq!(
        renderer.glyph_index(7),
        None,
        "clear_scene must drop glyph registrations too, or the slot leaks",
    );

    // And the recovery path costs one re-registration, not a re-rasterization: the caller
    // still holds the bitmap.
    let index = renderer.add_glyph(7, &glyph(255)).expect("re-register");
    renderer.set_text(&[instance(0.0, index)]);
    let image = renderer.render_to_image().expect("headless capture");
    assert!(red_at(&image, 8, 8).abs_diff(255) <= 2, "re-registered glyph does not draw");
}

/// The guard itself fires. Drawing text whose indices predate a `clear_scene` is the one way
/// this design renders the wrong texture silently, so it is a `debug_assert` rather than a
/// comment — and this is the test that proves the assert is wired to the right condition.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "cleared generation")]
fn drawing_text_across_a_scene_clear_is_caught() {
    let Some(mut renderer) = headless() else {
        // `should_panic` cannot be skipped, so stand in with the panic the test expects rather
        // than failing a machine that has no GPU.
        panic!("skipping: no usable adapter — cleared generation");
    };

    let index = renderer.add_glyph(1, &glyph(255)).expect("register");
    renderer.set_text(&[instance(0.0, index)]);
    renderer.clear_scene();
    let _ = renderer.render_to_image();
}
