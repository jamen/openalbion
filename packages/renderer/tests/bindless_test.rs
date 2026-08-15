//! The bindless texture array, exercised against a real device.
//!
//! This is AGENTS.md §12.8 step 1's verification (b) and (c): that the device actually
//! *grants* the features and limits the renderer asks for, and that a binding array of the
//! declared capacity validates. Neither is provable from the fact that `request_device`
//! returned — a granted feature with a default limit creates a device on which the array's
//! bind group layout cannot be built, and that would otherwise surface much later, in the
//! middle of migrating a pass.
//!
//! Needs a GPU, so it skips when there is not one, like the tests that need a Fable install.

use renderer::{MAX_BINDLESS_TEXTURES, MIN_BINDLESS_TEXTURES, Renderer};

fn headless() -> Option<Renderer<'static>> {
    match pollster::block_on(Renderer::new_headless([64, 64])) {
        Ok(renderer) => Some(renderer),
        Err(error) => {
            eprintln!("skipping: no usable adapter ({error})");
            None
        }
    }
}

/// The array is built, at a capacity a level fits in, with nothing registered yet.
///
/// The capacity assertion is the load-bearing one: reaching it means `request_device` was
/// given a raised `max_binding_array_elements_per_shader_stage` *and* the bind group layout
/// validated against it. With wgpu's default limits — which are 0 for binding arrays —
/// construction would have panicked before returning.
#[test]
fn the_array_is_built_and_empty() {
    let Some(renderer) = headless() else { return };

    let (registered, capacity) = renderer.bindless_stats();
    assert_eq!(registered, 0, "no pass registers a texture yet (§12.8 step 2)");
    assert!(
        (MIN_BINDLESS_TEXTURES..=MAX_BINDLESS_TEXTURES).contains(&capacity),
        "capacity {capacity} outside {MIN_BINDLESS_TEXTURES}..={MAX_BINDLESS_TEXTURES}",
    );
}

/// A frame still renders with the array bound but unused.
///
/// Every slot holds the fallback view, so this also exercises the "fully bound" property the
/// design depends on: the renderer never supplies fewer entries than the layout declares,
/// which is why `PARTIALLY_BOUND_BINDING_ARRAY` — unsupported on Metal — is never needed.
#[test]
fn a_frame_still_renders() {
    let Some(mut renderer) = headless() else {
        return;
    };

    let image = renderer.render_to_image().expect("headless capture");
    assert_eq!(image.width, 64);
    assert_eq!(image.height, 64);
    assert_eq!(image.rgba.len(), 64 * 64 * 4);
}
