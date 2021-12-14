use alloc::string::String;
use alloc::vec::Vec;

use crate::script::Item;
use crate::script_parser::ScriptParser;

#[derive(Debug)]
pub struct Wld {
    script: Vec<Item>,
}

impl Wld {
    pub fn parse(data: &str) -> Option<Self> {
        // TODO: Better error handling
        let script = ScriptParser::new().parse(data).ok()?;
        Some(Wld { script })
    }
}

// #[derive(Debug)]
// pub struct Wld {
//     pub start_initial_quests: Vec<String>,
//     pub map_uid_count: isize,
//     pub thing_manager_uid_count: isize,
//     pub maps: Vec<WldMap>,
//     pub regions: Vec<WldRegion>,
// }

// #[derive(Debug)]
// pub struct WldMap {
//     pub new_map: isize,
//     pub map_x: isize,
//     pub map_y: isize,
//     pub level_name: String,
//     pub level_script_name: String,
//     pub map_uid: isize,
//     pub is_sea: bool,
//     pub loaded_on_player_proximity: bool,
// }

// #[derive(Debug)]
// pub struct WldRegion {
//     pub new_region: isize,
//     pub region_name: String,
//     pub new_display_name: String,
//     pub region_def: String,
//     pub appear_on_world_map: bool,
//     pub mini_map_graphic: Option<String>,
//     pub mini_map_scale: f32,
//     pub mini_map_offset_x: f32,
//     pub mini_map_offset_y: f32,
//     pub world_map_offset_x: f32,
//     pub world_map_offset_y: f32,
//     pub name_graphic_offset_x: f32,
//     pub name_graphic_offset_y: f32,
//     pub mini_map_region_exit_text_offset_x: Vec<(String, f32)>,
//     pub mini_map_region_exit_text_offset_y: Vec<(String, f32)>,
//     pub contains_map: Vec<String>,
//     pub sees_map: Vec<String>,
// }
