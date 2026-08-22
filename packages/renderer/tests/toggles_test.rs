//! Subsystem toggles — the renderer half of the console's `Enable*` commands (AGENTS.md §13.5).
//!
//! The claim being pinned is "a toggle removes **exactly** its own subsystem", which is two
//! assertions rather than one: switching a subsystem off must make its pixels go away, and
//! switching the *other* four off must leave them alone. A test that only checked the first
//! would pass just as well for a toggle that turned off the whole frame.
//!
//! `EnableStaticMeshes` and `EnableAnimatedMeshes` are the pair most at risk here, because
//! unlike the others they are **not** two passes — both kinds draw through `ModelPass` and the
//! toggle is a filter inside it. So each of them has to be shown not to take the other's pixels.
//!
//! Needs a GPU, so it skips when there is not one.

use renderer::{
    AlphaMode, GlyphInstance, ImageFormat, Model, ModelInstance, ModelKind, ModelMaterial,
    ModelPrimitive, ModelSubMesh, ModelVertex, RenderToggles, Renderer, TextureImage,
};

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

/// A quad covering the whole target, drawn with an identity view-projection so its vertices
/// *are* clip coordinates. No level, no camera, no asset — the point here is which pass runs,
/// not what it draws.
///
/// `two_sided` so winding cannot decide the result (§3.11: Fable's meshes are clockwise-front
/// and the pass culls accordingly), and no diffuse map so it samples the 1×1 white fallback.
/// `z = 0.5` passes the reverse-Z depth test against `ClearPass`'s `0.0`.
fn fullscreen_quad() -> Model {
    let corner = |x: f32, y: f32| ModelVertex {
        position: [x, y, 0.5],
        normal: [0.0, 0.0, 1.0],
        uv: [0.0, 0.0],
    };
    Model {
        primitives: vec![ModelPrimitive {
            vertices: vec![
                corner(-1.0, -1.0),
                corner(1.0, -1.0),
                corner(1.0, 1.0),
                corner(-1.0, 1.0),
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            sub_meshes: vec![ModelSubMesh {
                material: 0,
                index_start: 0,
                index_count: 6,
            }],
        }],
        skin: None,
        materials: vec![ModelMaterial {
            diffuse_id: None,
            diffuse: None,
            alpha_mode: AlphaMode::Opaque,
            two_sided: true,
        }],
    }
}

fn red_at(image: &renderer::Image, x: u32, y: u32) -> u8 {
    image.rgba[((y * image.width + x) * 4) as usize]
}

/// Capture the centre pixel with `toggles` in force.
fn centre(renderer: &mut Renderer<'_>, toggles: RenderToggles) -> u8 {
    renderer.set_toggles(toggles);
    let image = renderer.render_to_image().expect("headless capture");
    red_at(&image, SIZE / 2, SIZE / 2)
}

#[test]
fn a_toggle_removes_its_own_subsystem_and_only_its_own() {
    let Some(mut renderer) = headless() else {
        return;
    };

    renderer.update_model_uniforms(glam::Mat4::IDENTITY.to_cols_array_2d());
    renderer
        .add_model(
            &fullscreen_quad(),
            &[ModelInstance {
                transform: glam::Mat4::IDENTITY.to_cols_array_2d(),
                colour: [1.0, 1.0, 1.0, 1.0],
            }],
            ModelKind::Static,
        )
        .expect("upload the quad");

    let all_on = centre(&mut renderer, RenderToggles::default());
    assert!(
        all_on > 0,
        "the quad drew nothing with every subsystem on, so this test cannot see a toggle at all",
    );

    // Its own toggle removes it, down to the clear colour.
    let off = centre(
        &mut renderer,
        RenderToggles {
            static_meshes: false,
            ..Default::default()
        },
    );
    assert_eq!(off, 0, "EnableStaticMeshes false left the mesh drawing");

    // The other four do not. This is the half that catches a toggle wired to the wrong pass —
    // and `animated_meshes` is in here because it shares `ModelPass` with this quad.
    let others_off = centre(
        &mut renderer,
        RenderToggles {
            sky: false,
            landscape: false,
            repeated_meshes: false,
            animated_meshes: false,
            static_meshes: true,
        },
    );
    assert_eq!(
        others_off, all_on,
        "turning off sky, landscape, foliage and animated meshes changed the static mesh pass",
    );

    // And back on again: a toggle is state, not a one-way door.
    assert_eq!(centre(&mut renderer, RenderToggles::default()), all_on);
}

/// The same claim for the animating half, and the direction that actually matters: an
/// animating mesh must answer to `EnableAnimatedMeshes` and **not** to `EnableStaticMeshes`.
///
/// Both kinds go through one pass with one pipeline, so a filter applied to the wrong field —
/// or not applied at all — is invisible in every other test. This is the one that would catch
/// it, and it is the mirror image of the static case above.
#[test]
fn an_animating_mesh_answers_only_to_its_own_toggle() {
    let Some(mut renderer) = headless() else {
        return;
    };

    renderer.update_model_uniforms(glam::Mat4::IDENTITY.to_cols_array_2d());
    renderer
        .add_model(
            &fullscreen_quad(),
            &[ModelInstance {
                transform: glam::Mat4::IDENTITY.to_cols_array_2d(),
                colour: [1.0, 1.0, 1.0, 1.0],
            }],
            ModelKind::Animated,
        )
        .expect("upload the quad");

    let all_on = centre(&mut renderer, RenderToggles::default());
    assert!(all_on > 0, "the animating quad drew nothing with everything on");

    let own_off = centre(
        &mut renderer,
        RenderToggles {
            animated_meshes: false,
            ..Default::default()
        },
    );
    assert_eq!(own_off, 0, "EnableAnimatedMeshes false left the mesh drawing");

    // The decisive one: the *static* toggle must not reach it.
    let static_off = centre(
        &mut renderer,
        RenderToggles {
            static_meshes: false,
            ..Default::default()
        },
    );
    assert_eq!(
        static_off, all_on,
        "EnableStaticMeshes false removed an animating mesh — the filter reads the wrong field",
    );

    // And the reported counts follow the kind, so `ShowStats` cannot disagree with the toggle.
    assert_eq!(renderer.model_stats_of(ModelKind::Animated), (1, 1));
    assert_eq!(renderer.model_stats_of(ModelKind::Static), (0, 0));
}

/// **The console must survive its own toggles.** Text is not a subsystem toggle can reach — a
/// console that could turn off the pass it draws through would have no way to say so.
#[test]
fn text_still_draws_with_every_subsystem_off() {
    let Some(mut renderer) = headless() else {
        return;
    };

    let slot = renderer
        .add_glyph(1, &TextureImage::single(8, 8, ImageFormat::R8, vec![255; 64]))
        .expect("register");
    renderer.set_text(&[GlyphInstance {
        rect: [0.0, 0.0, SIZE as f32, SIZE as f32],
        colour: [1.0, 1.0, 1.0, 1.0],
        texture_index: slot,
        _pad: [0; 3],
    }]);

    let lit = centre(
        &mut renderer,
        RenderToggles {
            sky: false,
            landscape: false,
            static_meshes: false,
            animated_meshes: false,
            repeated_meshes: false,
        },
    );
    assert!(lit > 250, "text read back {lit} with the world off, not ~255");
}

/// The panel behind the console's text is a [`GlyphInstance`] pointing at the bindless array's
/// 1×1 white fallback — no second pipeline and no upload (see `Renderer::solid_index`). What
/// makes that work is that the slot is *not* scene-scoped: `clear_scene` drops every
/// registration, and this one has to survive it or every level load would take the panel with
/// it.
#[test]
fn the_solid_slot_fills_a_rectangle_and_survives_a_scene_clear() {
    let Some(mut renderer) = headless() else {
        return;
    };

    let solid = renderer.solid_index();
    renderer.clear_scene();
    assert_eq!(
        renderer.solid_index(),
        solid,
        "the solid slot moved across a scene clear",
    );

    // Half alpha over the black clear, so the read-back proves the *instance colour* reached
    // the pixel rather than the texture's white.
    renderer.set_text(&[GlyphInstance {
        rect: [0.0, 0.0, SIZE as f32, SIZE as f32],
        colour: [1.0, 1.0, 1.0, 0.5],
        texture_index: solid,
        _pad: [0; 3],
    }]);

    let filled = centre(&mut renderer, RenderToggles::default());
    assert!(
        filled.abs_diff(128) <= 2,
        "the solid slot filled {filled}, not ~128 — it is not opaque white, or coverage is not \
         reaching alpha",
    );
}
