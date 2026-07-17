use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `UI_LOCALE_GRAPHICS_DEF` — C++ `NUISystem::CUILocaleGraphicsDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct UILocaleGraphicsDef {
        "WorldMapRegionName" => pub world_map_region_name: VecMap<u32, WorldMapNameGraphic>,
        "HelpScreenGraphics" => pub help_screen_graphics: Vec<u32>,
        "HelpRingPic" => pub help_ring_pic: u32,
    }
}
