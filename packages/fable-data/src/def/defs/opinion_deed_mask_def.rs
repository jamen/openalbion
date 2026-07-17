use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `OPINION_DEED_MASK` — C++ `COpinionDeedMaskDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct OpinionDeedMaskDef {
        "DeedEnabledDefault" => pub deed_enabled_default: bool,
        "DeedEnabled" => pub deed_enabled: VecMap<OpinionDeedType, bool>,
    }
}
