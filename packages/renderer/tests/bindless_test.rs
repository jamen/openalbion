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

use renderer::{
    AlphaMode, ImageFormat, MAX_BINDLESS_TEXTURES, MIN_BINDLESS_TEXTURES, Model, ModelMaterial,
    ModelPrimitive, ModelSubMesh, ModelVertex, Renderer, TextureImage,
};

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

/// The dedup, and the thing that would otherwise leak.
///
/// Two meshes sharing an asset id must take **one** slot, not two — that is the whole point of
/// keying the registry by the global asset id (AGENTS.md §3.11, §12.6). And `clear_scene`
/// must return the slots, or every scene load would leak the previous level's textures and
/// burn its capacity (§12.4).
#[test]
fn a_shared_asset_id_takes_one_slot_and_clearing_returns_it() {
    let Some(mut renderer) = headless() else {
        return;
    };

    // Two meshes referencing the same texture, and a third referencing a different one.
    renderer.add_model(&model_with_texture(7), &[Default::default()]).unwrap();
    assert!(renderer.has_texture(7), "id 7 is resident after the first mesh");
    assert_eq!(renderer.bindless_stats().0, 1);

    renderer.add_model(&model_with_texture(7), &[Default::default()]).unwrap();
    assert_eq!(
        renderer.bindless_stats().0,
        1,
        "the same asset id must reuse its slot, not take another",
    );

    renderer.add_model(&model_with_texture(9), &[Default::default()]).unwrap();
    assert_eq!(renderer.bindless_stats().0, 2, "a different id takes a new slot");

    renderer.clear_scene();
    assert_eq!(
        renderer.bindless_stats().0,
        0,
        "clear_scene must return the slots, or a scene load leaks the whole level",
    );
    assert!(!renderer.has_texture(7), "and the registrations with them");
}

/// A material with no diffuse map at all draws, rather than being dropped or claiming a slot.
/// Roughly a quarter of `graphics.big`'s materials have `base_texture_id == 0` (§3.11).
#[test]
fn a_material_without_a_texture_takes_no_slot() {
    let Some(mut renderer) = headless() else {
        return;
    };

    let mut model = model_with_texture(1);
    model.materials[0].diffuse_id = None;
    model.materials[0].diffuse = None;

    renderer.add_model(&model, &[Default::default()]).unwrap();
    assert_eq!(renderer.bindless_stats().0, 0);
    renderer.render_to_image().expect("still renders");
}

/// One triangle, one material, whose diffuse map is a 1×1 texture registered under `asset_id`.
fn model_with_texture(asset_id: u32) -> Model {
    Model {
        primitives: vec![ModelPrimitive {
            vertices: vec![
                ModelVertex { position: [0.0; 3], normal: [0.0, 0.0, 1.0], uv: [0.0; 2] },
                ModelVertex { position: [1.0, 0.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 0.0] },
                ModelVertex { position: [0.0, 1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 1.0] },
            ],
            indices: vec![0, 1, 2],
            sub_meshes: vec![ModelSubMesh { material: 0, index_start: 0, index_count: 3 }],
        }],
        materials: vec![ModelMaterial {
            diffuse_id: Some(asset_id),
            diffuse: Some(TextureImage {
                width: 1,
                height: 1,
                format: ImageFormat::Rgba8,
                levels: vec![vec![255, 255, 255, 255]],
            }),
            alpha_mode: AlphaMode::Opaque,
            two_sided: false,
        }],
    }
}
