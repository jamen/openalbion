//! Sky texture resolution and upload.
//!
//! Only *which* textures the sky shows lives here. The gradient colours that drive it come
//! from the environment colour LUT at `(ColourLookupColumn + keyframe, SkyGradient*Row)`
//! as an unfiltered integer texel fetch, which lands with the environment layer
//! (AGENTS.md §3.1, step 2).

use crate::files::Files;
use crate::renderer::Renderer;

/// The sky texture pair for a time of day: `(texture0, texture1, blend)`.
pub type SkyTextures = (Option<String>, Option<String>, f32);

/// Resolve which sky textures a theme wants at `time_of_day`.
///
/// Returns `None` when the theme is missing — the sky is optional, and a level without one
/// should render the rest of the world rather than fail.
pub fn sky_textures_at_time(
    files: &Files,
    theme_name: &str,
    time_of_day: f32,
) -> Option<SkyTextures> {
    files.environment_theme(theme_name).map(|theme| {
        let (tex0, tex1, blend) = theme.sky_textures_at_time(time_of_day);
        (tex0.map(String::from), tex1.map(String::from), blend)
    })
}

/// Upload a resolved sky texture pair. Failures are logged, not fatal.
///
/// `texture1` is only uploaded when it differs from `texture0` — the pass falls back to
/// slot 0 for the blend slot otherwise.
pub fn upload_sky_textures(
    files: &mut Files,
    renderer: &mut Renderer<'_>,
    texture0: Option<&str>,
    texture1: Option<&str>,
) {
    if let Some(name) = texture0 {
        upload_sky_texture(files, renderer, name, false);
    }
    if let Some(name) = texture1.filter(|n| Some(*n) != texture0) {
        upload_sky_texture(files, renderer, name, true);
    }
}

/// Read a sky texture from the textures archive, decode it, and upload it to the
/// renderer's primary (`secondary == false`) or blend (`secondary == true`) slot.
pub fn upload_sky_texture(
    files: &mut Files,
    renderer: &mut Renderer<'_>,
    name: &str,
    secondary: bool,
) {
    let (metadata, bytes) = match files.read_sky_texture(name) {
        Ok(asset) => asset,
        Err(error) => {
            tracing::warn!("Failed to read sky texture {name}: {error}");
            return;
        }
    };

    let image = match super::decode_texture(&metadata, &bytes) {
        Ok(image) => image,
        Err(error) => {
            tracing::warn!("Failed to decode sky texture {name}: {error}");
            return;
        }
    };

    tracing::debug!(
        "Sky texture slot {} ← {name} ({}x{} {:?})",
        secondary as u8,
        image.width,
        image.height,
        image.format,
    );

    if secondary {
        renderer.set_sky_texture1(&image);
    } else {
        renderer.set_sky_texture0(&image);
    }
}
