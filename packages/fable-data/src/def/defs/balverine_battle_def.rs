use crate::def_struct;

def_struct! {
    /// `CBalverineBattleDef` — C++ `CBalverineBattleDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct BalverineBattleDef {
        "SecondsBetweenLunges" => pub seconds_between_lunges: f32 = 7.0,
        "LungeAttackDamage" => pub lunge_attack_damage: f32 = 2.0,
        "TimeInTrees" => pub time_in_trees: f32 = 5.0,
    }
}
