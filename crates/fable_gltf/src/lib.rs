use std::io::Error;

use fable::Lev;

use gltf_json::root::Index;
use gltf_json::{Root,Node,Scene,Value,Mesh,Accessor,Buffer};
use gltf_json::buffer::{View,Target};
use gltf_json::accessor::{GenericComponentType,ComponentType,Type};
use gltf_json::mesh::{Primitive,Mode,Semantic};
use gltf_json::validation::Checked;

pub struct MeshConfig {
    pub cell_height_modifier: f32,
    pub cell_distance: f32,
    pub width: usize,
    pub height: usize,
}

pub fn encode_lev_to_mesh(lev: Lev, bin_file: &str) -> Result<(Vec<u8>, Root), Error> {
    let mut positions: Vec<u8> = Vec::new();

    let config = MeshConfig {
        cell_height_modifier: 2048f32,
        cell_distance: 1f32,
        width: (lev.header.width + 1) as usize,
        height: (lev.header.height + 1) as usize,
    };

    let mut x = 0f32;
    let mut y = 0f32;

    let max = lev.heightmap_cells.len();

    println!("lev cell len {}", max);
    println!("lev width {}", lev.header.width);
    println!("lev height {}", lev.header.height);
    println!("lev width * height {}", lev.header.width * lev.header.height);
    println!("lev (width + 1) * (height + 1) {}", (lev.header.width + 1) * (lev.header.height + 1));

    // [cell_1] [cell_2]
    // [cell_3] [cell_4]

    for (i, cell) in lev.heightmap_cells.iter().enumerate() {
        let cell_1_height = cell.height;

        // Neighbor right of cell
        let cell_2_height = if i + 1 % config.width > 0 {
            if let Some(neighbor_cell) = lev.heightmap_cells.get(i + 1) {
                neighbor_cell.height
            } else {
                cell_1_height
            }
        } else {
            cell_1_height
        };

        let cell_3_height = if i + config.width < max {
            if let Some(neighbor_cell) = lev.heightmap_cells.get(i + config.width) {
                neighbor_cell.height
            } else {
                cell_1_height
            }
        } else {
            cell_1_height
        };

        let cell_4_height = if i + config.width + 1 % config.width > 0 {
            if let Some(neighbor_cell) = lev.heightmap_cells.get(i + config.width + 1) {
                neighbor_cell.height
            } else {
                cell_1_height
            }
        } else {
            cell_1_height
        };

        positions.extend_from_slice(
            &[
                // *--.
                // | /
                // ./
                x.to_le_bytes(),
                (cell_1_height * config.cell_height_modifier).to_le_bytes(),
                y.to_le_bytes(),

                // .--*
                // | /
                // ./
                (x + config.cell_distance).to_le_bytes(),
                (cell_2_height * config.cell_height_modifier).to_le_bytes(),
                y.to_le_bytes(),

                // .--.
                // | /
                // */
                x.to_le_bytes(),
                (cell_3_height * config.cell_height_modifier).to_le_bytes(),
                (y + config.cell_distance).to_le_bytes(),

                //   /`
                //  / |
                // `--*
                (x + config.cell_distance).to_le_bytes(),
                (cell_4_height * config.cell_height_modifier).to_le_bytes(),
                (y + config.cell_distance).to_le_bytes(),

                //   /*
                //  / |
                // `--`
                (x + config.cell_distance).to_le_bytes(),
                (cell_2_height * config.cell_height_modifier).to_le_bytes(),
                y.to_le_bytes(),

                //   /`
                //  / |
                // *--`
                x.to_le_bytes(),
                (cell_3_height * config.cell_height_modifier).to_le_bytes(),
                (y + config.cell_distance).to_le_bytes(),
            ].concat()
        );

        x += 1f32;

        if i % config.width == 0 {
            x = 0f32;
            y += 1f32;
        }
    }

    let root = Root {
        accessors: vec![
            Accessor {
                buffer_view: Index::new(0),
                byte_offset: 0,
                count: (lev.heightmap_cells.len() * 6) as u32,
                component_type: Checked::Valid(GenericComponentType(ComponentType::F32)),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Checked::Valid(Type::Vec3),
                min: Some(Value::from(vec![-0.5f32, -0.5f32, 0.0f32])),
                max: Some(Value::from(vec![0.5f32, 0.5f32, 0.0f32])),
                normalized: false,
                sparse: None,
            }
        ],
        buffers:  vec![
            Buffer {
                byte_length: positions.len() as u32,
                uri: Some(bin_file.to_string()),
                extensions: Default::default(),
                extras: Default::default(),
            }
        ],
        buffer_views: vec![
            View {
                buffer: Index::new(0),
                byte_offset: Some(0),
                byte_length: positions.len() as u32,
                byte_stride: None,
                target: Some(Checked::Valid(Target::ElementArrayBuffer)),
                extras: Default::default(),
                extensions: Default::default(),
            },
        ],
        meshes: vec![
            Mesh {
                extensions: Default::default(),
                extras: Default::default(),
                primitives: vec![
                    Primitive {
                        attributes: {
                            let mut map = std::collections::HashMap::new();
                            map.insert(Checked::Valid(Semantic::Positions), Index::new(0));
                            map
                        },
                        extensions: Default::default(),
                        extras: Default::default(),
                        indices: None,
                        material: None,
                        mode: Checked::Valid(Mode::Triangles),
                        targets: None,
                    }
                ],
                weights: None,
            }
        ],
        nodes: vec![
            Node {
                camera: None,
                children: None,
                extensions: Default::default(),
                extras: Default::default(),
                matrix: None,
                mesh: Some(Index::new(0)),
                rotation: None,
                scale: None,
                translation: None,
                skin: None,
                weights: None,
            }
        ],
        scenes: vec![
            Scene {
                extensions: Default::default(),
                extras: Default::default(),
                nodes: vec![Index::new(0)],
            },
        ],
        .. Default::default()
    };

    Ok((positions, root))
}