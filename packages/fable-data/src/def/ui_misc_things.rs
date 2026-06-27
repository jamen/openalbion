use super::binary::control::{
    ID_BYTE_SIZE, ParseControlError, ParseControlErrorReason, SerializeControlError,
    SerializeControlErrorReason, parse_id, parse_scalar, parse_string, parse_wstr, serialize_id,
    serialize_scalar, serialize_string, serialize_wstr, string_control_byte_size,
    wstr_control_byte_size,
};
use crate::bytes::{put_le, put_null_terminated_utf8, take_le, take_null_terminated_utf8};

#[derive(Debug)]
pub struct UiMiscThingsDef {
    pub space_separator: String,
    pub comma_separator: String,
    pub new_line_separator: String,
    pub open_bracket: String,
    pub close_bracket: String,
    pub positive: String,
    pub weapon_value_string: String,
    pub weapon_aug_string: String,
    pub weapon_aug_none: String,
    pub weapon_weight_string: String,
    pub weapon_light_string: String,
    pub weapon_heavy_string: String,
    pub weapon_kills_string: String,
    pub weapon_cat_melee_string: String,
    pub weapon_cat_ranged_string: String,
    pub weapon_damage_string: String,
    pub trade_cost_string: String,
    pub colon_separator: String,
    pub trade_profit_string: String,
    pub trade_loss_string: String,
    pub trade_already_owns_string: String,
    pub trade_number_in_stock_string: String,
    pub trade_delivery_string: String,
    pub trade_no_delivery_string: String,
    pub trade_days_string: String,
    pub trade_buy_string: String,
    pub trade_sell_string: String,
    pub trade_wanted_string: String,
    pub quest_failed_string: String,
    pub failed_string: String,
    pub succeeded_string: String,
    pub plus: String,
    pub minus: String,
    pub core_graphic: u32,
    pub vignette_graphic: u32,
    pub optional_graphic: u32,
    pub feat_graphic: u32,
    pub objects_reward_string: String,
    pub none_string: String,
    pub check_guild_string: String,
    pub quest_starting_string: String,
    pub ring_center_x: f32,
    pub ring_center_y: f32,
    pub pc_ring_center_x: f32,
    pub pc_ring_center_y: f32,
    pub world_map_offset_x: f32,
    pub world_map_offset_y: f32,
    pub world_map_width: f32,
    pub world_map_height: f32,
    pub you_string: String,
    pub own_string: String,
    pub no_string: String,
    pub houses_string: String,
    pub house_string: String,
    pub in_string: String,
    pub shops_string: String,
    pub shop_string: String,
    pub there_string: String,
    pub are_string: String,
    pub is_string: String,
    pub for_string: String,
    pub sale_string: String,
    pub general_string: String,
    pub tatoo_string: String,
    pub barber_string: String,
    pub title_string: String,
    pub level_string: String,
    pub total_spells_in_palettes: u32,
    pub total_spells_in_container: u32,
    pub total_assignable_things: u32,
    pub log_book_basics_category_string: String,
    pub log_book_objects_category_string: String,
    pub log_book_towns_category_string: String,
    pub log_book_hero_category_string: String,
    pub log_book_combat_category_string: String,
    pub log_book_quest_category_string: String,
    pub log_book_story_category_string: String,
    pub log_book_basics_category_name_string: String,
    pub log_book_objects_category_name_string: String,
    pub log_book_towns_category_name_string: String,
    pub log_book_hero_category_name_string: String,
    pub log_book_combat_category_name_string: String,
    pub log_book_quest_category_name_string: String,
    pub log_book_story_category_name_string: String,
    pub map_paths: MapPaths,
    pub sound_up_down: String,
    pub sound_slider: String,
    pub sound_back: String,
    pub sound_forward: String,
    pub sound_error: String,
    pub sound_exit: String,
    pub hero_doll_tlx: f32,
    pub hero_doll_tly: f32,
    pub hero_doll_brx: f32,
    pub hero_doll_bry: f32,
    pub hero_doll_sphere_radius: f32,
    pub hero_doll_tlx_pc: f32,
    pub hero_doll_tly_pc: f32,
    pub hero_doll_brx_pc: f32,
    pub hero_doll_bry_pc: f32,
    pub hero_doll_sphere_radius_pc: f32,
    pub hero_doll_frame_tlx_pc: f32,
    pub hero_doll_frame_tly_pc: f32,
    pub hero_doll_frame_emulate_list_offset: f32,
    pub quest_start_screen_music: u32,
    pub quest_complete_screen_music: u32,
    pub quest_failure_screen_music: u32,
    pub death_screen_music: u32,
    pub count_up_sound: String,
    pub digit_count_time: f32,
    pub save_hero_graphic_index: u32,
    pub mini_map_graphics: MiniMapGraphics,
    pub sound_keyboard_up: String,
    pub sound_keyboard_down: String,
    pub sound_keyboard_left: String,
    pub sound_keyboard_right: String,
    pub sound_keyboard_enter_character: String,
    pub sound_keyboard_delete_character: String,
    pub sound_keyboard_done: String,
    pub front_end_music: String,
    pub keyboard_small_key_graphic: u32,
    pub keyboard_large_key_graphic: u32,
    pub time_in_secs_for_fade: f32,
    pub back_buffer_filter_saturation: f32,
    pub back_buffer_filter_contrast: f32,
    pub back_buffer_filter_brightness: f32,
    pub back_buffer_filter_tint_r: f32,
    pub back_buffer_filter_tint_g: f32,
    pub back_buffer_filter_tint_b: f32,
    pub back_buffer_filter_tint_scale: f32,
    pub back_buffer_diffuse_scale: f32,
    pub back_buffer_ambient_scale: f32,
    pub minimum_filter_color: f32,
}

