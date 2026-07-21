//! Def-type dispatch: maps a wire name to the concrete Rust def type.
//! Single-format (OG retail only).

use crate::def::prelude::*;
use crate::def::binary::control::{ParseControlError, SerializeControlError};
use crate::def::binary::def_binary::DefBody;

// ── GameBody enum + dispatch ──────────────────────────────────────────────────
//
// The `GameBody` enum and its four dispatch matches (`serialize`, `byte_size`,
// `visit_active`, and `parse_game_def`) are generated from the single canonical
// table in `for_each_game_def!` (see `game_def_table.rs`). Each def is listed
// there exactly once; this callback expands it into all five constructs.

macro_rules! def_gamebody {
    ($($variant:ident => [$($name:literal),+ $(,)?]),+ $(,)?) => {
        /// A parsed game-def body (the active variant selects the def type).
        #[derive(Debug, Clone)]
        pub enum GameBody {
            $( $variant($variant), )+
        }

        impl GameBody {
            pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
                match self {
                    $( Self::$variant(d) => $variant::serialize(d, out), )+
                }
            }

            pub fn byte_size(&self) -> usize {
                match self {
                    $( Self::$variant(d) => $variant::byte_size(d), )+
                }
            }

            /// Visit the active variant's fields via reflection (drives the
            /// semantic differ / SemVal decoder in `semantic.rs`).
            pub fn visit_active(&mut self, visitor: &mut dyn crate::def::visit::FieldVisitor) {
                use crate::def::visit::VisitFields as _;
                let mut visitor: &mut dyn crate::def::visit::FieldVisitor = visitor;
                match self {
                    $( Self::$variant(d) => $variant::visit_fields(d, &mut visitor), )+
                }
            }
        }

        /// Parse a game def body by type name. Returns `Ok(None)` when `name`
        /// isn't a known game def type (callers fall back to raw bytes); returns
        /// `Err` when the type is known but its body doesn't match the modeled
        /// layout — callers may then retry past an instance prefix rather than
        /// losing data.
        pub fn parse_game_def(
            name: &str,
            cur: &mut &[u8],
        ) -> Result<Option<DefBody>, ParseControlError> {
            Ok(Some(match name {
                $( $( $name => DefBody::Game(GameBody::$variant($variant::parse(cur)?)), )+ )+
                _ => return Ok(None),
            }))
        }
    };
}

crate::for_each_game_def!(def_gamebody);

