use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CGuardDef` — C++ `CGuardDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct GuardDef {
        "BribeCostWeaponOut" => pub bribe_cost_weapon_out: i32,
        "BribeCostTrespass" => pub bribe_cost_trespass: i32,
        "BribeCostVandalism" => pub bribe_cost_vandalism: i32,
        "BribeCostShoplifter" => pub bribe_cost_shoplifter: i32,
        "BribeCostPickPocket" => pub bribe_cost_pick_pocket: i32,
        "BribeCostAssault" => pub bribe_cost_assault: i32,
        "BribeCostGuardAssault" => pub bribe_cost_guard_assault: i32,
        "BribeCostGBH" => pub bribe_cost_gbh: i32,
        "BribeCostGuardGBH" => pub bribe_cost_guard_gbh: i32,
        "BribeCostMurder" => pub bribe_cost_murder: i32,
    }
}
