use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CActivateQuestDef` — C++ `CActivateQuestDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ActivateQuestDef {
        "ScriptName" => pub script_name: DefString,
        "LoadResources" => pub load_resources: bool = true,
    }
}
