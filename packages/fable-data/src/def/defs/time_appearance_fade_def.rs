use crate::def_struct;

def_struct! {
    /// `CTimeAppearanceFadeDef` — C++ `CTimeAppearanceFadeDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TimeAppearanceFadeDef {
        "FadeInStart" => pub fade_in_start: f32,
        "FadeInEnd" => pub fade_in_end: f32,
        "FadeOutStart" => pub fade_out_start: f32,
        "FadeOutEnd" => pub fade_out_end: f32,
    }
}
