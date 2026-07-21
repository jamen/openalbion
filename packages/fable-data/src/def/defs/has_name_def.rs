use crate::def_struct;

def_struct! {
    /// `CHasNameDef` — C++ `CHasNameDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct HasNameDef {
        "DefaultNameTag" => pub default_name_tag: u32,
        "Home" => pub home: u32,
        "Occupation" => pub occupation: u32,
    }
}
