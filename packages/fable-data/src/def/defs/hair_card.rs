use crate::DefStruct;
use crate::def::wire::DefIndex;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct HairCardDef {
    #[def("HairObject")]
    pub hair_object: DefIndex,
}
