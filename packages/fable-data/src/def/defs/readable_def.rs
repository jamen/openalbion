use crate::def_struct;

def_struct! {
    /// `CReadableDef` — C++ `CReadableDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ReadableDef {
        "UseInfoDisplay" => pub use_info_display: bool,
    }
}
