use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ActivateQuestDef {
    #[def("ScriptName")]
    pub script_name: DefString,
    #[def("LoadResources", default = true)]
    pub load_resources: bool,
}
