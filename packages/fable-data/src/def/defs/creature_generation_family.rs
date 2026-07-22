use crate::DefStruct;
use crate::def::enums::CreatureGeneratorGenerateType;
use crate::def::wire::DefIndex;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct CreatureGenerationFamilyDef {
    #[def("Creatures")]
    pub creatures: Vec<DefIndex>,
    #[def("DifficultyLevel")]
    pub difficulty_level: i32,
    #[def("GeneratorObject")]
    pub generator_object: DefIndex,
    #[def("GenerationTypes")]
    pub generation_types: Vec<CreatureGeneratorGenerateType>,
}
