use crate::def_struct;

def_struct! {
    /// `SPECIAL_ABILITIES_DRUNKENNESS_DEF` — C++ `CSpecialAbilitiesDrunkennessDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesDrunkennessDef {
        "DrunkenGraphic" => pub drunken_graphic: i32,
        "MaxAlpha" => pub max_alpha: i32,
        "USpeed" => pub u_speed: f32,
        "VSpeed" => pub v_speed: f32,
        "RotSpeed" => pub rot_speed: f32,
    }
}
