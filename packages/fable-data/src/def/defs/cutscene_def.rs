use crate::def_struct;

def_struct! {
    /// `CCutsceneDef` — C++ `CCutsceneDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CutsceneDef {
        "Macro" => pub macro_: Vec<String>,
        "SkipCond" => pub skip_cond: Vec<String>,
        "SetupCond" => pub setup_cond: Vec<String>,
        "Lights" => pub lights: Vec<String>,
        "LightScene" => pub light_scene: Vec<String>,
        "Sound" => pub sound: Vec<String>,
        "Answer0" => pub answer0: Vec<String>,
        "Answer1" => pub answer1: Vec<String>,
    }
}
