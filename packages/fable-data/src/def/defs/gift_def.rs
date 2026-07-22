use crate::DefStruct;

/// `CGiftDef` — C++ `CGiftDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct GiftDef {
    #[def("GiftType")]
    pub gift_type: i32,
    #[def("IsWeddingRing")]
    pub is_wedding_ring: bool,
}
