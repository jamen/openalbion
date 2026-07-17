use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCarriedReadableDef` — C++ `CCarriedReadableDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CarriedReadableDef {
        "TitleTextTag" => pub title_text_tag: DefString,
        "BodyTextTag" => pub body_text_tag: DefString,
    }
}
