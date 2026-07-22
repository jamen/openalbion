use crate::DefStruct;
use crate::def::enums::OpinionAttitudeType;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct BlurredAttitudesDef {
    #[def("Attitudes")]
    pub attitudes: Vec<OpinionAttitudeType>,
}
