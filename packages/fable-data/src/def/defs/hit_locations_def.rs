use crate::def_struct;

def_struct! {
    /// `CHitLocationsDef` — C++ `CHitLocationsDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct HitLocationsDef {
        "HitLocations" => pub hit_locations: Vec<i32>,
    }
}
