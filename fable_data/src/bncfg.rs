pub use crate::bncfg_parser::BncfgParser;

use std::collections::HashMap;

pub struct Bncfg {
    pub creature_type: String,
    pub group_settings: HashMap<String, Vec<BncfgScriptValue>>,
    pub bone_data: HashMap<String, Vec<BncfgScriptValue>>,
}

pub enum BncfgScriptValue {
    String(String),
    F32(f32),
}