// ── Sub-def table ──────────────────────────────────────
/// Whether entries of the named def type carry a sub-def table (`u16` count +
/// 12-byte records) between the entry preamble and the field controls.
/// Presence is a per-type property: these are the def classes deriving from
/// the sub-def bases (`CSubDefClassBase`/`CParentDefClassBase`), verified
/// against all three retail bins.
pub fn def_name_has_subdef_table(name: &str) -> bool {
    matches!(
        name,
        "ARMOUR"
            | "ATTACK_PATTERN"
            | "BRAIN"
            | "BUILDING"
            | "CAMERA_MANAGER"
            | "CAMERA_MANAGER_SET"
            | "CAMERA_MODE"
            | "CAreaOfEffectAttackDef"
            | "CBalverineBattleDef"
            | "CHeroPostcardGeneratorDef"
            | "CIdleSchedulerDef"
            | "CJackOfBladesBattleDef"
            | "CMazeBattleDef"
            | "COMBAT_DIALOGUE_DEF"
            | "COMBAT_SEQUENCE"
            | "COMBAT_TYPE"
            | "CONFIG_OPTIONS_DEFAULTS_DEF"
            | "CONTROL_SCHEME"
            | "CREATURE"
            | "CREATURE_ABILITY"
            | "CREATURE_GENERATION_FAMILY"
            | "CScorpionKingBattleDef"
            | "CScriptDef"
            | "CCutsceneDef"
            | "CRegionScriptDef"
            | "CThunderBattleDef"
            | "CTrollBattleDef"
            | "CWaspQueenBattleDef"
            | "CWhisperBattleDef"
            | "ENGINE"
            | "ENGINE_THEME"
            | "ENGINE_THEME_GROUP"
            | "ENGINE_VIDEO_OPTIONS"
            | "ENVIRONMENT"
            | "ENVIRONMENT_THEME_DAY"
            | "EXPRESSION"
            | "FACTION"
            | "FRONT_END"
            | "GLOBAL"
            | "HERO_ABILITY"
            | "HERO_COMBAT"
            | "HERO_MELEE_COMBAT_ABILITY"
            | "HERO_STATS"
            | "HIT_LOCATION"
            | "HOLY_SITE"
            | "INVENTORY_CATEGORY"
            | "INVENTORY_TYPE"
            | "LIGHTNING"
            | "LOCAL_DETAIL_GENERATOR"
            | "MARKER"
            | "MATERIAL"
            | "MELEE_COMBAT_KNOCKDOWN_EFFECTS"
            | "MESSAGE_EVENT"
            | "NOISE"
            | "OBJECT"
            | "OBJECT_FAMILY"
            | "OPINION_DEED_EFFECTS"
            | "OPINION_DEED_MASK"
            | "OPINION_PERSONALITY"
            | "OPINION_REACTION_MANAGER"
            | "OPINION_REACTION_MASK"
            | "OPINION_SOURCE"
            | "PHYSICAL_SWITCH"
            | "PLAYER"
            | "PLAYER_GUI"
            | "PLAYER_INVENTORY"
            | "PLAYER_MOVEMENT"
            | "REGION"
            | "SHOT"
            | "SIM_BUILDING"
            | "SIM_VOICES"
            | "SKY"
            | "SOUND_SETUP"
            | "SOUND_THEME"
            | "SPECIAL_ABILITIES_ASSASSIN_RUSH_DEF"
            | "SPECIAL_ABILITIES_BATTLE_CHARGE_DEF"
            | "SPECIAL_ABILITIES_BERSERK_DEF"
            | "SPECIAL_ABILITIES_BULLET_TIME_DEF"
            | "SPECIAL_ABILITIES_BURNT_EFFECT_DEF"
            | "SPECIAL_ABILITIES_CREATURE_TINT_DEF"
            | "SPECIAL_ABILITIES_DIVINE_WRATH_DEF"
            | "SPECIAL_ABILITIES_DRAIN_LIFE_DEF"
            | "SPECIAL_ABILITIES_DRUNKENNESS_DEF"
            | "SPECIAL_ABILITIES_ELECTROCUTED_EFFECT_DEF"
            | "SPECIAL_ABILITIES_ENFLAME_DEF"
            | "SPECIAL_ABILITIES_FIREBALL_SPELL_DEF"
            | "SPECIAL_ABILITIES_FORCE_PUSH_DEF"
            | "SPECIAL_ABILITIES_GHOST_SWORD_DEF"
            | "SPECIAL_ABILITIES_HEAL_LIFE_DEF"
            | "SPECIAL_ABILITIES_LIGHTNING_SPELL_DEF"
            | "SPECIAL_ABILITIES_MULTI_ARROW_DEF"
            | "SPECIAL_ABILITIES_MULTI_STRIKE_DEF"
            | "SPECIAL_ABILITIES_PHYSICAL_SHIELD_DEF"
            | "SPECIAL_ABILITIES_SUMMON_SPELL_DEF"
            | "SPECIAL_ABILITIES_THUNDER_LIGHTNING_STORM_DEF"
            | "SPECIAL_ABILITIES_TURNCOAT_SPELL_DEF"
            | "SPECIAL_ABILITIES_UNHOLY_POWER_DEF"
            | "SWITCH"
            | "THING"
            | "THING_GROUP"
            | "UI"
            | "UI_ICONS_DEF"
            | "UI_LOCALE_GRAPHICS_DEF"
            | "UI_MISC_THINGS_DEF"
            | "VILLAGE"
            | "VILLAGER_INTERACTION"
    )
}


pub fn parse_script_def(
    name: &str,
    cur: &mut &[u8],
) -> Result<DefBody, ParseControlError> {
    Ok(match parse_game_def(name, cur)? {
        Some(body) => body,
        None => DefBody::Unknown {
            name: name.to_string(),
            bytes: core::mem::take(cur).to_vec(),
        },
    })
}
