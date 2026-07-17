use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CAugmentationDef` — C++ `CAugmentationDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct AugmentationDef {
        "Name" => pub name: u32,
        "Type" => pub type_: ObjectAugmentationType,
        "InventoryGraphic" => pub inventory_graphic: u32,
        "DamageMultiplier" => pub damage_multiplier: f32,
        "ExperienceMultiplier" => pub experience_multiplier: f32,
        "FrequencyToAddHealthInSeconds" => pub frequency_to_add_health_in_seconds: f32,
        "HealthToAddPerIncrease" => pub health_to_add_per_increase: i32,
        "FrequencyToAddStaminaInSeconds" => pub frequency_to_add_stamina_in_seconds: f32,
        "StaminaToAddPerIncrease" => pub stamina_to_add_per_increase: i32,
    }
}
