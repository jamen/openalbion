//! `.lev` heightmap → [`TerrainData`].
//!
//! This is a placeholder architecture. The original builds a *linked list of layer meshes
//! per patch*, each with its own `Blend` / `CliffU` / `CliffV` per vertex and its own
//! foreground/background textures, drawn as N alpha-blended passes
//! (`CLandscapeLayerMesh::CVertex`, engine_landscape_layer_mesh.hpp:71 — AGENTS.md §3.4).
//! Porting `CEngineLandscapeMeshBuilder` (step 5.2) replaces the body of this module and
//! nothing in the renderer, which is the point of building it here.

use crate::files::Files;
use renderer::{ImageFormat, TerrainData, TerrainVertex, TextureImage};
use fable_data::lev::{Lev, LevHeightCell};

// UNVERIFIED: neither value is sourced from the game. LEV stores height as a
// normalised f32; the real world-space scale and cell pitch come from the landscape
// map/patch code (engine_landscape*.cpp), not from us. AGENTS.md §9.
pub const HEIGHT_SCALE: f32 = 2048.0;
const CELL_SIZE: f32 = 1.0;

/// Every terrain layer is decoded to this size so they can share one array texture.
/// Fable's ground textures are 256×256; anything else is rescaled by being skipped, which
/// the pass logs.
const LAYER_TEXTURE_SIZE: u32 = 256;

/// Build the landscape's geometry and layer textures for `lev`.
pub fn build_terrain(files: &mut Files, lev: &Lev) -> TerrainData {
    let (vertices, indices) = build_mesh(
        lev.header.width as usize,
        lev.header.height as usize,
        &lev.heightmap_cells,
    );
    let (layers, palette_to_layer) = build_layers(files, lev);

    let raw_min = lev
        .heightmap_cells
        .iter()
        .map(|c| c.height)
        .fold(f32::INFINITY, f32::min);
    let raw_max = lev
        .heightmap_cells
        .iter()
        .map(|c| c.height)
        .fold(f32::NEG_INFINITY, f32::max);
    tracing::debug!(
        "Terrain height range: raw [{raw_min:.2}, {raw_max:.2}], \
         scaled [{:.2}, {:.2}] (HEIGHT_SCALE {HEIGHT_SCALE} is UNVERIFIED — AGENTS.md §9)",
        raw_min * HEIGHT_SCALE,
        raw_max * HEIGHT_SCALE,
    );

    TerrainData {
        vertices,
        indices,
        layers,
        palette_to_layer,
    }
}

