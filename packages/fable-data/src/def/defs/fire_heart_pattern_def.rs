use crate::def_struct;

def_struct! {
    /// C++ `CFireHeartPatternDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct FireHeartPatternDef {
        "PadActive" => pub pad_active: Vec<bool>,
    }
}
