//! Parser for Fable's mesh assets (the "Bbm" / tagged 3D model format) stored in `.big` archives.
//!
//! A mesh holds a name, bounding volumes, helper points/dummies, a skeleton (bones + keyframes),
//! materials (texture ids), and a list of render primitives. Each primitive carries a vertex buffer
//! (position / normal / UV) and a triangle index buffer — enough to draw the model.
//!
//! Several sub-sections are LZO-compressed in place (see [`decompress_section`]); positions and
//! normals use a packed 11/11/10-bit format and UVs a 16-bit fixed-point format, both unpacked
//! here. Bone/cloth/particle data is parsed for byte-correctness but is not needed for rendering.
//!
//! Ported from the historical `model.rs` (2021), which fed the first textured model renderer.

use crate::bytes::{TakeError, UnexpectedEnd, take, take_bytes, take_null_terminated_bytes};
use bytemuck::{Pod, Zeroable};
use derive_more::{Display, Error, From};
use lzo::LzoError;
use std::collections::HashMap;

#[derive(Debug, Display, Error, From)]
pub enum MeshError {
    Take(TakeError),
    Bytes(UnexpectedEnd),
    Decompress(LzoError),
    #[from(skip)]
    #[display("invalid UTF-8 in mesh string")]
    Utf8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub name: String,
    pub animated: bool,
    pub bounding_sphere: BoundingSphere,
    pub bounding_box: BoundingBox,
    pub helper_points: Vec<HelperPoint>,
    pub helper_dummies: Vec<HelperDummy>,
    pub helper_point_names: Vec<u8>,
    pub helper_dummy_names: Vec<u8>,
    pub bones: Vec<Bone>,
    pub bone_keyframes: Vec<Keyframe>,
    pub bone_transforms: Vec<[f32; 16]>,
    pub bone_name_indices: Vec<u16>,
    pub bone_names: Vec<u8>,
    pub cloth: u8,
    pub transform_matrix: [f32; 12],
    pub materials: Vec<Material>,
    pub primitives: Vec<Primitive>,
}