/// `cell_width`/`cell_height` are the LEV's cell counts; the vertex grid has one more of
/// each, since vertices sit on cell corners.
fn build_mesh(
    cell_width: usize,
    cell_height: usize,
    cells: &[LevHeightCell],
) -> (Vec<TerrainVertex>, Vec<u32>) {
    let w = cell_width + 1;
    let h = cell_height + 1;

    let height_at = |col: usize, row: usize| -> f32 {
        cells
            .get(row * w + col)
            .map(|c| c.height * HEIGHT_SCALE)
            .unwrap_or(0.0)
    };

    let mut vertices = Vec::with_capacity(w * h);
    for row in 0..h {
        for col in 0..w {
            let z = height_at(col, row);

            let left = height_at(col.saturating_sub(1), row);
            let right = height_at((col + 1).min(w - 1), row);
            let down = height_at(col, row.saturating_sub(1));
            let up = height_at(col, (row + 1).min(h - 1));
            // Z-up: gradient in X and Y, up is +Z.
            let normal = normalize([-(right - left), -(up - down), 2.0 * CELL_SIZE]);

            let cell = &cells[row * w + col];

            vertices.push(TerrainVertex {
                position: [col as f32 * CELL_SIZE, row as f32 * CELL_SIZE, z],
                normal,
                theme_indices: [
                    cell.ground_theme.0,
                    cell.ground_theme.1,
                    cell.ground_theme.2,
                    0,
                ],
                blend: [
                    cell.ground_theme_strength.0,
                    cell.ground_theme_strength.1,
                    0,
                    0,
                ],
            });
        }
    }

    let mut indices = Vec::with_capacity((w - 1) * (h - 1) * 6);
    for row in 0..h.saturating_sub(1) {
        for col in 0..w.saturating_sub(1) {
            let a = (row * w + col) as u32;
            let b = a + 1;
            let c = a + w as u32;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    (vertices, indices)
}

/// Resolve the level's theme palette to a texture per layer, decoded to a common size and
/// format. Layers that fail to load are replaced by a flat magenta placeholder so the
/// layer indices the mesh refers to stay valid — a missing texture must not silently
/// re-point a vertex at a different theme.
fn build_layers(files: &mut Files, lev: &Lev) -> (Vec<TextureImage>, [u32; 256]) {
    let bundle = files.resolve_terrain_themes(&lev.header.heightmap_palette);

    let mut layers = Vec::with_capacity(bundle.texture_ids.len());
    for (layer, &tex_id) in bundle.texture_ids.iter().enumerate() {
        let image = load_layer(files, tex_id).unwrap_or_else(|error| {
            tracing::warn!("Terrain layer {layer} (texture id {tex_id}): {error} — placeholder");
            placeholder_layer()
        });
        layers.push(image);
    }

    let mut palette_to_layer = [0u32; 256];
    for (slot, &layer) in bundle.palette_to_layer.iter().enumerate() {
        palette_to_layer[slot] = layer as u32;
    }

    (layers, palette_to_layer)
}

fn load_layer(files: &mut Files, tex_id: i32) -> Result<TextureImage, String> {
    let (asset, data) = files.read_texture_by_id(tex_id as u32)?;
    let image = super::decode_texture_rgba(&asset, &data).map_err(|e| e.to_string())?;

    if image.width != LAYER_TEXTURE_SIZE || image.height != LAYER_TEXTURE_SIZE {
        return Err(format!(
            "is {}x{}, expected {LAYER_TEXTURE_SIZE}x{LAYER_TEXTURE_SIZE}",
            image.width, image.height,
        ));
    }

    Ok(image)
}

/// Flat magenta, at the layer array's size — an obviously wrong texture beats a silently
/// shifted layer index.
fn placeholder_layer() -> TextureImage {
    let texels = (LAYER_TEXTURE_SIZE * LAYER_TEXTURE_SIZE) as usize;
    TextureImage {
        width: LAYER_TEXTURE_SIZE,
        height: LAYER_TEXTURE_SIZE,
        format: ImageFormat::Rgba8,
        data: [255u8, 0, 255, 255].repeat(texels),
    }
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(height: f32) -> LevHeightCell {
        LevHeightCell {
            size: 0,
            version: 0,
            height,
            ground_theme: (0, 0, 0),
            ground_theme_strength: (0, 0),
            walkable: true,
            passover: false,
            sound_theme: 0,
            shore: false,
        }
    }

    /// Vertices sit on cell corners, so a `w`×`h` cell grid has `(w+1)*(h+1)` of them and
    /// two triangles per cell.
    #[test]
    fn mesh_has_one_vertex_per_grid_corner() {
        let (w, h) = (3usize, 2usize);
        let cells = vec![cell(0.0); (w + 1) * (h + 1)];

        let (vertices, indices) = build_mesh(w, h, &cells);

        assert_eq!(vertices.len(), (w + 1) * (h + 1));
        assert_eq!(indices.len(), w * h * 6);
        assert!(indices.iter().all(|&i| (i as usize) < vertices.len()));
    }

    /// Height is Z, and the grid lies in X/Y (AGENTS.md §3.6).
    #[test]
    fn height_is_the_z_axis() {
        let cells = vec![cell(0.25); 4];

        let (vertices, _) = build_mesh(1, 1, &cells);

        assert_eq!(vertices[0].position[..2], [0.0, 0.0]);
        assert!((vertices[0].position[2] - 0.25 * HEIGHT_SCALE).abs() < 1e-3);
        // A flat heightfield has every normal pointing straight up.
        assert!((vertices[0].normal[2] - 1.0).abs() < 1e-6);
    }
}
