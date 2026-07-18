use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CAppearanceModifierDef` — C++ `CAppearanceModifierDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct AppearanceModifierDef {
        "HideMaterials" => pub hide_materials: Vec<String>,
        "Graphics" => pub graphics: AppearanceModifierGraphics,
        "CoversBodyAreaFlags" => pub covers_body_area_flags: i32,
        "IsRemoveable" => pub is_removeable: bool = true,
        "HeroSuit" => pub hero_suit: i32,
        "SuitPart" => pub suit_part: ClothingSuitPart,
        "AppearanceType" => pub appearance_type: HeroAttachableAppearanceModifierType,
        "Scariness" => pub scariness: f32,
        "Attractiveness" => pub attractiveness: f32,
        "Goodstrength" => pub goodstrength: f32,
        "SoundRadiusMultiplier" => pub sound_radius_multiplier: f32 = 1.0,
        "VisibilityMultiplier" => pub visibility_multiplier: f32 = 1.0,
        "ArmourDefIndex" => pub armour_def_index: i32 = -1,
        "OveralProtectionWeighting" => pub overal_protection_weighting: f32,
    }
}
