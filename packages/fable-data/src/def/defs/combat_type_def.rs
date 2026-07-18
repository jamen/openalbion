use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `COMBAT_TYPE` — C++ `CCombatTypeDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CombatTypeDef {
        "DefenderCombatRingOccupationNumbers" => pub defender_combat_ring_occupation_numbers: Vec<i32>,
        "DefenderWheelOuterRingStartIndex" => pub defender_wheel_outer_ring_start_index: i32,
        "DefenderWheelAllowAttackerStrafing" => pub defender_wheel_allow_attacker_strafing: bool,
        "CombatSequence" => pub combat_sequence: Vec<String>,
        "CombatSequenceDef" => pub combat_sequence_def: Vec<i32>,
        "PreferOuterRing" => pub prefer_outer_ring: bool,
        "FrontFarLimit" => pub front_far_limit: f32,
        "FrontMiddleLimit" => pub front_middle_limit: f32,
        "FrontNearLimit" => pub front_near_limit: f32,
        "SideLimit" => pub side_limit: f32,
        "RearLimit" => pub rear_limit: f32,
        "PreferredCombatDistance" => pub preferred_combat_distance: f32,
        // `std::map<ECombatCreatureType, long>` — key-sorted (BTreeMap), value
        // is an ATTACK_PATTERN def index (verified against retail:
        // HOBBE_ATTACK_STYLE_BASE stores keys 0,1 → 417,418 =
        // ATTACK_PATTERN_MEDIUM/EASY). The decomp template's `EIdleStateGroup`
        // ValueT is a misattribution; the slot holds a `long` def index.
        "AttackPatterns" => pub attack_patterns: BTreeMap<i32, DefIndex>,
        "CombatCreatureType" => pub combat_creature_type: CombatCreatureType,
        "AlwaysAttackIfNearest" => pub always_attack_if_nearest: bool,
        "KnockdownCausesStun" => pub knockdown_causes_stun: bool,
        "RecoilStyle" => pub recoil_style: CombatStrikeRecoilStyle,
        "CanBeKnockedDown" => pub can_be_knocked_down: bool = true,
        "ComboContinueSpeed" => pub combo_continue_speed: f32,
        "CombatFOV" => pub combat_fov: f32 = 36.0,
        "StartsAsReadyForMelee" => pub starts_as_ready_for_melee: bool = true,
    }
}
