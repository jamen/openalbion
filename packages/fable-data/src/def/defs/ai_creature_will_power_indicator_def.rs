use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CAICreatureWillPowerIndicatorDef` — C++ `CAICreatureWillPowerIndicatorDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct AICreatureWillPowerIndicatorDef {
        "RechargeTime" => pub recharge_time: f32 = 5.0,
    }
}
