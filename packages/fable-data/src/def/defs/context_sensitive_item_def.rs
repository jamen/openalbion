use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CContextSensitiveItemDef` — C++ `CContextSensitiveItemDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ContextSensitiveItemDef {
        "Type" => pub type_: ContextSensitiveType,
        "Priority" => pub priority: i32,
        "SlotIndex" => pub slot_index: i32,
        "ExtraData1" => pub extra_data1: i32,
        "ExtraData2" => pub extra_data2: i32,
    }
}
