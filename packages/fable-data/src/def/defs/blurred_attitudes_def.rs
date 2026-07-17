use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CBlurredAttitudesDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct BlurredAttitudesDef {
        "Attitudes" => pub attitudes: Vec<OpinionAttitudeType>,
    }
}
