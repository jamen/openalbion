use crate::def::wire::DefIndex;
use crate::DefStruct;
use crate::def::wire::DefString;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct CarryingDef {
    #[def("AvailableCarrySlots")]
    pub available_carry_slots: Vec<DefIndex>,
    #[def("OverriddenDummyObject")]
    pub overridden_dummy_object: DefString,
}
