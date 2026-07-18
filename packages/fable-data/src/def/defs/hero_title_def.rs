use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CHeroTitleDef` — C++ `CHeroTitleDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct HeroTitleDef {
        "Title" => pub title: HeroTitle,
        "HeroTitleGuiTag" => pub hero_title_gui_tag: i32,
        "HeroTitleGreetToHeroTag" => pub hero_title_greet_to_hero_tag: i32,
        "HeroTitleCommentAtHeroTag" => pub hero_title_comment_at_hero_tag: i32,
        "HeroTitleCommentToSelfTag" => pub hero_title_comment_to_self_tag: i32,
        "RequiredRenownLevel" => pub required_renown_level: i32,
        "RequiredStrengthExpLevel" => pub required_strength_exp_level: i32,
        "RequiredSkillExpLevel" => pub required_skill_exp_level: i32,
        "RequiredWillExpLevel" => pub required_will_exp_level: i32,
        "IsBuyable" => pub is_buyable: bool,
        "IsAutomatic" => pub is_automatic: bool = true,
    }
}
