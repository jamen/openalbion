use crate::def_struct;

def_struct! {
    /// `SPECIAL_ABILITIES_CREATURE_TINT_DEF` — C++ `CSpecialAbilitiesCreatureTintDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesCreatureTintDef {
        "FadeInTimeSecs" => pub fade_in_time_secs: f32,
        "FadeOutTimeSecs" => pub fade_out_time_secs: f32,
        "Color" => pub color: Vec<i32>,
    }
}
