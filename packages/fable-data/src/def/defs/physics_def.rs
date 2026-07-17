use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CPhysicsDef` — C++ `CPhysicsDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct PhysicsDef {
        "Diameter" => pub diameter: f32,
        "Friction" => pub friction: f32,
        "Elasticity" => pub elasticity: f32,
        "InteractionFlags" => pub interaction_flags: i32,
        "AirResistance" => pub air_resistance: f32,
        "Mass" => pub mass: f32,
        "PushableByHero" => pub pushable_by_hero: bool,
    }
}
