use crate::DefStruct;
use crate::def::prelude::*;

/// `UI_LOCALE_GRAPHICS_DEF` — C++ `NUISystem::CUILocaleGraphicsDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct UILocaleGraphicsDef {
    #[def("WorldMapRegionName")]
    pub world_map_region_name: BTreeMap<u32, WorldMapNameGraphic>,
    #[def("HelpScreenGraphics")]
    pub help_screen_graphics: Vec<u32>,
    #[def("HelpRingPic")]
    pub help_ring_pic: u32,
}