impl UiMiscThingsDef {
    pub(crate) fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        Ok(Self {
            space_separator: parse_wstr(cur, "SpaceSeparator")?,
            comma_separator: parse_wstr(cur, "CommaSeparator")?,
            new_line_separator: parse_wstr(cur, "NewLineSeparator")?,
            open_bracket: parse_wstr(cur, "OpenBracket")?,
            close_bracket: parse_wstr(cur, "CloseBracket")?,
            positive: parse_wstr(cur, "Positive")?,
            weapon_value_string: parse_wstr(cur, "WeaponValueString")?,
            weapon_aug_string: parse_wstr(cur, "WeaponAugString")?,
            weapon_aug_none: parse_wstr(cur, "WeaponAugNone")?,
            weapon_weight_string: parse_wstr(cur, "WeaponWeightString")?,
            weapon_light_string: parse_wstr(cur, "WeaponLightString")?,
            weapon_heavy_string: parse_wstr(cur, "WeaponHeavyString")?,
            weapon_kills_string: parse_wstr(cur, "WeaponKillsString")?,
            weapon_cat_melee_string: parse_wstr(cur, "WeaponCatMeleeString")?,
            weapon_cat_ranged_string: parse_wstr(cur, "WeaponCatRangedString")?,
            weapon_damage_string: parse_wstr(cur, "WeaponDamageString")?,
            trade_cost_string: parse_wstr(cur, "TradeCostString")?,
            colon_separator: parse_wstr(cur, "ColonSeparator")?,
            trade_profit_string: parse_wstr(cur, "TradeProfitString")?,
            trade_loss_string: parse_wstr(cur, "TradeLossString")?,
            trade_already_owns_string: parse_wstr(cur, "TradeAlreadyOwnsString")?,
            trade_number_in_stock_string: parse_wstr(cur, "TradeNumberInStockString")?,
            trade_delivery_string: parse_wstr(cur, "TradeDeliveryString")?,
            trade_no_delivery_string: parse_wstr(cur, "TradeNoDeliveryString")?,
            trade_days_string: parse_wstr(cur, "TradeDaysString")?,
            trade_buy_string: parse_wstr(cur, "TradeBuyString")?,
            trade_sell_string: parse_wstr(cur, "TradeSellString")?,
            trade_wanted_string: parse_wstr(cur, "TradeWantedString")?,
            quest_failed_string: parse_wstr(cur, "QuestFailedString")?,
            failed_string: parse_wstr(cur, "FailedString")?,
            succeeded_string: parse_wstr(cur, "SucceededString")?,
            plus: parse_wstr(cur, "Plus")?,
            minus: parse_wstr(cur, "Minus")?,
            core_graphic: parse_scalar::<u32>(cur, "CoreGraphic")?,
            vignette_graphic: parse_scalar::<u32>(cur, "VignetteGraphic")?,
            optional_graphic: parse_scalar::<u32>(cur, "OptionalGraphic")?,
            feat_graphic: parse_scalar::<u32>(cur, "FeatGraphic")?,
            objects_reward_string: parse_wstr(cur, "ObjectsRewardString")?,
            none_string: parse_wstr(cur, "NoneString")?,
            check_guild_string: parse_wstr(cur, "CheckGuildString")?,
            quest_starting_string: parse_wstr(cur, "QuestStartingString")?,
            ring_center_x: parse_scalar::<f32>(cur, "RingCenterX")?,
            ring_center_y: parse_scalar::<f32>(cur, "RingCenterY")?,
            pc_ring_center_x: parse_scalar::<f32>(cur, "PCRingCenterX")?,
            pc_ring_center_y: parse_scalar::<f32>(cur, "PCRingCenterY")?,
            world_map_offset_x: parse_scalar::<f32>(cur, "WorldMapOffsetX")?,
            world_map_offset_y: parse_scalar::<f32>(cur, "WorldMapOffsetY")?,
            world_map_width: parse_scalar::<f32>(cur, "WorldMapWidth")?,
            world_map_height: parse_scalar::<f32>(cur, "WorldMapHeight")?,
            you_string: parse_wstr(cur, "YouString")?,
            own_string: parse_wstr(cur, "OwnString")?,
            no_string: parse_wstr(cur, "NoString")?,
            houses_string: parse_wstr(cur, "HousesString")?,
            house_string: parse_wstr(cur, "HouseString")?,
            in_string: parse_wstr(cur, "InString")?,
            shops_string: parse_wstr(cur, "ShopsString")?,
            shop_string: parse_wstr(cur, "ShopString")?,
            there_string: parse_wstr(cur, "ThereString")?,
            are_string: parse_wstr(cur, "AreString")?,
            is_string: parse_wstr(cur, "IsString")?,
            for_string: parse_wstr(cur, "ForString")?,
            sale_string: parse_wstr(cur, "SaleString")?,
            general_string: parse_wstr(cur, "GeneralString")?,
            tatoo_string: parse_wstr(cur, "TatooString")?,
            barber_string: parse_wstr(cur, "BarberString")?,
            title_string: parse_wstr(cur, "TitleString")?,
            level_string: parse_wstr(cur, "LevelString")?,
            total_spells_in_palettes: parse_scalar::<u32>(cur, "TotalSpellsInPalettes")?,
            total_spells_in_container: parse_scalar::<u32>(cur, "TotalSpellsInContainer")?,
            total_assignable_things: parse_scalar::<u32>(cur, "TotalAssignableThings")?,
            log_book_basics_category_string: parse_wstr(cur, "LogBookBasicsCategoryString")?,
            log_book_objects_category_string: parse_wstr(cur, "LogBookObjectsCategoryString")?,
            log_book_towns_category_string: parse_wstr(cur, "LogBookTownsCategoryString")?,
            log_book_hero_category_string: parse_wstr(cur, "LogBookHeroCategoryString")?,
            log_book_combat_category_string: parse_wstr(cur, "LogBookCombatCategoryString")?,
            log_book_quest_category_string: parse_wstr(cur, "LogBookQuestCategoryString")?,
            log_book_story_category_string: parse_wstr(cur, "LogBookStoryCategoryString")?,
            log_book_basics_category_name_string: parse_wstr(cur, "LogBookBasicsCategoryNameString")?,
            log_book_objects_category_name_string: parse_wstr(cur, "LogBookObjectsCategoryNameString")?,
            log_book_towns_category_name_string: parse_wstr(cur, "LogBookTownsCategoryNameString")?,
            log_book_hero_category_name_string: parse_wstr(cur, "LogBookHeroCategoryNameString")?,
            log_book_combat_category_name_string: parse_wstr(cur, "LogBookCombatCategoryNameString")?,
            log_book_quest_category_name_string: parse_wstr(cur, "LogBookQuestCategoryNameString")?,
            log_book_story_category_name_string: parse_wstr(cur, "LogBookStoryCategoryNameString")?,
            map_paths: MapPaths::parse(cur)?,
            sound_up_down: parse_string(cur, "SoundUpDown")?.to_string(),
            sound_slider: parse_string(cur, "SoundSlider")?.to_string(),
            sound_back: parse_string(cur, "SoundBack")?.to_string(),
            sound_forward: parse_string(cur, "SoundForward")?.to_string(),
            sound_error: parse_string(cur, "SoundError")?.to_string(),
            sound_exit: parse_string(cur, "SoundExit")?.to_string(),
            hero_doll_tlx: parse_scalar::<f32>(cur, "HeroDollTLX")?,
            hero_doll_tly: parse_scalar::<f32>(cur, "HeroDollTLY")?,
            hero_doll_brx: parse_scalar::<f32>(cur, "HeroDollBRX")?,
            hero_doll_bry: parse_scalar::<f32>(cur, "HeroDollBRY")?,
            hero_doll_sphere_radius: parse_scalar::<f32>(cur, "HeroDollSphereRadius")?,
            hero_doll_tlx_pc: parse_scalar::<f32>(cur, "HeroDollTLX_PC")?,
            hero_doll_tly_pc: parse_scalar::<f32>(cur, "HeroDollTLY_PC")?,
            hero_doll_brx_pc: parse_scalar::<f32>(cur, "HeroDollBRX_PC")?,
            hero_doll_bry_pc: parse_scalar::<f32>(cur, "HeroDollBRY_PC")?,
            hero_doll_sphere_radius_pc: parse_scalar::<f32>(cur, "HeroDollSphereRadius_PC")?,
            hero_doll_frame_tlx_pc: parse_scalar::<f32>(cur, "HeroDollFrameTLX_PC")?,
            hero_doll_frame_tly_pc: parse_scalar::<f32>(cur, "HeroDollFrameTLY_PC")?,
            hero_doll_frame_emulate_list_offset: parse_scalar::<f32>(cur, "HeroDollFrameEmulateListOffset")?,
            quest_start_screen_music: parse_scalar::<u32>(cur, "QuestStartScreenMusic")?,
            quest_complete_screen_music: parse_scalar::<u32>(cur, "QuestCompleteScreenMusic")?,
            quest_failure_screen_music: parse_scalar::<u32>(cur, "QuestFailureScreenMusic")?,
            death_screen_music: parse_scalar::<u32>(cur, "DeathScreenMusic")?,
            count_up_sound: parse_string(cur, "CountUpSound")?.to_string(),
            digit_count_time: parse_scalar::<f32>(cur, "DigitCountTime")?,
            save_hero_graphic_index: parse_scalar::<u32>(cur, "SaveHeroGraphicIndex")?,
            mini_map_graphics: MiniMapGraphics::parse(cur)?,
            sound_keyboard_up: parse_string(cur, "SoundKeyboardUp")?.to_string(),
            sound_keyboard_down: parse_string(cur, "SoundKeyboardDown")?.to_string(),
            sound_keyboard_left: parse_string(cur, "SoundKeyboardLeft")?.to_string(),
            sound_keyboard_right: parse_string(cur, "SoundKeyboardRight")?.to_string(),
            sound_keyboard_enter_character: parse_string(cur, "SoundKeyboardEnterCharacter")?.to_string(),
            sound_keyboard_delete_character: parse_string(cur, "SoundKeyboardDeleteCharacter")?.to_string(),
            sound_keyboard_done: parse_string(cur, "SoundKeyboardDone")?.to_string(),
            front_end_music: parse_wstr(cur, "FrontEndMusic")?,
            keyboard_small_key_graphic: parse_scalar::<u32>(cur, "KeyboardSmallKeyGraphic")?,
            keyboard_large_key_graphic: parse_scalar::<u32>(cur, "KeyboardLargeKeyGraphic")?,
            time_in_secs_for_fade: parse_scalar::<f32>(cur, "TimeInSecsForFade")?,
            back_buffer_filter_saturation: parse_scalar::<f32>(cur, "BackBufferFilterSaturation")?,
            back_buffer_filter_contrast: parse_scalar::<f32>(cur, "BackBufferFilterContrast")?,
            back_buffer_filter_brightness: parse_scalar::<f32>(cur, "BackBufferFilterBrightness")?,
            back_buffer_filter_tint_r: parse_scalar::<f32>(cur, "BackBufferFilterTintR")?,
            back_buffer_filter_tint_g: parse_scalar::<f32>(cur, "BackBufferFilterTintG")?,
            back_buffer_filter_tint_b: parse_scalar::<f32>(cur, "BackBufferFilterTintB")?,
            back_buffer_filter_tint_scale: parse_scalar::<f32>(cur, "BackBufferFilterTintScale")?,
            back_buffer_diffuse_scale: parse_scalar::<f32>(cur, "BackBufferDiffuseScale")?,
            back_buffer_ambient_scale: parse_scalar::<f32>(cur, "BackBufferAmbientScale")?,
            minimum_filter_color: parse_scalar::<f32>(cur, "MinimumFilterColor")?,
        })
    }

    pub(crate) fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        serialize_wstr(out, "SpaceSeparator", &self.space_separator)?;
        serialize_wstr(out, "CommaSeparator", &self.comma_separator)?;
        serialize_wstr(out, "NewLineSeparator", &self.new_line_separator)?;
        serialize_wstr(out, "OpenBracket", &self.open_bracket)?;
        serialize_wstr(out, "CloseBracket", &self.close_bracket)?;
        serialize_wstr(out, "Positive", &self.positive)?;
        serialize_wstr(out, "WeaponValueString", &self.weapon_value_string)?;
        serialize_wstr(out, "WeaponAugString", &self.weapon_aug_string)?;
        serialize_wstr(out, "WeaponAugNone", &self.weapon_aug_none)?;
        serialize_wstr(out, "WeaponWeightString", &self.weapon_weight_string)?;
        serialize_wstr(out, "WeaponLightString", &self.weapon_light_string)?;
        serialize_wstr(out, "WeaponHeavyString", &self.weapon_heavy_string)?;
        serialize_wstr(out, "WeaponKillsString", &self.weapon_kills_string)?;
        serialize_wstr(out, "WeaponCatMeleeString", &self.weapon_cat_melee_string)?;
        serialize_wstr(out, "WeaponCatRangedString", &self.weapon_cat_ranged_string)?;
        serialize_wstr(out, "WeaponDamageString", &self.weapon_damage_string)?;
        serialize_wstr(out, "TradeCostString", &self.trade_cost_string)?;
        serialize_wstr(out, "ColonSeparator", &self.colon_separator)?;
        serialize_wstr(out, "TradeProfitString", &self.trade_profit_string)?;
        serialize_wstr(out, "TradeLossString", &self.trade_loss_string)?;
        serialize_wstr(out, "TradeAlreadyOwnsString", &self.trade_already_owns_string)?;
        serialize_wstr(out, "TradeNumberInStockString", &self.trade_number_in_stock_string)?;
        serialize_wstr(out, "TradeDeliveryString", &self.trade_delivery_string)?;
        serialize_wstr(out, "TradeNoDeliveryString", &self.trade_no_delivery_string)?;
        serialize_wstr(out, "TradeDaysString", &self.trade_days_string)?;
        serialize_wstr(out, "TradeBuyString", &self.trade_buy_string)?;
        serialize_wstr(out, "TradeSellString", &self.trade_sell_string)?;
        serialize_wstr(out, "TradeWantedString", &self.trade_wanted_string)?;
        serialize_wstr(out, "QuestFailedString", &self.quest_failed_string)?;
        serialize_wstr(out, "FailedString", &self.failed_string)?;
        serialize_wstr(out, "SucceededString", &self.succeeded_string)?;
        serialize_wstr(out, "Plus", &self.plus)?;
        serialize_wstr(out, "Minus", &self.minus)?;
        serialize_scalar::<u32>(out, "CoreGraphic", self.core_graphic)?;
        serialize_scalar::<u32>(out, "VignetteGraphic", self.vignette_graphic)?;
        serialize_scalar::<u32>(out, "OptionalGraphic", self.optional_graphic)?;
        serialize_scalar::<u32>(out, "FeatGraphic", self.feat_graphic)?;
        serialize_wstr(out, "ObjectsRewardString", &self.objects_reward_string)?;
        serialize_wstr(out, "NoneString", &self.none_string)?;
        serialize_wstr(out, "CheckGuildString", &self.check_guild_string)?;
        serialize_wstr(out, "QuestStartingString", &self.quest_starting_string)?;
        serialize_scalar::<f32>(out, "RingCenterX", self.ring_center_x)?;
        serialize_scalar::<f32>(out, "RingCenterY", self.ring_center_y)?;
        serialize_scalar::<f32>(out, "PCRingCenterX", self.pc_ring_center_x)?;
        serialize_scalar::<f32>(out, "PCRingCenterY", self.pc_ring_center_y)?;
        serialize_scalar::<f32>(out, "WorldMapOffsetX", self.world_map_offset_x)?;
        serialize_scalar::<f32>(out, "WorldMapOffsetY", self.world_map_offset_y)?;
        serialize_scalar::<f32>(out, "WorldMapWidth", self.world_map_width)?;
        serialize_scalar::<f32>(out, "WorldMapHeight", self.world_map_height)?;
        serialize_wstr(out, "YouString", &self.you_string)?;
        serialize_wstr(out, "OwnString", &self.own_string)?;
        serialize_wstr(out, "NoString", &self.no_string)?;
        serialize_wstr(out, "HousesString", &self.houses_string)?;
        serialize_wstr(out, "HouseString", &self.house_string)?;
        serialize_wstr(out, "InString", &self.in_string)?;
        serialize_wstr(out, "ShopsString", &self.shops_string)?;
        serialize_wstr(out, "ShopString", &self.shop_string)?;
        serialize_wstr(out, "ThereString", &self.there_string)?;
        serialize_wstr(out, "AreString", &self.are_string)?;
        serialize_wstr(out, "IsString", &self.is_string)?;
        serialize_wstr(out, "ForString", &self.for_string)?;
        serialize_wstr(out, "SaleString", &self.sale_string)?;
        serialize_wstr(out, "GeneralString", &self.general_string)?;
        serialize_wstr(out, "TatooString", &self.tatoo_string)?;
        serialize_wstr(out, "BarberString", &self.barber_string)?;
        serialize_wstr(out, "TitleString", &self.title_string)?;
        serialize_wstr(out, "LevelString", &self.level_string)?;
        serialize_scalar::<u32>(out, "TotalSpellsInPalettes", self.total_spells_in_palettes)?;
        serialize_scalar::<u32>(out, "TotalSpellsInContainer", self.total_spells_in_container)?;
        serialize_scalar::<u32>(out, "TotalAssignableThings", self.total_assignable_things)?;
        serialize_wstr(out, "LogBookBasicsCategoryString", &self.log_book_basics_category_string)?;
        serialize_wstr(out, "LogBookObjectsCategoryString", &self.log_book_objects_category_string)?;
        serialize_wstr(out, "LogBookTownsCategoryString", &self.log_book_towns_category_string)?;
        serialize_wstr(out, "LogBookHeroCategoryString", &self.log_book_hero_category_string)?;
        serialize_wstr(out, "LogBookCombatCategoryString", &self.log_book_combat_category_string)?;
        serialize_wstr(out, "LogBookQuestCategoryString", &self.log_book_quest_category_string)?;
        serialize_wstr(out, "LogBookStoryCategoryString", &self.log_book_story_category_string)?;
        serialize_wstr(out, "LogBookBasicsCategoryNameString", &self.log_book_basics_category_name_string)?;
        serialize_wstr(out, "LogBookObjectsCategoryNameString", &self.log_book_objects_category_name_string)?;
        serialize_wstr(out, "LogBookTownsCategoryNameString", &self.log_book_towns_category_name_string)?;
        serialize_wstr(out, "LogBookHeroCategoryNameString", &self.log_book_hero_category_name_string)?;
        serialize_wstr(out, "LogBookCombatCategoryNameString", &self.log_book_combat_category_name_string)?;
        serialize_wstr(out, "LogBookQuestCategoryNameString", &self.log_book_quest_category_name_string)?;
        serialize_wstr(out, "LogBookStoryCategoryNameString", &self.log_book_story_category_name_string)?;
        self.map_paths.serialize(out)?;
        serialize_string(out, "SoundUpDown", &self.sound_up_down)?;
        serialize_string(out, "SoundSlider", &self.sound_slider)?;
        serialize_string(out, "SoundBack", &self.sound_back)?;
        serialize_string(out, "SoundForward", &self.sound_forward)?;
        serialize_string(out, "SoundError", &self.sound_error)?;
        serialize_string(out, "SoundExit", &self.sound_exit)?;
        serialize_scalar::<f32>(out, "HeroDollTLX", self.hero_doll_tlx)?;
        serialize_scalar::<f32>(out, "HeroDollTLY", self.hero_doll_tly)?;
        serialize_scalar::<f32>(out, "HeroDollBRX", self.hero_doll_brx)?;
        serialize_scalar::<f32>(out, "HeroDollBRY", self.hero_doll_bry)?;
        serialize_scalar::<f32>(out, "HeroDollSphereRadius", self.hero_doll_sphere_radius)?;
        serialize_scalar::<f32>(out, "HeroDollTLX_PC", self.hero_doll_tlx_pc)?;
        serialize_scalar::<f32>(out, "HeroDollTLY_PC", self.hero_doll_tly_pc)?;
        serialize_scalar::<f32>(out, "HeroDollBRX_PC", self.hero_doll_brx_pc)?;
        serialize_scalar::<f32>(out, "HeroDollBRY_PC", self.hero_doll_bry_pc)?;
        serialize_scalar::<f32>(out, "HeroDollSphereRadius_PC", self.hero_doll_sphere_radius_pc)?;
        serialize_scalar::<f32>(out, "HeroDollFrameTLX_PC", self.hero_doll_frame_tlx_pc)?;
        serialize_scalar::<f32>(out, "HeroDollFrameTLY_PC", self.hero_doll_frame_tly_pc)?;
        serialize_scalar::<f32>(out, "HeroDollFrameEmulateListOffset", self.hero_doll_frame_emulate_list_offset)?;
        serialize_scalar::<u32>(out, "QuestStartScreenMusic", self.quest_start_screen_music)?;
        serialize_scalar::<u32>(out, "QuestCompleteScreenMusic", self.quest_complete_screen_music)?;
        serialize_scalar::<u32>(out, "QuestFailureScreenMusic", self.quest_failure_screen_music)?;
        serialize_scalar::<u32>(out, "DeathScreenMusic", self.death_screen_music)?;
        serialize_string(out, "CountUpSound", &self.count_up_sound)?;
        serialize_scalar::<f32>(out, "DigitCountTime", self.digit_count_time)?;
        serialize_scalar::<u32>(out, "SaveHeroGraphicIndex", self.save_hero_graphic_index)?;
        self.mini_map_graphics.serialize(out)?;
        serialize_string(out, "SoundKeyboardUp", &self.sound_keyboard_up)?;
        serialize_string(out, "SoundKeyboardDown", &self.sound_keyboard_down)?;
        serialize_string(out, "SoundKeyboardLeft", &self.sound_keyboard_left)?;
        serialize_string(out, "SoundKeyboardRight", &self.sound_keyboard_right)?;
        serialize_string(out, "SoundKeyboardEnterCharacter", &self.sound_keyboard_enter_character)?;
        serialize_string(out, "SoundKeyboardDeleteCharacter", &self.sound_keyboard_delete_character)?;
        serialize_string(out, "SoundKeyboardDone", &self.sound_keyboard_done)?;
        serialize_wstr(out, "FrontEndMusic", &self.front_end_music)?;
        serialize_scalar::<u32>(out, "KeyboardSmallKeyGraphic", self.keyboard_small_key_graphic)?;
        serialize_scalar::<u32>(out, "KeyboardLargeKeyGraphic", self.keyboard_large_key_graphic)?;
        serialize_scalar::<f32>(out, "TimeInSecsForFade", self.time_in_secs_for_fade)?;
        serialize_scalar::<f32>(out, "BackBufferFilterSaturation", self.back_buffer_filter_saturation)?;
        serialize_scalar::<f32>(out, "BackBufferFilterContrast", self.back_buffer_filter_contrast)?;
        serialize_scalar::<f32>(out, "BackBufferFilterBrightness", self.back_buffer_filter_brightness)?;
        serialize_scalar::<f32>(out, "BackBufferFilterTintR", self.back_buffer_filter_tint_r)?;
        serialize_scalar::<f32>(out, "BackBufferFilterTintG", self.back_buffer_filter_tint_g)?;
        serialize_scalar::<f32>(out, "BackBufferFilterTintB", self.back_buffer_filter_tint_b)?;
        serialize_scalar::<f32>(out, "BackBufferFilterTintScale", self.back_buffer_filter_tint_scale)?;
        serialize_scalar::<f32>(out, "BackBufferDiffuseScale", self.back_buffer_diffuse_scale)?;
        serialize_scalar::<f32>(out, "BackBufferAmbientScale", self.back_buffer_ambient_scale)?;
        serialize_scalar::<f32>(out, "MinimumFilterColor", self.minimum_filter_color)?;
        Ok(())
    }

    pub(crate) fn byte_size(&self) -> usize {
        wstr_control_byte_size(&self.space_separator)
            + wstr_control_byte_size(&self.comma_separator)
            + wstr_control_byte_size(&self.new_line_separator)
            + wstr_control_byte_size(&self.open_bracket)
            + wstr_control_byte_size(&self.close_bracket)
            + wstr_control_byte_size(&self.positive)
            + wstr_control_byte_size(&self.weapon_value_string)
            + wstr_control_byte_size(&self.weapon_aug_string)
            + wstr_control_byte_size(&self.weapon_aug_none)
            + wstr_control_byte_size(&self.weapon_weight_string)
            + wstr_control_byte_size(&self.weapon_light_string)
            + wstr_control_byte_size(&self.weapon_heavy_string)
            + wstr_control_byte_size(&self.weapon_kills_string)
            + wstr_control_byte_size(&self.weapon_cat_melee_string)
            + wstr_control_byte_size(&self.weapon_cat_ranged_string)
            + wstr_control_byte_size(&self.weapon_damage_string)
            + wstr_control_byte_size(&self.trade_cost_string)
            + wstr_control_byte_size(&self.colon_separator)
            + wstr_control_byte_size(&self.trade_profit_string)
            + wstr_control_byte_size(&self.trade_loss_string)
            + wstr_control_byte_size(&self.trade_already_owns_string)
            + wstr_control_byte_size(&self.trade_number_in_stock_string)
            + wstr_control_byte_size(&self.trade_delivery_string)
            + wstr_control_byte_size(&self.trade_no_delivery_string)
            + wstr_control_byte_size(&self.trade_days_string)
            + wstr_control_byte_size(&self.trade_buy_string)
            + wstr_control_byte_size(&self.trade_sell_string)
            + wstr_control_byte_size(&self.trade_wanted_string)
            + wstr_control_byte_size(&self.quest_failed_string)
            + wstr_control_byte_size(&self.failed_string)
            + wstr_control_byte_size(&self.succeeded_string)
            + wstr_control_byte_size(&self.plus)
            + wstr_control_byte_size(&self.minus)
            + (ID_BYTE_SIZE + size_of::<u32>())
            + (ID_BYTE_SIZE + size_of::<u32>())
            + (ID_BYTE_SIZE + size_of::<u32>())
            + (ID_BYTE_SIZE + size_of::<u32>())
            + wstr_control_byte_size(&self.objects_reward_string)
            + wstr_control_byte_size(&self.none_string)
            + wstr_control_byte_size(&self.check_guild_string)
            + wstr_control_byte_size(&self.quest_starting_string)
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + wstr_control_byte_size(&self.you_string)
            + wstr_control_byte_size(&self.own_string)
            + wstr_control_byte_size(&self.no_string)
            + wstr_control_byte_size(&self.houses_string)
            + wstr_control_byte_size(&self.house_string)
            + wstr_control_byte_size(&self.in_string)
            + wstr_control_byte_size(&self.shops_string)
            + wstr_control_byte_size(&self.shop_string)
            + wstr_control_byte_size(&self.there_string)
            + wstr_control_byte_size(&self.are_string)
            + wstr_control_byte_size(&self.is_string)
            + wstr_control_byte_size(&self.for_string)
            + wstr_control_byte_size(&self.sale_string)
            + wstr_control_byte_size(&self.general_string)
            + wstr_control_byte_size(&self.tatoo_string)
            + wstr_control_byte_size(&self.barber_string)
            + wstr_control_byte_size(&self.title_string)
            + wstr_control_byte_size(&self.level_string)
            + (ID_BYTE_SIZE + size_of::<u32>())
            + (ID_BYTE_SIZE + size_of::<u32>())
            + (ID_BYTE_SIZE + size_of::<u32>())
            + wstr_control_byte_size(&self.log_book_basics_category_string)
            + wstr_control_byte_size(&self.log_book_objects_category_string)
            + wstr_control_byte_size(&self.log_book_towns_category_string)
            + wstr_control_byte_size(&self.log_book_hero_category_string)
            + wstr_control_byte_size(&self.log_book_combat_category_string)
            + wstr_control_byte_size(&self.log_book_quest_category_string)
            + wstr_control_byte_size(&self.log_book_story_category_string)
            + wstr_control_byte_size(&self.log_book_basics_category_name_string)
            + wstr_control_byte_size(&self.log_book_objects_category_name_string)
            + wstr_control_byte_size(&self.log_book_towns_category_name_string)
            + wstr_control_byte_size(&self.log_book_hero_category_name_string)
            + wstr_control_byte_size(&self.log_book_combat_category_name_string)
            + wstr_control_byte_size(&self.log_book_quest_category_name_string)
            + wstr_control_byte_size(&self.log_book_story_category_name_string)
            + self.map_paths.byte_size()
            + string_control_byte_size(&self.sound_up_down)
            + string_control_byte_size(&self.sound_slider)
            + string_control_byte_size(&self.sound_back)
            + string_control_byte_size(&self.sound_forward)
            + string_control_byte_size(&self.sound_error)
            + string_control_byte_size(&self.sound_exit)
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<u32>())
            + (ID_BYTE_SIZE + size_of::<u32>())
            + (ID_BYTE_SIZE + size_of::<u32>())
            + (ID_BYTE_SIZE + size_of::<u32>())
            + string_control_byte_size(&self.count_up_sound)
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<u32>())
            + self.mini_map_graphics.byte_size()
            + string_control_byte_size(&self.sound_keyboard_up)
            + string_control_byte_size(&self.sound_keyboard_down)
            + string_control_byte_size(&self.sound_keyboard_left)
            + string_control_byte_size(&self.sound_keyboard_right)
            + string_control_byte_size(&self.sound_keyboard_enter_character)
            + string_control_byte_size(&self.sound_keyboard_delete_character)
            + string_control_byte_size(&self.sound_keyboard_done)
            + wstr_control_byte_size(&self.front_end_music)
            + (ID_BYTE_SIZE + size_of::<u32>())
            + (ID_BYTE_SIZE + size_of::<u32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
    }
}

