use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CPhysicsDef` — C++ `CPhysicsDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct PhysicsDef {
        "Diameter" => pub diameter: f32,
        "Friction" => pub friction: f32 = 1.0,
        "Elasticity" => pub elasticity: f32,
        "InteractionFlags" => pub interaction_flags: i32 = 22,
        "AirResistance" => pub air_resistance: f32,
        "Mass" => pub mass: f32 = 50.0,
        "PushableByHero" => pub pushable_by_hero: bool = true,
    }
}
