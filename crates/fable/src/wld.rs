pub mod decode;
pub mod encode;

use crate::script::ScriptField;

#[derive(Debug,PartialEq)]
pub struct Wld {
    start_initial_quests: Vec<ScriptField>,
    map_uid_count: ScriptField,
    thing_manager_uid_count: ScriptField,
    maps: Vec<WldMap>,
    regions: Vec<WldRegion>,
}

#[derive(Debug,PartialEq)]
pub struct WldMap {
    new_map: ScriptField,
    instrs: Vec<ScriptField>,
}

#[derive(Debug,PartialEq)]
pub struct WldRegion {
    new_region: ScriptField,
    instrs: Vec<ScriptField>,
}