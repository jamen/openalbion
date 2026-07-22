use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct BlurredAttitudesDef {
    #[def("Attitudes")]
    pub attitudes: Vec<OpinionAttitudeType>,
}
