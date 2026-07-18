use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCreatureNavigationDef` — C++ `CCreatureNavigationDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CreatureNavigationDef {
        "MinPathCost" => pub min_path_cost: f32,
        "MaxPathCost" => pub max_path_cost: f32,
        "Radius" => pub radius: f32,
        "AvoidDynamicObstacles" => pub avoid_dynamic_obstacles: bool = true,
    }
}
