use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `FRONT_END` — C++ `CFrontEndDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct FrontEndDef {
        "vAttractModeMovie" => pub v_attract_mode_movie: Vec<String>,
        "ErrorMessageBackgroundGraphic" => pub error_message_background_graphic: i32,
        "ButtonABigGraphic" => pub button_a_big_graphic: i32,
        "ButtonBBigGraphic" => pub button_b_big_graphic: i32,
    }
}

