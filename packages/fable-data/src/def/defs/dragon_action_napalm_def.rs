use crate::def_struct;

def_struct! {
    /// `CDragonActionNapalmDef` — C++ `CDragonActionNapalmDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct DragonActionNapalmDef {
        "ChargeUpTime" => pub charge_up_time: f32,
        "WindSpeed" => pub wind_speed: f32,
        "TimeBeforeAddOfMouthEffect" => pub time_before_add_of_mouth_effect: f32,
        "TimeBeforeAddExplosion" => pub time_before_add_explosion: f32,
    }
}
