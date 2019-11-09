use std::io::Error;

use fable_format::Lev;

use gltf_json::root::Index;
use gltf_json::{Root,Node,Scene,Value,Mesh,Accessor,Buffer};
use gltf_json::buffer::{View,Target};
use gltf_json::accessor::{GenericComponentType,ComponentType,Type};
use gltf_json::mesh::{Primitive,Mode,Semantic};
use gltf_json::validation::Checked;

pub fn compile_lev_to_mesh(lev: Lev, bin_file: &str) -> Result<(Vec<u8>, Root), Error> {
    let mut positions: Vec<u8> = Vec::new();

    let width = (lev.header.width + 1) as usize;
    let mut x = 0f32;
    let mut z = 0f32;

    for (i, cell) in lev.heightmap_cells.iter().enumerate() {
        positions.extend_from_slice(
            &[
                x.to_le_bytes(),
                (cell.height * 2048f32).to_le_bytes(),
                z.to_le_bytes(),
            ].concat()
        );

        if i % width == 0 {
            z += 5f32;
            x = 0f32;
        }

        x += 5f32;
    }

    let root = Root {
        accessors: vec![
            Accessor {
                buffer_view: Index::new(0),
                byte_offset: 0,
                count: lev.heightmap_cells.len() as u32,
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