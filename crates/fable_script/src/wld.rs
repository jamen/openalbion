pub mod decode;
pub mod encode;

use crate::shared::Instr;

#[derive(Debug,PartialEq)]
pub struct Wld {
    start_initial_quests: Vec<Instr>,
    map_uid_count: Instr,
    thing_manager_uid_count: Instr,
    maps: Vec<WldMap>,
    regions: Vec<WldRegion>,
}

#[derive(Debug,PartialEq)]
pub struct WldMap {
    new_map: Instr,
    instrs: Vec<Instr>,
}

#[derive(Debug,PartialEq)]
pub struct WldRegion {
    new_region: Instr,
    instrs: Vec<Instr>,
}