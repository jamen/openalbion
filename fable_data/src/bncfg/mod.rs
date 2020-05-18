//! Bone config.
//!
//! These are found at `Data/Bones`.
//!
//! ## Format Description
//!
//! | Section        | Description                                                     |
//! |----------------|-----------------------------------------------------------------|
//! | Creature type  | A string representing the type of creature these bones are for. |
//! | Group settings | A map of group names to sets of bone names                      |
//! | Bone data      | A map of bone names to `Vector3<f32>` positions.                |
//!
//! The format is human readable, so the files can be referenced by eye.
//!

mod decode;
mod encode;

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