#[derive(Debug)]
pub struct MapPaths {
    pub paths: Vec<String>,
}

impl MapPaths {
    const NAME: &'static str = "MapPaths";

    fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        let _id = parse_id(cur, Self::NAME)?;
        let count = take_le::<u32>(cur).map_err(|inner| ParseControlError {
            name: Self::NAME,
            reason: ParseControlErrorReason::ListCount(inner),
        })?;
        let mut paths = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let path = take_null_terminated_utf8(cur)
                .map_err(|inner| ParseControlError {
                    name: Self::NAME,
                    reason: ParseControlErrorReason::Utf8(inner),
                })?
                .to_owned();
            paths.push(path);
        }
        Ok(Self { paths })
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        serialize_id(out, Self::NAME)?;
        put_le(out, &(self.paths.len() as u32)).map_err(|inner| SerializeControlError {
            name: Self::NAME,
            reason: SerializeControlErrorReason::ListCount(inner),
        })?;
        for (i, path) in self.paths.iter().enumerate() {
            put_null_terminated_utf8(out, path).map_err(|inner| SerializeControlError {
                name: Self::NAME,
                reason: SerializeControlErrorReason::ListItem(i, inner),
            })?;
        }
        Ok(())
    }

    fn byte_size(&self) -> usize {
        ID_BYTE_SIZE
            + size_of::<u32>()
            + self
                .paths
                .iter()
                .map(|path| path.len() + size_of::<u8>())
                .sum::<usize>()
    }
}

