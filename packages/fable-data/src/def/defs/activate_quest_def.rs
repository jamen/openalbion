use crate::DefStruct;
use crate::def::prelude::*;

/// `CActivateQuestDef` — C++ `CActivateQuestDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ActivateQuestDef {
    #[def("ScriptName")]
    pub script_name: DefString,
    #[def("LoadResources", default = true)]
    pub load_resources: bool,
}
