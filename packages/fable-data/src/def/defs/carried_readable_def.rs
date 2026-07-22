use crate::DefStruct;
use crate::def::prelude::*;

/// `CCarriedReadableDef` — C++ `CCarriedReadableDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct CarriedReadableDef {
    #[def("TitleTextTag")]
    pub title_text_tag: DefString,
    #[def("BodyTextTag")]
    pub body_text_tag: DefString,
}
