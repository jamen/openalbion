//! `CReplaceableMeshDef` — C++ `CReplaceableMeshDef`.

use crate::{def_struct, wire_struct};
use crate::def::prelude::*;

wire_struct! {
    /// One entry of `NGraphicAppearance::CReplaceableMeshes` — original PC release layout.
    pub struct ReplaceableMeshesEntry {
        pub bank_index: i32,
        pub anim_step: f32,
        pub render_size_x: f32,
        pub type_: u8,
        pub additive_alpha: u8,
    }
}

wire_struct! {
    /// C++ `NGraphicAppearance::CReplaceableMeshes` — original PC release layout.
    pub struct ReplaceableMeshes {
        pub vector: Vec<ReplaceableMeshesEntry>,
    }
}

def_struct! {
    /// `CReplaceableMeshDef` — original PC release.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ReplaceableMeshDef {
        "Meshes" => pub meshes: ReplaceableMeshes,
    }
}
