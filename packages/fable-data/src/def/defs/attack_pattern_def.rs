use crate::DefStruct;

/// `ATTACK_PATTERN` — C++ `CAttackPatternDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct AttackPatternDef {
    #[def("AttackPattern")]
    pub attack_pattern: Vec<i32>,
}
