mod decode;
mod encode;

use crate::script::Field;

#[derive(Debug,PartialEq)]
pub struct Wld {
    start_initial_quests: Vec<Field>,
    map_uid_count: Field,
    thing_manager_uid_count: Field,
    maps: Vec<WldMap>,
    regions: Vec<WldRegion>,
}

#[derive(Debug,PartialEq)]
pub struct WldMap {
    new_map: Field,
    instrs: Vec<Field>,
}

#[derive(Debug,PartialEq)]
pub struct WldRegion {
    new_region: Field,
    instrs: Vec<Field>,
}