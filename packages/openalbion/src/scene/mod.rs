//! Scene assembly: Fable's assets in, renderer inputs out.
//!
//! This is the only place where the two vocabularies meet. The renderer knows about
//! vertices, images and shader constants; `files` knows about `.big` archives, `.lev`
//! heightmaps and compiled defs; nothing crosses between them except through here.
//!
//! That is deliberate. AGENTS.md §0 traces the sky/landscape fix loop to passes inventing
//! their own data lookups; a pass that is handed a finished `TerrainData` has nothing
//! left to invent. It also means this module — not a shader — is where provenance
//! logging belongs: "which byte became which constant" is a conversion question.

pub mod camera_path;
pub mod local_detail;
pub mod model;
pub mod sky;
pub mod terrain;
pub mod texture;
pub mod things;

pub use self::camera_path::{CameraPath, find_camera_paths, rank_camera_paths};
pub use self::local_detail::{LevelLocalDetail, build_local_detail, merge_local_detail};
pub use self::model::build_model;
pub use self::sky::{sky_textures_at_time, upload_sky_textures};
pub use self::terrain::build_region_terrain;
pub use self::texture::decode_texture;
pub use self::things::{merge_things, resolve_things};
