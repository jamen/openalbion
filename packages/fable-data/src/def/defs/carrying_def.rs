use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCarryingDef` — C++ `CCarryingDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CarryingDef {
        "AvailableCarrySlots" => pub available_carry_slots: Vec<i32>,
        "OverriddenDummyObject" => pub overridden_dummy_object: DefString,
    }
}
