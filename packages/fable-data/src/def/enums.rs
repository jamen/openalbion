//! Def enum + flag types (merged frontend + game def sets).

//! Def enum types.
//!
//! Generated from the validated def spec (`fable-decomp/defs-spec.json`, itself
//! validated byte-exactly against every entry of the retail `game.bin`,
//! `frontend.bin`, and `script.bin`) plus the game's own `keyboard_keys.h`.
//! Each variant records its original C++ enumerator name — text defs refer to
//! enum values by those symbols, so they are kept for lowering and future text
//! emission.
//!
//! Two shapes:
//! - [`def_enum!`] — a strict, closed Rust enum. Used where the compiled data
//!   provably stays inside the C++ enum table (verified by scanning all three
//!   bins). Parsing an out-of-table value is an error.
//! - [`def_flags!`] — a bit-set newtype. Used for the "enums" the game actually
//!   ORs together (e.g. `TABLE_EXPANSION_HORIZONTAL | TABLE_EXPANSION_VERTICAL`
//!   occurs in game.bin) or that legitimately carry empty/unlisted values.

/// A def enum: a closed `i32`-repr enum with a total mapping to/from the wire
/// value and the C++ enumerator symbols used in text defs.
pub trait DefEnum: Sized + Copy {
    /// Table lookup; `None` for values outside the C++ enum.
    fn from_i32(value: i32) -> Option<Self>;
    fn to_i32(self) -> i32;
}

macro_rules! def_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident: i32 {
            $( $(#[$vmeta:meta])* $variant:ident = $value:literal => $symbol:literal, )+
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(i32)]
        pub enum $name {
            $( $(#[$vmeta])* $variant = $value, )+
        }

        impl $name {
            /// The original C++ enumerator name, as used by text defs.
            pub const fn symbol(self) -> &'static str {
                match self { $( Self::$variant => $symbol, )+ }
            }

            /// Look up a variant by its C++ enumerator name.
            pub fn from_symbol(symbol: &str) -> Option<Self> {
                match symbol {
                    $( $symbol => Some(Self::$variant), )+
                    _ => None,
                }
            }
        }

        impl crate::def::enums::DefEnum for $name {
            fn from_i32(value: i32) -> Option<Self> {
                match value {
                    $( $value => Some(Self::$variant), )+
                    _ => None,
                }
            }

            fn to_i32(self) -> i32 {
                self as i32
            }
        }

        impl crate::def::visit::DefDefault for $name {
            fn def_default() -> Self {
                // The first variant is the conventional default/zero value.
                [$( Self::$variant ),+][0]
            }
        }

        impl crate::def::wire::Wire for $name {
            fn parse(
                cur: &mut &[u8],
            ) -> Result<Self, crate::def::wire::ParseWireError> {
                use crate::def::enums::DefEnum;
                let value = <i32 as crate::def::wire::Wire>::parse(cur)?;
                Self::from_i32(value)
                    .ok_or(crate::def::wire::ParseWireError::InvalidEnumValue { value })
            }

            fn serialize(
                &self,
                out: &mut &mut [u8],
            ) -> Result<(), crate::bytes::UnexpectedEnd> {
                use crate::def::enums::DefEnum;
                crate::def::wire::Wire::serialize(&self.to_i32(), out)
            }

            fn wire_size(&self) -> usize {
                size_of::<i32>()
            }
        }

        impl crate::def::visit::EnumSlot for $name {
            fn get_i32(&self) -> i32 {
                use crate::def::enums::DefEnum;
                self.to_i32()
            }

            fn set_i32(&mut self, value: i32) -> Result<(), i32> {
                use crate::def::enums::DefEnum;
                *self = Self::from_i32(value).ok_or(value)?;
                Ok(())
            }
        }

        impl crate::def::visit::AsField for $name {
            fn as_field(&mut self) -> crate::def::visit::FieldRef<'_> {
                crate::def::visit::FieldRef::Enum(self)
            }
        }
    };
}

