use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `HIT_LOCATION` — C++ `CHitLocationDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct HitLocationDef {
        "PhysicalPrimitive" => pub physical_primitive: PhysicalPrimitiveInit,
        "Armour" => pub armour: i32,
        "HitDamageEffect" => pub hit_damage_effect: i32,
        "HitBlockedEffect" => pub hit_blocked_effect: i32,
        "VulnerableEffect" => pub vulnerable_effect: i32,
        "HitResponseAction" => pub hit_response_action: i32,
        "HitResponseToEpicSpellsAction" => pub hit_response_to_epic_spells_action: i32,
        "HitKnockdownAction" => pub hit_knockdown_action: i32,
        "HitKnockdownDeathAction" => pub hit_knockdown_death_action: i32,
        "BlockResponseAction" => pub block_response_action: i32,
        "Flags" => pub flags: i32,
        "DefendableAgainstShots" => pub defendable_against_shots: bool,
        "DefendableAgainstMelee" => pub defendable_against_melee: bool,
        "Priority" => pub priority: i32,
        "Default" => pub default: bool,
        "DecapitateOnHit" => pub decapitate_on_hit: bool,
        "PlayGetHitResponse" => pub play_get_hit_response: bool,
        "EnableOnCreate" => pub enable_on_create: bool,
        "HitLocationCodelet" => pub hit_location_codelet: i32,
    }
}
