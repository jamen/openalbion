//! `CDegradableDef` — C++ `CDegradableDef`.

use crate::{def_struct, wire_struct};
use crate::def::prelude::*;

wire_struct! {
    /// C++ `CDegradableInfo` — original PC release layout.
    pub struct DegradableInfo {
        pub health_percentage: f32,
        pub bank_index: i32,
        pub anim_step: f32,
        pub render_size_x: f32,
        pub type_: u8,
        pub additive_alpha: u8,
        pub smash_particle_emitter: i32,
        pub blocks_navigation: bool,
        pub skip: [u8; 4],
    }
}

def_struct! {
    /// `CDegradableDef` — original PC release.
    #[derive(Debug, Clone, PartialEq)]
    pub struct DegradableDef {
        "Degradable" => pub degradable: bool,
        "GraphicType" => pub graphic_type: EngineGraphicType,
        "InitiallyBlocksNavigation" => pub initially_blocks_navigation: bool,
        "Degradations" => pub degradations: Vec<DegradableInfo>,
    }
}
