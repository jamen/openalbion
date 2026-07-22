use crate::DefStruct;
use crate::def::prelude::*;

/// `CIdleSchedulerDef` — C++ `CIdleSchedulerDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct IdleSchedulerDef {
    #[def("Period")]
    pub period: VecMap<IdleStateGroup, i32>,
}
