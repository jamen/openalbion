use crate::DefStruct;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct HairCardDef {
    #[def("HairObject")]
    pub hair_object: i32,
}
