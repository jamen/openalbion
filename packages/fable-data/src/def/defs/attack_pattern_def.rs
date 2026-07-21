use crate::def_struct;

def_struct! {
    /// `ATTACK_PATTERN` — C++ `CAttackPatternDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct AttackPatternDef {
        "AttackPattern" => pub attack_pattern: Vec<i32>,
    }
}
