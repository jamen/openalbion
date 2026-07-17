use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CObjectAugmentationsDef` — C++ `CObjectAugmentationsDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ObjectAugmentationsDef {
        "NumberOfSlots" => pub number_of_slots: i32,
        "MaximumNumberOfSlots" => pub maximum_number_of_slots: i32,
        "AugmentationParticles" => pub augmentation_particles: VecMap<ObjectAugmentationType, ObjectAugmentationParticleSet>,
        "EmptySlotName" => pub empty_slot_name: u32,
        "InitialAugmentations" => pub initial_augmentations: Vec<i32>,
        "SlotUpgradeCosts" => pub slot_upgrade_costs: Vec<i32>,
        "InitSounds" => pub init_sounds: VecMap<ObjectAugmentationType, DefString>,
        "LoopingSounds" => pub looping_sounds: VecMap<ObjectAugmentationType, DefString>,
        "FireProjectileTrailEffect" => pub fire_projectile_trail_effect: i32,
        "LightningProjectileTrailEffect" => pub lightning_projectile_trail_effect: i32,
        "FireAndLightningProjectileTrailEffect" => pub fire_and_lightning_projectile_trail_effect: i32,
        "FireProjectileOnHitEffect" => pub fire_projectile_on_hit_effect: i32,
        "LightningProjectileOnHitEffect" => pub lightning_projectile_on_hit_effect: i32,
        "FireAndLightningProjectileOnHitEffect" => pub fire_and_lightning_projectile_on_hit_effect: i32,
        "WeaponTrailAttachmentPoint" => pub weapon_trail_attachment_point: String,
    }
}
