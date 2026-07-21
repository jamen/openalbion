use crate::def_struct;

def_struct! {
    /// `CGiftDef` — C++ `CGiftDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct GiftDef {
        "GiftType" => pub gift_type: i32,
        "IsWeddingRing" => pub is_wedding_ring: bool,
    }
}
