use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `MELEE_COMBAT_KNOCKDOWN_EFFECTS` — C++ `CMeleeCombatKnockdownEffects`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct MeleeCombatKnockdownEffects {
        "BlendInFrames" => pub blend_in_frames: i32,
        "BlendOutFrames" => pub blend_out_frames: i32,
        "AttackerLightingChannel" => pub attacker_lighting_channel: i32,
        "TargetLightingChannel" => pub target_lighting_channel: i32,
        "EnvironmentTheme" => pub environment_theme: i32,
        "CameraFOVChange" => pub camera_fov_change: bool,
        "CameraPan" => pub camera_pan: bool,
        "AttackerAndTargetSpeedMultiplier" => pub attacker_and_target_speed_multiplier: f32,
        "PauseEntities" => pub pause_entities: bool,
        "NonAttackingEntities" => pub non_attacking_entities: bool,
        "AttackerInvulnerable" => pub attacker_invulnerable: bool,
    }
}
