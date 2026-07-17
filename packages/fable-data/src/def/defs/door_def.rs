use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CDoorDef` — C++ `CDoorDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct DoorDef {
        "TriggerType" => pub trigger_type: DoorTriggerType,
        "TriggerRadius" => pub trigger_radius: f32,
        "OpenCollisionMesh" => pub open_collision_mesh: i32,
        "ClosedCollisionMesh" => pub closed_collision_mesh: i32,
        "PostVandalismCrimeIfDestroyed" => pub post_vandalism_crime_if_destroyed: bool,
        "LockPickable" => pub lock_pickable: bool,
        "PreventLockingByOwner" => pub prevent_locking_by_owner: bool,
    }
}