impl Mesh {
    /// Decode a mesh from a `.big` asset's bytes.
    pub fn decode(mut data: &[u8]) -> Result<Mesh, MeshError> {
        let i = &mut data;

        let name = read_str_nul(i)?;
        let animated = read_u8(i)? > 0;
        let bounding_sphere = BoundingSphere::parse(i)?;
        let bounding_box = BoundingBox::parse(i)?;
        let helper_point_count = read_u16(i)?;
        let helper_dummy_count = read_u16(i)?;
        let helper_names_size = read_u32(i)?;
        let _padding = read_u16(i)?;

        let helper_points = decode_helper_points(i, helper_point_count)?;
        let helper_dummies = decode_helper_dummies(i, helper_dummy_count)?;
        let (helper_point_names, helper_dummy_names) = decode_helper_names(i, helper_names_size)?;

        let material_count = read_u32(i)?;
        let primitive_count = read_u32(i)?;
        let bone_count = read_u32(i)?;
        let bone_names_len = read_u32(i)?;
        let cloth = read_u8(i)?;
        let _unknown_1 = read_u16(i)?;
        let _unknown_2 = read_u16(i)?;

        let (bone_name_indices, bone_names) = decode_bone_names(i, bone_count, bone_names_len)?;
        let bones = decode_bones(i, bone_count)?;
        let bone_keyframes = decode_bone_keyframes(i, bone_count)?;
        let bone_transforms = decode_bone_transforms(i, bone_count)?;
        let transform_matrix = read_mat3x4(i)?;

        let mut materials = Vec::with_capacity(material_count as usize);
        for _ in 0..material_count {
            materials.push(Material::parse(i)?);
        }

        let mut primitives = Vec::with_capacity(primitive_count as usize);
        for _ in 0..primitive_count {
            primitives.push(Primitive::parse(i)?);
        }

        Ok(Mesh {
            name,
            animated,
            bounding_sphere,
            bounding_box,
            helper_points,
            helper_dummies,
            helper_point_names,
            helper_dummy_names,
            bones,
            bone_keyframes,
            bone_transforms,
            bone_name_indices,
            bone_names,
            cloth,
            transform_matrix,
            materials,
            primitives,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingSphere {
    pub center: [f32; 3],
    pub radius: f32,
}

impl BoundingSphere {
    fn parse(i: &mut &[u8]) -> Result<BoundingSphere, MeshError> {
        Ok(BoundingSphere {
            center: read_vec3(i)?,
            radius: read_f32(i)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl BoundingBox {
    fn parse(i: &mut &[u8]) -> Result<BoundingBox, MeshError> {
        Ok(BoundingBox {
            min: read_vec3(i)?,
            max: read_vec3(i)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HelperPoint {
    pub bank_id: u32,
    pub point: [f32; 3],
    pub bone_index: i32,
}

fn decode_helper_points(i: &mut &[u8], count: u16) -> Result<Vec<HelperPoint>, MeshError> {
    let mut points = Vec::with_capacity(count as usize);
    if count > 0 {
        let data = decompress_section(i, 20 * count as usize)?;
        let d = &mut &data[..];
        for _ in 0..count {
            points.push(HelperPoint {
                bank_id: read_u32(d)?,
                point: read_vec3(d)?,
                bone_index: read_i32(d)?,
            });
        }
    }
    Ok(points)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HelperDummy {
    pub bank_id: u32,
    pub transform: [f32; 12],
    pub bone_index: i32,
}

fn decode_helper_dummies(i: &mut &[u8], count: u16) -> Result<Vec<HelperDummy>, MeshError> {
    let mut dummies = Vec::with_capacity(count as usize);
    if count > 0 {
        let data = decompress_section(i, 56 * count as usize)?;
        let d = &mut &data[..];
        for _ in 0..count {
            dummies.push(HelperDummy {
                bank_id: read_u32(d)?,
                transform: read_mat3x4(d)?,
                bone_index: read_i32(d)?,
            });
        }
    }
    Ok(dummies)
}

/// Returns `(helper_point_names, helper_dummy_names)` as raw blobs.
fn decode_helper_names(i: &mut &[u8], size: u32) -> Result<(Vec<u8>, Vec<u8>), MeshError> {
    if size == 0 {
        return Ok((Vec::new(), Vec::new()));
    }
    let names = decompress_section(i, size as usize)?;
    let d = &mut &names[..];
    let point_index_size = read_u16(d)?;
    let point_names = take_bytes(d, point_index_size.saturating_sub(2) as usize)?.to_vec();
    let dummy_names = take_bytes(d, size.saturating_sub(point_index_size as u32) as usize)?.to_vec();
    Ok((point_names, dummy_names))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bone {
    pub name_crc: u32,
    pub parent: i32,
    pub original_child_count: i32,
    pub matrix: [f32; 12],
}

fn decode_bone_names(
    i: &mut &[u8],
    bone_count: u32,
    bone_names_len: u32,
) -> Result<(Vec<u16>, Vec<u8>), MeshError> {
    if bone_count == 0 || bone_names_len == 0 {
        return Ok((Vec::new(), Vec::new()));
    }
    let indices_data = decompress_section(i, 2 * (bone_count - 1) as usize)?;
    let d = &mut &indices_data[..];
    let mut indices = Vec::with_capacity((bone_count - 1) as usize);
    for _ in 0..bone_count - 1 {
        indices.push(read_u16(d)?);
    }
    let names = decompress_section(i, bone_names_len as usize)?;
    Ok((indices, names))
}

fn decode_bones(i: &mut &[u8], bone_count: u32) -> Result<Vec<Bone>, MeshError> {
    let mut bones = Vec::with_capacity(bone_count as usize);
    if bone_count > 0 {
        let data = decompress_section(i, 60 * bone_count as usize)?;
        let d = &mut &data[..];
        for _ in 0..bone_count {
            bones.push(Bone {
                name_crc: read_u32(d)?,
                parent: read_i32(d)?,
                original_child_count: read_i32(d)?,
                matrix: read_mat3x4(d)?,
            });
        }
    }
    Ok(bones)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keyframe {
    pub rotation: [f32; 4],
    pub translation: [f32; 4],
    pub scale: [f32; 4],
}

fn decode_bone_keyframes(i: &mut &[u8], bone_count: u32) -> Result<Vec<Keyframe>, MeshError> {
    let mut keyframes = Vec::with_capacity(bone_count as usize);
    if bone_count > 0 {
        let data = decompress_section(i, 48 * bone_count as usize)?;
        let d = &mut &data[..];
        for _ in 0..bone_count {
            keyframes.push(Keyframe {
                rotation: read_vec4(d)?,
                translation: read_vec4(d)?,
                scale: read_vec4(d)?,
            });
        }
    }
    Ok(keyframes)
}

fn decode_bone_transforms(i: &mut &[u8], bone_count: u32) -> Result<Vec<[f32; 16]>, MeshError> {
    let mut transforms = Vec::with_capacity(bone_count as usize);
    if bone_count > 0 {
        let data = decompress_section(i, 64 * bone_count as usize)?;
        let d = &mut &data[..];
        for _ in 0..bone_count {
            transforms.push(read_mat4(d)?);
        }
    }
    Ok(transforms)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub id: u32,
    pub name: String,
    pub decal_id: u32,
    pub base_texture_id: u32,
    pub bumpmap_texture_id: u32,
    pub reflect_texture_id: u32,
    pub illumination_texture_id: u32,
    pub map_flags: u32,
    pub self_illumination: u32,
    pub two_sided: bool,
    pub transparent: bool,
    pub boolean_alpha: bool,
    pub degenerate_triangles: u16,
}

impl Material {
    fn parse(i: &mut &[u8]) -> Result<Material, MeshError> {
        Ok(Material {
            id: read_u32(i)?,
            name: read_str_nul(i)?,
            decal_id: read_u32(i)?,
            base_texture_id: read_u32(i)?,
            bumpmap_texture_id: read_u32(i)?,
            reflect_texture_id: read_u32(i)?,
            illumination_texture_id: read_u32(i)?,
            map_flags: read_u32(i)?,
            self_illumination: read_u32(i)?,
            two_sided: read_u8(i)? != 0,
            transparent: read_u8(i)? != 0,
            boolean_alpha: read_u8(i)? != 0,
            degenerate_triangles: read_u16(i)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrimitiveBlock {
    pub primitive_count: u32,
    pub start_index: u32,
    pub is_strip: bool,
    pub change_flags: u8,
    pub degenerate_triangles: bool,
}

impl PrimitiveBlock {
    fn parse(i: &mut &[u8]) -> Result<PrimitiveBlock, MeshError> {
        Ok(PrimitiveBlock {
            primitive_count: read_u32(i)?,
            start_index: read_u32(i)?,
            is_strip: read_u8(i)? != 0,
            change_flags: read_u8(i)?,
            degenerate_triangles: read_u8(i)? != 0,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrimitiveStaticBlock {
    pub base: PrimitiveBlock,
    pub material_index: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveAnimatedBlock {
    pub base: PrimitiveBlock,
    pub vertex_count: u32,
    pub bones_per_vertex: u16,
    pub paletted_flag: bool,
    pub groups: Vec<u8>,
}

/// A render vertex. `#[repr(C)]` + `Pod` so it can be uploaded to the GPU directly.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Primitive {
    pub material_index: u32,
    pub repeating_mesh_reps: u32,
    pub bounding_sphere: BoundingSphere,
    pub average_texture_stretch: f32,
    pub vertex_count: u32,
    pub triangle_count: u32,
    pub index_count: u32,
    pub init_flags: u32,
    pub static_blocks: Vec<PrimitiveStaticBlock>,
    pub animated_blocks: Vec<PrimitiveAnimatedBlock>,
    pub pos_scale: [f32; 4],
    pub pos_bias: [f32; 4],
    pub vertex_size: u32,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub cloth_primitives: Vec<ClothPrimitive>,
}

impl Primitive {
    fn parse(i: &mut &[u8]) -> Result<Primitive, MeshError> {
        let material_index = read_u32(i)?;
        let repeating_mesh_reps = read_u32(i)?;
        let bounding_sphere = BoundingSphere::parse(i)?;
        let average_texture_stretch = read_f32(i)?;
        let _static_block_count_hint = read_u32(i)?;
        let _animated_block_count_hint = read_u32(i)?;
        let vertex_count = read_u32(i)?;
        let triangle_count = read_u32(i)?;
        let index_count = read_u32(i)?;
        let init_flags = read_u32(i)?;
        let static_block_count = read_u32(i)?;
        let animated_block_count = read_u32(i)?;

        let mut static_blocks = Vec::with_capacity(static_block_count as usize);
        for _ in 0..static_block_count {
            let base = PrimitiveBlock::parse(i)?;
            static_blocks.push(PrimitiveStaticBlock {
                base,
                material_index: read_u32(i)?,
            });
        }

        let mut animated_blocks = Vec::with_capacity(animated_block_count as usize);
        for _ in 0..animated_block_count {
            let base = PrimitiveBlock::parse(i)?;
            let vertex_count = read_u32(i)?;
            let bones_per_vertex = read_u16(i)?;
            let paletted_flag = read_u8(i)? != 0;
            let groups_len = read_u8(i)?;
            let groups = take_bytes(i, groups_len as usize)?.to_vec();
            animated_blocks.push(PrimitiveAnimatedBlock {
                base,
                vertex_count,
                bones_per_vertex,
                paletted_flag,
                groups,
            });
        }

        let pos_scale = read_vec4(i)?;
        let pos_bias = read_vec4(i)?;
        let vertex_size = read_u32(i)?;
        let _padding = read_u32(i)?;

        let reps = if repeating_mesh_reps < 2 {
            1
        } else {
            repeating_mesh_reps as usize
        };

        // Repeating ("scatter") meshes store `reps` copies of the base mesh; the index buffer
        // references all of them, so decode the full `vertex_count * reps` vertex set.
        let total_vertices = vertex_count as usize * reps;
        let vertex_data = decompress_section(i, total_vertices * vertex_size as usize)?;
        let vd = &mut &vertex_data[..];
        let mut vertices = Vec::with_capacity(total_vertices);
        for _ in 0..total_vertices {
            vertices.push(decode_vertex(
                vd,
                vertex_size,
                animated_block_count > 0,
                repeating_mesh_reps > 0,
                init_flags,
                pos_scale,
                pos_bias,
            )?);
        }

        let index_data = decompress_section(i, 2 * index_count as usize * reps)?;

        // Expand the static and animated blocks' triangle lists / strips into a flat index list.
        let mut indices = Vec::new();
        for block in &static_blocks {
            let mut view = &index_data[block.base.start_index as usize * 2..];
            expand_block(&mut view, &block.base, &mut indices)?;
        }
        let mut animated_view = &index_data[..];
        for block in &animated_blocks {
            expand_block(&mut animated_view, &block.base, &mut indices)?;
        }

        let cloth_count = read_u32(i)?;
        let mut cloth_primitives = Vec::with_capacity(cloth_count as usize);
        for _ in 0..cloth_count {
            cloth_primitives.push(ClothPrimitive::parse(i)?);
        }

        Ok(Primitive {
            material_index,
            repeating_mesh_reps,
            bounding_sphere,
            average_texture_stretch,
            vertex_count,
            triangle_count,
            index_count,
            init_flags,
            static_blocks,
            animated_blocks,
            pos_scale,
            pos_bias,
            vertex_size,
            vertices,
            indices,
            cloth_primitives,
        })
    }
}

/// Read one triangle list / strip block out of an index buffer into `indices`.
///
/// For a triangle strip, each step emits a triangle from a sliding window of three; for a list,
/// each step consumes a fresh triple. (This mirrors the historical decoder; strip winding is not
/// flipped per-triangle.)
fn expand_block(view: &mut &[u8], block: &PrimitiveBlock, indices: &mut Vec<u16>) -> Result<(), MeshError> {
    for _ in 0..block.primitive_count {
        if block.is_strip {
            let v1 = read_u16(view)?;
            let mut peek = &view[..];
            let v2 = read_u16(&mut peek)?;
            let v3 = read_u16(&mut peek)?;
            indices.extend_from_slice(&[v1, v2, v3]);
        } else {
            let v1 = read_u16(view)?;
            let v2 = read_u16(view)?;
            let v3 = read_u16(view)?;
            indices.extend_from_slice(&[v1, v2, v3]);
        }
    }
    Ok(())
}

fn decode_vertex(
    data: &mut &[u8],
    vertex_size: u32,
    animated: bool,
    repeating: bool,
    init_flags: u32,
    pos_scale: [f32; 4],
    pos_bias: [f32; 4],
) -> Result<Vertex, MeshError> {
    // Each vertex occupies `vertex_size` bytes; parse the attributes we use out of that window.
    let mut v = take_bytes(data, vertex_size as usize)?;

    let mut pos = if !repeating && (init_flags & 0b10000) != 0b10000 {
        read_packed_vec3(&mut v)?
    } else {
        read_vec3(&mut v)?
    };
    pos[0] = pos[0] * pos_scale[0] + pos_bias[0];
    pos[1] = pos[1] * pos_scale[1] + pos_bias[1];
    pos[2] = pos[2] * pos_scale[2] + pos_bias[2];

    // Per-vertex bone weights/indices (skipped — not used for static rendering).
    if !repeating && animated {
        let _ = take_bytes(&mut v, 8);
    }

    let normal = if !repeating {
        read_packed_vec3(&mut v)?
    } else {
        read_vec3(&mut v)?
    };

    let uv = if !repeating {
        [decode_uv(read_u16(&mut v)?), decode_uv(read_u16(&mut v)?)]
    } else {
        [read_f32(&mut v)?, read_f32(&mut v)?]
    };

    // Optional extra vector attribute.
    if !repeating && (init_flags & 0b10) == 0b10 {
        let _ = take_bytes(&mut v, 8);
    }

    Ok(Vertex { pos, normal, uv })
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClothPrimitive {
    pub cloth_id: u32,
    pub material_id: u32,
    pub constraints: Vec<ParticleConstraint>,
    pub particles: Particles,
    pub triangle_indices: Vec<[u16; 3]>,
    pub quad_indices: Vec<[u16; 4]>,
    pub render_vertices: Vec<[u16; 2]>,
    pub non_sim_positions: Vec<[f32; 3]>,
    pub indexed_texture_coords: Vec<[f32; 2]>,
    pub particle_indices: Vec<i32>,
    pub vertex_indices: Vec<i32>,
    pub average_patch_size: f32,
    pub bezier_enabled: bool,
    pub export_particles: HashMap<String, u32>,
}

impl ClothPrimitive {
    fn parse(i: &mut &[u8]) -> Result<ClothPrimitive, MeshError> {
        let cloth_id = read_u32(i)?;
        let material_id = read_u32(i)?;
        let cloth_data_len = read_u32(i)?;
        let cloth = decompress_section(i, cloth_data_len as usize)?;
        let d = &mut &cloth[..];

        let _unknown_1 = read_u32(d)?;

        let constraints_len = read_u32(d)?;
        let mut constraints_data = take_bytes(d, constraints_len as usize)?;
        let mut constraints = Vec::new();
        while let Some(c) = ParticleConstraint::parse(&mut constraints_data)? {
            constraints.push(c);
        }

        let particles = Particles::parse(d)?;

        let triangle_count = read_u32(d)?;
        let mut triangle_indices = Vec::with_capacity(triangle_count as usize);
        for _ in 0..triangle_count {
            triangle_indices.push([read_u16(d)?, read_u16(d)?, read_u16(d)?]);
        }

        let quad_count = read_u32(d)?;
        let mut quad_indices = Vec::with_capacity(quad_count as usize);
        for _ in 0..quad_count {
            quad_indices.push([read_u16(d)?, read_u16(d)?, read_u16(d)?, read_u16(d)?]);
        }

        let render_vertex_count = read_u32(d)?;
        let mut render_vertices = Vec::with_capacity(render_vertex_count as usize);
        for _ in 0..render_vertex_count {
            render_vertices.push([read_u16(d)?, read_u16(d)?]);
        }

        let non_sim_count = read_u32(d)?;
        let mut non_sim_positions = Vec::with_capacity(non_sim_count as usize);
        for _ in 0..non_sim_count {
            non_sim_positions.push(read_vec3(d)?);
        }

        let tex_coord_count = read_u32(d)?;
        let mut indexed_texture_coords = Vec::with_capacity(tex_coord_count as usize);
        for _ in 0..tex_coord_count {
            indexed_texture_coords.push([read_f32(d)?, read_f32(d)?]);
        }

        let particle_index_count = read_u32(d)?;
        let mut particle_indices = Vec::with_capacity(particle_index_count as usize);
        for _ in 0..particle_index_count {
            particle_indices.push(read_i32(d)?);
        }

        let vertex_index_count = read_u32(d)?;
        let mut vertex_indices = Vec::with_capacity(vertex_index_count as usize);
        for _ in 0..vertex_index_count {
            vertex_indices.push(read_i32(d)?);
        }

        let average_patch_size = read_f32(d)?;
        let bezier_enabled = read_u8(d)? != 0;

        let export_count = read_u32(d)?;
        let mut export_particles = HashMap::with_capacity(export_count as usize);
        for _ in 0..export_count {
            let name = read_str_u32_prefix(d)?;
            export_particles.insert(name, read_u32(d)?);
        }

        Ok(ClothPrimitive {
            cloth_id,
            material_id,
            constraints,
            particles,
            triangle_indices,
            quad_indices,
            render_vertices,
            non_sim_positions,
            indexed_texture_coords,
            particle_indices,
            vertex_indices,
            average_patch_size,
            bezier_enabled,
            export_particles,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParticleConstraint {
    Repeat(u32),
    RepeatEnd,
    Distance { ids: [u32; 2], distance: f32, strength: f32 },
    DistanceMinMax { ids: [u32; 2], min: f32, max: f32 },
    Unbend { ids: [u32; 3], strength: f32 },
    SphereCollision { center: [f32; 3], radius: f32 },
}

impl ParticleConstraint {
    /// Returns `Ok(None)` at the natural end of the constraint list (unknown type or no data left).
    fn parse(i: &mut &[u8]) -> Result<Option<ParticleConstraint>, MeshError> {
        if i.len() < 12 {
            return Ok(None);
        }
        let type_id = read_u32(i)?;
        let repeat_count = read_u32(i)?;
        let size = read_u32(i)?;
        Ok(match (type_id, size) {
            (0, 0) => Some(ParticleConstraint::Repeat(repeat_count)),
            (1, 0) => Some(ParticleConstraint::RepeatEnd),
            (2, 16) => Some(ParticleConstraint::Distance {
                ids: [read_u32(i)?, read_u32(i)?],
                distance: read_f32(i)?,
                strength: read_f32(i)?,
            }),
            (3, 16) => Some(ParticleConstraint::DistanceMinMax {
                ids: [read_u32(i)?, read_u32(i)?],
                min: read_f32(i)?,
                max: read_f32(i)?,
            }),
            (4, 16) => Some(ParticleConstraint::Unbend {
                ids: [read_u32(i)?, read_u32(i)?, read_u32(i)?],
                strength: read_f32(i)?,
            }),
            (5, 16) => Some(ParticleConstraint::SphereCollision {
                center: read_vec3(i)?,
                radius: read_f32(i)?,
            }),
            _ => None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Particles {
    pub timestep: f32,
    pub timestep_changed: f32,
    pub timestep_multiplier: f32,
    pub positions: Vec<[f32; 3]>,
    pub sim_alphas: Vec<f32>,
    pub gravity_strength: f32,
    pub wind_strength: f32,
    pub dragging_enabled: bool,
    pub dragging_rotational: bool,
    pub dragging_strength: f32,
    pub acceleration_enabled: bool,
    pub global_damping: f32,
}

impl Particles {
    fn parse(i: &mut &[u8]) -> Result<Particles, MeshError> {
        let timestep = read_f32(i)?;
        let timestep_changed = read_f32(i)?;
        let timestep_multiplier = read_f32(i)?;
        let count = read_u32(i)?;

        let mut positions = Vec::with_capacity(count as usize);
        for _ in 0..count {
            positions.push(read_vec3(i)?);
        }
        let mut sim_alphas = Vec::with_capacity(count as usize);
        for _ in 0..count {
            sim_alphas.push(read_f32(i)?);
        }

        Ok(Particles {
            timestep,
            timestep_changed,
            timestep_multiplier,
            positions,
            sim_alphas,
            gravity_strength: read_f32(i)?,
            wind_strength: read_f32(i)?,
            dragging_enabled: read_u8(i)? != 0,
            dragging_rotational: read_u8(i)? != 0,
            dragging_strength: read_f32(i)?,
            acceleration_enabled: read_u8(i)? != 0,
            global_damping: read_f32(i)?,
        })
    }
}

// --- byte helpers ---------------------------------------------------------------------------

fn read_u8(i: &mut &[u8]) -> Result<u8, MeshError> {
    Ok(take::<u8>(i)?)
}
fn read_u16(i: &mut &[u8]) -> Result<u16, MeshError> {
    Ok(take::<u16>(i)?.to_le())
}
fn read_u32(i: &mut &[u8]) -> Result<u32, MeshError> {
    Ok(take::<u32>(i)?.to_le())
}
fn read_i32(i: &mut &[u8]) -> Result<i32, MeshError> {
    Ok(take::<i32>(i)?.to_le())
}
fn read_f32(i: &mut &[u8]) -> Result<f32, MeshError> {
    Ok(f32::from_bits(take::<u32>(i)?.to_le()))
}
fn read_vec3(i: &mut &[u8]) -> Result<[f32; 3], MeshError> {
    Ok([read_f32(i)?, read_f32(i)?, read_f32(i)?])
}
fn read_vec4(i: &mut &[u8]) -> Result<[f32; 4], MeshError> {
    Ok([read_f32(i)?, read_f32(i)?, read_f32(i)?, read_f32(i)?])
}
fn read_mat<const N: usize>(i: &mut &[u8]) -> Result<[f32; N], MeshError> {
    let mut m = [0.0; N];
    for x in &mut m {
        *x = read_f32(i)?;
    }
    Ok(m)
}
fn read_mat3x4(i: &mut &[u8]) -> Result<[f32; 12], MeshError> {
    read_mat::<12>(i)
}
fn read_mat4(i: &mut &[u8]) -> Result<[f32; 16], MeshError> {
    read_mat::<16>(i)
}
fn read_str_nul(i: &mut &[u8]) -> Result<String, MeshError> {
    let bytes = take_null_terminated_bytes(i)?;
    Ok(str::from_utf8(bytes).map_err(|_| MeshError::Utf8)?.to_owned())
}
fn read_str_u32_prefix(i: &mut &[u8]) -> Result<String, MeshError> {
    let len = read_u32(i)? as usize;
    let bytes = take_bytes(i, len)?;
    Ok(str::from_utf8(bytes).map_err(|_| MeshError::Utf8)?.to_owned())
}
fn read_packed_vec3(i: &mut &[u8]) -> Result<[f32; 3], MeshError> {
    Ok(unpack_packed_vec3(read_u32(i)?))
}

/// 16-bit fixed-point UV component: `value / 2048 - 8`.
fn decode_uv(x: u16) -> f32 {
    x as f32 * 0.00048828 - 8.0
}

/// Unpack a 3D vector where X and Y are 11-bit minifloats and Z is a 10-bit minifloat.
fn unpack_packed_vec3(v: u32) -> [f32; 3] {
    let xe = (v >> 6) & 0x1f;
    let xm = v & 0x3f;
    let ye = (v >> 17) & 0x1f;
    let ym = (v >> 11) & 0x3f;
    let ze = (v >> 27) & 0x1f;
    let zm = (v >> 22) & 0x1f;
    [
        unpack_component(xe, xm, 0x40, 17),
        unpack_component(ye, ym, 0x40, 17),
        unpack_component(ze, zm, 0x20, 18),
    ]
}

fn unpack_component(exp_bits: u32, mantissa_bits: u32, denorm_mask: u32, shift: u32) -> f32 {
    let mut mantissa = mantissa_bits;
    let exponent = if exp_bits == 0x1f {
        0x7f80_0000 | (mantissa_bits << 17)
    } else if mantissa_bits != 0 {
        let mut e = 1u32;
        loop {
            e = e.wrapping_sub(1);
            mantissa <<= 1;
            if (mantissa & denorm_mask) == 0 {
                break;
            }
        }
        mantissa &= 0x1f;
        e
    } else {
        112u32.wrapping_neg()
    };
    f32::from_bits((exponent.wrapping_add(112) << 23) | (mantissa << shift))
}

/// Decompress a "semi-compressed" section: an optional LZO chunk followed by raw bytes, totalling
/// `size` decompressed bytes. A `u16` (or `u32` if `0xFFFF`) prefix gives the compressed length.
fn decompress_section(i: &mut &[u8], mut size: usize) -> Result<Vec<u8>, MeshError> {
    let short_len = read_u16(i)?;
    let compressed_len = if short_len == 0xFFFF {
        read_u32(i)? as usize
    } else {
        short_len as usize
    };

    let mut out = Vec::with_capacity(size);
    if compressed_len > 0 {
        let input = take_bytes(i, compressed_len)?;
        let decompressed = lzo::decompress(input, size)?;
        size -= decompressed.len();
        out.extend(decompressed);
    }
    if size > 0 {
        out.extend(take_bytes(i, size)?);
    }
    Ok(out)
}
