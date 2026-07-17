use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CAppearanceDef` — C++ `CAppearanceDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct AppearanceDef {
        "Graphic" => pub graphic: EngineGraphic,
        "Animation" => pub animation: AnimationSet,
        "OutlineEffectAlphaRef" => pub outline_effect_alpha_ref: i32,
        "OutlineEffectAlphaBias" => pub outline_effect_alpha_bias: i32,
        "OverrideLightingChannel" => pub override_lighting_channel: bool,
        "LightingChannel" => pub lighting_channel: LightingChannel,
        "FadeOutWhenCloseGraphic" => pub fade_out_when_close_graphic: EngineGraphic,
        "HasFadeOutWhenCloseGraphic" => pub has_fade_out_when_close_graphic: bool,
        "FadeOutGraphicRenderFadeDistance" => pub fade_out_graphic_render_fade_distance: f32,
        "FadeOutGraphicNearRenderFadeDistance" => pub fade_out_graphic_near_render_fade_distance: f32,
        "FadeOutGraphicNearRenderClipDistance" => pub fade_out_graphic_near_render_clip_distance: f32,
    }
}
