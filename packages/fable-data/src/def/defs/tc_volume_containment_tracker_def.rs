use crate::def_struct;

def_struct! {
    /// `CTCVolumeContainmentTrackerDef` — C++ `CTCVolumeContainmentTrackerDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TCVolumeContainmentTrackerDef {
        "UpdateEveryFrame" => pub update_every_frame: bool,
        "CalculateBlendedLightingChannel" => pub calculate_blended_lighting_channel: bool,
        "LightingChannelSphereModelRadius" => pub lighting_channel_sphere_model_radius: f32,
        "LightingChannelSphereModelZOffset" => pub lighting_channel_sphere_model_z_offset: f32,
    }
}
