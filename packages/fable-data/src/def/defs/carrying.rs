use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct CarryingDef {
    #[def("AvailableCarrySlots")]
    pub available_carry_slots: Vec<i32>,
    #[def("OverriddenDummyObject")]
    pub overridden_dummy_object: DefString,
}
