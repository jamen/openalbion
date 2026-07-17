use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CMultiStaticMeshEntryDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct MultiStaticMeshEntryDef {
        "Mesh" => pub mesh: i32,
        "OverrideLightingChannel" => pub override_lighting_channel: bool,
        "OverrideRenderFadeDistance" => pub override_render_fade_distance: bool,
        "RenderFadeDistance" => pub render_fade_distance: f32,
        "LightingChannel" => pub lighting_channel: LightingChannel,
    }
}