#[derive(Debug)]
pub struct MiniMapGraphics {
    pub graphics: Vec<(String, i32)>,
}

impl MiniMapGraphics {
    const NAME: &'static str = "MiniMapGraphics";

    fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        let _id = parse_id(cur, Self::NAME)?;
        let count = take_le::<u32>(cur).map_err(|inner| ParseControlError {
            name: Self::NAME,
            reason: ParseControlErrorReason::ListCount(inner),
        })?;
        let mut graphics = Vec::with_capacity(count as usize);
        for i in 0..count as usize {
            let key = take_null_terminated_utf8(cur)
                .map_err(|inner| ParseControlError {
                    name: Self::NAME,
                    reason: ParseControlErrorReason::Utf8(inner),
                })?
                .to_owned();
            let value = take_le::<i32>(cur).map_err(|inner| ParseControlError {
                name: Self::NAME,
                reason: ParseControlErrorReason::ListItem(i, inner),
            })?;
            graphics.push((key, value));
        }
        Ok(Self { graphics })
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        serialize_id(out, Self::NAME)?;
        put_le(out, &(self.graphics.len() as u32)).map_err(|inner| SerializeControlError {
            name: Self::NAME,
            reason: SerializeControlErrorReason::ListCount(inner),
        })?;
        for (i, (key, value)) in self.graphics.iter().enumerate() {
            put_null_terminated_utf8(out, key).map_err(|inner| SerializeControlError {
                name: Self::NAME,
                reason: SerializeControlErrorReason::ListItem(i, inner),
            })?;
            put_le(out, value).map_err(|inner| SerializeControlError {
                name: Self::NAME,
                reason: SerializeControlErrorReason::ListItem(i, inner),
            })?;
        }
        Ok(())
    }

    fn byte_size(&self) -> usize {
        ID_BYTE_SIZE
            + size_of::<u32>()
            + self
                .graphics
                .iter()
                .map(|(key, _)| key.len() + size_of::<u8>() + size_of::<i32>())
                .sum::<usize>()
    }
}

