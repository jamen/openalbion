use crate::DefStruct;
use crate::def::prelude::*;

/// C++ `CBlurredAttitudesDef` (sub-component def).
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct BlurredAttitudesDef {
    #[def("Attitudes")]
    pub attitudes: Vec<OpinionAttitudeType>,
}
