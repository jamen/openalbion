use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CTavernGameCardBaseDef` — C++ `CTavernGameCardBaseDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TavernGameCardBaseDef {
        "ControlType" => pub control_type: TavernGameControlType,
        "CardWidth" => pub card_width: f32,
        "CardHeight" => pub card_height: f32,
        "ImpulseScaleXbox" => pub impulse_scale_xbox: f32,
        "ImpulseScalePC" => pub impulse_scale_pc: f32,
        "PointerModel" => pub pointer_model: DefIndex,
        "MoveSpeed" => pub move_speed: f32,
        "TurnSpeed" => pub turn_speed: f32,
        "PointerHeightOffset" => pub pointer_height_offset: f32,
        "PointerParticleEffect" => pub pointer_particle_effect: DefIndex,
        "HighlightWidth" => pub highlight_width: f32,
        "HighlightColourGood" => pub highlight_colour_good: RGBColour,
        "HighlightColourBad" => pub highlight_colour_bad: RGBColour,
        "CardSeparation" => pub card_separation: f32,
        "TableSeparation" => pub table_separation: f32,
        "PackOffset" => pub pack_offset: Vector2D,
        "CardOrders" => pub card_orders: Vec<CardPositionsDef>,
        "CardPack" => pub card_pack: Vec<CardDef>,
    }
}
