pub mod decode;
pub mod encode;

use std::collections::HashMap;

// Human readable format.

struct Bncfg {
    group_settings: HashMap<String, Vec<BncfgValue>>,
    bone_data: HashMap<String, Vec<BncfgValue>>,
}

enum BncfgValue {
    String(String),
    F32(f32),
}