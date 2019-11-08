pub mod decode;
pub mod encode;

use std::collections::HashMap;

// Human readable format.

pub struct Bncfg {
    pub group_settings: HashMap<String, Vec<BncfgValue>>,
    pub bone_data: HashMap<String, Vec<BncfgValue>>,
}

pub enum BncfgValue {
    String(String),
    F32(f32),
}