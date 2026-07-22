
use crate::{DefStruct, WireStruct};
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, WireStruct)]
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

/// `CDegradableDef` — original PC release.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct DegradableDef {
    #[def("Degradable")]
    pub degradable: bool,
    #[def("GraphicType")]
    pub graphic_type: EngineGraphicType,
    #[def("InitiallyBlocksNavigation", default = true)]
    pub initially_blocks_navigation: bool,
    #[def("Degradations")]
    pub degradations: Vec<DegradableInfo>,
}
