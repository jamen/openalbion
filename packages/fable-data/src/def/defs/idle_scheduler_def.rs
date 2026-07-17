use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CIdleSchedulerDef` — C++ `CIdleSchedulerDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct IdleSchedulerDef {
        "Period" => pub period: VecMap<IdleStateGroup, i32>,
    }
}
