use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CREATURE_GENERATION_FAMILY` — C++ `CCreatureGenerationFamilyDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CreatureGenerationFamilyDef {
        "Creatures" => pub creatures: Vec<i32>,
        "DifficultyLevel" => pub difficulty_level: i32,
        "GeneratorObject" => pub generator_object: i32,
        "GenerationTypes" => pub generation_types: Vec<CreatureGeneratorGenerateType>,
    }
}
