use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `EXPRESSION` — C++ `CExpressionDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ExpressionDef {
        "Animation" => pub animation: DefString,
        "AnimationInto" => pub animation_into: DefString,
        "AnimationLoop" => pub animation_loop: DefString,
        "AnimationOutOf" => pub animation_out_of: DefString,
        "AnimationToOther" => pub animation_to_other: DefString,
        "AnimationToOtherInto" => pub animation_to_other_into: DefString,
        "AnimationToOtherLoop" => pub animation_to_other_loop: DefString,
        "AnimationToOtherOutOf" => pub animation_to_other_out_of: DefString,
        "SpriteDuration" => pub sprite_duration: f32,
        "SpriteGraphic" => pub sprite_graphic: i32,
        "SpriteOverrideText" => pub sprite_override_text: DefString,
        "SoundCriteria" => pub sound_criteria: DefString,
        "LoopSoundCriteria" => pub loop_sound_criteria: DefString,
        "DummyObject" => pub dummy_object: DefIndex,
        "GameEventType" => pub game_event_type: GameEventType,
        "ScriptName" => pub script_name: DefString,
        "ScriptLoadResources" => pub script_load_resources: bool,
        "DeactivateScriptAtEnd" => pub deactivate_script_at_end: bool,
        "PerformAfterScript" => pub perform_after_script: bool,
        "OpinionDeedType" => pub opinion_deed_type: OpinionDeedType,
        "Extendable" => pub extendable: bool,
        "SheatheWeapons" => pub sheathe_weapons: bool,
        "ReplaceHeldItemWhenDone" => pub replace_held_item_when_done: bool,
        "TextTag" => pub text_tag: u32,
        "ExpressionType" => pub expression_type: ExpressionInventoryType,
    }
}