macro_rules! def_flags {
    (
        $(#[$meta:meta])*
        pub struct $name:ident: i32 {
            $( $(#[$vmeta:meta])* $flag:ident = $value:literal => $symbol:literal, )+
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub struct $name(pub i32);

        #[allow(non_upper_case_globals)]
        impl $name {
            $( $(#[$vmeta])* pub const $flag: Self = Self($value); )+

            pub const fn from_i32(value: i32) -> Self {
                Self(value)
            }

            pub const fn to_i32(self) -> i32 {
                self.0
            }

            pub const fn is_empty(self) -> bool {
                self.0 == 0
            }

            pub const fn contains(self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }

            /// The C++ enumerator name, when this is exactly one known flag.
            pub const fn symbol(self) -> Option<&'static str> {
                // Some flag sets alias a value (e.g. a `NONE = 0`), so later arms
                // for the same value are unreachable — the first symbol wins.
                #[allow(unreachable_patterns)]
                match self.0 {
                    $( $value => Some($symbol), )+
                    _ => None,
                }
            }

            /// Look up a single flag by its C++ enumerator name.
            pub fn from_symbol(symbol: &str) -> Option<Self> {
                match symbol {
                    $( $symbol => Some(Self::$flag), )+
                    _ => None,
                }
            }
        }

        impl crate::def::visit::DefDefault for $name {
            fn def_default() -> Self {
                Self(0)
            }
        }

        impl core::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }

        impl core::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        impl crate::def::wire::Wire for $name {
            fn parse(
                cur: &mut &[u8],
            ) -> Result<Self, crate::def::wire::ParseWireError> {
                Ok(Self::from_i32(<i32 as crate::def::wire::Wire>::parse(cur)?))
            }

            fn serialize(
                &self,
                out: &mut &mut [u8],
            ) -> Result<(), crate::bytes::UnexpectedEnd> {
                crate::def::wire::Wire::serialize(&self.to_i32(), out)
            }

            fn wire_size(&self) -> usize {
                size_of::<i32>()
            }
        }

        impl crate::def::visit::FlagsSlot for $name {
            fn get_i32(&self) -> i32 {
                self.0
            }

            fn set_i32(&mut self, value: i32) {
                self.0 = value;
            }
        }

        impl crate::def::visit::AsField for $name {
            fn as_field(&mut self) -> crate::def::visit::FieldRef<'_> {
                crate::def::visit::FieldRef::Flags(self)
            }
        }
    };
}

// Make the macros importable by the generated `game::_enums_generated` module.
pub(crate) use {def_enum, def_flags};

def_enum! {
    /// UI element type.
    ///
    /// C++ `NUISystem::EType`.
    pub enum UiType: i32 {
        Sprite = 0 => "UI_TYPE_SPRITE",
        MorphingSprite = 1 => "UI_TYPE_MORPHING_SPRITE",
        Table = 2 => "UI_TYPE_TABLE",
        Mesh = 3 => "UI_TYPE_MESH",
        Composite = 4 => "UI_TYPE_COMPOSITE",
        ChangingStateComposite = 5 => "UI_TYPE_CHANGING_STATE_COMPOSITE",
        Text = 6 => "UI_TYPE_TEXT",
        MenuEntry = 7 => "UI_TYPE_MENU_ENTRY",
        List = 8 => "UI_TYPE_LIST",
        Viewport = 9 => "UI_TYPE_VIEWPORT",
        FrontendScreen = 10 => "UI_TYPE_FRONTEND_SCREEN",
        FrontendButton = 11 => "UI_TYPE_FRONTEND_BUTTON",
        FrontendList = 12 => "UI_TYPE_FRONTEND_LIST",
        ScrollingViewport = 13 => "UI_TYPE_SCROLLING_VIEWPORT",
        ListArrow = 14 => "UI_TYPE_LIST_ARROW",
        Slider = 15 => "UI_TYPE_SLIDER",
        TextSlider = 16 => "UI_TYPE_TEXT_SLIDER",
        Movie = 17 => "UI_TYPE_MOVIE",
        SwappingStateComposite = 18 => "UI_TYPE_SWAPPING_STATE_COMPOSITE",
        ScrollingComposite = 19 => "UI_TYPE_SCROLLING_COMPOSITE",
        TextContainer = 20 => "UI_TYPE_TEXT_CONTAINER",
        ZoomingComposite = 21 => "UI_TYPE_ZOOMING_COMPOSITE",
        ComponentContainer = 22 => "UI_TYPE_COMPONENT_CONTAINER",
        SpellContainer = 23 => "UI_TYPE_SPELL_CONTAINER",
        SpellContainerList = 24 => "UI_TYPE_SPELL_CONTAINER_LIST",
        YesNo = 25 => "UI_TYPE_YESNO",
        Ok = 26 => "UI_TYPE_OK",
        ParticleEffect = 27 => "UI_TYPE_PARTICLE_EFFECT",
        ControllerDisconnect = 28 => "UI_TYPE_CONTROLLERDISCONNECT",
        DirtyDisc = 29 => "UI_TYPE_DIRTYDISC",
        IconText = 30 => "UI_TYPE_ICON_TEXT",
        DynamicList = 31 => "UI_TYPE_DYNAMIC_LIST",
        MousePointer = 32 => "UI_TYPE_MOUSE_POINTER",
        Hoverable = 33 => "UI_TYPE_HOVERABLE",
        Clickable = 34 => "UI_TYPE_CLICKABLE",
        Draggable = 35 => "UI_TYPE_DRAGGABLE",
        DraggableInto = 36 => "UI_TYPE_DRAGGABLE_INTO",
        EditBox = 37 => "UI_TYPE_EDIT_BOX",
        NavigationButton = 38 => "UI_TYPE_NAVIGATION_BUTTON",
        KeyRedefiner = 39 => "UI_TYPE_KEY_REDEFINER",
        RedefinerList = 40 => "UI_TYPE_REDEFINER_LIST",
        Scrollbar = 41 => "UI_TYPE_SCROLLBAR",
        ScrollbarOutside = 42 => "UI_TYPE_SCROLLBAR_OUTSIDE",
        ScrollableList = 43 => "UI_TYPE_SCROLLABLE_LIST",
    }
}

def_flags! {
    /// Table growth direction. Declared as an enum in C++ but OR-combined in real data (game.bin has `3`).
    ///
    /// C++ `NUISystem::ETableExpansionTypes`.
    pub struct TableExpansion: i32 {
        HORIZONTAL = 1 => "TABLE_EXPANSION_HORIZONTAL",
        VERTICAL = 2 => "TABLE_EXPANSION_VERTICAL",
    }
}

def_enum! {
    /// Text alignment.
    ///
    /// C++ `NUISystem::ETextAlignement`.
    pub enum TextAlignment: i32 {
        Left = 0 => "LEFT",
        Center = 1 => "CENTER",
        Right = 2 => "RIGHT",
    }
}

def_enum! {
    /// Order in which a UI state change propagates.
    ///
    /// C++ `NUISystem::EStateChangeType`.
    pub enum StateChangeType: i32 {
        Simultaneous = 0 => "STATE_CHANGE_SIMULTANEOUS",
        ParentFirst = 1 => "STATE_CHANGE_PARENT_FIRST",
        ChildrenFirst = 2 => "STATE_CHANGE_CHILDREN_FIRST",
        ParentOnly = 3 => "STATE_CHANGE_PARENT_ONLY",
        ChildrenOnly = 4 => "STATE_CHANGE_CHILDREN_ONLY",
    }
}

def_enum! {
    /// UI action fired by buttons and menu entries.
    ///
    /// C++ `NUISystem::EActionType`.
    pub enum ActionType: i32 {
        TypeNone = 0 => "UI_ACTION_TYPE_NONE",
        TypeWield = 1 => "UI_ACTION_TYPE_WIELD",
        TypeAugment = 2 => "UI_ACTION_TYPE_AUGMENT",
        TypeChangestate = 3 => "UI_ACTION_TYPE_CHANGESTATE",
        TypeTakeBoast = 4 => "UI_ACTION_TYPE_TAKE_BOAST",
        TypeTeleport = 5 => "UI_ACTION_TYPE_TELEPORT",
        TypeTakeQuest = 6 => "UI_ACTION_TYPE_TAKE_QUEST",
        TypeBuyStat = 7 => "UI_ACTION_TYPE_BUY_STAT",
        TypeBuyAbility = 8 => "UI_ACTION_TYPE_BUY_ABILITY",
        TypeNextScreenOptions = 9 => "UI_ACTION_TYPE_NEXT_SCREEN_OPTIONS",
        TypeNextScreenLiveAware = 10 => "UI_ACTION_TYPE_NEXT_SCREEN_LIVE_AWARE",
        TypeNextScreenExtras = 11 => "UI_ACTION_TYPE_NEXT_SCREEN_EXTRAS",
        TypeNextScreenAudioOptions = 12 => "UI_ACTION_TYPE_NEXT_SCREEN_AUDIO_OPTIONS",
        TypeNextScreenVideoOptions = 13 => "UI_ACTION_TYPE_NEXT_SCREEN_VIDEO_OPTIONS",
        TypeNextScreenCameraOptions = 14 => "UI_ACTION_TYPE_NEXT_SCREEN_CAMERA_OPTIONS",
        TypeNewGame = 15 => "UI_ACTION_TYPE_NEW_GAME",
        TypeNextScreenProfiles = 16 => "UI_ACTION_TYPE_NEXT_SCREEN_PROFILES",
        TypeLoadGame = 17 => "UI_ACTION_TYPE_LOAD_GAME",
        TypeLogin = 18 => "UI_ACTION_TYPE_LOGIN",
        TypeFriends = 19 => "UI_ACTION_TYPE_FRIENDS",
        TypeAppearOffline = 20 => "UI_ACTION_TYPE_APPEAR_OFFLINE",
        TypeDelete = 21 => "UI_ACTION_TYPE_DELETE",
        TypeAddChild = 22 => "UI_ACTION_TYPE_ADD_CHILD",
        TypeAddChildAugmentation = 23 => "UI_ACTION_TYPE_ADD_CHILD_AUGMENTATION",
        TypeDeleteAll = 24 => "UI_ACTION_TYPE_DELETE_ALL",
        TypeUseItem = 25 => "UI_ACTION_TYPE_USE_ITEM",
        TypeItemAssignLeft = 26 => "UI_ACTION_TYPE_ITEM_ASSIGN_LEFT",
        TypeItemAssignRight = 27 => "UI_ACTION_TYPE_ITEM_ASSIGN_RIGHT",
        TypeItemAssignDown = 28 => "UI_ACTION_TYPE_ITEM_ASSIGN_DOWN",
        TypeItemAssignUp = 29 => "UI_ACTION_TYPE_ITEM_ASSIGN_UP",
        TypeClothingWear = 30 => "UI_ACTION_TYPE_CLOTHING_WEAR",
        TypeUpdateMannequinClothing = 31 => "UI_ACTION_TYPE_UPDATE_MANNEQUIN_CLOTHING",
        TypeBuy = 36 => "UI_ACTION_TYPE_BUY",
        TypeSell = 37 => "UI_ACTION_TYPE_SELL",
        TypeDeletePrevious = 38 => "UI_ACTION_TYPE_DELETE_PREVIOUS",
        TypeResetClothingMannequin = 39 => "UI_ACTION_TYPE_RESET_CLOTHING_MANNEQUIN",
        TypeSetBrightness = 40 => "UI_ACTION_TYPE_SET_BRIGHTNESS",
        TypeSetSound = 41 => "UI_ACTION_TYPE_SET_SOUND",
        TypeSetMusic = 42 => "UI_ACTION_TYPE_SET_MUSIC",
        TypeSetCameraRotation = 43 => "UI_ACTION_TYPE_SET_CAMERA_ROTATION",
        TypeSetCameraUpDown = 44 => "UI_ACTION_TYPE_SET_CAMERA_UP_DOWN",
        TypeSetVibration = 45 => "UI_ACTION_TYPE_SET_VIBRATION",
        TypeSetHeadphones = 46 => "UI_ACTION_TYPE_SET_HEADPHONES",
        TypeSetSubtitles = 47 => "UI_ACTION_TYPE_SET_SUBTITLES",
        TypeOpenMap = 48 => "UI_ACTION_TYPE_OPEN_MAP",
        TypeOpenAbilities = 49 => "UI_ACTION_TYPE_OPEN_ABILITIES",
        TypeOpenWeapons = 50 => "UI_ACTION_TYPE_OPEN_WEAPONS",
        TypeOpenStats = 51 => "UI_ACTION_TYPE_OPEN_STATS",
        TypeOpenClothing = 52 => "UI_ACTION_TYPE_OPEN_CLOTHING",
        TypeOpenItems = 53 => "UI_ACTION_TYPE_OPEN_ITEMS",
        TypeCloseMap = 54 => "UI_ACTION_TYPE_CLOSE_MAP",
        TypeCloseAbilities = 55 => "UI_ACTION_TYPE_CLOSE_ABILITIES",
        TypeCloseWeapons = 56 => "UI_ACTION_TYPE_CLOSE_WEAPONS",
        TypeCloseStats = 57 => "UI_ACTION_TYPE_CLOSE_STATS",
        TypeCloseClothing = 58 => "UI_ACTION_TYPE_CLOSE_CLOTHING",
        TypeCloseItems = 59 => "UI_ACTION_TYPE_CLOSE_ITEMS",
        TypeCloseMenu = 60 => "UI_ACTION_TYPE_CLOSE_MENU",
        TypeGuildSealRecall = 61 => "UI_ACTION_TYPE_GUILD_SEAL_RECALL",
        TypeLoad = 62 => "UI_ACTION_TYPE_LOAD",
        TypeSave = 63 => "UI_ACTION_TYPE_SAVE",
        TypeClosePauseMenu = 64 => "UI_ACTION_TYPE_CLOSE_PAUSE_MENU",
        TypePlayMovie = 65 => "UI_ACTION_TYPE_PLAY_MOVIE",
        TypeNextScreenProfilesSavedGames = 66 => "UI_ACTION_TYPE_NEXT_SCREEN_PROFILES_SAVED_GAMES",
        TypeNextScreenCredits = 67 => "UI_ACTION_TYPE_NEXT_SCREEN_CREDITS",
        TypeConstructWeaponsList = 68 => "UI_ACTION_TYPE_CONSTRUCT_WEAPONS_LIST",
        TypeConstructClothingList = 69 => "UI_ACTION_TYPE_CONSTRUCT_CLOTHING_LIST",
        TypeConstructItemsList = 70 => "UI_ACTION_TYPE_CONSTRUCT_ITEMS_LIST",
        TypeConstructSkillsList = 71 => "UI_ACTION_TYPE_CONSTRUCT_SKILLS_LIST",
        TypeConstructQuestsList = 72 => "UI_ACTION_TYPE_CONSTRUCT_QUESTS_LIST",
        TypeConstructMapList = 73 => "UI_ACTION_TYPE_CONSTRUCT_MAP_LIST",
        TypeConstructStatsList = 74 => "UI_ACTION_TYPE_CONSTRUCT_STATS_LIST",
        TypeChangeChildState = 75 => "UI_ACTION_TYPE_CHANGE_CHILD_STATE",
        TypeConstructExperienceList = 76 => "UI_ACTION_TYPE_CONSTRUCT_EXPERIENCE_LIST",
        TypeConstructPersonalityList = 77 => "UI_ACTION_TYPE_CONSTRUCT_PERSONALITY_LIST",
        TypeSetHud = 78 => "UI_ACTION_TYPE_SET_HUD",
        TypeCloseQuickAccessMenu = 79 => "UI_ACTION_TYPE_CLOSE_QUICK_ACCESS_MENU",
        TypeCloseTeleportMenu = 80 => "UI_ACTION_TYPE_CLOSE_TELEPORT_MENU",
        TypeBuyBulk = 81 => "UI_ACTION_TYPE_BUY_BULK",
        TypeSellBulk = 82 => "UI_ACTION_TYPE_SELL_BULK",
        TypeSetTutorials = 83 => "UI_ACTION_TYPE_SET_TUTORIALS",
        TypeSetShowBuddyNames = 84 => "UI_ACTION_TYPE_SET_SHOW_BUDDY_NAMES",
        TypeQuitToFrontEnd = 85 => "UI_ACTION_TYPE_QUIT_TO_FRONT_END",
        TypePreviousScreen = 86 => "UI_ACTION_TYPE_PREVIOUS_SCREEN",
        TypeAcceptChanges = 87 => "UI_ACTION_TYPE_ACCEPT_CHANGES",
        TypeSetDialogue = 88 => "UI_ACTION_TYPE_SET_DIALOGUE",
        TypeXlive = 89 => "UI_ACTION_TYPE_XLIVE",
        TypeXliveBack = 90 => "UI_ACTION_TYPE_XLIVE_BACK",
        TypeLogout = 91 => "UI_ACTION_TYPE_LOGOUT",
        TypeDemos = 92 => "UI_ACTION_TYPE_DEMOS",
        TypeSetGuildMaster = 93 => "UI_ACTION_TYPE_SET_GUILD_MASTER",
        TypeSetHudTooltips = 94 => "UI_ACTION_TYPE_SET_HUD_TOOLTIPS",
        TypeUndoBuyAbility = 200 => "UI_ACTION_TYPE_UNDO_BUY_ABILITY",
        TypeUndoBuyStat = 201 => "UI_ACTION_TYPE_UNDO_BUY_STAT",
        TypeGiveExclusiveInput = 202 => "UI_ACTION_TYPE_GIVE_EXCLUSIVE_INPUT",
        TypeRemoveExclusiveInput = 203 => "UI_ACTION_TYPE_REMOVE_EXCLUSIVE_INPUT",
        TypeSendBackEvent = 204 => "UI_ACTION_TYPE_SEND_BACK_EVENT",
        TypeShowSpellSelection = 205 => "UI_ACTION_TYPE_SHOW_SPELL_SELECTION",
        TypeSetSpellToAssign = 206 => "UI_ACTION_TYPE_SET_SPELL_TO_ASSIGN",
        TypeTeleportToBoast = 207 => "UI_ACTION_TYPE_TELEPORT_TO_BOAST",
        TypeDropQuest = 208 => "UI_ACTION_TYPE_DROP_QUEST",
        TypeAssignSpell = 209 => "UI_ACTION_TYPE_ASSIGN_SPELL",
        TypeUnassignSpell = 210 => "UI_ACTION_TYPE_UNASSIGN_SPELL",
        TypeResetAssignedSpells = 211 => "UI_ACTION_TYPE_RESET_ASSIGNED_SPELLS",
        TypeChangeAlpha = 212 => "UI_ACTION_TYPE_CHANGE_ALPHA",
        TypeChangeNumber = 213 => "UI_ACTION_TYPE_CHANGE_NUMBER",
        TypeDeleteProfile = 214 => "UI_ACTION_TYPE_DELETE_PROFILE",
        TypeGoToDeleteProfileScreen = 215 => "UI_ACTION_TYPE_GO_TO_DELETE_PROFILE_SCREEN",
        TypePopMapState = 216 => "UI_ACTION_TYPE_POP_MAP_STATE",
        TypeNextScreenInvalidProfile = 219 => "UI_ACTION_TYPE_NEXT_SCREEN_INVALID_PROFILE",
        TypeNextScreenInvalidSave = 220 => "UI_ACTION_TYPE_NEXT_SCREEN_INVALID_SAVE",
        TypeFreeSpaceOnT = 221 => "UI_ACTION_TYPE_FREE_SPACE_ON_T",
        TypeFreeSpaceOnU = 222 => "UI_ACTION_TYPE_FREE_SPACE_ON_U",
        TypeFlashDpadDown = 223 => "UI_ACTION_TYPE_FLASH_DPAD_DOWN",
        TypeFlashDpadUp = 224 => "UI_ACTION_TYPE_FLASH_DPAD_UP",
        TypeFlashDpadLeft = 225 => "UI_ACTION_TYPE_FLASH_DPAD_LEFT",
        TypeFlashDpadRight = 226 => "UI_ACTION_TYPE_FLASH_DPAD_RIGHT",
        TypeStopDpadFlashing = 227 => "UI_ACTION_TYPE_STOP_DPAD_FLASHING",
        TypeConstructLogbookList = 228 => "UI_ACTION_TYPE_CONSTRUCT_LOGBOOK_LIST",
        TypeGoToMainMenuFromStart = 229 => "UI_ACTION_TYPE_GO_TO_MAIN_MENU_FROM_START",
        TypeSendOwnedEvent = 230 => "UI_ACTION_TYPE_SEND_OWNED_EVENT",
        TypeSendEvent = 231 => "UI_ACTION_TYPE_SEND_EVENT",
        TypeMoveComponent = 232 => "UI_ACTION_TYPE_MOVE_COMPONENT",
        TypeExitLiveGui = 233 => "UI_ACTION_TYPE_EXIT_LIVE_GUI",
        TypeOpenPcSkillsMenu = 234 => "UI_ACTION_TYPE_OPEN_PC_SKILLS_MENU",
        TypeConstructExpressionsList = 235 => "UI_ACTION_TYPE_CONSTRUCT_EXPRESSIONS_LIST",
        TypeScrollActiveListUp = 236 => "UI_ACTION_TYPE_SCROLL_ACTIVE_LIST_UP",
        TypeScrollActiveListDown = 237 => "UI_ACTION_TYPE_SCROLL_ACTIVE_LIST_DOWN",
        TypeOpenPcInventoryMenu = 238 => "UI_ACTION_TYPE_OPEN_PC_INVENTORY_MENU",
        TypeOpenPcOptionsMenu = 239 => "UI_ACTION_TYPE_OPEN_PC_OPTIONS_MENU",
        TypeConstructPcStyleCardsList = 240 => "UI_ACTION_TYPE_CONSTRUCT_PC_STYLE_CARDS_LIST",
        TypeOpenPcBuyTradingGoodsList = 241 => "UI_ACTION_TYPE_OPEN_PC_BUY_TRADING_GOODS_LIST",
        TypeClosePcBuyTradingGoodsList = 242 => "UI_ACTION_TYPE_CLOSE_PC_BUY_TRADING_GOODS_LIST",
        TypeConstructMagicListPc = 243 => "UI_ACTION_TYPE_CONSTRUCT_MAGIC_LIST_PC",
        TypeAssignSpellPc = 244 => "UI_ACTION_TYPE_ASSIGN_SPELL_PC",
        TypeObserveEvent = 245 => "UI_ACTION_TYPE_OBSERVE_EVENT",
        TypeSetSpellToAssignPc = 246 => "UI_ACTION_TYPE_SET_SPELL_TO_ASSIGN_PC",
        TypeResetAssignedSpellsPc = 247 => "UI_ACTION_TYPE_RESET_ASSIGNED_SPELLS_PC",
        TypeUnassignSpellPc = 248 => "UI_ACTION_TYPE_UNASSIGN_SPELL_PC",
        TypeIgnoreEvent = 249 => "UI_ACTION_TYPE_IGNORE_EVENT",
        TypeNextScreenProfilesForDelete = 250 => "UI_ACTION_TYPE_NEXT_SCREEN_PROFILES_FOR_DELETE",
        TypeAssignExpressionItemPc = 251 => "UI_ACTION_TYPE_ASSIGN_EXPRESSION_ITEM_PC",
        TypeUnassignExpressionItemPc = 252 => "UI_ACTION_TYPE_UNASSIGN_EXPRESSION_ITEM_PC",
        TypeSetExpressionItemToAssignPc = 253 => "UI_ACTION_TYPE_SET_EXPRESSION_ITEM_TO_ASSIGN_PC",
        TypeOpenPcMapMenu = 254 => "UI_ACTION_TYPE_OPEN_PC_MAP_MENU",
        TypeOpenPcStatusMenu = 255 => "UI_ACTION_TYPE_OPEN_PC_STATUS_MENU",
        TypeOpenPcLogBook = 256 => "UI_ACTION_TYPE_OPEN_PC_LOG_BOOK",
        TypeConstructPcQuestsList = 257 => "UI_ACTION_TYPE_CONSTRUCT_PC_QUESTS_LIST",
        TypeSetResolution = 258 => "UI_ACTION_TYPE_SET_RESOLUTION",
        TypeSetShadowDetail = 259 => "UI_ACTION_TYPE_SET_SHADOW_DETAIL",
        TypeSetDecals = 260 => "UI_ACTION_TYPE_SET_DECALS",
        TypeSetWeatherEffects = 261 => "UI_ACTION_TYPE_SET_WEATHER_EFFECTS",
        TypeSetVerticalSync = 262 => "UI_ACTION_TYPE_SET_VERTICAL_SYNC",
        TypeSetGlowEffects = 263 => "UI_ACTION_TYPE_SET_GLOW_EFFECTS",
        TypeSetReverseStereo = 264 => "UI_ACTION_TYPE_SET_REVERSE_STEREO",
        TypeSetAntialiasing = 265 => "UI_ACTION_TYPE_SET_ANTIALIASING",
        TypeSetTextureDetail = 266 => "UI_ACTION_TYPE_SET_TEXTURE_DETAIL",
        TypeSetMeshResolution = 267 => "UI_ACTION_TYPE_SET_MESH_RESOLUTION",
        TypeSetWaterReflection = 268 => "UI_ACTION_TYPE_SET_WATER_REFLECTION",
        TypeSetLandscapeDetail = 269 => "UI_ACTION_TYPE_SET_LANDSCAPE_DETAIL",
        TypeSetParticleDetail = 270 => "UI_ACTION_TYPE_SET_PARTICLE_DETAIL",
        TypeAddObserver = 271 => "UI_ACTION_TYPE_ADD_OBSERVER",
        TypeScrollActiveListUntilChildContaining = 272 => "UI_ACTION_TYPE_SCROLL_ACTIVE_LIST_UNTIL_CHILD_CONTAINING",
        TypeGoBack = 273 => "UI_ACTION_TYPE_GO_BACK",
        TypeCloseActiveMenu = 274 => "UI_ACTION_TYPE_CLOSE_ACTIVE_MENU",
        TypeRemoveObserver = 275 => "UI_ACTION_TYPE_REMOVE_OBSERVER",
        TypeTavernGameBetDown = 276 => "UI_ACTION_TYPE_TAVERN_GAME_BET_DOWN",
        TypeTavernGameBetUp = 277 => "UI_ACTION_TYPE_TAVERN_GAME_BET_UP",
        TypeRespawn = 278 => "UI_ACTION_TYPE_RESPAWN",
        TypeContinue = 279 => "UI_ACTION_TYPE_CONTINUE",
        TypeOpenPcSellTradingGoodsList = 280 => "UI_ACTION_TYPE_OPEN_PC_SELL_TRADING_GOODS_LIST",
        TypeOpenPcWantedTradingGoodsList = 281 => "UI_ACTION_TYPE_OPEN_PC_WANTED_TRADING_GOODS_LIST",
        TypeConstructPcExperienceList = 282 => "UI_ACTION_TYPE_CONSTRUCT_PC_EXPERIENCE_LIST",
        TypeGoToRedefineKeysMenu = 283 => "UI_ACTION_TYPE_GO_TO_REDEFINE_KEYS_MENU",
        TypeResetKeys = 284 => "UI_ACTION_TYPE_RESET_KEYS",
        TypeAssignSpellToItemSlotPc = 285 => "UI_ACTION_TYPE_ASSIGN_SPELL_TO_ITEM_SLOT_PC",
        TypeConstructStyleCardsList = 286 => "UI_ACTION_TYPE_CONSTRUCT_STYLE_CARDS_LIST",
        TypeScrollDescriptionDown = 287 => "UI_ACTION_TYPE_SCROLL_DESCRIPTION_DOWN",
        TypeScrollDescriptionUp = 288 => "UI_ACTION_TYPE_SCROLL_DESCRIPTION_UP",
        TypeConstructPcPersonalityList = 289 => "UI_ACTION_TYPE_CONSTRUCT_PC_PERSONALITY_LIST",
        TypeSendOwnedEventForceObservation = 291 => "UI_ACTION_TYPE_SEND_OWNED_EVENT_FORCE_OBSERVATION",
        TypeGoToMainMenuFromProfileList = 292 => "UI_ACTION_TYPE_GO_TO_MAIN_MENU_FROM_PROFILE_LIST",
        TypeNewProfile = 293 => "UI_ACTION_TYPE_NEW_PROFILE",
        TypeNewProfileReturnPressed = 294 => "UI_ACTION_TYPE_NEW_PROFILE_RETURN_PRESSED",
        TypeNewProfileEscapePressed = 295 => "UI_ACTION_TYPE_NEW_PROFILE_ESCAPE_PRESSED",
        TypeQuitGame = 296 => "UI_ACTION_TYPE_QUIT_GAME",
        TypeNextScreenOptionsSubMenu = 297 => "UI_ACTION_TYPE_NEXT_SCREEN_OPTIONS_SUB_MENU",
        TypeSetMeshDetail = 298 => "UI_ACTION_TYPE_SET_MESH_DETAIL",
        TypeSetEffectsDetail = 299 => "UI_ACTION_TYPE_SET_EFFECTS_DETAIL",
        TypeSetScreenAspectRatio = 300 => "UI_ACTION_TYPE_SET_SCREEN_ASPECT_RATIO",
        TypeApplyProfileValues = 301 => "UI_ACTION_TYPE_APPLY_PROFILE_VALUES",
        TypeCloseActiveTradeMenu = 302 => "UI_ACTION_TYPE_CLOSE_ACTIVE_TRADE_MENU",
        TypeScrollActiveListDownMax = 303 => "UI_ACTION_TYPE_SCROLL_ACTIVE_LIST_DOWN_MAX",
        TypeScrollActiveListUpMax = 304 => "UI_ACTION_TYPE_SCROLL_ACTIVE_LIST_UP_MAX",
        TypeScrollDescriptionDownMax = 305 => "UI_ACTION_TYPE_SCROLL_DESCRIPTION_DOWN_MAX",
        TypeScrollDescriptionUpMax = 306 => "UI_ACTION_TYPE_SCROLL_DESCRIPTION_UP_MAX",
        TypeScrollDescriptionDownOnePixel = 307 => "UI_ACTION_TYPE_SCROLL_DESCRIPTION_DOWN_ONE_PIXEL",
        TypeScrollDescriptionUpOnePixel = 308 => "UI_ACTION_TYPE_SCROLL_DESCRIPTION_UP_ONE_PIXEL",
        TypeCloseTradeMenuIfLeaf = 309 => "UI_ACTION_TYPE_CLOSE_TRADE_MENU_IF_LEAF",
        TypeExitLiveGuiIfLeaf = 310 => "UI_ACTION_TYPE_EXIT_LIVE_GUI_IF_LEAF",
        TypeResetKeysWasd = 311 => "UI_ACTION_TYPE_RESET_KEYS_WASD",
        TypeNextScreenOptionsScoreboard = 312 => "UI_ACTION_TYPE_NEXT_SCREEN_OPTIONS_SCOREBOARD",
        TypeSetControlMethod = 313 => "UI_ACTION_TYPE_SET_CONTROL_METHOD",
        TypeGotoQuitPrompt = 314 => "UI_ACTION_TYPE_GOTO_QUIT_PROMPT",
        TypePcQuitToFrontEnd = 315 => "UI_ACTION_TYPE_PC_QUIT_TO_FRONT_END",
        TypePcAcceptVideoChanges = 316 => "UI_ACTION_TYPE_PC_ACCEPT_VIDEO_CHANGES",
        TypeSetRefreshRate = 317 => "UI_ACTION_TYPE_SET_REFRESH_RATE",
        TypeSetCameraSensitivity = 318 => "UI_ACTION_TYPE_SET_CAMERA_SENSITIVITY",
        TypeSetBowCamera = 319 => "UI_ACTION_TYPE_SET_BOW_CAMERA",
        TypeSetCameraResetting = 320 => "UI_ACTION_TYPE_SET_CAMERA_RESETTING",
        TypeGotoAboutScreen = 321 => "UI_ACTION_TYPE_GOTO_ABOUT_SCREEN",
        TypeCloseFrame = 322 => "UI_ACTION_TYPE_CLOSE_FRAME",
        TypeSetShowTargetingStatus = 323 => "UI_ACTION_TYPE_SET_SHOW_TARGETING_STATUS",
        TypeRestoreDefaultsGameplay = 324 => "UI_ACTION_TYPE_RESTORE_DEFAULTS_GAMEPLAY",
        TypeRestoreDefaultsVideo = 325 => "UI_ACTION_TYPE_RESTORE_DEFAULTS_VIDEO",
        TypeRestoreDefaultsAudio = 326 => "UI_ACTION_TYPE_RESTORE_DEFAULTS_AUDIO",
        TypeSetExpressionItemToAssignSwappingPc = 327 => "UI_ACTION_TYPE_SET_EXPRESSION_ITEM_TO_ASSIGN_SWAPPING_PC",
        TypeSetSpellToAssignSwappingPc = 328 => "UI_ACTION_TYPE_SET_SPELL_TO_ASSIGN_SWAPPING_PC",
        TypeSetSpellToAssignInItemsSwappingPc = 329 => "UI_ACTION_TYPE_SET_SPELL_TO_ASSIGN_IN_ITEMS_SWAPPING_PC",
        AddMeshChild = 2000 => "UI_ACTION_ADD_MESH_CHILD",
        LoadWeaponDesc = 2001 => "UI_ACTION_LOAD_WEAPON_DESC",
        LoadClothingDesc = 2002 => "UI_ACTION_LOAD_CLOTHING_DESC",
        LoadItemDesc = 2003 => "UI_ACTION_LOAD_ITEM_DESC",
        LoadWeaponMenuEntryName = 2004 => "UI_ACTION_LOAD_WEAPON_MENU_ENTRY_NAME",
        LoadClothingMenuEntryName = 2005 => "UI_ACTION_LOAD_CLOTHING_MENU_ENTRY_NAME",
        LoadItemMenuEntryName = 2006 => "UI_ACTION_LOAD_ITEM_MENU_ENTRY_NAME",
        LoadSellItemDesc = 2007 => "UI_ACTION_LOAD_SELL_ITEM_DESC",
        LoadSellActionMenu = 2008 => "UI_ACTION_LOAD_SELL_ACTION_MENU",
        LoadBuyItemDesc = 2009 => "UI_ACTION_LOAD_BUY_ITEM_DESC",
        LoadBuyActionMenu = 2010 => "UI_ACTION_LOAD_BUY_ACTION_MENU",
        LoadWantedItemDesc = 2011 => "UI_ACTION_LOAD_WANTED_ITEM_DESC",
        LoadQuickMenuExpressionsIcon = 2012 => "UI_ACTION_LOAD_QUICK_MENU_EXPRESSIONS_ICON",
        LoadFileDesc = 2013 => "UI_ACTION_LOAD_FILE_DESC",
        LoadFileSavegameMinimap = 2014 => "UI_ACTION_LOAD_FILE_SAVEGAME_MINIMAP",
        RemoveMeshChild = 2015 => "UI_ACTION_REMOVE_MESH_CHILD",
        UnloadWeaponDesc = 2016 => "UI_ACTION_UNLOAD_WEAPON_DESC",
        UnloadClothingDesc = 2017 => "UI_ACTION_UNLOAD_CLOTHING_DESC",
        UnloadItemDesc = 2018 => "UI_ACTION_UNLOAD_ITEM_DESC",
        UnloadSellItemDesc = 2019 => "UI_ACTION_UNLOAD_SELL_ITEM_DESC",
        UnloadSellActionMenu = 2020 => "UI_ACTION_UNLOAD_SELL_ACTION_MENU",
        UnloadBuyItemDesc = 2021 => "UI_ACTION_UNLOAD_BUY_ITEM_DESC",
        UnloadBuyActionMenu = 2022 => "UI_ACTION_UNLOAD_BUY_ACTION_MENU",
        UnloadWantedItemDesc = 2023 => "UI_ACTION_UNLOAD_WANTED_ITEM_DESC",
        TypeCheatMorality = 3000 => "UI_ACTION_TYPE_CHEAT_MORALITY",
        TypeCheatRenown = 3001 => "UI_ACTION_TYPE_CHEAT_RENOWN",
        TypeCloseBoastMenu = 3002 => "UI_ACTION_TYPE_CLOSE_BOAST_MENU",
        TypePlaySound = 3003 => "UI_ACTION_TYPE_PLAY_SOUND",
        TypeTakeQuestForBoast = 3004 => "UI_ACTION_TYPE_TAKE_QUEST_FOR_BOAST",
        TypeOpenPcMsnChat = 3005 => "UI_ACTION_TYPE_OPEN_PC_MSN_CHAT",
        TypeActivateMsnConversation = 3006 => "UI_ACTION_TYPE_ACTIVATE_MSN_CONVERSATION",
        TypeSendMessage = 3007 => "UI_ACTION_TYPE_SEND_MESSAGE",
        TypeSelectContact = 3008 => "UI_ACTION_TYPE_SELECT_CONTACT",
        TypeScrollViewportUp = 3009 => "UI_ACTION_TYPE_SCROLL_VIEWPORT_UP",
        TypeScrollViewportDown = 3010 => "UI_ACTION_TYPE_SCROLL_VIEWPORT_DOWN",
        TypeScrollListDown = 3011 => "UI_ACTION_TYPE_SCROLL_LIST_DOWN",
        TypeScrollListUp = 3012 => "UI_ACTION_TYPE_SCROLL_LIST_UP",
        TypeOpenScoreboard = 3013 => "UI_ACTION_TYPE_OPEN_SCOREBOARD",
        TypeChooseClan = 3014 => "UI_ACTION_TYPE_CHOOSE_CLAN",
        TypePhotoCaption = 3015 => "UI_ACTION_TYPE_PHOTO_CAPTION",
        TypeDiscardPhoto = 3016 => "UI_ACTION_TYPE_DISCARD_PHOTO",
        TypeGoToScoreboardScreen = 3017 => "UI_ACTION_TYPE_GO_TO_SCOREBOARD_SCREEN",
        TypeActivateEditbox = 3018 => "UI_ACTION_TYPE_ACTIVATE_EDITBOX",
        TypeDeactivateEditbox = 3019 => "UI_ACTION_TYPE_DEACTIVATE_EDITBOX",
        TypeSetEditboxValues = 3020 => "UI_ACTION_TYPE_SET_EDITBOX_VALUES",
        TypeScoreboardOk = 3021 => "UI_ACTION_TYPE_SCOREBOARD_OK",
        TypeLoadPhoto = 3022 => "UI_ACTION_TYPE_LOAD_PHOTO",
        TypeScrollPhotoDown = 3023 => "UI_ACTION_TYPE_SCROLL_PHOTO_DOWN",
        TypeScrollPhotoUp = 3024 => "UI_ACTION_TYPE_SCROLL_PHOTO_UP",
        TypeNextScreenStart = 3025 => "UI_ACTION_TYPE_NEXT_SCREEN_START",
        TypeOpenPhotojournal = 4000 => "UI_ACTION_TYPE_OPEN_PHOTOJOURNAL",
        TypeOpenPhoto = 4001 => "UI_ACTION_TYPE_OPEN_PHOTO",
        TypeExitPhotojournalCapture = 4002 => "UI_ACTION_TYPE_EXIT_PHOTOJOURNAL_CAPTURE",
    }
}

def_enum! {
    /// Sprite slot of a UI table (key of `UiDef::sprites`).
    ///
    /// C++ `NUISystem::ETableSprites`.
    pub enum TableSprites: i32 {
        TopLeft = 0 => "TABLE_SPRITES_TOP_LEFT",
        TopRight = 1 => "TABLE_SPRITES_TOP_RIGHT",
        BottomLeft = 2 => "TABLE_SPRITES_BOTTOM_LEFT",
        BottomRight = 3 => "TABLE_SPRITES_BOTTOM_RIGHT",
        TopMiddle = 4 => "TABLE_SPRITES_TOP_MIDDLE",
        BottomMiddle = 5 => "TABLE_SPRITES_BOTTOM_MIDDLE",
        MiddleLeft = 6 => "TABLE_SPRITES_MIDDLE_LEFT",
        MiddleRight = 7 => "TABLE_SPRITES_MIDDLE_RIGHT",
        SeparationBottom = 8 => "TABLE_SPRITES_SEPARATION_BOTTOM",
        SeparationTop = 9 => "TABLE_SPRITES_SEPARATION_TOP",
        SeparationLeft = 10 => "TABLE_SPRITES_SEPARATION_LEFT",
        SeparationRight = 11 => "TABLE_SPRITES_SEPARATION_RIGHT",
        Cross = 12 => "TABLE_SPRITES_CROSS",
        Amount = 13 => "TABLE_SPRITES_AMOUNT",
    }
}

def_enum! {
    /// Engine graphic/mesh kind.
    ///
    /// C++ `EEngineGraphicType`.
    pub enum EngineGraphicType: i32 {
        EngineGraphicNull = 0 => "ENGINE_GRAPHIC_NULL",
        EngineGraphicSprite = 1 => "ENGINE_GRAPHIC_SPRITE",
        EngineGraphic3dsprite = 2 => "ENGINE_GRAPHIC_3DSPRITE",
        EngineGraphicGeneratedEffect = 3 => "ENGINE_GRAPHIC_GENERATED_EFFECT",
        EngineGraphicAnimatingMesh = 4 => "ENGINE_GRAPHIC_ANIMATING_MESH",
        EngineGraphicStaticMesh = 5 => "ENGINE_GRAPHIC_STATIC_MESH",
        MaxNoEngineGraphicTypes = 6 => "MAX_NO_ENGINE_GRAPHIC_TYPES",
    }
}

def_flags! {
    /// 2D sprite render flags. `0` (no flags) occurs in real data.
    ///
    /// C++ `EEngineSprite2DFlag`.
    pub struct Sprite2dFlags: i32 {
        CENTRE_ON_POS = 1 => "ENGINE_2D_SPRITE_CENTRE_ON_POS",
        ENABLE_FILTERING = 2 => "ENGINE_2D_SPRITE_ENABLE_FILTERING",
    }
}

def_enum! {
    /// Bindable game action.
    ///
    /// C++ `EGameAction`.
    pub enum GameAction: i32 {
        Null = 0 => "GAME_ACTION_NULL",
        LockTarget = 1 => "GAME_ACTION_LOCK_TARGET",
        OpenInventory = 2 => "GAME_ACTION_OPEN_INVENTORY",
        OpenInGameMenu = 3 => "GAME_ACTION_OPEN_IN_GAME_MENU",
        ToggleMiniMap = 4 => "GAME_ACTION_TOGGLE_MINI_MAP",
        Pause = 5 => "GAME_ACTION_PAUSE",
        Interact = 6 => "GAME_ACTION_INTERACT",
        Block = 7 => "GAME_ACTION_BLOCK",
        SpecialAttack = 8 => "GAME_ACTION_SPECIAL_ATTACK",
        Attack = 9 => "GAME_ACTION_ATTACK",
        FireRangedWeapon = 10 => "GAME_ACTION_FIRE_RANGED_WEAPON",
        UnarmedAttack = 11 => "GAME_ACTION_UNARMED_ATTACK",
        ArmedMeleeAttack = 12 => "GAME_ACTION_ARMED_MELEE_ATTACK",
        UnsheatheMeleeWeapon = 13 => "GAME_ACTION_UNSHEATHE_MELEE_WEAPON",
        UnsheatheRangedWeapon = 14 => "GAME_ACTION_UNSHEATHE_RANGED_WEAPON",
        SheatheMeleeWeapon = 15 => "GAME_ACTION_SHEATHE_MELEE_WEAPON",
        SheatheRangedWeapon = 16 => "GAME_ACTION_SHEATHE_RANGED_WEAPON",
        ToggleSilentMove = 17 => "GAME_ACTION_TOGGLE_SILENT_MOVE",
        ToggleCinematicAndUserCamera = 18 => "GAME_ACTION_TOGGLE_CINEMATIC_AND_USER_CAMERA",
        ToggleFirstPersonView = 19 => "GAME_ACTION_TOGGLE_FIRST_PERSON_VIEW",
        SkipPastText = 20 => "GAME_ACTION_SKIP_PAST_TEXT",
        SkipCutScene = 21 => "GAME_ACTION_SKIP_CUT_SCENE",
        AnswerQuestionYes = 22 => "GAME_ACTION_ANSWER_QUESTION_YES",
        AnswerQuestionNo = 23 => "GAME_ACTION_ANSWER_QUESTION_NO",
        AnswerQuestionThird = 24 => "GAME_ACTION_ANSWER_QUESTION_THIRD",
        InventorySelect = 25 => "GAME_ACTION_INVENTORY_SELECT",
        AttractExperienceOrbs = 26 => "GAME_ACTION_ATTRACT_EXPERIENCE_ORBS",
        RotateGuiScreensLeft = 27 => "GAME_ACTION_ROTATE_GUI_SCREENS_LEFT",
        RotateGuiScreensRight = 28 => "GAME_ACTION_ROTATE_GUI_SCREENS_RIGHT",
        Jump = 29 => "GAME_ACTION_JUMP",
        Sprint = 30 => "GAME_ACTION_SPRINT",
        Run = 31 => "GAME_ACTION_RUN",
        Sneak = 32 => "GAME_ACTION_SNEAK",
        InventoryA = 33 => "GAME_ACTION_INVENTORY_A",
        InventoryB = 34 => "GAME_ACTION_INVENTORY_B",
        InventoryX = 35 => "GAME_ACTION_INVENTORY_X",
        InventoryY = 36 => "GAME_ACTION_INVENTORY_Y",
        InventoryUp = 37 => "GAME_ACTION_INVENTORY_UP",
        InventoryDown = 38 => "GAME_ACTION_INVENTORY_DOWN",
        InventoryLeft = 39 => "GAME_ACTION_INVENTORY_LEFT",
        InventoryRight = 40 => "GAME_ACTION_INVENTORY_RIGHT",
        InventoryWhite = 41 => "GAME_ACTION_INVENTORY_WHITE",
        TavernGamesInstructions = 42 => "GAME_ACTION_TAVERN_GAMES_INSTRUCTIONS",
        FishingReelIn = 43 => "GAME_ACTION_FISHING_REEL_IN",
        FishingCancel = 44 => "GAME_ACTION_FISHING_CANCEL",
        ToggleFirstPersonTargeting = 45 => "GAME_ACTION_TOGGLE_FIRST_PERSON_TARGETING",
        FirstPersonTargetLock = 46 => "GAME_ACTION_FIRST_PERSON_TARGET_LOCK",
        FirstPersonZoomIn = 47 => "GAME_ACTION_FIRST_PERSON_ZOOM_IN",
        GeneralLeavePlayerMode = 48 => "GAME_ACTION_GENERAL_LEAVE_PLAYER_MODE",
        DebugJump1 = 49 => "GAME_ACTION_DEBUG_JUMP_1",
        DebugJump2 = 50 => "GAME_ACTION_DEBUG_JUMP_2",
        DebugCamera = 51 => "GAME_ACTION_DEBUG_CAMERA",
        DebugShift = 52 => "GAME_ACTION_DEBUG_SHIFT",
        TakePhotoForPhotojournal = 53 => "GAME_ACTION_TAKE_PHOTO_FOR_PHOTOJOURNAL",
        AssignableSpecialMove = 54 => "GAME_ACTION_ASSIGNABLE_SPECIAL_MOVE",
        QuickAccessItem = 55 => "GAME_ACTION_QUICK_ACCESS_ITEM",
        ContextSensitiveItem = 56 => "GAME_ACTION_CONTEXT_SENSITIVE_ITEM",
        CycleThroughSpells = 57 => "GAME_ACTION_CYCLE_THROUGH_SPELLS",
        CoinGolfCancelAim = 58 => "GAME_ACTION_COIN_GOLF_CANCEL_AIM",
        ConfirmResetToFrontEnd = 59 => "GAME_ACTION_CONFIRM_RESET_TO_FRONT_END",
        Movement = 60 => "GAME_ACTION_MOVEMENT",
        CameraRotate = 61 => "GAME_ACTION_CAMERA_ROTATE",
        CameraRotateLeft = 62 => "GAME_ACTION_CAMERA_ROTATE_LEFT",
        CameraRotateRight = 63 => "GAME_ACTION_CAMERA_ROTATE_RIGHT",
        CameraZoomIn = 64 => "GAME_ACTION_CAMERA_ZOOM_IN",
        CameraZoomOut = 65 => "GAME_ACTION_CAMERA_ZOOM_OUT",
        OracleMinigameUp = 66 => "GAME_ACTION_ORACLE_MINIGAME_UP",
        OracleMinigameDown = 67 => "GAME_ACTION_ORACLE_MINIGAME_DOWN",
        OracleMinigameLeft = 68 => "GAME_ACTION_ORACLE_MINIGAME_LEFT",
        OracleMinigameRight = 69 => "GAME_ACTION_ORACLE_MINIGAME_RIGHT",
        OracleMinigameQuit = 70 => "GAME_ACTION_ORACLE_MINIGAME_QUIT",
        MoveMouseOnGui = 71 => "GAME_ACTION_MOVE_MOUSE_ON_GUI",
        ToggleLiveGui = 72 => "GAME_ACTION_TOGGLE_LIVE_GUI",
        OpenExpressionMenu = 73 => "GAME_ACTION_OPEN_EXPRESSION_MENU",
        ToggleDeactivateLockTarget = 74 => "GAME_ACTION_TOGGLE_DEACTIVATE_LOCK_TARGET",
        FirstPersonLookaround = 75 => "GAME_ACTION_FIRST_PERSON_LOOKAROUND",
        InventoryUnselect = 76 => "GAME_ACTION_INVENTORY_UNSELECT",
        CameraMoveDoubleAxis = 77 => "GAME_ACTION_CAMERA_MOVE_DOUBLE_AXIS",
        ChargeGuildSeal = 78 => "GAME_ACTION_CHARGE_GUILD_SEAL",
        TavernGameMovement = 79 => "GAME_ACTION_TAVERN_GAME_MOVEMENT",
        TavernGameCamera = 80 => "GAME_ACTION_TAVERN_GAME_CAMERA",
        TavernGameActionButton = 81 => "GAME_ACTION_TAVERN_GAME_ACTION_BUTTON",
        TavernGameAlternateButton = 82 => "GAME_ACTION_TAVERN_GAME_ALTERNATE_BUTTON",
        TavernGameQuit = 83 => "GAME_ACTION_TAVERN_GAME_QUIT",
        ProjectileTargetingAnalogueZoom = 84 => "GAME_ACTION_PROJECTILE_TARGETING_ANALOGUE_ZOOM",
        TogglePassiveAggressiveMode = 85 => "GAME_ACTION_TOGGLE_PASSIVE_AGGRESSIVE_MODE",
        ActivateSpellMode = 86 => "GAME_ACTION_ACTIVATE_SPELL_MODE",
        ExpressionShift = 87 => "GAME_ACTION_EXPRESSION_SHIFT",
        ScrollDescriptionUp = 88 => "GAME_ACTION_SCROLL_DESCRIPTION_UP",
        ScrollDescriptionDown = 89 => "GAME_ACTION_SCROLL_DESCRIPTION_DOWN",
        OpenWeaponsMenu = 90 => "GAME_ACTION_OPEN_WEAPONS_MENU",
        OpenClothingMenu = 91 => "GAME_ACTION_OPEN_CLOTHING_MENU",
        OpenItemsMenu = 92 => "GAME_ACTION_OPEN_ITEMS_MENU",
        OpenCurrentQuestsMenu = 93 => "GAME_ACTION_OPEN_CURRENT_QUESTS_MENU",
        CycleThroughSpellsKeyboard = 94 => "GAME_ACTION_CYCLE_THROUGH_SPELLS_KEYBOARD",
        ToggleKillEverythingMode = 95 => "GAME_ACTION_TOGGLE_KILL_EVERYTHING_MODE",
        OpenMagicMenu = 96 => "GAME_ACTION_OPEN_MAGIC_MENU",
        OpenExpressionsMenu = 97 => "GAME_ACTION_OPEN_EXPRESSIONS_MENU",
        OpenPersonalityMenu = 98 => "GAME_ACTION_OPEN_PERSONALITY_MENU",
        OpenLogbookMenu = 99 => "GAME_ACTION_OPEN_LOGBOOK_MENU",
        OpenMapMenu = 100 => "GAME_ACTION_OPEN_MAP_MENU",
        ScrollMenu = 101 => "GAME_ACTION_SCROLL_MENU",
        ZoomMapOut = 102 => "GAME_ACTION_ZOOM_MAP_OUT",
        ZoomMapIn = 103 => "GAME_ACTION_ZOOM_MAP_IN",
        ScrollMapLeft = 104 => "GAME_ACTION_SCROLL_MAP_LEFT",
        ScrollMapRight = 105 => "GAME_ACTION_SCROLL_MAP_RIGHT",
        ScrollMapDown = 106 => "GAME_ACTION_SCROLL_MAP_DOWN",
        ScrollMapUp = 107 => "GAME_ACTION_SCROLL_MAP_UP",
        OptionsSliderLeft = 108 => "GAME_ACTION_OPTIONS_SLIDER_LEFT",
        OptionsSliderRight = 109 => "GAME_ACTION_OPTIONS_SLIDER_RIGHT",
        DigitalAnalogueZoomIn = 110 => "GAME_ACTION_DIGITAL_ANALOGUE_ZOOM_IN",
        DigitalAnalogueZoomOut = 111 => "GAME_ACTION_DIGITAL_ANALOGUE_ZOOM_OUT",
        ToggleViewHeroMode = 112 => "GAME_ACTION_TOGGLE_VIEW_HERO_MODE",
        CentreCamera = 113 => "GAME_ACTION_CENTRE_CAMERA",
        Betting = 114 => "GAME_ACTION_BETTING",
        Count = 115 => "GAME_ACTION_COUNT",
        // Anniversary retail additions (values 116-127, names not recovered)
        GameAction116 = 116 => "GAME_ACTION_116",
        GameAction117 = 117 => "GAME_ACTION_117",
        GameAction118 = 118 => "GAME_ACTION_118",
        GameAction119 = 119 => "GAME_ACTION_119",
        GameAction120 = 120 => "GAME_ACTION_120",
        GameAction121 = 121 => "GAME_ACTION_121",
        GameAction122 = 122 => "GAME_ACTION_122",
        GameAction123 = 123 => "GAME_ACTION_123",
        GameAction124 = 124 => "GAME_ACTION_124",
        GameAction125 = 125 => "GAME_ACTION_125",
        GameAction126 = 126 => "GAME_ACTION_126",
        GameAction127 = 127 => "GAME_ACTION_127",
    }
}

def_enum! {
    /// Input controller kind.
    ///
    /// C++ `EControllerType`.
    pub enum ControllerType: i32 {
        None = 0 => "CONTROLLER_NONE",
        XboxPad = 1 => "CONTROLLER_XBOX_PAD",
        Keyboard = 2 => "CONTROLLER_KEYBOARD",
        Mouse = 3 => "CONTROLLER_MOUSE",
    }
}

def_enum! {
    /// Xbox pad button.
    ///
    /// C++ `EXboxControllerButton`.
    pub enum XboxControllerButton: i32 {
        UndefinedButton = 0 => "XBOX_PAD_UNDEFINED_BUTTON",
        XButton = 1 => "XBOX_PAD_X_BUTTON",
        YButton = 2 => "XBOX_PAD_Y_BUTTON",
        BlackButton = 3 => "XBOX_PAD_BLACK_BUTTON",
        AButton = 4 => "XBOX_PAD_A_BUTTON",
        BButton = 5 => "XBOX_PAD_B_BUTTON",
        WhiteButton = 6 => "XBOX_PAD_WHITE_BUTTON",
        LeftTrigger = 7 => "XBOX_PAD_LEFT_TRIGGER",
        RightTrigger = 8 => "XBOX_PAD_RIGHT_TRIGGER",
        LeftStickButton = 9 => "XBOX_PAD_LEFT_STICK_BUTTON",
        RightStickButton = 10 => "XBOX_PAD_RIGHT_STICK_BUTTON",
        StartButton = 11 => "XBOX_PAD_START_BUTTON",
        BackButton = 12 => "XBOX_PAD_BACK_BUTTON",
        DpadUpButton = 13 => "XBOX_PAD_DPAD_UP_BUTTON",
        DpadDownButton = 14 => "XBOX_PAD_DPAD_DOWN_BUTTON",
        DpadLeftButton = 15 => "XBOX_PAD_DPAD_LEFT_BUTTON",
        DpadRightButton = 16 => "XBOX_PAD_DPAD_RIGHT_BUTTON",
        LeftAnalogueStick = 17 => "XBOX_PAD_LEFT_ANALOGUE_STICK",
        RightAnalogueStick = 18 => "XBOX_PAD_RIGHT_ANALOGUE_STICK",
    }
}

def_enum! {
    /// Mouse button or movement binding.
    ///
    /// C++ `EMouseButtonControl`.
    pub enum MouseButtonControl: i32 {
        ButtonNullControl = 0 => "MOUSE_BUTTON_NULL_CONTROL",
        ButtonLeftControl = 1 => "MOUSE_BUTTON_LEFT_CONTROL",
        ButtonRightControl = 2 => "MOUSE_BUTTON_RIGHT_CONTROL",
        ButtonMiddleControl = 3 => "MOUSE_BUTTON_MIDDLE_CONTROL",
        Movement = 4 => "MOUSE_MOVEMENT",
        WheelMovement = 5 => "MOUSE_WHEEL_MOVEMENT",
        WheelMovementUp = 6 => "MOUSE_WHEEL_MOVEMENT_UP",
        WheelMovementDown = 7 => "MOUSE_WHEEL_MOVEMENT_DOWN",
        Button4Control = 8 => "MOUSE_BUTTON_4_CONTROL",
        Button5Control = 9 => "MOUSE_BUTTON_5_CONTROL",
        Button6Control = 10 => "MOUSE_BUTTON_6_CONTROL",
        Button7Control = 11 => "MOUSE_BUTTON_7_CONTROL",
        Button8Control = 12 => "MOUSE_BUTTON_8_CONTROL",
    }
}

def_enum! {
    /// Keyboard key binding.
    ///
    /// C++ `EInputKey` (`Data/Defs/keyboard_keys.h`). The `NO_INPUT_KEYS` count
    /// enumerator is omitted.
    pub enum InputKey: i32 {
        Null = 0 => "KB_NULL",
        Esc = 1 => "KB_ESC",
        Num1 = 2 => "KB_1",
        Num2 = 3 => "KB_2",
        Num3 = 4 => "KB_3",
        Num4 = 5 => "KB_4",
        Num5 = 6 => "KB_5",
        Num6 = 7 => "KB_6",
        Num7 = 8 => "KB_7",
        Num8 = 9 => "KB_8",
        Num9 = 10 => "KB_9",
        Num0 = 11 => "KB_0",
        Minus = 12 => "KB_MINUS",
        Equals = 13 => "KB_EQUALS",
        Backspace = 14 => "KB_BACKSPACE",
        Tab = 15 => "KB_TAB",
        Q = 16 => "KB_Q",
        W = 17 => "KB_W",
        E = 18 => "KB_E",
        R = 19 => "KB_R",
        T = 20 => "KB_T",
        Y = 21 => "KB_Y",
        U = 22 => "KB_U",
        I = 23 => "KB_I",
        O = 24 => "KB_O",
        P = 25 => "KB_P",
        Lbracket = 26 => "KB_LBRACKET",
        Rbracket = 27 => "KB_RBRACKET",
        Return = 28 => "KB_RETURN",
        Lcontrol = 29 => "KB_LCONTROL",
        A = 30 => "KB_A",
        S = 31 => "KB_S",
        D = 32 => "KB_D",
        F = 33 => "KB_F",
        G = 34 => "KB_G",
        H = 35 => "KB_H",
        J = 36 => "KB_J",
        K = 37 => "KB_K",
        L = 38 => "KB_L",
        Semicolon = 39 => "KB_SEMICOLON",
        Apostrophe = 40 => "KB_APOSTROPHE",
        Hash = 41 => "KB_HASH",
        Lshift = 42 => "KB_LSHIFT",
        Backslash = 43 => "KB_BACKSLASH",
        Z = 44 => "KB_Z",
        X = 45 => "KB_X",
        C = 46 => "KB_C",
        V = 47 => "KB_V",
        B = 48 => "KB_B",
        N = 49 => "KB_N",
        M = 50 => "KB_M",
        Comma = 51 => "KB_COMMA",
        Fullstop = 52 => "KB_FULLSTOP",
        Slash = 53 => "KB_SLASH",
        Rshift = 54 => "KB_RSHIFT",
        Pmultiply = 55 => "KB_PMULTIPLY",
        Lalt = 56 => "KB_LALT",
        Space = 57 => "KB_SPACE",
        Capslock = 58 => "KB_CAPSLOCK",
        F1 = 59 => "KB_F1",
        F2 = 60 => "KB_F2",
        F3 = 61 => "KB_F3",
        F4 = 62 => "KB_F4",
        F5 = 63 => "KB_F5",
        F6 = 64 => "KB_F6",
        F7 = 65 => "KB_F7",
        F8 = 66 => "KB_F8",
        F9 = 67 => "KB_F9",
        F10 = 68 => "KB_F10",
        Numlock = 69 => "KB_NUMLOCK",
        Scrolllock = 70 => "KB_SCROLLLOCK",
        P7 = 71 => "KB_P7",
        P8 = 72 => "KB_P8",
        P9 = 73 => "KB_P9",
        Pminus = 74 => "KB_PMINUS",
        P4 = 75 => "KB_P4",
        P5 = 76 => "KB_P5",
        P6 = 77 => "KB_P6",
        Pplus = 78 => "KB_PPLUS",
        P1 = 79 => "KB_P1",
        P2 = 80 => "KB_P2",
        P3 = 81 => "KB_P3",
        P0 = 82 => "KB_P0",
        Pfullstop = 83 => "KB_PFULLSTOP",
        F11 = 84 => "KB_F11",
        F12 = 85 => "KB_F12",
        F13 = 86 => "KB_F13",
        F14 = 87 => "KB_F14",
        F15 = 88 => "KB_F15",
        Kana = 89 => "KB_KANA",
        Convert = 90 => "KB_CONVERT",
        Noconvert = 91 => "KB_NOCONVERT",
        Yen = 92 => "KB_YEN",
        Pequals = 93 => "KB_PEQUALS",
        Circumflex = 94 => "KB_CIRCUMFLEX",
        At = 95 => "KB_AT",
        Colon = 96 => "KB_COLON",
        Underline = 97 => "KB_UNDERLINE",
        Kanji = 98 => "KB_KANJI",
        Stop = 99 => "KB_STOP",
        Ax = 100 => "KB_AX",
        Unlabeled = 101 => "KB_UNLABELED",
        Penter = 102 => "KB_PENTER",
        Rcontrol = 103 => "KB_RCONTROL",
        Pcomma = 104 => "KB_PCOMMA",
        Pdivide = 105 => "KB_PDIVIDE",
        Sysrq = 106 => "KB_SYSRQ",
        Ralt = 107 => "KB_RALT",
        Home = 108 => "KB_HOME",
        Up = 109 => "KB_UP",
        Pageup = 110 => "KB_PAGEUP",
        Left = 111 => "KB_LEFT",
        Right = 112 => "KB_RIGHT",
        End = 113 => "KB_END",
        Down = 114 => "KB_DOWN",
        Pagedown = 115 => "KB_PAGEDOWN",
        Insert = 116 => "KB_INSERT",
        Delete = 117 => "KB_DELETE",
        Lwin = 118 => "KB_LWIN",
        Rwin = 119 => "KB_RWIN",
        Apps = 120 => "KB_APPS",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_round_trip_and_symbols() {
        assert_eq!(UiType::from_i32(32), Some(UiType::MousePointer));
        assert_eq!(UiType::MousePointer.to_i32(), 32);
        assert_eq!(UiType::MousePointer.symbol(), "UI_TYPE_MOUSE_POINTER");
        assert_eq!(UiType::from_symbol("UI_TYPE_MOUSE_POINTER"), Some(UiType::MousePointer));
        assert_eq!(UiType::from_i32(99), None);
    }

    #[test]
    fn controller_dispatch_values() {
        // The CActionInputControl slot dispatch relies on these exact values.
        assert_eq!(ControllerType::XboxPad.to_i32(), 1);
        assert_eq!(ControllerType::Keyboard.to_i32(), 2);
        assert_eq!(ControllerType::Mouse.to_i32(), 3);
    }

    #[test]
    fn keyboard_keys_sequential_start() {
        assert_eq!(InputKey::from_i32(0), Some(InputKey::Null));
        assert_eq!(InputKey::from_i32(12), Some(InputKey::Minus));
    }

    #[test]
    fn flags_lossless() {
        // game.bin contains OR'd expansion values; must round-trip.
        let both = TableExpansion::from_i32(3);
        assert!(both.contains(TableExpansion::HORIZONTAL));
        assert!(both.contains(TableExpansion::VERTICAL));
        assert_eq!(both.to_i32(), 3);
        assert_eq!((TableExpansion::HORIZONTAL | TableExpansion::VERTICAL).to_i32(), 3);
        assert!(Sprite2dFlags::from_i32(0).is_empty());
    }
}

// ── generated game.bin enums ──────────────────────────────────

def_enum! {
    /// C++ `EActionRegisteredType`.
    pub enum ActionRegisteredType: i32 {
        NULL = 0 => "ACTION_NULL",
        JACKOFBLADESHITRESPONSE = 1 => "ACTION_JACK_OF_BLADES_HIT_RESPONSE",
        SCORPIONKINGHITRESPONSE = 2 => "ACTION_SCORPION_KING_HIT_RESPONSE",
        MAZECAUSEFORCEPUSHHITRESPONSE = 3 => "ACTION_MAZE_CAUSE_FORCE_PUSH_HIT_RESPONSE",
        ROCKTROLLDEATH = 4 => "ACTION_ROCK_TROLL_DEATH",
        KINGSCORPIONDEATH = 5 => "ACTION_KING_SCORPION_DEATH",
        WASPQUEENDEATH = 6 => "ACTION_WASP_QUEEN_DEATH",
        COMBATGENERICPROJECTILEWEAPONAIM = 7 => "ACTION_COMBAT_GENERIC_PROJECTILE_WEAPON_AIM",
        COMBATGENERICPROJECTILEWEAPONFIRE = 8 => "ACTION_COMBAT_GENERIC_PROJECTILE_WEAPON_FIRE",
        COMBATGENERICPROJECTILEWEAPONLOAD = 9 => "ACTION_COMBAT_GENERIC_PROJECTILE_WEAPON_LOAD",
        COMBATGENERICLEADERCOMMAND = 10 => "ACTION_COMBAT_GENERIC_LEADER_COMMAND",
        COMBATGENERICBOAST = 11 => "ACTION_COMBAT_GENERIC_BOAST",
        COMBATUNBLOCKABLEATTACK = 12 => "ACTION_COMBAT_UNBLOCKABLE_ATTACK",
        COMBATHOPBACK = 13 => "ACTION_COMBAT_HOP_BACK",
        COMBATSTRAFEFORWARD = 14 => "ACTION_COMBAT_STRAFE_FORWARD",
        COMBATSTRAFEBACKWARD = 15 => "ACTION_COMBAT_STRAFE_BACKWARD",
        COMBATSTRAFELEFT = 16 => "ACTION_COMBAT_STRAFE_LEFT",
        COMBATSTRAFERIGHT = 17 => "ACTION_COMBAT_STRAFE_RIGHT",
        COMBATSTRAFEBACKLEFT = 18 => "ACTION_COMBAT_STRAFE_BACK_LEFT",
        COMBATSTRAFEBACKRIGHT = 19 => "ACTION_COMBAT_STRAFE_BACK_RIGHT",
        COMBATCHARGE = 20 => "ACTION_COMBAT_CHARGE",
        COMBATIDLE = 21 => "ACTION_COMBAT_IDLE",
        BREAKINTOMELEE = 22 => "ACTION_BREAK_INTO_MELEE",
        COMBATSUMMONCREATURES = 23 => "ACTION_COMBAT_SUMMON_CREATURES",
        COMBATATTACKLUNGE = 24 => "ACTION_COMBAT_ATTACK_LUNGE",
        COMBATATTACKMAIN = 25 => "ACTION_COMBAT_ATTACK_MAIN",
        COMBATATTACKKNOCKDOWN = 26 => "ACTION_COMBAT_ATTACK_KNOCKDOWN",
        COMBATATTACKSIDE = 27 => "ACTION_COMBAT_ATTACK_SIDE",
        COMBATATTACKSHORTRANGE = 28 => "ACTION_COMBAT_ATTACK_SHORT_RANGE",
        BANDITKINGHITRESPONSE = 29 => "ACTION_BANDIT_KING_HIT_RESPONSE",
        BANDITKINGSTUCKHITRESPONSE = 30 => "ACTION_BANDIT_KING_STUCK_HIT_RESPONSE",
        HOBBESPELLCASTERAIM = 31 => "ACTION_HOBBE_SPELLCASTER_AIM",
        HOBBESPELLCASTERFIRE = 32 => "ACTION_HOBBE_SPELLCASTER_FIRE",
        HOBBELUNGE = 33 => "ACTION_HOBBE_LUNGE",
        TENTACLEHITRESPONSE = 34 => "ACTION_TENTACLE_HIT_RESPONSE",
        SCREAMERDIE = 35 => "ACTION_SCREAMER_DIE",
        SCREAMERDRAINATTACK = 36 => "ACTION_SCREAMER_DRAIN_ATTACK",
        SCREAMERDRAINOUTOF = 37 => "ACTION_SCREAMER_DRAIN_OUT_OF",
        SCREAMERADVANCE = 38 => "ACTION_SCREAMER_ADVANCE",
        SCREAMERBACKOFF = 39 => "ACTION_SCREAMER_BACK_OFF",
        SCREAMERIDLE = 40 => "ACTION_SCREAMER_IDLE",
        COMBATBODGESIDEATTACK = 41 => "ACTION_COMBAT_BODGE_SIDE_ATTACK",
        COMBATTURNSTRIKELEFT = 42 => "ACTION_COMBAT_TURN_STRIKE_LEFT",
        COMBATTURNSTRIKERIGHT = 43 => "ACTION_COMBAT_TURN_STRIKE_RIGHT",
        COMBATCHARGESTRIKE = 44 => "ACTION_COMBAT_CHARGE_STRIKE",
        NYMPHGETHIT = 45 => "ACTION_NYMPH_GET_HIT",
        NYMPHGETHITDIE = 46 => "ACTION_NYMPH_GET_HIT_DIE",
        BALVERINELUNGEATTACK = 47 => "ACTION_BALVERINE_LUNGE_ATTACK",
        BALVERINEBREAKOFFFROMCOMBAT = 48 => "ACTION_BALVERINE_BREAK_OFF_FROM_COMBAT",
        BALVERINEBREAKOFFFROMCOMBATLONG = 49 => "ACTION_BALVERINE_BREAK_OFF_FROM_COMBAT_LONG",
        SUMMONERFLAMESLICE = 50 => "ACTION_SUMMONER_FLAME_SLICE",
        SUMMONERUNSHEATHESTRIKE = 51 => "ACTION_SUMMONER_UNSHEATHE_STRIKE",
        SUMMONERSTRIKE = 52 => "ACTION_SUMMONER_STRIKE",
        BATTLECHARGE = 53 => "ACTION_BATTLE_CHARGE",
        SUMMONERDIE = 54 => "ACTION_SUMMONER_DIE",
    }
}

def_enum! {
    /// C++ `EClockHandType`.
    pub enum ClockHandType: i32 {
        SECOND = 0 => "CLOCKHAND_SECOND",
        MINUTE = 1 => "CLOCKHAND_MINUTE",
        HOUR = 2 => "CLOCKHAND_HOUR",
    }
}

def_enum! {
    /// C++ `EClothingSuitPart`.
    pub enum ClothingSuitPart: i32 {
        NULL = 0 => "CLOTHING_SUIT_NULL",
        HEAD = 1 => "CLOTHING_SUIT_HEAD",
        BODY = 2 => "CLOTHING_SUIT_BODY",
        HANDS = 3 => "CLOTHING_SUIT_HANDS",
        LEGS = 4 => "CLOTHING_SUIT_LEGS",
        FEET = 5 => "CLOTHING_SUIT_FEET",
    }
}

def_enum! {
    /// C++ `ECombatCreatureType`.
    pub enum CombatCreatureType: i32 {
        HERO = 0 => "CREATURE_TYPE_HERO",
        RIVALHERO = 1 => "CREATURE_TYPE_RIVAL_HERO",
        FODDERCREATURE = 2 => "CREATURE_TYPE_FODDER_CREATURE",
        COMBATHUMANOID = 3 => "CREATURE_TYPE_COMBAT_HUMANOID",
        COMBATANIMAL = 4 => "CREATURE_TYPE_COMBAT_ANIMAL",
        VILLAGERMALE = 5 => "CREATURE_TYPE_VILLAGER_MALE",
        VILLAGERFEMALE = 6 => "CREATURE_TYPE_VILLAGER_FEMALE",
        VILLAGERCHILD = 7 => "CREATURE_TYPE_VILLAGER_CHILD",
        GUARD = 8 => "CREATURE_TYPE_GUARD",
    }
}

def_enum! {
    /// C++ `ECombatSequenceInterruptionType`.
    pub enum CombatSequenceInterruptionType: i32 {
        INTERRUPTNULL = 0 => "COMBAT_SEQUENCE_INTERRUPT_NULL",
        INTERRUPTABLE = 1 => "COMBAT_SEQUENCE_INTERRUPTABLE",
        INTERRUPTABLEDUETOZONECHANGE = 2 => "COMBAT_SEQUENCE_INTERRUPTABLE_DUE_TO_ZONE_CHANGE",
    }
}

def_enum! {
    /// C++ `ECombatSequenceIsValidType`.
    pub enum CombatSequenceIsValidType: i32 {
        VALIDNULL = 0 => "COMBAT_SEQUENCE_IS_VALID_NULL",
        VALIDTARGETBLOCKING = 1 => "COMBAT_SEQUENCE_IS_VALID_TARGET_BLOCKING",
        VALIDLOADRANGEDWEAPON = 2 => "COMBAT_SEQUENCE_IS_VALID_LOAD_RANGED_WEAPON",
        VALIDFIREATTARGET = 3 => "COMBAT_SEQUENCE_IS_VALID_FIRE_AT_TARGET",
        VALIDISTARGETHEALTHOVER75 = 4 => "COMBAT_SEQUENCE_IS_VALID_IS_TARGET_HEALTH_OVER_75",
        VALIDISMYHEALTHBELOW30 = 5 => "COMBAT_SEQUENCE_IS_VALID_IS_MY_HEALTH_BELOW_30",
        VALIDISBALVERINEABLETOLUNGE = 6 => "COMBAT_SEQUENCE_IS_VALID_IS_BALVERINE_ABLE_TO_LUNGE",
        VALIDABLETOSUMMON = 7 => "COMBAT_SEQUENCE_IS_VALID_ABLE_TO_SUMMON",
        TARGETINLINEOFSIGHT = 8 => "COMBAT_SEQUENCE_IS_TARGET_IN_LINE_OF_SIGHT",
    }
}

def_enum! {
    /// C++ `ECombatSequenceOnStartModuleType`.
    pub enum CombatSequenceOnStartModuleType: i32 {
        NULL = 0 => "COMBAT_SEQUENCE_ON_START_NULL",
        CONTINUEAIMING = 1 => "COMBAT_SEQUENCE_ON_START_CONTINUE_AIMING",
    }
}

def_enum! {
    /// C++ `ECombatSequenceOnStopModuleType`.
    pub enum CombatSequenceOnStopModuleType: i32 {
        NULL = 0 => "COMBAT_SEQUENCE_ON_STOP_NULL",
    }
}

def_enum! {
    /// C++ `ECombatSequenceType`.
    pub enum CombatSequenceType: i32 {
        NULL = 0 => "COMBAT_SEQUENCE_NULL",
        MELEE = 1 => "COMBAT_SEQUENCE_MELEE",
        BREAKINTOMELEE = 2 => "COMBAT_SEQUENCE_BREAK_INTO_MELEE",
        BOAST = 3 => "COMBAT_SEQUENCE_BOAST",
        LEADER = 4 => "COMBAT_SEQUENCE_LEADER",
    }
}

def_enum! {
    /// C++ `ECombatStrikeRecoilStyle`.
    pub enum CombatStrikeRecoilStyle: i32 {
        RECOILBREAKHANDEDNESS = 0 => "RECOIL_BREAK_HANDEDNESS",
        RECOILMAINTATINHANDEDNESS = 1 => "RECOIL_MAINTATIN_HANDEDNESS",
        RECOILNONE = 2 => "RECOIL_NONE",
        MAXNUMBEROFRECOILTYPES = 3 => "MAX_NUMBER_OF_RECOIL_TYPES",
    }
}

def_enum! {
    /// C++ `ECompositeBlendType`.
    pub enum CompositeBlendType: i32 {
        NULL = 0 => "COMPOSITE_BLEND_NULL",
        ADDITIVE = 1 => "COMPOSITE_BLEND_ADDITIVE",
        ALPHA = 2 => "COMPOSITE_BLEND_ALPHA",
        SOLID = 3 => "COMPOSITE_BLEND_SOLID",
        MULTIPLY = 4 => "COMPOSITE_BLEND_MULTIPLY",
    }
}

def_enum! {
    /// C++ `EContextSensitiveType`.
    pub enum ContextSensitiveType: i32 {
        NULL = 0 => "CONTEXT_SENSITIVE_NULL",
        GUILDSEAL = 1 => "CONTEXT_SENSITIVE_GUILD_SEAL",
        LAMP = 2 => "CONTEXT_SENSITIVE_LAMP",
        HEALTH = 3 => "CONTEXT_SENSITIVE_HEALTH",
        MANA = 4 => "CONTEXT_SENSITIVE_MANA",
        EXPRESSION = 5 => "CONTEXT_SENSITIVE_EXPRESSION",
        OPINIONEXPRESSION = 6 => "CONTEXT_SENSITIVE_OPINION_EXPRESSION",
        GIFT = 7 => "CONTEXT_SENSITIVE_GIFT",
        MARKER = 8 => "CONTEXT_SENSITIVE_MARKER",
        TROPHY = 9 => "CONTEXT_SENSITIVE_TROPHY",
        SCRIPT = 10 => "CONTEXT_SENSITIVE_SCRIPT",
    }
}

def_enum! {
    /// C++ `EControlledMovementType`.
    pub enum ControlledMovementType: i32 {
        NULL = 0 => "CONTROLLED_MOVEMENT_NULL",
        WALKING = 1 => "CONTROLLED_MOVEMENT_WALKING",
        FLYING = 2 => "CONTROLLED_MOVEMENT_FLYING",
        FIRSTPERSON = 3 => "CONTROLLED_MOVEMENT_FIRST_PERSON",
    }
}

def_enum! {
    /// C++ `ECreatureAbility`.
    pub enum CreatureAbility: i32 {
        CREATUREABILITYTYPEATTACK = 0 => "CREATURE_ABILITY_TYPE_ATTACK",
        CREATUREABILITYTYPEFLOURISH = 1 => "CREATURE_ABILITY_TYPE_FLOURISH",
        CREATUREABILITYTYPEFLOURISH360CW = 2 => "CREATURE_ABILITY_TYPE_FLOURISH_360_CW",
        CREATUREABILITYTYPEFLOURISH360ACW = 3 => "CREATURE_ABILITY_TYPE_FLOURISH_360_ACW",
        CREATUREABILITYTYPEFLOURISHUPTHRUSTLEFT = 4 => "CREATURE_ABILITY_TYPE_FLOURISH_UPTHRUST_LEFT",
        CREATUREABILITYTYPEFLOURISHUPTHRUSTRIGHT = 5 => "CREATURE_ABILITY_TYPE_FLOURISH_UPTHRUST_RIGHT",
        CREATUREABILITYTYPEFLOURISHMAXIMUMDAMAGE = 6 => "CREATURE_ABILITY_TYPE_FLOURISH_MAXIMUM_DAMAGE",
        CREATUREABILITYTYPEBREAKBLOCK = 7 => "CREATURE_ABILITY_TYPE_BREAK_BLOCK",
        MAXNUMBEROFCREATUREABILITIES = 8 => "MAX_NUMBER_OF_CREATURE_ABILITIES",
    }
}

def_enum! {
    /// C++ `ECreatureGeneratorGenerateType`.
    pub enum CreatureGeneratorGenerateType: i32 {
        NORMAL = 0 => "GENERATE_NORMAL",
        AMBUSHDROPIN = 1 => "GENERATE_AMBUSH_DROP_IN",
        AMBUSHJUMPOUT = 2 => "GENERATE_AMBUSH_JUMP_OUT",
        GENERATORANIMATION = 3 => "GENERATE_GENERATOR_ANIMATION",
        UNDEAD = 4 => "GENERATE_UNDEAD",
    }
}

def_flags! {
    /// C++ `ECreatureInteractionType`.
    pub struct CreatureInteractionType: i32 {
        NULL = 0 => "CREATURE_INTERACTION_NULL",
        CONVERSATION = 1 => "CREATURE_INTERACTION_CONVERSATION",
        TAG = 2 => "CREATURE_INTERACTION_TAG",
        MULTI_TAG = 4 => "CREATURE_INTERACTION_MULTI_TAG",
        PURCHASING = 8 => "CREATURE_INTERACTION_PURCHASING",
    }
}

def_enum! {
    /// C++ `ECreatureType`.
    pub enum CreatureType: i32 {
        NOTHUMAN = 0 => "NOT_HUMAN",
        HUMANCHILD = 1 => "HUMAN_CHILD",
        HUMANADULT = 2 => "HUMAN_ADULT",
        HUMANELDERLY = 3 => "HUMAN_ELDERLY",
    }
}

def_enum! {
    /// C++ `ECrimeSeverity`.
    pub enum CrimeSeverity: i32 {
        NONE = 0 => "CRIME_SEVERITY_NONE",
        MINOR = 1 => "CRIME_SEVERITY_MINOR",
        MODERATE = 2 => "CRIME_SEVERITY_MODERATE",
        SERIOUS = 3 => "CRIME_SEVERITY_SERIOUS",
    }
}

def_enum! {
    /// C++ `EDamageAttribute`.
    pub enum DamageAttribute: i32 {
        NULL = -1 => "DAMAGE_NULL",
        MELEE = 0 => "DAMAGE_MELEE",
        MELEEUNARMED = 1 => "DAMAGE_MELEE_UNARMED",
        LIGHTNING = 2 => "DAMAGE_LIGHTNING",
        FIRE = 3 => "DAMAGE_FIRE",
        PROJECTILE = 4 => "DAMAGE_PROJECTILE",
        EXPLOSION = 5 => "DAMAGE_EXPLOSION",
        DRAIN = 6 => "DAMAGE_DRAIN",
        DRAINHEAL = 7 => "DAMAGE_DRAIN_HEAL",
        GENERICWILL = 8 => "DAMAGE_GENERIC_WILL",
        DIVINEWRATH = 9 => "DAMAGE_DIVINE_WRATH",
        UNHOLYPOWER = 10 => "DAMAGE_UNHOLY_POWER",
    }
}

def_enum! {
    /// C++ `EDoorTriggerType`.
    pub enum DoorTriggerType: i32 {
        ONPERSON = 0 => "DOOR_TRIGGER_ON_PERSON",
        MANUAL = 1 => "DOOR_TRIGGER_MANUAL",
    }
}

def_enum! {
    /// C++ `EExpressionInventoryType`.
    pub enum ExpressionInventoryType: i32 {
        EXPRESSIONINVENTORYSOCIAL = 0 => "EXPRESSION_INVENTORY_SOCIAL",
        EXPRESSIONINVENTORYRENOWN = 1 => "EXPRESSION_INVENTORY_RENOWN",
        EXPRESSIONINVENTORYALIGNMENT = 2 => "EXPRESSION_INVENTORY_ALIGNMENT",
        EXPRESSIONINVENTORYSTEALTH = 3 => "EXPRESSION_INVENTORY_STEALTH",
        NUMINVENTORYEXPRESSIONS = 4 => "NUM_INVENTORY_EXPRESSIONS",
    }
}

def_enum! {
    /// C++ `EFeatAttackType`.
    pub enum FeatAttackType: i32 {
        ANY = 0 => "FAT_ATTACK_ANY",
        SWORD = 1 => "FAT_ATTACK_SWORD",
        BOW = 2 => "FAT_ATTACK_BOW",
        HANDS = 3 => "FAT_ATTACK_HANDS",
        WILL = 4 => "FAT_ATTACK_WILL",
    }
}

def_enum! {
    /// C++ `EGameEventType`.
    pub enum GameEventType: i32 {
        GAMEEVENTNULL = 0 => "GAME_EVENT_NULL",
        GAMEEVENTUPDATEFRAME = 1 => "GAME_EVENT_UPDATE_FRAME",
        GAMEEVENTQUIT = 2 => "GAME_EVENT_QUIT",
        GAMEEVENTSETEXCLUSIVEMODE = 3 => "GAME_EVENT_SET_EXCLUSIVE_MODE",
        GAMEEVENTSETDISPLAYMODE = 4 => "GAME_EVENT_SET_DISPLAY_MODE",
        GAMEEVENTSETEDITORMODE = 5 => "GAME_EVENT_SET_EDITOR_MODE",
        GAMEEVENTFIRSTPERSONVIEWSTART = 6 => "GAME_EVENT_FIRST_PERSON_VIEW_START",
        GAMEEVENTCREATURECHEAT = 7 => "GAME_EVENT_CREATURE_CHEAT",
        GAMEEVENTPLAYERRESPAWN = 8 => "GAME_EVENT_PLAYER_RESPAWN",
        GAMEEVENTSETPAUSEMODE = 9 => "GAME_EVENT_SET_PAUSE_MODE",
        GAMEEVENTSETSLOWMOTION = 10 => "GAME_EVENT_SET_SLOW_MOTION",
        GAMEEVENTSETFREECAMERAMODE = 11 => "GAME_EVENT_SET_FREE_CAMERA_MODE",
        GAMEEVENTUSEFREECAMERA = 12 => "GAME_EVENT_USE_FREE_CAMERA",
        GAMEEVENTCONTROLLEDCREATUREBLOCK = 13 => "GAME_EVENT_CONTROLLED_CREATURE_BLOCK",
        GAMEEVENTTEMPWORLDEVENT = 14 => "GAME_EVENT_TEMP_WORLD_EVENT",
        GAMEEVENTAPPLYSCRIPTEDMAPBRUSHES = 15 => "GAME_EVENT_APPLY_SCRIPTED_MAP_BRUSHES",
        GAMEEVENTCREATUREMOVEMENT = 16 => "GAME_EVENT_CREATURE_MOVEMENT",
        GAMEEVENTCREATUREUSEOBJECT = 17 => "GAME_EVENT_CREATURE_USE_OBJECT",
        GAMEEVENTCONTROLLEDCREATURETALK = 18 => "GAME_EVENT_CONTROLLED_CREATURE_TALK",
        GAMEEVENTCONTROLLEDCREATURELEARNEXPRESSION = 19 => "GAME_EVENT_CONTROLLED_CREATURE_LEARN_EXPRESSION",
        GAMEEVENTCONTROLLEDCREATUREDROPOBJECT = 20 => "GAME_EVENT_CONTROLLED_CREATURE_DROP_OBJECT",
        GAMEEVENTCONTROLLEDCREATUREZTARGET = 21 => "GAME_EVENT_CONTROLLED_CREATURE_ZTARGET",
        GAMEEVENTCONTROLLEDCREATUREUSEABILITY = 22 => "GAME_EVENT_CONTROLLED_CREATURE_USE_ABILITY",
        GAMEEVENTCONTROLLEDCREATURESTARTSNEAK = 23 => "GAME_EVENT_CONTROLLED_CREATURE_START_SNEAK",
        GAMEEVENTCONTROLLEDCREATURESTOPSNEAK = 24 => "GAME_EVENT_CONTROLLED_CREATURE_STOP_SNEAK",
        GAMEEVENTCLICKPASTTEXT = 25 => "GAME_EVENT_CLICK_PAST_TEXT",
        GAMEEVENTOPENHEROINFOSCREEN = 26 => "GAME_EVENT_OPEN_HERO_INFO_SCREEN",
        GAMEEVENTCLOSEHEROINFOSCREEN = 27 => "GAME_EVENT_CLOSE_HERO_INFO_SCREEN",
        GAMEEVENTCLOSEINGAMEMENU = 28 => "GAME_EVENT_CLOSE_IN_GAME_MENU",
        GAMEEVENTQUESTIONANSWERED = 29 => "GAME_EVENT_QUESTION_ANSWERED",
        GAMEEVENTCONTROLLEDCREATURESTARTTHROWOBJECT = 30 => "GAME_EVENT_CONTROLLED_CREATURE_START_THROW_OBJECT",
        GAMEEVENTCONTROLLEDCREATUREENDTHROWOBJECT = 31 => "GAME_EVENT_CONTROLLED_CREATURE_END_THROW_OBJECT",
        GAMEEVENTCONTROLLEDCREATURESHEATHEWEAPON = 32 => "GAME_EVENT_CONTROLLED_CREATURE_SHEATHE_WEAPON",
        GAMEEVENTCONTROLLEDCREATUREUNSHEATHEMELEEWEAPON = 33 => "GAME_EVENT_CONTROLLED_CREATURE_UNSHEATHE_MELEE_WEAPON",
        GAMEEVENTCONTROLLEDCREATUREUNSHEATHERANGEDWEAPON = 34 => "GAME_EVENT_CONTROLLED_CREATURE_UNSHEATHE_RANGED_WEAPON",
        GAMEEVENTCONTROLLEDCREATUREPLAYERINTERACTION = 35 => "GAME_EVENT_CONTROLLED_CREATURE_PLAYER_INTERACTION",
        GAMEEVENTCONTROLLEDCREATUREGIVEITEMTOTARGET = 36 => "GAME_EVENT_CONTROLLED_CREATURE_GIVE_ITEM_TO_TARGET",
        GAMEEVENTCONTROLLEDCREATUREUNFREEZECONTROLS = 37 => "GAME_EVENT_CONTROLLED_CREATURE_UNFREEZE_CONTROLS",
        GAMEEVENTCONTROLLEDCREATUREROLL = 38 => "GAME_EVENT_CONTROLLED_CREATURE_ROLL",
        GAMEEVENTUSEQUICKACCESSITEM = 39 => "GAME_EVENT_USE_QUICK_ACCESS_ITEM",
        GAMEEVENTUSEQUICKACCESSITEMINCUTSCENE = 40 => "GAME_EVENT_USE_QUICK_ACCESS_ITEM_IN_CUTSCENE",
        GAMEEVENTCHARGEQUICKACCESSITEM = 41 => "GAME_EVENT_CHARGE_QUICK_ACCESS_ITEM",
        GAMEEVENTPUTAWAY = 42 => "GAME_EVENT_PUT_AWAY",
        GAMEEVENTCREATURESTRAFE = 43 => "GAME_EVENT_CREATURE_STRAFE",
        GAMEEVENTMOVEHEROTOREGION = 44 => "GAME_EVENT_MOVE_HERO_TO_REGION",
        GAMEEVENTFIRSTPERSONTARGETING = 45 => "GAME_EVENT_FIRST_PERSON_TARGETING",
        GAMEEVENTCONTROLLEDCREATUREDEFAULT = 46 => "GAME_EVENT_CONTROLLED_CREATURE_DEFAULT",
        GAMEEVENTEXPRESSIONFOLLOW = 47 => "GAME_EVENT_EXPRESSION_FOLLOW",
        GAMEEVENTEXPRESSIONWAIT = 48 => "GAME_EVENT_EXPRESSION_WAIT",
        GAMEEVENTUSEPROJECTILEWEAPON = 49 => "GAME_EVENT_USE_PROJECTILE_WEAPON",
        GAMEEVENTCONTROLLEDCREATUREJUMP = 50 => "GAME_EVENT_CONTROLLED_CREATURE_JUMP",
        GAMEEVENTSPIRITMOVEMENT = 51 => "GAME_EVENT_SPIRIT_MOVEMENT",
        GAMEEVENTOPENHEROCENTREDOOR = 52 => "GAME_EVENT_OPEN_HERO_CENTRE_DOOR",
        GAMEEVENTCLOSEHEROCENTREDOOR = 53 => "GAME_EVENT_CLOSE_HERO_CENTRE_DOOR",
        GAMEEVENTSPIRITATTACK = 54 => "GAME_EVENT_SPIRIT_ATTACK",
        GAMEEVENTCREATURESPRINT = 55 => "GAME_EVENT_CREATURE_SPRINT",
        GAMEEVENTSPIRITRETURNTOHERO = 56 => "GAME_EVENT_SPIRIT_RETURN_TO_HERO",
        GAMEEVENTSKIPCUTSCENE = 57 => "GAME_EVENT_SKIP_CUT_SCENE",
        GAMEEVENTUSEPROJECTILEWEAPONTHIRDPERSON = 58 => "GAME_EVENT_USE_PROJECTILE_WEAPON_THIRD_PERSON",
        GAMEEVENTCHARGEUPWILLSPELL = 59 => "GAME_EVENT_CHARGE_UP_WILL_SPELL",
        GAMEEVENTLOADGAMEFROMINGAMEMENU = 60 => "GAME_EVENT_LOAD_GAME_FROM_IN_GAME_MENU",
        GAMEEVENTCONTROLLEDCREATUREBLOCKEND = 61 => "GAME_EVENT_CONTROLLED_CREATURE_BLOCK_END",
        GAMEEVENTCONTROLLEDCREATUREDEACTIVATEZTARGET = 62 => "GAME_EVENT_CONTROLLED_CREATURE_DEACTIVATE_ZTARGET",
        GAMEEVENTREMOVECURRENTMODE = 63 => "GAME_EVENT_REMOVE_CURRENT_MODE",
        GAMEEVENTCONTROLLEDCREATURELIGHTNING = 64 => "GAME_EVENT_CONTROLLED_CREATURE_LIGHTNING",
        GAMEEVENTCONTROLLEDCREATUREDEACTIVATELIGHTNING = 65 => "GAME_EVENT_CONTROLLED_CREATURE_DEACTIVATE_LIGHTNING",
        GAMEEVENTCLOSELIVEGUI = 66 => "GAME_EVENT_CLOSE_LIVE_GUI",
        GAMEEVENTCLOSEPHOTOCAPTURE = 67 => "GAME_EVENT_CLOSE_PHOTO_CAPTURE",
        GAMEEVENTTAKETHEBLOODYSCREENSHOT = 68 => "GAME_EVENT_TAKE_THE_BLOODY_SCREENSHOT",
        GAMEEVENTTOGGLECONSOLE = 69 => "GAME_EVENT_TOGGLE_CONSOLE",
        GAMEEVENTOPENPCSKILLSMENU = 70 => "GAME_EVENT_OPEN_PC_SKILLS_MENU",
        GAMEEVENTOPENPCMSNCHATMENU = 71 => "GAME_EVENT_OPEN_PC_MSN_CHAT_MENU",
        GAMEEVENTOPENPCINVENTORYMENU = 72 => "GAME_EVENT_OPEN_PC_INVENTORY_MENU",
        GAMEEVENTOPENPCOPTIONSMENU = 73 => "GAME_EVENT_OPEN_PC_OPTIONS_MENU",
        GAMEEVENTOPENPCPHOTOJOURNALMENU = 74 => "GAME_EVENT_OPEN_PC_PHOTO_JOURNAL_MENU",
        GAMEEVENTOPENPCMAPMENU = 75 => "GAME_EVENT_OPEN_PC_MAP_MENU",
        GAMEEVENTOPENPCBUYTRADINGMENU = 76 => "GAME_EVENT_OPEN_PC_BUY_TRADING_MENU",
        GAMEEVENTOPENPCSELLTRADINGMENU = 77 => "GAME_EVENT_OPEN_PC_SELL_TRADING_MENU",
        GAMEEVENTOPENPCWANTEDTRADINGMENU = 78 => "GAME_EVENT_OPEN_PC_WANTED_TRADING_MENU",
        GAMEEVENTOPENPCSTATUSMENU = 79 => "GAME_EVENT_OPEN_PC_STATUS_MENU",
        GAMEEVENTOPENPCSCOREBOARD = 80 => "GAME_EVENT_OPEN_PC_SCOREBOARD",
        GAMEEVENTDELETEMENUCOMPONENTS = 81 => "GAME_EVENT_DELETE_MENU_COMPONENTS",
        GAMEEVENTTOGGLEPASSIVEAGGRESSIVEMODE = 82 => "GAME_EVENT_TOGGLE_PASSIVE_AGGRESSIVE_MODE",
        GAMEEVENTDELETETRADEACTIVEMENU = 83 => "GAME_EVENT_DELETE_TRADE_ACTIVE_MENU",
        GAMEEVENTTOGGLEKILLEVERYTHINGMODE = 84 => "GAME_EVENT_TOGGLE_KILL_EVERYTHING_MODE",
        GAMEEVENTTOGGLEVIEWHEROMODE = 85 => "GAME_EVENT_TOGGLE_VIEW_HERO_MODE",
        GAMEEVENTCENTRECAMERA = 86 => "GAME_EVENT_CENTRE_CAMERA",
        NOGAMEEVENTS = 87 => "NO_GAME_EVENTS",
    }
}

def_enum! {
    /// C++ `EGiftType`.
    pub enum GiftType: i32 {
        FRIENDLY = 0 => "GIFT_TYPE_FRIENDLY",
        ROMANTIC = 1 => "GIFT_TYPE_ROMANTIC",
        OFFENSIVE = 2 => "GIFT_TYPE_OFFENSIVE",
    }
}

def_enum! {
    /// C++ `EHeroAbility`.
    pub enum HeroAbility: i32 {
        HEROABILITYNULL = 0 => "HERO_ABILITY_NULL",
        HEROABILITYFORCEPUSH = 1 => "HERO_ABILITY_FORCE_PUSH",
        HEROABILITYTIMESPELL = 2 => "HERO_ABILITY_TIME_SPELL",
        HEROABILITYENFLAMESPELL = 3 => "HERO_ABILITY_ENFLAME_SPELL",
        HEROABILITYPHYSICALSHIELDSPELL = 4 => "HERO_ABILITY_PHYSICAL_SHIELD_SPELL",
        HEROABILITYTURNCOATSPELL = 5 => "HERO_ABILITY_TURNCOAT_SPELL",
        HEROABILITYDRAINLIFESPELL = 6 => "HERO_ABILITY_DRAIN_LIFE_SPELL",
        HEROABILITYRAISEDEADSPELL = 7 => "HERO_ABILITY_RAISE_DEAD_SPELL",
        HEROABILITYBERSERK = 8 => "HERO_ABILITY_BERSERK",
        HEROABILITYDOUBLESTRIKE = 9 => "HERO_ABILITY_DOUBLE_STRIKE",
        HEROABILITYSUMMONSPELL = 10 => "HERO_ABILITY_SUMMON_SPELL",
        HEROABILITYLIGHTNINGSPELL = 11 => "HERO_ABILITY_LIGHTNING_SPELL",
        HEROABILITYBATTLECHARGE = 12 => "HERO_ABILITY_BATTLE_CHARGE",
        HEROABILITYASSASSINRUSH = 13 => "HERO_ABILITY_ASSASSIN_RUSH",
        HEROABILITYHEALLIFESPELL = 14 => "HERO_ABILITY_HEAL_LIFE_SPELL",
        HEROABILITYGHOSTSWORDSPELL = 15 => "HERO_ABILITY_GHOST_SWORD_SPELL",
        HEROABILITYFIREBALLSPELL = 16 => "HERO_ABILITY_FIREBALL_SPELL",
        HEROABILITYMULTIARROW = 17 => "HERO_ABILITY_MULTI_ARROW",
        HEROABILITYDIVINEWRATHSPELL = 18 => "HERO_ABILITY_DIVINE_WRATH_SPELL",
        HEROABILITYUNHOLYPOWERSPELL = 19 => "HERO_ABILITY_UNHOLY_POWER_SPELL",
        MAXNUMBEROFHEROABILITIES = 20 => "MAX_NUMBER_OF_HERO_ABILITIES",
    }
}

def_enum! {
    /// C++ `EHeroAttachableAppearanceModifierType`.
    pub enum HeroAttachableAppearanceModifierType: i32 {
        APPEARANCEHAIR = 0 => "APPEARANCE_HAIR",
        APPEARANCEHORN = 1 => "APPEARANCE_HORN",
        APPEARANCECLOTHING = 2 => "APPEARANCE_CLOTHING",
        NOOFAPPEARANCEMODIFIERTYPES = 3 => "NO_OF_APPEARANCE_MODIFIER_TYPES",
    }
}

def_enum! {
    /// C++ `EHeroExperienceStatCategory`.
    pub enum HeroExperienceStatCategory: i32 {
        HEROSTATSTRENGTH = 0 => "HERO_STAT_STRENGTH",
        HEROSTATSKILL = 1 => "HERO_STAT_SKILL",
        HEROSTATWILL = 2 => "HERO_STAT_WILL",
        NUMBEROFHEROSTATCATEGORIES = 3 => "NUMBER_OF_HERO_STAT_CATEGORIES",
    }
}

def_enum! {
    /// C++ `EHeroTitle`.
    pub enum HeroTitle: i32 {
        NONE = 0 => "TITLE_NONE",
        REAPER = 1 => "TITLE_REAPER",
        SHADOWHUNTER = 2 => "TITLE_SHADOWHUNTER",
        MALEFICUS = 3 => "TITLE_MALEFICUS",
        DEATHBRINGER = 4 => "TITLE_DEATHBRINGER",
        ASSASSIN = 5 => "TITLE_ASSASSIN",
        NECROMANCER = 6 => "TITLE_NECROMANCER",
        AVATAR = 7 => "TITLE_AVATAR",
        PILGRIM = 8 => "TITLE_PILGRIM",
        LIBERATOR = 9 => "TITLE_LIBERATOR",
        PALADIN = 10 => "TITLE_PALADIN",
        DRUID = 11 => "TITLE_DRUID",
        RANGER = 12 => "TITLE_RANGER",
        RUNEMASTER = 13 => "TITLE_RUNEMASTER",
        HOOD = 14 => "TITLE_HOOD",
        GLADIATOR = 15 => "TITLE_GLADIATOR",
        SABRE = 16 => "TITLE_SABRE",
        ARROWDODGER = 17 => "TITLE_ARROWDODGER",
        PIEMASTER = 18 => "TITLE_PIEMASTER",
        CHICKENCHASER = 19 => "TITLE_CHICKEN_CHASER",
        ARSEFACE = 20 => "TITLE_ARSEFACE",
        JACK = 21 => "TITLE_JACK",
        MAZE = 22 => "TITLE_MAZE",
        SCARLETROBE = 23 => "TITLE_SCARLET_ROBE",
        SCYTHE = 24 => "TITLE_SCYTHE",
        THUNDER = 25 => "TITLE_THUNDER",
        WHISPER = 26 => "TITLE_WHISPER",
        TWINBLADE = 27 => "TITLE_TWINBLADE",
        BRIARROSE = 28 => "TITLE_BRIAR_ROSE",
        LADYGREY = 29 => "TITLE_LADY_GREY",
        GUILDMASTER = 30 => "TITLE_GUILDMASTER",
        SCORPIONSLAYER = 31 => "TITLE_SCORPION_SLAYER",
        DEATHBRINGER_ = 32 => "TITLE_DEATH_BRINGER",
    }
}

def_enum! {
    /// C++ `EHeroTrainingStatus`.
    pub enum HeroTrainingStatus: i32 {
        GRADUATED = 0 => "TRAINING_STATUS_GRADUATED",
        APPRENTICE = 1 => "TRAINING_STATUS_APPRENTICE",
        BOY = 2 => "TRAINING_STATUS_BOY",
    }
}

def_flags! {
    /// C++ `EIdleStateGroup`.
    pub struct IdleStateGroup: i32 {
        NULL = 0 => "ISG_NULL",
        NOT_BOTHERED = 1 => "ISG_NOT_BOTHERED",
        CLEAN_HOME = 2 => "ISG_CLEAN_HOME",
        SIT_HOME = 3 => "ISG_SIT_HOME",
        SIT_OUTSIDE = 4 => "ISG_SIT_OUTSIDE",
        SIT_AROUND_FIRE = 5 => "ISG_SIT_AROUND_FIRE",
        WATCH_FIRE = 6 => "ISG_WATCH_FIRE",
        GAZE_HOME = 7 => "ISG_GAZE_HOME",
        GAZE_OUTSIDE = 8 => "ISG_GAZE_OUTSIDE",
        LOOK_AT_INTERESTING_THINGS = 9 => "ISG_LOOK_AT_INTERESTING_THINGS",
        WANDER_AROUND_SHOPS = 10 => "ISG_WANDER_AROUND_SHOPS",
        KID_WANDER_NEAR_KIDS = 11 => "ISG_KID_WANDER_NEAR_KIDS",
    }
}

def_enum! {
    /// C++ `ELightingChannel`.
    pub enum LightingChannel: i32 {
        MAIN = 0 => "LIGHTING_CHANNEL_MAIN",
        INDOORS = 1 => "LIGHTING_CHANNEL_INDOORS",
        INDOORS2 = 2 => "LIGHTING_CHANNEL_INDOORS_2",
        INDOORS3 = 3 => "LIGHTING_CHANNEL_INDOORS_3",
        INDOORS4 = 4 => "LIGHTING_CHANNEL_INDOORS_4",
        EPICSPELL = 5 => "LIGHTING_CHANNEL_EPIC_SPELL",
        COUNT = 6 => "LIGHTING_CHANNEL_COUNT",
    }
}

def_enum! {
    /// C++ `EMessageEventType`.
    pub enum MessageEventType: i32 {
        PLAYERGIVEITEM = 0 => "MESSAGE_EVENT_PLAYER_GIVE_ITEM",
        CALLCHILDRENHOME = 1 => "MESSAGE_EVENT_CALL_CHILDREN_HOME",
        CALLSPOUSEHOME = 2 => "MESSAGE_EVENT_CALL_SPOUSE_HOME",
        PLAYERARSON = 3 => "MESSAGE_EVENT_PLAYER_ARSON",
        ATTACK = 4 => "MESSAGE_EVENT_ATTACK",
        MURDER = 5 => "MESSAGE_EVENT_MURDER",
        PLAYERARSONEMOTIONALREACTION = 6 => "MESSAGE_EVENT_PLAYER_ARSON_EMOTIONAL_REACTION",
        PLAYERATTACKEMOTIONALREACTION = 7 => "MESSAGE_EVENT_PLAYER_ATTACK_EMOTIONAL_REACTION",
        PLAYERMURDEREMOTIONALREACTION = 8 => "MESSAGE_EVENT_PLAYER_MURDER_EMOTIONAL_REACTION",
        DEATH = 9 => "MESSAGE_EVENT_DEATH",
        GOINGFORHELP = 10 => "MESSAGE_EVENT_GOING_FOR_HELP",
        YELLFORHELP = 11 => "MESSAGE_EVENT_YELL_FOR_HELP",
        YELLISEEHIM = 12 => "MESSAGE_EVENT_YELL_I_SEE_HIM",
        CONVERSATIONSTART = 13 => "MESSAGE_EVENT_CONVERSATION_START",
        CONVERSATIONJOIN = 14 => "MESSAGE_EVENT_CONVERSATION_JOIN",
        CONVERSATIONEND = 15 => "MESSAGE_EVENT_CONVERSATION_END",
        CONVERSATIONANSWERYESORNO = 16 => "MESSAGE_EVENT_CONVERSATION_ANSWER_YES_OR_NO",
        CONVERSATIONCLICKPAST = 17 => "MESSAGE_EVENT_CONVERSATION_CLICK_PAST",
        GAMEINFOCLICKPAST = 18 => "MESSAGE_EVENT_GAME_INFO_CLICK_PAST",
        BUYITEM = 19 => "MESSAGE_EVENT_BUY_ITEM",
        PLAYERINTERACTION = 20 => "MESSAGE_EVENT_PLAYER_INTERACTION",
        GAMEOFTAGSTART = 21 => "MESSAGE_EVENT_GAME_OF_TAG_START",
        GAMEOFTAGJOIN = 22 => "MESSAGE_EVENT_GAME_OF_TAG_JOIN",
        GAMEOFTAGSOMEONETAGGED = 23 => "MESSAGE_EVENT_GAME_OF_TAG_SOMEONE_TAGGED",
        HOPSCOTCHPLAYING = 24 => "MESSAGE_EVENT_HOPSCOTCH_PLAYING",
        FOUNDCORPSE = 25 => "MESSAGE_EVENT_FOUND_CORPSE",
        FOUNDUNCONSCIOUSPERSON = 26 => "MESSAGE_EVENT_FOUND_UNCONSCIOUS_PERSON",
        GUARDSEENPLAYERSWORD = 27 => "MESSAGE_EVENT_GUARD_SEEN_PLAYER_SWORD",
        THEFT = 28 => "MESSAGE_EVENT_THEFT",
        TRESPASS = 29 => "MESSAGE_EVENT_TRESPASS",
        DEALINGWITHTRESPASS = 30 => "MESSAGE_EVENT_DEALING_WITH_TRESPASS",
        DEALINGWITHNOISEINHOUSE = 31 => "MESSAGE_EVENT_DEALING_WITH_NOISE_IN_HOUSE",
        HEROREPUTATIONEVENT = 32 => "MESSAGE_EVENT_HERO_REPUTATION_EVENT",
        CROWDFORMING = 33 => "MESSAGE_EVENT_CROWD_FORMING",
        CROWDDISPERSING = 34 => "MESSAGE_EVENT_CROWD_DISPERSING",
        HEROPODIUM = 35 => "MESSAGE_EVENT_HERO_PODIUM",
        LEVELLOADED = 36 => "MESSAGE_EVENT_LEVEL_LOADED",
        LEVELUNLOADED = 37 => "MESSAGE_EVENT_LEVEL_UNLOADED",
        REGIONLOADED = 38 => "MESSAGE_EVENT_REGION_LOADED",
        REGIONUNLOADED = 39 => "MESSAGE_EVENT_REGION_UNLOADED",
        REGIONPREUNLOAD = 40 => "MESSAGE_EVENT_REGION_PREUNLOAD",
        BOASTMADE = 41 => "MESSAGE_EVENT_BOAST_MADE",
        EXPRESSIONPERFORMED = 42 => "MESSAGE_EVENT_EXPRESSION_PERFORMED",
        SCRIPTEDCAMERAEVENT = 43 => "MESSAGE_EVENT_SCRIPTED_CAMERA_EVENT",
        REQUESTGOSSIP = 44 => "MESSAGE_EVENT_REQUEST_GOSSIP",
        REQUESTGOSSIPREQUEST = 45 => "MESSAGE_EVENT_REQUEST_GOSSIP_REQUEST",
        REQUESTGAMEOFTAG = 46 => "MESSAGE_EVENT_REQUEST_GAME_OF_TAG",
        APPRENTICEPRACTICE = 47 => "MESSAGE_EVENT_APPRENTICE_PRACTICE",
        PAYMENTREQUEST = 48 => "MESSAGE_EVENT_PAYMENT_REQUEST",
        PAYMENTCANCELLED = 49 => "MESSAGE_EVENT_PAYMENT_CANCELLED",
        SKIPCUTSCENE = 50 => "MESSAGE_EVENT_SKIP_CUT_SCENE",
        QUESTCOMPLETED = 51 => "MESSAGE_EVENT_QUEST_COMPLETED",
        QUESTFAILED = 52 => "MESSAGE_EVENT_QUEST_FAILED",
        QUESTCOMPLETEDBEFORESCREENSHOWN = 53 => "MESSAGE_EVENT_QUEST_COMPLETED_BEFORE_SCREEN_SHOWN",
        QUESTFAILEDBEFORESCREENSHOWN = 54 => "MESSAGE_EVENT_QUEST_FAILED_BEFORE_SCREEN_SHOWN",
        QUESTACCEPTED = 55 => "MESSAGE_EVENT_QUEST_ACCEPTED",
        FEATACCEPTED = 56 => "MESSAGE_EVENT_FEAT_ACCEPTED",
        HAIRTYPECHANGED = 57 => "MESSAGE_EVENT_HAIR_TYPE_CHANGED",
        BEARDTYPECHANGED = 58 => "MESSAGE_EVENT_BEARD_TYPE_CHANGED",
        MOUSTACHETYPECHANGED = 59 => "MESSAGE_EVENT_MOUSTACHE_TYPE_CHANGED",
        TELEPORTERUSED = 60 => "MESSAGE_EVENT_TELEPORTER_USED",
        GUILDSEALUSED = 61 => "MESSAGE_EVENT_GUILD_SEAL_USED",
        GAMESAVEDMANUALLY = 62 => "MESSAGE_EVENT_GAME_SAVED_MANUALLY",
        FISHINGGAMEFINISHED = 63 => "MESSAGE_EVENT_FISHING_GAME_FINISHED",
        TAVERNGAMEFINISHED = 64 => "MESSAGE_EVENT_TAVERN_GAME_FINISHED",
        HEROREWARDEDFROMCONTAINER = 65 => "MESSAGE_EVENT_HERO_REWARDED_FROM_CONTAINER",
        HEROSLEPT = 66 => "MESSAGE_EVENT_HERO_SLEPT",
        HEROFIREDRANGEDWEAPON = 67 => "MESSAGE_EVENT_HERO_FIRED_RANGED_WEAPON",
        HEROCASTSPELL = 68 => "MESSAGE_EVENT_HERO_CAST_SPELL",
        HEROPICKEDPOCKET = 69 => "MESSAGE_EVENT_HERO_PICKED_POCKET",
        HEROPICKEDLOCK = 70 => "MESSAGE_EVENT_HERO_PICKED_LOCK",
        HEROSTOLENOBJECT = 71 => "MESSAGE_EVENT_HERO_STOLEN_OBJECT",
        CHESTOPENINGCANCELLED = 72 => "MESSAGE_EVENT_CHEST_OPENING_CANCELLED",
        LEAVINGQUESTSTARTSCREEN = 73 => "MESSAGE_EVENT_LEAVING_QUEST_START_SCREEN",
        LEAVINGEXPERIENCESPENDSCREEN = 74 => "MESSAGE_EVENT_LEAVING_EXPERIENCE_SPEND_SCREEN",
        ACTIONMODEBUTTONPRESSED = 75 => "MESSAGE_EVENT_ACTION_MODE_BUTTON_PRESSED",
    }
}

def_enum! {
    /// C++ `EMinimapThemeType`.
    pub enum MinimapThemeType: i32 {
        NONE = 0 => "MINIMAP_THEME_TYPE_NONE",
        GRASS = 1 => "MINIMAP_THEME_TYPE_GRASS",
        WATER = 2 => "MINIMAP_THEME_TYPE_WATER",
        SNOW = 3 => "MINIMAP_THEME_TYPE_SNOW",
        CLIFF = 4 => "MINIMAP_THEME_TYPE_CLIFF",
        EARTH = 5 => "MINIMAP_THEME_TYPE_EARTH",
        FOLIAGE = 6 => "MINIMAP_THEME_TYPE_FOLIAGE",
        WOOD = 7 => "MINIMAP_THEME_TYPE_WOOD",
        BUILDING = 8 => "MINIMAP_THEME_TYPE_BUILDING",
    }
}

def_enum! {
    /// C++ `ENavigatorType`.
    pub enum NavigatorType: i32 {
        GROUND = 1 => "NAV_INIT_GROUND",
        FLYER = 2 => "NAV_INIT_FLYER",
    }
}

def_enum! {
    /// C++ `ENoiseType`.
    pub enum NoiseType: i32 {
        CONTINUOUS = 0 => "NOISE_TYPE_CONTINUOUS",
        ONCEONLY = 1 => "NOISE_TYPE_ONCE_ONLY",
    }
}

def_flags! {
    /// C++ `EObjectAugmentationType`.
    pub struct ObjectAugmentationType: i32 {
        NULL = 0 => "AUGMENTATION_NULL",
        EXTRA_DAMAGE = 1 => "AUGMENTATION_EXTRA_DAMAGE",
        SILVER = 2 => "AUGMENTATION_SILVER",
        FLAME = 4 => "AUGMENTATION_FLAME",
        LIGHTNING = 8 => "AUGMENTATION_LIGHTNING",
        DIAMOND = 16 => "AUGMENTATION_DIAMOND",
        HEALTH_REGENERATION = 32 => "AUGMENTATION_HEALTH_REGENERATION",
        STAMINA_REGENERATION = 64 => "AUGMENTATION_STAMINA_REGENERATION",
        EXPERIENCE_INCREASE = 128 => "AUGMENTATION_EXPERIENCE_INCREASE",
        HOBBE_KILLER = 256 => "AUGMENTATION_HOBBE_KILLER",
        BANDIT_KILLER = 512 => "AUGMENTATION_BANDIT_KILLER",
    }
}

def_flags! {
    /// C++ `EOpinion`.
    pub struct Opinion: i32 {
        MORALITY = 0 => "OPINION_MORALITY",
        RENOWN = 1 => "OPINION_RENOWN",
        SCARINESS = 2 => "OPINION_SCARINESS",
        AGREEABLENESS = 3 => "OPINION_AGREEABLENESS",
        ATTRACTIVENESS = 4 => "OPINION_ATTRACTIVENESS",
        LAST = 5 => "OPINION_LAST",
    }
}

def_flags! {
    /// C++ `EOpinionAttitudeType`.
    pub struct OpinionAttitudeType: i32 {
        NONE = 0 => "OPINION_ATTITUDE_TYPE_NONE",
        RESPECT = 1 => "OPINION_ATTITUDE_TYPE_RESPECT",
        AWE = 2 => "OPINION_ATTITUDE_TYPE_AWE",
        DISDAIN = 3 => "OPINION_ATTITUDE_TYPE_DISDAIN",
        FEAR = 4 => "OPINION_ATTITUDE_TYPE_FEAR",
        FRIENDLINESS = 5 => "OPINION_ATTITUDE_TYPE_FRIENDLINESS",
        WORSHIP = 6 => "OPINION_ATTITUDE_TYPE_WORSHIP",
        RIDICULOUS = 7 => "OPINION_ATTITUDE_TYPE_RIDICULOUS",
        OFFENSIVE = 8 => "OPINION_ATTITUDE_TYPE_OFFENSIVE",
        AGREEABLE = 9 => "OPINION_ATTITUDE_TYPE_AGREEABLE",
        UGLY = 10 => "OPINION_ATTITUDE_TYPE_UGLY",
        ATTRACTED = 11 => "OPINION_ATTITUDE_TYPE_ATTRACTED",
        LOVE = 12 => "OPINION_ATTITUDE_TYPE_LOVE",
        WIFE_FIRST = 13 => "OPINION_ATTITUDE_TYPE_WIFE_FIRST",
        WIFE_LOVE = 13 => "OPINION_ATTITUDE_TYPE_WIFE_LOVE",
        WIFE_LIKE = 14 => "OPINION_ATTITUDE_TYPE_WIFE_LIKE",
        WIFE_NEUTRAL = 15 => "OPINION_ATTITUDE_TYPE_WIFE_NEUTRAL",
        WIFE_DISLIKE = 16 => "OPINION_ATTITUDE_TYPE_WIFE_DISLIKE",
        WIFE_HATE = 17 => "OPINION_ATTITUDE_TYPE_WIFE_HATE",
        LAST = 18 => "OPINION_ATTITUDE_TYPE_LAST",
    }
}

def_flags! {
    /// C++ `EOpinionDeedType`.
    pub struct OpinionDeedType: i32 {
        NONE = 0 => "OPINION_DEED_TYPE_NONE",
        CRIME_WEAPON_OUT = 1 => "OPINION_DEED_TYPE_CRIME_WEAPON_OUT",
        CRIME_TRESPASS_THIRD = 2 => "OPINION_DEED_TYPE_CRIME_TRESPASS_THIRD",
        CRIME_VANDALISM = 3 => "OPINION_DEED_TYPE_CRIME_VANDALISM",
        CRIME_LOCKPICKING = 4 => "OPINION_DEED_TYPE_CRIME_LOCKPICKING",
        CRIME_PICK_POCKETS = 5 => "OPINION_DEED_TYPE_CRIME_PICK_POCKETS",
        CRIME_THEFT = 6 => "OPINION_DEED_TYPE_CRIME_THEFT",
        CRIME_ASSAULT = 7 => "OPINION_DEED_TYPE_CRIME_ASSAULT",
        CRIME_GBH = 8 => "OPINION_DEED_TYPE_CRIME_GBH",
        CRIME_MURDER = 9 => "OPINION_DEED_TYPE_CRIME_MURDER",
        CRIME_TRESPASS_FIRST = 10 => "OPINION_DEED_TYPE_CRIME_TRESPASS_FIRST",
        CRIME_TRESPASS_SECOND = 11 => "OPINION_DEED_TYPE_CRIME_TRESPASS_SECOND",
        EXPRESSION_HEROIC_STANCE = 12 => "OPINION_DEED_TYPE_EXPRESSION_HEROIC_STANCE",
        EXPRESSION_FLIRT = 13 => "OPINION_DEED_TYPE_EXPRESSION_FLIRT",
        EXPRESSION_APOLOGY_NO_CRIME = 14 => "OPINION_DEED_TYPE_EXPRESSION_APOLOGY_NO_CRIME",
        EXPRESSION_SNEER = 15 => "OPINION_DEED_TYPE_EXPRESSION_SNEER",
        EXPRESSION_EVIL_LAUGH = 16 => "OPINION_DEED_TYPE_EXPRESSION_EVIL_LAUGH",
        EXPRESSION_BATTLE_CRY = 17 => "OPINION_DEED_TYPE_EXPRESSION_BATTLE_CRY",
        EXPRESSION_PELVIC_THRUST = 18 => "OPINION_DEED_TYPE_EXPRESSION_PELVIC_THRUST",
        EXPRESSION_MIDDLE_FINGER = 19 => "OPINION_DEED_TYPE_EXPRESSION_MIDDLE_FINGER",
        EXPRESSION_BELCH = 20 => "OPINION_DEED_TYPE_EXPRESSION_BELCH",
        EXPRESSION_FART = 21 => "OPINION_DEED_TYPE_EXPRESSION_FART",
        EXPRESSION_VICTORY_PUMP = 22 => "OPINION_DEED_TYPE_EXPRESSION_VICTORY_PUMP",
        EXPRESSION_CLAP = 23 => "OPINION_DEED_TYPE_EXPRESSION_CLAP",
        EXPRESSION_GIGGLE = 24 => "OPINION_DEED_TYPE_EXPRESSION_GIGGLE",
        EXPRESSION_SHIT = 25 => "OPINION_DEED_TYPE_EXPRESSION_SHIT",
        EXPRESSION_THANKS = 26 => "OPINION_DEED_TYPE_EXPRESSION_THANKS",
        EXPRESSION_COCK_A_DOODLE_DO = 27 => "OPINION_DEED_TYPE_EXPRESSION_COCK_A_DOODLE_DO",
        EXPRESSION_CROTCH_GRAB = 28 => "OPINION_DEED_TYPE_EXPRESSION_CROTCH_GRAB",
        EXPRESSION_KISS_MY_ASS = 29 => "OPINION_DEED_TYPE_EXPRESSION_KISS_MY_ASS",
        EXPRESSION_FLAMENCO = 30 => "OPINION_DEED_TYPE_EXPRESSION_FLAMENCO",
        EXPRESSION_COSSACK = 31 => "OPINION_DEED_TYPE_EXPRESSION_COSSACK",
        EXPRESSION_AIR_GUITAR = 32 => "OPINION_DEED_TYPE_EXPRESSION_AIR_GUITAR",
        EXPRESSION_BALLET = 33 => "OPINION_DEED_TYPE_EXPRESSION_BALLET",
        EXPRESSION_SATURDAY_NIGHT_FEVER = 34 => "OPINION_DEED_TYPE_EXPRESSION_SATURDAY_NIGHT_FEVER",
        EXPRESSION_TAP = 35 => "OPINION_DEED_TYPE_EXPRESSION_TAP",
        EXPRESSION_Y = 36 => "OPINION_DEED_TYPE_EXPRESSION_Y",
        EXPRESSION_M = 37 => "OPINION_DEED_TYPE_EXPRESSION_M",
        EXPRESSION_C = 38 => "OPINION_DEED_TYPE_EXPRESSION_C",
        EXPRESSION_A = 39 => "OPINION_DEED_TYPE_EXPRESSION_A",
        EXPRESSION_THREATEN_SMALL = 40 => "OPINION_DEED_TYPE_EXPRESSION_THREATEN_SMALL",
        EXPRESSION_THREATEN_LARGE = 41 => "OPINION_DEED_TYPE_EXPRESSION_THREATEN_LARGE",
        SCRIPT_ACTION_ANNOYING_SMALL = 42 => "OPINION_DEED_TYPE_SCRIPT_ACTION_ANNOYING_SMALL",
        SCRIPT_ACTION_ANNOYING_LARGE = 43 => "OPINION_DEED_TYPE_SCRIPT_ACTION_ANNOYING_LARGE",
        SCRIPT_ACTION_NICE_SMALL = 44 => "OPINION_DEED_TYPE_SCRIPT_ACTION_NICE_SMALL",
        SCRIPT_ACTION_NICE_LARGE = 45 => "OPINION_DEED_TYPE_SCRIPT_ACTION_NICE_LARGE",
        FOLLOWER_ACCEPT = 46 => "OPINION_DEED_TYPE_FOLLOWER_ACCEPT",
        FOLLOWER_REFUSE = 47 => "OPINION_DEED_TYPE_FOLLOWER_REFUSE",
        FOLLOWER_DISMISSED = 48 => "OPINION_DEED_TYPE_FOLLOWER_DISMISSED",
        FOLLOWER_QUIT = 49 => "OPINION_DEED_TYPE_FOLLOWER_QUIT",
        FOLLOWER_ENEMYSPOTTED = 50 => "OPINION_DEED_TYPE_FOLLOWER_ENEMYSPOTTED",
        FOLLOWER_IDLEEXCITED = 51 => "OPINION_DEED_TYPE_FOLLOWER_IDLEEXCITED",
        FOLLOWER_IDLEBORED = 52 => "OPINION_DEED_TYPE_FOLLOWER_IDLEBORED",
        WITNESSED_ASSAULT_OR_GBH = 53 => "OPINION_DEED_TYPE_WITNESSED_ASSAULT_OR_GBH",
        TOO_FREQUENT_OTHER_DEED = 54 => "OPINION_DEED_TYPE_TOO_FREQUENT_OTHER_DEED",
        SHOW_TROPHY_EVIL = 55 => "OPINION_DEED_TYPE_SHOW_TROPHY_EVIL",
        SHOW_TROPHY_GOOD = 56 => "OPINION_DEED_TYPE_SHOW_TROPHY_GOOD",
        SHOW_TROPHY_BORED = 57 => "OPINION_DEED_TYPE_SHOW_TROPHY_BORED",
        KILL_BAD_GUY = 58 => "OPINION_DEED_TYPE_KILL_BAD_GUY",
        MURDER_SPOUSE = 59 => "OPINION_DEED_TYPE_MURDER_SPOUSE",
        RECEIVE_GIFT_ROMANTIC = 60 => "OPINION_DEED_TYPE_RECEIVE_GIFT_ROMANTIC",
        RECEIVE_GIFT_FRIENDLY = 61 => "OPINION_DEED_TYPE_RECEIVE_GIFT_FRIENDLY",
        RECEIVE_GIFT_OFFENSIVE = 62 => "OPINION_DEED_TYPE_RECEIVE_GIFT_OFFENSIVE",
        RECEIVE_WEDDING_RING = 63 => "OPINION_DEED_TYPE_RECEIVE_WEDDING_RING",
        BOAST_ANTICIPATION = 64 => "OPINION_DEED_TYPE_BOAST_ANTICIPATION",
        BOAST_ENCOURAGE_FIRST = 65 => "OPINION_DEED_TYPE_BOAST_ENCOURAGE_FIRST",
        BOAST_ENCOURAGE_MIDDLE = 66 => "OPINION_DEED_TYPE_BOAST_ENCOURAGE_MIDDLE",
        BOAST_ENCOURAGE_FINAL = 67 => "OPINION_DEED_TYPE_BOAST_ENCOURAGE_FINAL",
        BOAST_WELL_WISHES = 68 => "OPINION_DEED_TYPE_BOAST_WELL_WISHES",
        BOAST_ANNOYED_NO_BOASTING = 69 => "OPINION_DEED_TYPE_BOAST_ANNOYED_NO_BOASTING",
        COMMENT_AT_HERO = 70 => "OPINION_DEED_TYPE_COMMENT_AT_HERO",
        COMMENT_TO_SELF = 71 => "OPINION_DEED_TYPE_COMMENT_TO_SELF",
        COMMENT_ABOUT_HERO = 72 => "OPINION_DEED_TYPE_COMMENT_ABOUT_HERO",
        GENERIC_INCOMPREHENSION = 73 => "OPINION_DEED_TYPE_GENERIC_INCOMPREHENSION",
        HIGH_PRIORITY_INCOMPREHENSION = 74 => "OPINION_DEED_TYPE_HIGH_PRIORITY_INCOMPREHENSION",
        HUSBAND_RAGE = 75 => "OPINION_DEED_TYPE_HUSBAND_RAGE",
        TAVERN_GAME_WIN = 76 => "OPINION_DEED_TYPE_TAVERN_GAME_WIN",
        INDOORS_GREETING = 77 => "OPINION_DEED_TYPE_INDOORS_GREETING",
        APOLOGY_ACCEPTED = 78 => "OPINION_DEED_TYPE_APOLOGY_ACCEPTED",
        APOLOGY_REFUSED = 79 => "OPINION_DEED_TYPE_APOLOGY_REFUSED",
        WIFE_GREETED = 80 => "OPINION_DEED_TYPE_WIFE_GREETED",
        WIFE_TIME_SINCE_SEEING = 81 => "OPINION_DEED_TYPE_WIFE_TIME_SINCE_SEEING",
        WIFE_GIFT_RECEIVE_ALREADY_GOT = 82 => "OPINION_DEED_TYPE_WIFE_GIFT_RECEIVE_ALREADY_GOT",
        WIFE_JUSTMARRIED = 83 => "OPINION_DEED_TYPE_WIFE_JUSTMARRIED",
        WIFE_GIFT_WANTED = 84 => "OPINION_DEED_TYPE_WIFE_GIFT_WANTED",
        WIFE_WITNESSED_FLIRT = 85 => "OPINION_DEED_TYPE_WIFE_WITNESSED_FLIRT",
        WIFE_HOUSE_DRESSING_GOOD = 86 => "OPINION_DEED_TYPE_WIFE_HOUSE_DRESSING_GOOD",
        WIFE_HOUSE_DRESSING_BAD = 87 => "OPINION_DEED_TYPE_WIFE_HOUSE_DRESSING_BAD",
        WIFE_DIVORCE_WARNING = 88 => "OPINION_DEED_TYPE_WIFE_DIVORCE_WARNING",
        WIFE_DIVORCE_OCCURRED = 89 => "OPINION_DEED_TYPE_WIFE_DIVORCE_OCCURRED",
        WIFE_SEX_OFFER_TO_GO_TO_BED = 90 => "OPINION_DEED_TYPE_WIFE_SEX_OFFER_TO_GO_TO_BED",
        WIFE_SEX_COMMENT_AFTERWARDS = 91 => "OPINION_DEED_TYPE_WIFE_SEX_COMMENT_AFTERWARDS",
        LAST = 92 => "OPINION_DEED_TYPE_LAST",
    }
}

def_flags! {
    /// C++ `EOpinionReactionType`.
    pub struct OpinionReactionType: i32 {
        NONE = 0 => "OPINION_REACTION_TYPE_NONE",
        ATTITUDE_RESPECT = 1 => "OPINION_REACTION_TYPE_ATTITUDE_RESPECT",
        ATTITUDE_AWE = 2 => "OPINION_REACTION_TYPE_ATTITUDE_AWE",
        ATTITUDE_DISDAIN = 3 => "OPINION_REACTION_TYPE_ATTITUDE_DISDAIN",
        ATTITUDE_FEAR = 4 => "OPINION_REACTION_TYPE_ATTITUDE_FEAR",
        ATTITUDE_FRIENDLINESS = 5 => "OPINION_REACTION_TYPE_ATTITUDE_FRIENDLINESS",
        ATTITUDE_WORSHIP = 6 => "OPINION_REACTION_TYPE_ATTITUDE_WORSHIP",
        ATTITUDE_RIDICULOUS = 7 => "OPINION_REACTION_TYPE_ATTITUDE_RIDICULOUS",
        ATTITUDE_OFFENSIVE = 8 => "OPINION_REACTION_TYPE_ATTITUDE_OFFENSIVE",
        ATTITUDE_AGREEABLE = 9 => "OPINION_REACTION_TYPE_ATTITUDE_AGREEABLE",
        ATTITUDE_UGLY = 10 => "OPINION_REACTION_TYPE_ATTITUDE_UGLY",
        ATTITUDE_ATTRACTED = 11 => "OPINION_REACTION_TYPE_ATTITUDE_ATTRACTED",
        ATTITUDE_LOVE = 12 => "OPINION_REACTION_TYPE_ATTITUDE_LOVE",
        ATTITUDE_WIFE_LOVE = 13 => "OPINION_REACTION_TYPE_ATTITUDE_WIFE_LOVE",
        ATTITUDE_WIFE_LIKE = 14 => "OPINION_REACTION_TYPE_ATTITUDE_WIFE_LIKE",
        ATTITUDE_WIFE_NEUTRAL = 15 => "OPINION_REACTION_TYPE_ATTITUDE_WIFE_NEUTRAL",
        ATTITUDE_WIFE_DISLIKE = 16 => "OPINION_REACTION_TYPE_ATTITUDE_WIFE_DISLIKE",
        ATTITUDE_WIFE_HATE = 17 => "OPINION_REACTION_TYPE_ATTITUDE_WIFE_HATE",
        ANGRY_POINT = 18 => "OPINION_REACTION_TYPE_ANGRY_POINT",
        BACK_AWAY = 19 => "OPINION_REACTION_TYPE_BACK_AWAY",
        BELLY_LAUGH = 20 => "OPINION_REACTION_TYPE_BELLY_LAUGH",
        BOO = 21 => "OPINION_REACTION_TYPE_BOO",
        BOWING_LARGE = 22 => "OPINION_REACTION_TYPE_BOWING_LARGE",
        BOWING_SMALL = 23 => "OPINION_REACTION_TYPE_BOWING_SMALL",
        CALLING_CHILDREN = 24 => "OPINION_REACTION_TYPE_CALLING_CHILDREN",
        CHEER_LARGE = 25 => "OPINION_REACTION_TYPE_CHEER_LARGE",
        CHEER_SMALL = 26 => "OPINION_REACTION_TYPE_CHEER_SMALL",
        CLAP_LARGE = 27 => "OPINION_REACTION_TYPE_CLAP_LARGE",
        CLAP_SMALL = 28 => "OPINION_REACTION_TYPE_CLAP_SMALL",
        COMMENT_ABOUT_HERO = 29 => "OPINION_REACTION_TYPE_COMMENT_ABOUT_HERO",
        COMMENT_AT_HERO = 30 => "OPINION_REACTION_TYPE_COMMENT_AT_HERO",
        COMMENT_TO_SELF = 31 => "OPINION_REACTION_TYPE_COMMENT_TO_SELF",
        COWER = 32 => "OPINION_REACTION_TYPE_COWER",
        DISMISS = 33 => "OPINION_REACTION_TYPE_DISMISS",
        FLEE = 34 => "OPINION_REACTION_TYPE_FLEE",
        FOLLOW_CLOSE = 35 => "OPINION_REACTION_TYPE_FOLLOW_CLOSE",
        FOLLOW_FAR = 36 => "OPINION_REACTION_TYPE_FOLLOW_FAR",
        FRIENDLY_GREET = 37 => "OPINION_REACTION_TYPE_FRIENDLY_GREET",
        GET_OUT = 38 => "OPINION_REACTION_TYPE_GET_OUT",
        GROVEL_LARGE = 39 => "OPINION_REACTION_TYPE_GROVEL_LARGE",
        GROVEL_SMALL = 40 => "OPINION_REACTION_TYPE_GROVEL_SMALL",
        HERO_IMITATION_PLAY = 41 => "OPINION_REACTION_TYPE_HERO_IMITATION_PLAY",
        HEROTITLE_AT_HERO = 42 => "OPINION_REACTION_TYPE_HEROTITLE_AT_HERO",
        HEROTITLE_TO_SELF = 43 => "OPINION_REACTION_TYPE_HEROTITLE_TO_SELF",
        HIDE = 44 => "OPINION_REACTION_TYPE_HIDE",
        MARRIAGE_COMMENT = 45 => "OPINION_REACTION_TYPE_MARRIAGE_COMMENT",
        OFFER_GIFT_FRIENDLY = 46 => "OPINION_REACTION_TYPE_OFFER_GIFT_FRIENDLY",
        OFFER_GIFT_WORSHIP = 47 => "OPINION_REACTION_TYPE_OFFER_GIFT_WORSHIP",
        PICK_FIGHT = 48 => "OPINION_REACTION_TYPE_PICK_FIGHT",
        POINT = 49 => "OPINION_REACTION_TYPE_POINT",
        POINT_LAUGH = 50 => "OPINION_REACTION_TYPE_POINT_LAUGH",
        RESPECTFUL_GREET = 51 => "OPINION_REACTION_TYPE_RESPECTFUL_GREET",
        SHAKE_FIST = 52 => "OPINION_REACTION_TYPE_SHAKE_FIST",
        SHOUT_AWE = 53 => "OPINION_REACTION_TYPE_SHOUT_AWE",
        SNIGGER = 54 => "OPINION_REACTION_TYPE_SNIGGER",
        THUMBS_DOWN = 55 => "OPINION_REACTION_TYPE_THUMBS_DOWN",
        WATCH = 56 => "OPINION_REACTION_TYPE_WATCH",
        WIFE_FEELING_LOVE = 57 => "OPINION_REACTION_TYPE_WIFE_FEELING_LOVE",
        WIFE_FIRST = 57 => "OPINION_REACTION_TYPE_WIFE_FIRST",
        WIFE_GENERAL_LOVE = 58 => "OPINION_REACTION_TYPE_WIFE_GENERAL_LOVE",
        WIFE_TOHERSELF_LOVE = 59 => "OPINION_REACTION_TYPE_WIFE_TOHERSELF_LOVE",
        WIFE_CLOTHING_LOVE = 60 => "OPINION_REACTION_TYPE_WIFE_CLOTHING_LOVE",
        WIFE_FEELING_LIKE = 61 => "OPINION_REACTION_TYPE_WIFE_FEELING_LIKE",
        WIFE_GENERAL_LIKE = 62 => "OPINION_REACTION_TYPE_WIFE_GENERAL_LIKE",
        WIFE_TOHERSELF_LIKE = 63 => "OPINION_REACTION_TYPE_WIFE_TOHERSELF_LIKE",
        WIFE_CLOTHING_LIKE = 64 => "OPINION_REACTION_TYPE_WIFE_CLOTHING_LIKE",
        WIFE_FEELING_NEUTRAL = 65 => "OPINION_REACTION_TYPE_WIFE_FEELING_NEUTRAL",
        WIFE_GENERAL_NEUTRAL = 66 => "OPINION_REACTION_TYPE_WIFE_GENERAL_NEUTRAL",
        WIFE_TOHERSELF_NEUTRAL = 67 => "OPINION_REACTION_TYPE_WIFE_TOHERSELF_NEUTRAL",
        WIFE_CLOTHING_NEUTRAL = 68 => "OPINION_REACTION_TYPE_WIFE_CLOTHING_NEUTRAL",
        WIFE_FEELING_DISLIKE = 69 => "OPINION_REACTION_TYPE_WIFE_FEELING_DISLIKE",
        WIFE_GENERAL_DISLIKE = 70 => "OPINION_REACTION_TYPE_WIFE_GENERAL_DISLIKE",
        WIFE_TOHERSELF_DISLIKE = 71 => "OPINION_REACTION_TYPE_WIFE_TOHERSELF_DISLIKE",
        WIFE_CLOTHING_DISLIKE = 72 => "OPINION_REACTION_TYPE_WIFE_CLOTHING_DISLIKE",
        WIFE_FEELING_HATE = 73 => "OPINION_REACTION_TYPE_WIFE_FEELING_HATE",
        WIFE_GENERAL_HATE = 74 => "OPINION_REACTION_TYPE_WIFE_GENERAL_HATE",
        WIFE_TOHERSELF_HATE = 75 => "OPINION_REACTION_TYPE_WIFE_TOHERSELF_HATE",
        WIFE_CLOTHING_HATE = 76 => "OPINION_REACTION_TYPE_WIFE_CLOTHING_HATE",
        WIFE_JUSTMARRIED = 77 => "OPINION_REACTION_TYPE_WIFE_JUSTMARRIED",
        WIFE_GIFT_WANTED = 78 => "OPINION_REACTION_TYPE_WIFE_GIFT_WANTED",
        LAST = 79 => "OPINION_REACTION_TYPE_LAST",
        WIFE_LAST = 79 => "OPINION_REACTION_TYPE_WIFE_LAST",
    }
}

def_flags! {
    /// C++ `EOpinionTargetingConditionType`.
    pub struct OpinionTargetingConditionType: i32 {
        NONE = 0 => "OPINION_TARGETING_CONDITION_TYPE_NONE",
        NOT_TARGETED = 1 => "OPINION_TARGETING_CONDITION_TYPE_NOT_TARGETED",
        TARGETED_BUT_NOT_Z = 2 => "OPINION_TARGETING_CONDITION_TYPE_TARGETED_BUT_NOT_Z",
        NOT_Z_TARGETED = 3 => "OPINION_TARGETING_CONDITION_TYPE_NOT_Z_TARGETED",
        Z_TARGETED = 4 => "OPINION_TARGETING_CONDITION_TYPE_Z_TARGETED",
        ANY_TARGETED = 5 => "OPINION_TARGETING_CONDITION_TYPE_ANY_TARGETED",
    }
}

def_flags! {
    /// C++ `EPSwitchTriggerType`.
    pub struct PSwitchTriggerType: i32 {
        PSWITCH_TRIGGER_ON_PLAYER = 1 => "PSWITCH_TRIGGER_ON_PLAYER",
        USE = 2 => "PSWITCH_TRIGGER_ON_PLAYER_USE",
    }
}

def_enum! {
    /// C++ `EPerceivedThingType`.
    pub enum PerceivedThingType: i32 {
        PERCEIVEDTHINGFINDPLAYER = 0 => "PERCEIVED_THING_FIND_PLAYER",
        MAXNOPERCEIVEDTHINGTYPES = 1 => "MAX_NO_PERCEIVED_THING_TYPES",
    }
}

def_enum! {
    /// C++ `EPointLightChannelEffect`.
    pub enum PointLightChannelEffect: i32 {
        LOCALCHANNEL = 0 => "POINT_LIGHT_EFFECT_LOCAL_CHANNEL",
        ALLINTERNALS = 1 => "POINT_LIGHT_EFFECT_ALL_INTERNALS",
        EXTERNALS = 2 => "POINT_LIGHT_EFFECT_EXTERNALS",
        ALL = 3 => "POINT_LIGHT_EFFECT_ALL",
    }
}

def_enum! {
    /// C++ `EQuakeLength`.
    pub enum QuakeLength: i32 {
        SHORT = 0 => "QUAKE_LENGTH_SHORT",
        MEDIUM = 1 => "QUAKE_LENGTH_MEDIUM",
        LONG = 2 => "QUAKE_LENGTH_LONG",
        CONTINUOUS = 3 => "QUAKE_LENGTH_CONTINUOUS",
    }
}

def_enum! {
    /// C++ `EQuakeStrength`.
    pub enum QuakeStrength: i32 {
        WEAK = 0 => "QUAKE_STRENGTH_WEAK",
        MEDIUM = 1 => "QUAKE_STRENGTH_MEDIUM",
        STRONG = 2 => "QUAKE_STRENGTH_STRONG",
        MADNESS = 3 => "QUAKE_STRENGTH_MADNESS",
    }
}

def_enum! {
    /// C++ `EReactionSpeechType`.
    pub enum ReactionSpeechType: i32 {
        REACTIONSPEECHNULL = 0 => "REACTION_SPEECH_NULL",
        REACTIONSPEECHCALLOVERHERE = 1 => "REACTION_SPEECH_CALL_OVER_HERE",
        REACTIONSPEECHFAWNINGGREETING = 2 => "REACTION_SPEECH_FAWNING_GREETING",
        REACTIONSPEECHFRIENDLYGREETING = 3 => "REACTION_SPEECH_FRIENDLY_GREETING",
        REACTIONSPEECHSTRANGERSGREETING = 4 => "REACTION_SPEECH_STRANGERS_GREETING",
        REACTIONSPEECHNERVOUS = 5 => "REACTION_SPEECH_NERVOUS",
        REACTIONSPEECHSCARED = 6 => "REACTION_SPEECH_SCARED",
        REACTIONSPEECHTERRIFIED = 7 => "REACTION_SPEECH_TERRIFIED",
        REACTIONSPEECHINSULTED = 8 => "REACTION_SPEECH_INSULTED",
        REACTIONSPEECHANGRY = 9 => "REACTION_SPEECH_ANGRY",
        REACTIONSPEECHHATEFUL = 10 => "REACTION_SPEECH_HATEFUL",
        REACTIONSPEECHTHREATOFRETRIBUTION = 11 => "REACTION_SPEECH_THREAT_OF_RETRIBUTION",
        REACTIONSPEECHPROMISEOFRETRIBUTION = 12 => "REACTION_SPEECH_PROMISE_OF_RETRIBUTION",
        REACTIONSPEECHDISMISSIVE = 13 => "REACTION_SPEECH_DISMISSIVE",
        REACTIONSPEECHBOTHERED = 14 => "REACTION_SPEECH_BOTHERED",
        REACTIONSPEECHHARASSED = 15 => "REACTION_SPEECH_HARASSED",
        REACTIONSPEECHRIDICULING = 16 => "REACTION_SPEECH_RIDICULING",
        REACTIONSPEECHINSULTS = 17 => "REACTION_SPEECH_INSULTS",
        REACTIONSPEECHATTRACTED = 18 => "REACTION_SPEECH_ATTRACTED",
        REACTIONSPEECHLOVING = 19 => "REACTION_SPEECH_LOVING",
        REACTIONSPEECHSURPRISEDATABUSE = 20 => "REACTION_SPEECH_SURPRISED_AT_ABUSE",
        REACTIONSPEECHLOVINGANDSHOCKEDATABUSE = 21 => "REACTION_SPEECH_LOVING_AND_SHOCKED_AT_ABUSE",
        REACTIONSPEECHUNIMPRESSEDRESPONSETOTHREAT = 22 => "REACTION_SPEECH_UNIMPRESSED_RESPONSE_TO_THREAT",
        REACTIONSPEECHRIDICULINGRESPONSETOTHREAT = 23 => "REACTION_SPEECH_RIDICULING_RESPONSE_TO_THREAT",
        REACTIONSPEECHLOVINGRESPONSETOFLIRT = 24 => "REACTION_SPEECH_LOVING_RESPONSE_TO_FLIRT",
        REACTIONSPEECHATTRACTEDRESPONSETOFLIRT = 25 => "REACTION_SPEECH_ATTRACTED_RESPONSE_TO_FLIRT",
        REACTIONSPEECHNEUTRALREFUSALOFFLIRT = 26 => "REACTION_SPEECH_NEUTRAL_REFUSAL_OF_FLIRT",
        REACTIONSPEECHNEGATIVERESPONSETOFLIRT = 27 => "REACTION_SPEECH_NEGATIVE_RESPONSE_TO_FLIRT",
        REACTIONSPEECHFEARFULREFUSALOFFLIRT = 28 => "REACTION_SPEECH_FEARFUL_REFUSAL_OF_FLIRT",
        REACTIONSPEECHGRATEFULACCEPTANCEOFBRIBE = 29 => "REACTION_SPEECH_GRATEFUL_ACCEPTANCE_OF_BRIBE",
        REACTIONSPEECHPACIFIEDACCEPTANCEOFBRIBE = 30 => "REACTION_SPEECH_PACIFIED_ACCEPTANCE_OF_BRIBE",
        REACTIONSPEECHFRIENDLYREJECTIONOFBRIBE = 31 => "REACTION_SPEECH_FRIENDLY_REJECTION_OF_BRIBE",
        REACTIONSPEECHUNFRIENDLYREJECTIONOFBRIBE = 32 => "REACTION_SPEECH_UNFRIENDLY_REJECTION_OF_BRIBE",
        REACTIONSPEECHWARNAWAY = 33 => "REACTION_SPEECH_WARN_AWAY",
        REACTIONSPEECHGIVEAWAY = 34 => "REACTION_SPEECH_GIVE_AWAY",
        REACTIONSPEECHREPORTCRIME = 35 => "REACTION_SPEECH_REPORT_CRIME",
        REACTIONSPEECHBODYFOUND = 36 => "REACTION_SPEECH_BODY_FOUND",
        REACTIONSPEECHREPORTBODYFOUND = 37 => "REACTION_SPEECH_REPORT_BODY_FOUND",
        REACTIONSPEECHGOSSIP = 38 => "REACTION_SPEECH_GOSSIP",
        REACTIONSPEECHYAWN = 39 => "REACTION_SPEECH_YAWN",
        REACTIONSPEECHSNORE = 40 => "REACTION_SPEECH_SNORE",
        REACTIONSPEECHCHEER = 41 => "REACTION_SPEECH_CHEER",
        REACTIONSPEECHSOB = 42 => "REACTION_SPEECH_SOB",
        REACTIONSPEECHCRYOUT = 43 => "REACTION_SPEECH_CRY_OUT",
        REACTIONSPEECHBATTLECRY = 44 => "REACTION_SPEECH_BATTLE_CRY",
        REACTIONSPEECHLYNCHCRY = 45 => "REACTION_SPEECH_LYNCH_CRY",
        REACTIONSPEECHWOUNDED = 46 => "REACTION_SPEECH_WOUNDED",
        REACTIONSPEECHDIE = 47 => "REACTION_SPEECH_DIE",
        REACTIONSPEECHGUARDKILL = 48 => "REACTION_SPEECH_GUARD_KILL",
        REACTIONSPEECHGUARDARREST = 49 => "REACTION_SPEECH_GUARD_ARREST",
        REACTIONSPEECHGUARDSECURITYSWEEP = 50 => "REACTION_SPEECH_GUARD_SECURITY_SWEEP",
        REACTIONSPEECHGUARDWARNING1 = 51 => "REACTION_SPEECH_GUARD_WARNING_1",
        REACTIONSPEECHGUARDWARNING2 = 52 => "REACTION_SPEECH_GUARD_WARNING_2",
        REACTIONSPEECHGUARDWARNING3 = 53 => "REACTION_SPEECH_GUARD_WARNING_3",
        REACTIONSPEECHGUARDWARNINGENDANDTHANKS = 54 => "REACTION_SPEECH_GUARD_WARNING_END_AND_THANKS",
        REACTIONSPEECHNORESPECT = 55 => "REACTION_SPEECH_NO_RESPECT",
        MAXNOREACTIONSPEECHTYPES = 56 => "MAX_NO_REACTION_SPEECH_TYPES",
    }
}

def_enum! {
    /// C++ `EReverbEnvironmentType`.
    pub enum ReverbEnvironmentType: i32 {
        NULL = 0 => "REVERB_ENVIRONMENT_NULL",
        EXTERNAL = 1 => "REVERB_ENVIRONMENT_EXTERNAL",
        CAVE = 2 => "REVERB_ENVIRONMENT_CAVE",
        HALL = 3 => "REVERB_ENVIRONMENT_HALL",
        GUILD = 4 => "REVERB_ENVIRONMENT_GUILD",
        GUILDSMALL = 5 => "REVERB_ENVIRONMENT_GUILD_SMALL",
        SMALLROOM = 6 => "REVERB_ENVIRONMENT_SMALL_ROOM",
        SCHOOL = 7 => "REVERB_ENVIRONMENT_SCHOOL",
    }
}

def_enum! {
    /// C++ `EScriptingStateGroups`.
    pub enum ScriptingStateGroups: i32 {
        NONE = 0 => "ESSG_NONE",
        PERFORMACTIONPHYSICAL = 1 => "ESSG_PERFORM_ACTION_PHYSICAL",
        PERFORMACTIONVERBAL = 2 => "ESSG_PERFORM_ACTION_VERBAL",
        PERFORMACTIONAURAL = 3 => "ESSG_PERFORM_ACTION_AURAL",
        WANDERNEAR = 4 => "ESSG_WANDER_NEAR",
        FOLLOWPATH = 5 => "ESSG_FOLLOW_PATH",
        FOLLOWRANDOM = 6 => "ESSG_FOLLOW_RANDOM",
        FOLLOWNEAREST = 7 => "ESSG_FOLLOW_NEAREST",
        WALKTORANDOM = 8 => "ESSG_WALK_TO_RANDOM",
        WALKTONEARESTDIFFERENT = 9 => "ESSG_WALK_TO_NEAREST_DIFFERENT",
        RUNATHEROANDATTACKUNTILDEAD = 10 => "ESSG_RUN_AT_HERO_AND_ATTACK_UNTIL_DEAD",
    }
}

def_enum! {
    /// C++ `ESex`.
    pub enum Sex: i32 {
        SEXNULL = 0 => "SEX_NULL",
        SEXMALE = 1 => "SEX_MALE",
        SEXFEMALE = 2 => "SEX_FEMALE",
        NOOFSEXES = 3 => "NO_OF_SEXES",
    }
}

def_flags! {
    /// C++ `ESwitchTriggerType`.
    pub struct SwitchTriggerType: i32 {
        PLAYER_ONLY_ONCE_IN_AREA = 1 => "SWITCH_TRIGGER_PLAYER_ONLY_ONCE_IN_AREA",
        PLAYER_ONLY_MULTIPLE_TIMES_IN_AREA = 2 => "SWITCH_TRIGGER_PLAYER_ONLY_MULTIPLE_TIMES_IN_AREA",
        PLAYER_ONLY_RESET_WHEN_LEAVES = 3 => "SWITCH_TRIGGER_PLAYER_ONLY_RESET_WHEN_LEAVES",
        PLAYER_SHOW_AREA_NAME = 4 => "SWITCH_TRIGGER_PLAYER_SHOW_AREA_NAME",
        PLAYER_CHANGE_ENVIRONMENT_THEME = 5 => "SWITCH_TRIGGER_PLAYER_CHANGE_ENVIRONMENT_THEME",
        ONCE_ON_ITEM_APPLICATION = 6 => "SWITCH_TRIGGER_ONCE_ON_ITEM_APPLICATION",
    }
}

def_enum! {
    /// C++ `ETavernGameControlType`.
    pub enum TavernGameControlType: i32 {
        RELATIVE = 0 => "ETGCT_RELATIVE",
        ABSOLUTE = 1 => "ETGCT_ABSOLUTE",
    }
}

def_enum! {
    /// C++ `EThingCreatureProperty`.
    pub enum ThingCreatureProperty: i32 {
        NULL = 0 => "THING_CREATURE_PROPERTY_NULL",
        ISMINION = 1 => "THING_CREATURE_PROPERTY_IS_MINION",
        ANNOYABLEBYKIDS = 2 => "THING_CREATURE_PROPERTY_ANNOYABLE_BY_KIDS",
        GUARD = 3 => "THING_CREATURE_PROPERTY_GUARD",
        FIREFLY = 4 => "THING_CREATURE_PROPERTY_FIREFLY",
    }
}

def_enum! {
    /// C++ `ETrapTriggerType`.
    pub enum TrapTriggerType: i32 {
        MANUAL = 0 => "TRAP_TRIGGER_MANUAL",
        PROXIMITY = 1 => "TRAP_TRIGGER_PROXIMITY",
    }
}

def_enum! {
    /// C++ `ETrapType`.
    pub enum TrapType: i32 {
        TRIGGERONCEONLY = 0 => "TRAP_TYPE_TRIGGER_ONCE_ONLY",
        TRIGGERANDRESET = 1 => "TRAP_TYPE_TRIGGER_AND_RESET",
        TRIGGERANDPLAYCONTINUOUS = 2 => "TRAP_TYPE_TRIGGER_AND_PLAY_CONTINUOUS",
        PLAYCONTINUOUS = 3 => "TRAP_TYPE_PLAY_CONTINUOUS",
        TRIGGERANDDIE = 4 => "TRAP_TYPE_TRIGGER_AND_DIE",
    }
}

def_enum! {
    /// C++ `ETutorialCategory`.
    pub enum TutorialCategory: i32 {
        NONE = 0 => "TUTORIAL_CATEGORY_NONE",
        ABILITYASSIGNING = 1 => "TUTORIAL_CATEGORY_ABILITY_ASSIGNING",
        ABILITYCYCLING = 2 => "TUTORIAL_CATEGORY_ABILITY_CYCLING",
        BASICOBJECTS = 3 => "TUTORIAL_CATEGORY_BASIC_OBJECTS",
        BED = 4 => "TUTORIAL_CATEGORY_BED",
        BOASTING = 5 => "TUTORIAL_CATEGORY_BOASTING",
        CAMERA = 6 => "TUTORIAL_CATEGORY_CAMERA",
        CHARITYSHOP = 7 => "TUTORIAL_CATEGORY_CHARITY_SHOP",
        CHEST = 8 => "TUTORIAL_CATEGORY_CHEST",
        COMBATMULTIPLIER = 9 => "TUTORIAL_CATEGORY_COMBAT_MULTIPLIER",
        CREATUREDROP = 10 => "TUTORIAL_CATEGORY_CREATURE_DROP",
        DYING = 11 => "TUTORIAL_CATEGORY_DYING",
        DEMONDOOR = 12 => "TUTORIAL_CATEGORY_DEMON_DOOR",
        DOOR = 13 => "TUTORIAL_CATEGORY_DOOR",
        EXPERIENCE = 14 => "TUTORIAL_CATEGORY_EXPERIENCE",
        EXPERIENCESPENDING = 15 => "TUTORIAL_CATEGORY_EXPERIENCE_SPENDING",
        EXPRESSION = 16 => "TUTORIAL_CATEGORY_EXPRESSION",
        FLIRTING = 17 => "TUTORIAL_CATEGORY_FLIRTING",
        FLOURISHINGMOVE = 18 => "TUTORIAL_CATEGORY_FLOURISHING_MOVE",
        GOLDMARKERS = 19 => "TUTORIAL_CATEGORY_GOLDMARKERS",
        GUILDSEAL = 20 => "TUTORIAL_CATEGORY_GUILD_SEAL",
        INTERACTING = 21 => "TUTORIAL_CATEGORY_INTERACTING",
        INVENTORY = 22 => "TUTORIAL_CATEGORY_INVENTORY",
        INVENTORYASSIGNING = 23 => "TUTORIAL_CATEGORY_INVENTORY_ASSIGNING",
        LEVELLINGUP = 24 => "TUTORIAL_CATEGORY_LEVELLING_UP",
        MORALITY = 25 => "TUTORIAL_CATEGORY_MORALITY",
        MOVEMENT = 26 => "TUTORIAL_CATEGORY_MOVEMENT",
        QUEST = 27 => "TUTORIAL_CATEGORY_QUEST",
        QUESTCARD = 28 => "TUTORIAL_CATEGORY_QUEST_CARD",
        RENOWN = 29 => "TUTORIAL_CATEGORY_RENOWN",
        TAKINGQUESTS = 30 => "TUTORIAL_CATEGORY_TAKING_QUESTS",
        TELEPORTING = 31 => "TUTORIAL_CATEGORY_TELEPORTING",
        TRADEITEM = 32 => "TUTORIAL_CATEGORY_TRADE_ITEM",
        SEARCHING = 33 => "TUTORIAL_CATEGORY_SEARCHING",
        SNEAK = 34 => "TUTORIAL_CATEGORY_SNEAK",
        BUILDINGOWNERSHIP = 35 => "TUTORIAL_CATEGORY_BUILDING_OWNERSHIP",
        FISHINGGAME = 36 => "TUTORIAL_CATEGORY_FISHING_GAME",
        ORACLEGAME = 37 => "TUTORIAL_CATEGORY_ORACLE_GAME",
        WORLDMAP = 38 => "TUTORIAL_CATEGORY_WORLD_MAP",
        ALCOHOL = 39 => "TUTORIAL_CATEGORY_ALCOHOL",
        AUGMENTATION = 40 => "TUTORIAL_CATEGORY_AUGMENTATION",
        ARMOUR = 41 => "TUTORIAL_CATEGORY_ARMOUR",
        BOMB = 42 => "TUTORIAL_CATEGORY_BOMB",
        CLOTHES = 43 => "TUTORIAL_CATEGORY_CLOTHES",
        FOOD = 44 => "TUTORIAL_CATEGORY_FOOD",
        FISHINGROD = 45 => "TUTORIAL_CATEGORY_FISHING_ROD",
        GIFT = 46 => "TUTORIAL_CATEGORY_GIFT",
        HAIRSTYLE = 47 => "TUTORIAL_CATEGORY_HAIRSTYLE",
        POTION = 48 => "TUTORIAL_CATEGORY_POTION",
        RESURRECTIONPHIAL = 49 => "TUTORIAL_CATEGORY_RESURRECTION_PHIAL",
        SILVERKEY = 50 => "TUTORIAL_CATEGORY_SILVER_KEY",
        SPADE = 51 => "TUTORIAL_CATEGORY_SPADE",
        TATTOO = 52 => "TUTORIAL_CATEGORY_TATTOO",
        TROPHY = 53 => "TUTORIAL_CATEGORY_TROPHY",
        WEAPON = 54 => "TUTORIAL_CATEGORY_WEAPON",
        WEAPONLEGENDARY = 55 => "TUTORIAL_CATEGORY_WEAPON_LEGENDARY",
        APOLOGY = 56 => "TUTORIAL_CATEGORY_APOLOGY",
        BATTLECRY = 57 => "TUTORIAL_CATEGORY_BATTLE_CRY",
        BELCH = 58 => "TUTORIAL_CATEGORY_BELCH",
        EVILLAUGH = 59 => "TUTORIAL_CATEGORY_EVIL_LAUGH",
        FART = 60 => "TUTORIAL_CATEGORY_FART",
        FLIRT = 61 => "TUTORIAL_CATEGORY_FLIRT",
        FOLLOW = 62 => "TUTORIAL_CATEGORY_FOLLOW",
        GIGGLE = 63 => "TUTORIAL_CATEGORY_GIGGLE",
        HEROICSTANCE = 64 => "TUTORIAL_CATEGORY_HEROIC_STANCE",
        MIDDLEFINGER = 65 => "TUTORIAL_CATEGORY_MIDDLE_FINGER",
        PELVICTHRUST = 66 => "TUTORIAL_CATEGORY_PELVIC_THRUST",
        PICKLOCK = 67 => "TUTORIAL_CATEGORY_PICKLOCK",
        PICKPOCKET = 68 => "TUTORIAL_CATEGORY_PICKPOCKET",
        SHIT = 69 => "TUTORIAL_CATEGORY_SHIT",
        SNEER = 70 => "TUTORIAL_CATEGORY_SNEER",
        STEAL = 71 => "TUTORIAL_CATEGORY_STEAL",
        THANKS = 72 => "TUTORIAL_CATEGORY_THANKS",
        VICTORYPUMP = 73 => "TUTORIAL_CATEGORY_VICTORY_PUMP",
        WAIT = 74 => "TUTORIAL_CATEGORY_WAIT",
        COCKADOODLEDO = 75 => "TUTORIAL_CATEGORY_COCK_A_DOODLE_DO",
        CROTCHGRAB = 76 => "TUTORIAL_CATEGORY_CROTCH_GRAB",
        KISSMYASS = 77 => "TUTORIAL_CATEGORY_KISS_MY_ASS",
        FLAMENCO = 78 => "TUTORIAL_CATEGORY_FLAMENCO",
        COSSACK = 79 => "TUTORIAL_CATEGORY_COSSACK",
        AIRGUITAR = 80 => "TUTORIAL_CATEGORY_AIR_GUITAR",
        BALLET = 81 => "TUTORIAL_CATEGORY_BALLET",
        SATURDAYNIGHTFEVER = 82 => "TUTORIAL_CATEGORY_SATURDAY_NIGHT_FEVER",
        TAP = 83 => "TUTORIAL_CATEGORY_TAP",
        Y = 84 => "TUTORIAL_CATEGORY_Y",
        M = 85 => "TUTORIAL_CATEGORY_M",
        C = 86 => "TUTORIAL_CATEGORY_C",
        A = 87 => "TUTORIAL_CATEGORY_A",
        CRIMEWEAPONOUT = 88 => "TUTORIAL_CATEGORY_CRIME_WEAPONOUT",
        CRIMETRESPASSING = 89 => "TUTORIAL_CATEGORY_CRIME_TRESPASSING",
        CRIMEVANDALISM = 90 => "TUTORIAL_CATEGORY_CRIME_VANDALISM",
        CRIMETHEFT = 91 => "TUTORIAL_CATEGORY_CRIME_THEFT",
        CRIMEASSAULT = 92 => "TUTORIAL_CATEGORY_CRIME_ASSAULT",
        CRIMEGBH = 93 => "TUTORIAL_CATEGORY_CRIME_GBH",
        CRIMEMURDER = 94 => "TUTORIAL_CATEGORY_CRIME_MURDER",
        COUNT = 95 => "TUTORIAL_CATEGORY_COUNT",
    }
}

def_enum! {
    /// C++ `EWallMountEffects`.
    pub enum WallMountEffects: i32 {
        NONE = 0 => "WALL_MOUNT_EFFECT_NONE",
        TELEPORT = 1 => "WALL_MOUNT_EFFECT_TELEPORT",
        HEAL = 2 => "WALL_MOUNT_EFFECT_HEAL",
    }
}

def_enum! {
    /// C++ `EWaterType`.
    pub enum WaterType: i32 {
        NULL = 0 => "WATER_TYPE_NULL",
        LAKE = 1 => "WATER_TYPE_LAKE",
        RIVER = 2 => "WATER_TYPE_RIVER",
        SEA = 3 => "WATER_TYPE_SEA",
        REFLECTIVESEA = 4 => "WATER_TYPE_REFLECTIVE_SEA",
        NONREFLECTIVESEA = 5 => "WATER_TYPE_NON_REFLECTIVE_SEA",
        OLD = 6 => "WATER_TYPE_OLD",
        DUMMYSHOREPOINT = 7 => "WATER_TYPE_DUMMY_SHORE_POINT",
        ICE = 8 => "WATER_TYPE_ICE",
        COUNT = 9 => "WATER_TYPE_COUNT",
    }
}

def_enum! {
    /// C++ `EWeaponClass`.
    pub enum WeaponClass: i32 {
        UNARMED = 0 => "WC_UNARMED",
        LIGHT = 1 => "WC_LIGHT",
        HEAVY = 2 => "WC_HEAVY",
        PROJECTILE = 3 => "WC_PROJECTILE",
    }
}

def_enum! {
    /// C++ `EWeaponType`.
    pub enum WeaponType: i32 {
        SWORD = 0 => "WT_SWORD",
        AXE = 1 => "WT_AXE",
        HAMMER = 2 => "WT_HAMMER",
        BOW = 3 => "WT_BOW",
        CROSSBOW = 4 => "WT_CROSSBOW",
        BOLT = 5 => "WT_BOLT",
        ARROW = 6 => "WT_ARROW",
        THROWING = 7 => "WT_THROWING",
    }
}

def_flags! {
    /// C++ `EWorldMapNameGraphic`.
    pub struct WorldMapNameGraphic: i32 {
        BANDIT_CAMP = 0 => "WORLD_MAP_NAME_GRAPHIC_BANDIT_CAMP",
        BANDIT_CAMP_TENT = 1 => "WORLD_MAP_NAME_GRAPHIC_BANDIT_CAMP_TENT",
        BANDIT_CAMP_ELITES = 2 => "WORLD_MAP_NAME_GRAPHIC_BANDIT_CAMP_ELITES",
        BANDIT_CAMP_PATH = 3 => "WORLD_MAP_NAME_GRAPHIC_BANDIT_CAMP_PATH",
        BANDIT_CAMP_GATE = 4 => "WORLD_MAP_NAME_GRAPHIC_BANDIT_CAMP_GATE",
        BOWERSTONE = 5 => "WORLD_MAP_NAME_GRAPHIC_BOWERSTONE",
        DARKWOOD = 6 => "WORLD_MAP_NAME_GRAPHIC_DARKWOOD",
        DARWOOD_SWAMP = 7 => "WORLD_MAP_NAME_GRAPHIC_DARWOOD_SWAMP",
        DARKWOOD_LAKE = 8 => "WORLD_MAP_NAME_GRAPHIC_DARKWOOD_LAKE",
        DARKWOOD_CAMP = 9 => "WORLD_MAP_NAME_GRAPHIC_DARKWOOD_CAMP",
        CHAPEL_SKORM = 10 => "WORLD_MAP_NAME_GRAPHIC_CHAPEL_SKORM",
        ANCIENT_CULLIS = 11 => "WORLD_MAP_NAME_GRAPHIC_ANCIENT_CULLIS",
        DARKWOOD_WEIR = 12 => "WORLD_MAP_NAME_GRAPHIC_DARKWOOD_WEIR",
        FISHER_CREEK = 13 => "WORLD_MAP_NAME_GRAPHIC_FISHER_CREEK",
        LYCHFIELD_GRAVEYARD = 14 => "WORLD_MAP_NAME_GRAPHIC_LYCHFIELD_GRAVEYARD",
        HEADMANS_HILL = 15 => "WORLD_MAP_NAME_GRAPHIC_HEADMANS_HILL",
        PRISON_PATH = 16 => "WORLD_MAP_NAME_GRAPHIC_PRISON_PATH",
        GIBBET_WOODS = 17 => "WORLD_MAP_NAME_GRAPHIC_GIBBET_WOODS",
        GRAVEYARD_PATH = 18 => "WORLD_MAP_NAME_GRAPHIC_GRAVEYARD_PATH",
        CIRCLE_DEAD = 19 => "WORLD_MAP_NAME_GRAPHIC_CIRCLE_DEAD",
        GREATWOOD_CAVES = 20 => "WORLD_MAP_NAME_GRAPHIC_GREATWOOD_CAVES",
        GREATWOOD_LAKE = 21 => "WORLD_MAP_NAME_GRAPHIC_GREATWOOD_LAKE",
        GREATWOOD = 22 => "WORLD_MAP_NAME_GRAPHIC_GREATWOOD",
        GUILD = 23 => "WORLD_MAP_NAME_GRAPHIC_GUILD",
        LOOKOUT_POINT = 24 => "WORLD_MAP_NAME_GRAPHIC_LOOKOUT_POINT",
        WINDMILL_HILL = 25 => "WORLD_MAP_NAME_GRAPHIC_WINDMILL_HILL",
        BOWERSTONE_JAIL = 26 => "WORLD_MAP_NAME_GRAPHIC_BOWERSTONE_JAIL",
        OAKVALE = 27 => "WORLD_MAP_NAME_GRAPHIC_OAKVALE",
        ORCHARD_FARM = 28 => "WORLD_MAP_NAME_GRAPHIC_ORCHARD_FARM",
        WITCHWOOD = 29 => "WORLD_MAP_NAME_GRAPHIC_WITCHWOOD",
        WITCHWOOD_LAKE = 30 => "WORLD_MAP_NAME_GRAPHIC_WITCHWOOD_LAKE",
        TEMPLE_AVO = 31 => "WORLD_MAP_NAME_GRAPHIC_TEMPLE_AVO",
        WITCHWOOD_STONES = 32 => "WORLD_MAP_NAME_GRAPHIC_WITCHWOOD_STONES",
        PICNIC_AREA = 33 => "WORLD_MAP_NAME_GRAPHIC_PICNIC_AREA",
        ROSE_COTTAGE = 34 => "WORLD_MAP_NAME_GRAPHIC_ROSE_COTTAGE",
        ARENA = 35 => "WORLD_MAP_NAME_GRAPHIC_ARENA",
        KNOTHOLE_GLADE = 36 => "WORLD_MAP_NAME_GRAPHIC_KNOTHOLE_GLADE",
        HOOK_COAST = 37 => "WORLD_MAP_NAME_GRAPHIC_HOOK_COAST",
        BARROW_FIELDS = 38 => "WORLD_MAP_NAME_GRAPHIC_BARROW_FIELDS",
        GREY_HOUSE = 39 => "WORLD_MAP_NAME_GRAPHIC_GREY_HOUSE",
        GREATWOOD_GORGE = 40 => "WORLD_MAP_NAME_GRAPHIC_GREATWOOD_GORGE",
        HOBBE_CAVE = 41 => "WORLD_MAP_NAME_GRAPHIC_HOBBE_CAVE",
        HEADMANS_CAVE = 42 => "WORLD_MAP_NAME_GRAPHIC_HEADMANS_CAVE",
        GREATWOOD_CULLIS = 43 => "WORLD_MAP_NAME_GRAPHIC_GREATWOOD_CULLIS",
        PRISON = 44 => "WORLD_MAP_NAME_GRAPHIC_PRISON",
        LOST_BAY = 45 => "WORLD_MAP_NAME_GRAPHIC_LOST_BAY",
        NECROPOLIS = 46 => "WORLD_MAP_NAME_GRAPHIC_NECROPOLIS",
        ARCHONS_SHRINE = 47 => "WORLD_MAP_NAME_GRAPHIC_ARCHONS_SHRINE",
        SNOWSPIRE = 48 => "WORLD_MAP_NAME_GRAPHIC_SNOWSPIRE",
        ARCHONS_FOLLY = 49 => "WORLD_MAP_NAME_GRAPHIC_ARCHONS_FOLLY",
        NORTHERN_WASTES_FOOTHILLS = 50 => "WORLD_MAP_NAME_GRAPHIC_NORTHERN_WASTES_FOOTHILLS",
        BORDELLO = 51 => "WORLD_MAP_NAME_GRAPHIC_BORDELLO",
        NUMBER = 52 => "WORLD_MAP_NAME_GRAPHIC_NUMBER",
    }
}

def_enum! {
    /// C++ `NSpeechGainManager::EDialogueLayer`.
    pub enum DialogueLayer: i32 {
        FOREGROUND = 0 => "DIALOGUE_LAYER_FOREGROUND",
        MIDGROUND = 1 => "DIALOGUE_LAYER_MIDGROUND",
        BACKGROUND = 2 => "DIALOGUE_LAYER_BACKGROUND",
        LAST = 3 => "DIALOGUE_LAYER_LAST",
    }
}
