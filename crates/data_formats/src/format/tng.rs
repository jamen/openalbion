use crate::util::{
    slice::TakeSliceExt,
    text::{Lexer, Location, Token, TokenKind},
};

#[derive(Clone, Debug)]
pub struct Tng {
    // pub raw_tng: RawTng,
}

// pub struct Tng {
//     // TODO: Use hashmap?
//     sections: Vec<TngSection>,
// }

// pub struct TngSection {
//     // TODO: Use hashmap?
//     items: Vec<TngSectionItem>,
// }

// pub enum TngSectionItem {
//     Thing(TngThing),
//     Marker(TngMarker),
//     Object(TngObject),
// }

// pub struct TngThing {}

// pub struct TngMarker {}

// pub struct TngObject {}

#[derive(Clone, Debug)]
pub struct TngParseError {
    raw: Option<RawTngParseError>,
}

impl Tng {
    pub fn parse(source: &str) -> Result<Tng, TngParseError> {
        let tokens = Lexer::tokenize(source).map_err(|_loc| TngParseError { raw: None })?;

        let raw_tng = RawTng::parse(&tokens).map_err(|raw| TngParseError { raw: Some(raw) })?;

        Self::parse_raw_tng(raw_tng)
    }

    fn parse_raw_tng(raw_tng: RawTng) -> Result<Tng, TngParseError> {
        // TODO: Temporary, to test if the RawTng parsing works
        // Ok(Self { raw_tng })
        Ok(Self {})
    }
}

#[derive(Clone, Debug)]
struct RawTng {
    list: Vec<RawTngPair>,
}

impl RawTng {
    fn parse(mut tokens: &[Token]) -> Result<RawTng, RawTngParseError> {
        let mut raw_tng = RawTng { list: vec![] };

        while !tokens.is_empty() {
            let pair = RawTngPair::parse(&mut tokens)?;
            raw_tng.list.push(pair);
            skip_whitespace(&mut tokens)?;
        }

        Ok(raw_tng)
    }
}

#[derive(Clone, Debug)]
struct RawTngPair {
    key: RawTngKey,
    value: RawTngValue,
}

impl RawTngPair {
    fn parse(mut tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        let key = RawTngKey::parse(&mut tokens)?;
        let value = RawTngValue::parse(&mut tokens)?;
        Ok(RawTngPair { key, value })
    }
}

#[derive(Clone, Debug)]
struct RawTngKey {
    ident: RawTngKeyIdentifier,
    path: Vec<RawTngKeyIndex>,
}

impl RawTngKey {
    fn parse(mut tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        let ident = RawTngKeyIdentifier::parse(&mut tokens)?;

        let mut path = vec![];

        loop {
            let maybe_sep_token = tokens.first().ok_or_else(|| RawTngParseError {
                location: None,
                kind: E::UnexpectedEOF,
            })?;

            if (maybe_sep_token.kind == T::Whitespace && maybe_sep_token.text == " ")
                || (maybe_sep_token.kind == T::Symbol && maybe_sep_token.text == ";")
            {
                break;
            }

            let index = RawTngKeyIndex::parse(tokens)?;

            path.push(index);
        }

        Ok(RawTngKey { ident, path })
    }
}

#[derive(Clone, Debug)]
enum RawTngKeyIndex {
    Array(u64),
    Property(RawTngKeyProperty),
    Call,
}

impl RawTngKeyIndex {
    fn parse(tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        let index_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        match (index_token.kind, index_token.text) {
            (T::Symbol, ".") => Ok(RawTngKeyIndex::Property(RawTngKeyProperty::parse(tokens)?)),
            (T::Symbol, "[") => Ok(Self::parse_array_index(tokens)?),
            (T::Symbol, "(") => Ok(Self::parse_call(tokens)?),
            _ => Err(RawTngParseError {
                location: Some(index_token.location),
                kind: E::UnexpectedToken,
            }),
        }
    }

    fn parse_array_index(tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        let &integer_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        if integer_token.kind != T::Integer {
            return Err(RawTngParseError {
                location: Some(integer_token.location),
                kind: E::UnexpectedToken,
            });
        }

        let index = match integer_token.text.parse::<u64>() {
            Ok(index) => Ok(Self::Array(index)),
            Err(_) => Err(RawTngParseError {
                location: Some(integer_token.location),
                kind: E::ParseIntError,
            }),
        }?;

        let closing_bracket_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        if closing_bracket_token.kind != T::Symbol && closing_bracket_token.text != "]" {
            return Err(RawTngParseError {
                location: Some(integer_token.location),
                kind: E::UnexpectedToken,
            });
        }

        Ok(index)
    }

    fn parse_call(tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        // I haven't seen any instance of this syntax where arguments are supplied
        // So we simply look for ")" and move on

        let paren_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        if paren_token.kind != T::Symbol || paren_token.text != ")" {
            return Err(RawTngParseError {
                location: Some(paren_token.location),
                kind: E::UnexpectedToken,
            });
        }

        Ok(RawTngKeyIndex::Call)
    }
}

#[derive(Clone, Debug)]
enum RawTngValue {
    Integer(i32),
    Uid(u64),
    Float(f32),
    Boolean(bool),
    Identifier(String),
    String(String),
    Struct(RawTngStructName, Vec<RawTngValue>),
    Empty,
}

impl RawTngValue {
    fn parse(tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        let value = Self::parse_value(tokens)?;

        Self::parse_closing(tokens)?;

        Ok(value)
    }

    fn parse_value(tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        skip_spaces(tokens)?;

        // NOTE: Instead of taking the token off the stack we just look at it.
        let value_token = tokens.first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        match (value_token.kind, value_token.text) {
            (T::Integer, _) => Self::parse_integer(tokens),
            (T::Uid, _) => Self::parse_uid(tokens),
            (T::Float, _) => Self::parse_float(tokens),
            // (T::String, _) => Self::parse_string(tokens),
            (T::Symbol, "\"") => Self::parse_string(tokens),
            (T::Identifier, "TRUE") => Ok(RawTngValue::Boolean(true)),
            (T::Identifier, "FALSE") => Ok(RawTngValue::Boolean(false)),
            (T::Identifier, _) => Self::parse_ident(tokens),
            (T::Symbol, ";") => Ok(RawTngValue::Empty),
            _ => Err(RawTngParseError {
                location: Some(value_token.location),
                kind: E::UnexpectedToken,
            }),
        }
    }

    fn parse_integer(tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        let integer_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        if integer_token.kind != T::Integer {
            return Err(RawTngParseError {
                location: Some(integer_token.location),
                kind: E::UnexpectedToken,
            });
        }

        match integer_token.text.parse::<i32>() {
            Ok(integer) => Ok(Self::Integer(integer)),
            Err(_) => Err(RawTngParseError {
                location: Some(integer_token.location),
                kind: E::ParseIntError,
            }),
        }
    }

    fn parse_uid(tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        let uid_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        if uid_token.kind != T::Uid {
            return Err(RawTngParseError {
                location: Some(uid_token.location),
                kind: E::UnexpectedToken,
            });
        }

        match uid_token.text.parse::<u64>() {
            Ok(integer) => Ok(Self::Uid(integer)),
            Err(_) => Err(RawTngParseError {
                location: Some(uid_token.location),
                kind: E::ParseIntError,
            }),
        }
    }

    fn parse_float(tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        let float_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        if float_token.kind != T::Float {
            return Err(RawTngParseError {
                location: Some(float_token.location),
                kind: E::UnexpectedToken,
            });
        }

        match float_token.text.parse::<f32>() {
            Ok(integer) => Ok(Self::Float(integer)),
            Err(_) => Err(RawTngParseError {
                location: Some(float_token.location),
                kind: E::ParseIntError,
            }),
        }
    }

    fn parse_string(tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        let &open_quote = tokens.grab_first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        if open_quote.kind != T::Symbol || open_quote.text != "\"" {
            return Err(RawTngParseError {
                location: Some(open_quote.location),
                kind: E::UnexpectedToken,
            });
        }

        let &string_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        if string_token.kind != T::String {
            return Err(RawTngParseError {
                location: Some(string_token.location),
                kind: E::UnexpectedToken,
            });
        }

        let &close_quote = tokens.grab_first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        if close_quote.kind != T::Symbol || close_quote.text != "\"" {
            return Err(RawTngParseError {
                location: Some(close_quote.location),
                kind: E::UnexpectedToken,
            });
        }

        Ok(Self::String(string_token.text.to_string()))
    }

    fn parse_ident(tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        let &ident_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        if ident_token.kind != T::Identifier {
            return Err(RawTngParseError {
                location: Some(ident_token.location),
                kind: E::UnexpectedToken,
            });
        }

        skip_spaces(tokens)?;

        // Look at next token to determine if ident may be struct-like or series-like.
        let next_token = tokens.first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        if next_token.kind == T::Symbol && next_token.text == "(" {
            let args = Self::parse_struct_args(tokens)?;
            let ident = ident_token.text.to_string();
            return Ok(Self::Struct(ident, args));
        }

        // Handle the case of identifiers having spaces in them, like "Holy Site".
        if next_token.kind == T::Identifier {
            let mut ident_series = vec![ident_token.text.to_string()];

            loop {
                skip_spaces(tokens)?;

                let next_in_series_token = tokens.first().ok_or_else(|| RawTngParseError {
                    location: None,
                    kind: E::UnexpectedEOF,
                })?;

                match (next_in_series_token.kind, next_in_series_token.text) {
                    (T::Symbol, ";") => break,
                    (T::Identifier, ident) => ident_series.push(ident.to_string()),
                    _ => {
                        return Err(RawTngParseError {
                            location: Some(ident_token.location),
                            kind: E::UnexpectedToken,
                        })
                    }
                }

                let _ = tokens.grab_first();
            }

            let ident = ident_series.join(" ");

            return Ok(Self::Identifier(ident));
        }

        Ok(Self::Identifier(ident_token.text.to_string()))
    }

    fn parse_struct_args(tokens: &mut &[Token]) -> Result<Vec<Self>, RawTngParseError> {
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        let &open_paren_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        if open_paren_token.kind != T::Symbol || open_paren_token.text != "(" {
            return Err(RawTngParseError {
                location: Some(open_paren_token.location),
                kind: E::UnexpectedToken,
            });
        }

        let mut args = Vec::new();

        loop {
            skip_spaces(tokens)?;

            let arg = Self::parse_value(tokens)?;

            args.push(arg);

            let &next_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
                location: None,
                kind: E::UnexpectedEOF,
            })?;

            match (next_token.kind, next_token.text) {
                (T::Symbol, ")") => break,
                (T::Symbol, ",") => {}
                _ => {
                    return Err(RawTngParseError {
                        location: Some(next_token.location),
                        kind: E::UnexpectedToken,
                    });
                }
            }
        }

        Ok(args)
    }

    fn parse_closing(tokens: &mut &[Token]) -> Result<(), RawTngParseError> {
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        skip_spaces(tokens)?;

        let close_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        if close_token.kind != T::Symbol && close_token.text != ";" {
            return Err(RawTngParseError {
                location: Some(close_token.location),
                kind: E::UnexpectedToken,
            });
        }

        Ok(())
    }
}

fn skip_spaces(tokens: &mut &[Token]) -> Result<(), RawTngParseError> {
    loop {
        match tokens.first() {
            Some(Token {
                kind: TokenKind::Whitespace,
                text: " ",
                ..
            }) => {
                let _ = tokens.grab_first();
            }
            _ => return Ok(()),
        }
    }
}

fn skip_whitespace(tokens: &mut &[Token]) -> Result<(), RawTngParseError> {
    loop {
        match tokens.first() {
            Some(Token {
                kind: TokenKind::Whitespace,
                ..
            }) => {
                let _ = tokens.grab_first();
            }
            _ => return Ok(()),
        }
    }
}

#[derive(Clone, Debug)]
struct RawTngParseError {
    location: Option<Location>,
    kind: RawTngParseErrorKind,
}

#[derive(Copy, Clone, Debug)]
enum RawTngParseErrorKind {
    UnexpectedEOF,
    UnexpectedToken,
    UnrecognizedIdentifier,
    ParseIntError,
    ParseBoolError,
}

#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
enum RawTngKeyProperty {
    AnimationSpeed,
    Duration,
    Event,
    FOV,
    LookDirection,
    PauseTime,
    Position,
    RollAngle,
    ShuttleSpeed,
    Type,
    X,
    Y,
    Z,
    pos,
    size,
}

impl RawTngKeyProperty {
    fn parse(tokens: &mut &[Token]) -> Result<RawTngKeyProperty, RawTngParseError> {
        use RawTngKeyProperty as P;
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        let ident_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        if ident_token.kind != T::Identifier {
            return Err(RawTngParseError {
                location: Some(ident_token.location),
                kind: E::UnexpectedToken,
            });
        }

        Ok(match ident_token.text {
            "AnimationSpeed" => P::AnimationSpeed,
            "Duration" => P::Duration,
            "Event" => P::Event,
            "FOV" => P::FOV,
            "LookDirection" => P::LookDirection,
            "PauseTime" => P::PauseTime,
            "Position" => P::Position,
            "RollAngle" => P::RollAngle,
            "ShuttleSpeed" => P::ShuttleSpeed,
            "Type" => P::Type,
            "X" => P::X,
            "Y" => P::Y,
            "Z" => P::Z,
            "pos" => P::pos,
            "size" => P::size,
            _ => {
                return Err(RawTngParseError {
                    location: Some(ident_token.location),
                    kind: E::UnrecognizedIdentifier,
                })
            }
        })
    }
}

#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
enum RawTngKeyIdentifier {
    ActivateOnActivate,
    Active,
    ActiveCreatureLimit,
    AllowRightStickRotation,
    AllowRightStickZoom,
    AllowZTarget,
    AllowedToFollowHero,
    AlreadyRead,
    Angle,
    AnimationName,
    AtmosName,
    AugmentationDefNames,
    AutoGoBehind,
    AutoGoBehindTime,
    BestWitnessesAheadToDate,
    BooleanHusbandAppearances,
    BoughtForAmount,
    BribePool,
    CageRadius,
    CameraTrackUID,
    CanBeCourted,
    CanBeMarried,
    CanComeBetweenCameraAndHero,
    ChestOpen,
    Colour,
    ContainerContents,
    ContinueAIWithInformation,
    CoordAxisFwd,
    CoordAxisUp,
    CoordBase,
    CourtingBlocked,
    CreateTC,
    CreatureFamilies,
    CurrentDressLevel,
    CurrentIsHeroCriminal,
    CutInto,
    CutOutOf,
    DayNextRentIsDue,
    DeactivateAfterSetTime,
    DefinitionType,
    DisplayTime,
    DivorcedHero,
    DoorTriggerType,
    DoorType2,
    EnableCreatureAutoPlacing,
    EnableFollowersEnemyProxy,
    End,
    EndCTCAIScratchpad,
    EndCTCActionUseBed,
    EndCTCActionUseReadable,
    EndCTCActionUseScriptedHook,
    EndCTCActivationReceptorCreatureGenerator,
    EndCTCActivationReceptorDoor,
    EndCTCActivationTrigger,
    EndCTCAtmosPlayer,
    EndCTCBoastingArea,
    EndCTCBuyableHouse,
    EndCTCCameraPointFixedPoint,
    EndCTCCameraPointGeneralCase,
    EndCTCCameraPointScripted,
    EndCTCCameraPointScriptedSpline,
    EndCTCCameraPointTrack,
    EndCTCCarriedActionUseRead,
    EndCTCChest,
    EndCTCContainerRewardHero,
    EndCTCCreatureGenerator,
    EndCTCCreatureGeneratorCreator,
    EndCTCCreatureOpinionOfHero,
    EndCTCDCameraPoint,
    EndCTCDNavigationSeed,
    EndCTCDParticleEmitter,
    EndCTCDRegionEntrance,
    EndCTCDRegionExit,
    EndCTCDiggingSpot,
    EndCTCDoor,
    EndCTCEditor,
    EndCTCEnemy,
    EndCTCExplodingObject,
    EndCTCFishingSpot,
    EndCTCGuard,
    EndCTCHero,
    EndCTCHeroCentreDoorMarker,
    EndCTCInfoDisplay,
    EndCTCInventoryItem,
    EndCTCLight,
    EndCTCObjectAugmentations,
    EndCTCOwnedEntity,
    EndCTCPhysicsLight,
    EndCTCPhysicsNavigator,
    EndCTCPhysicsStandard,
    EndCTCPreCalculatedNavigationRoute,
    EndCTCRandomAppearanceMorph,
    EndCTCSearchableContainer,
    EndCTCShapeManager,
    EndCTCShop,
    EndCTCSpotLight,
    EndCTCStealableItemLocation,
    EndCTCStockItem,
    EndCTCTalk,
    EndCTCTargeted,
    EndCTCTeleporter,
    EndCTCTrophy,
    EndCTCVillage,
    EndCTCVillageMember,
    EndCTCWallMount,
    EndCTCWife,
    EndPos,
    EndThing,
    EntranceConnectedToUID,
    EnvironmentDef,
    FOV,
    FactionName,
    FireDamage,
    Flicker,
    ForSale,
    ForceConfirmation,
    ForcedAttitude,
    FrameEnteredAttitudeHate,
    FrameEnteredLoveWithHusbandPresentAtHome,
    FrameGotMarriedToThePlayer,
    FrameLastAwareOfHusband,
    FrameLastBribeAdded,
    FrameLastConsideredGivingGift,
    FrameLastCrimeSeen,
    FrameLastCulledGiftsReceived,
    FrameLastEvaluatedGiftOpinion,
    FrameLastEvaluatedLoveAttitude,
    FrameLastGaveDivorceWarning,
    FrameLastGaveSexOffer,
    FrameLastReceivedApology,
    FrameLastReceivedNiceGift,
    FrameLastReducedOpinion,
    FramePendingCrimesAdded,
    FramePlayerLastSeenByGuard,
    FrameToCheckAppearanceChanges,
    FrameToDecayNumberOfTimesHit,
    FramesAfterActivationToDeactivate,
    FriendsWithEverythingFlag,
    GameTextDefName,
    GenerationRadius,
    GiftGivingOpinionDistanceFromMax,
    GiftGivingPriceValue,
    GiftToGiveDef,
    GreetedFlag,
    HasBeenInLoveWithPlayer,
    HasBeenInitiallyPopulated,
    HasInformation,
    Health,
    HeightOffset,
    HeroIsSubject,
    HeroOpinionEnemy,
    Hidden,
    HiddenOnMiniMap,
    HomeBuildingUID,
    HouseDressingLevelLastCommentedOn,
    IndependantObject,
    InitialPosX,
    InitialPosY,
    InitialPosZ,
    InnerRadius,
    InteractedFlag,
    InventoryUID,
    Inverted,
    IsCoordBaseRelativeToParent,
    IsCoordsRelativeToMap,
    IsEnemyBecauseOfCrime,
    IsResidential,
    IsScripted,
    JustMarried,
    KeyCameras,
    LastCrimeSeenSeverity,
    LastFatnessChangePoint,
    LastOpinionReactionFrame,
    LastWeaponEquippedID,
    Limbo,
    LinkedToUID1,
    LinkedToUID2,
    LockedInPlace,
    LookDirection,
    LookDirectionEnd,
    LookVector,
    LoveAttitudeValue,
    MaxDamage,
    MessageRadius,
    Mountable,
    NavLayer0,
    NavLayer1,
    NavLayer2,
    NavLayer3,
    NavLayer4,
    NavLayer5,
    NavLayer6,
    NavLayer7,
    NavPosition0,
    NavPosition1,
    NavPosition2,
    NavPosition3,
    NavPosition4,
    NavPosition5,
    NavPosition6,
    NavPosition7,
    NeedsToChangeBrain,
    NewThing,
    NumKeyCameras,
    NumShapes,
    NumTriggers,
    NumberOfStepsOnRoute,
    NumberOfTimesHit,
    NumberOfTimesToSearch,
    ObjectScale,
    Open,
    OuterRadius,
    Overridden,
    OverridingBrainName,
    OwnedByHero,
    OwnedByPlayer,
    OwnerUID,
    ParticleTypeName,
    PermittedToRegionFollow,
    Player,
    PositionX,
    PositionY,
    PositionZ,
    PrecCalculatedNavigationRouteVersion,
    Price,
    RHSetForwardX,
    RHSetForwardY,
    RHSetForwardZ,
    RHSetUpX,
    RHSetUpY,
    RHSetUpZ,
    Radius,
    RadiusToBeWithin,
    RadiusToTakeItemsBackTo,
    ReceivedWeddingRing,
    ReceptorUID,
    RegionFollowingOverriddenFromScript,
    Rented,
    ReplacementObject,
    RespondingToFollowAndWait,
    ReversedOnMiniMap,
    SavedInGame,
    ScriptData,
    ScriptName,
    ScriptNameOfAllGeneratedCreatures,
    Seed,
    SelfTerminate,
    SelfTrigger,
    SelfTriggerRadius,
    SelfTriggerResetInterval,
    Shape,
    SoundName,
    Start,
    StartCTCAIScratchpad,
    StartCTCActionUseBed,
    StartCTCActionUseReadable,
    StartCTCActionUseScriptedHook,
    StartCTCActivationReceptorCreatureGenerator,
    StartCTCActivationReceptorDoor,
    StartCTCActivationTrigger,
    StartCTCAtmosPlayer,
    StartCTCBoastingArea,
    StartCTCBuyableHouse,
    StartCTCCameraPointFixedPoint,
    StartCTCCameraPointGeneralCase,
    StartCTCCameraPointScripted,
    StartCTCCameraPointScriptedSpline,
    StartCTCCameraPointTrack,
    StartCTCCarriedActionUseRead,
    StartCTCChest,
    StartCTCContainerRewardHero,
    StartCTCCreatureGenerator,
    StartCTCCreatureGeneratorCreator,
    StartCTCCreatureOpinionOfHero,
    StartCTCDCameraPoint,
    StartCTCDNavigationSeed,
    StartCTCDParticleEmitter,
    StartCTCDRegionEntrance,
    StartCTCDRegionExit,
    StartCTCDiggingSpot,
    StartCTCDoor,
    StartCTCEditor,
    StartCTCEnemy,
    StartCTCExplodingObject,
    StartCTCFishingSpot,
    StartCTCGuard,
    StartCTCHero,
    StartCTCHeroCentreDoorMarker,
    StartCTCInfoDisplay,
    StartCTCInventoryItem,
    StartCTCLight,
    StartCTCObjectAugmentations,
    StartCTCOwnedEntity,
    StartCTCPhysicsLight,
    StartCTCPhysicsNavigator,
    StartCTCPhysicsStandard,
    StartCTCPreCalculatedNavigationRoute,
    StartCTCRandomAppearanceMorph,
    StartCTCSearchableContainer,
    StartCTCShapeManager,
    StartCTCShop,
    StartCTCSpotLight,
    StartCTCStealableItemLocation,
    StartCTCStockItem,
    StartCTCTalk,
    StartCTCTargeted,
    StartCTCTeleporter,
    StartCTCTrophy,
    StartCTCVillage,
    StartCTCVillageMember,
    StartCTCWallMount,
    StartCTCWife,
    StartPos,
    Stealable,
    StringLength,
    SwitchableNavigationTCAdded,
    Targetable,
    TeleportToRegionEntrance,
    Tension,
    TestAngleBeforeActivation,
    TextTag,
    TextTagBack,
    ThingGamePersistent,
    ThingLevelPersistent,
    ThingToCalculateRouteToUID,
    ThingUID,
    TimeToChangeEnvironmentDef,
    TimeToPlay,
    ToleranceToBeingHitOverride,
    TotalGenerationLimit,
    TrackThing,
    TransitionTime,
    TriggerOnActivate,
    TriggerRadius,
    TriggeredByThing,
    TriggeredOnCreatureProximity,
    TrophyID,
    UID,
    Usable,
    UseableByHero,
    UsingRelativeCoords,
    UsingRelativeOrientation,
    ValidAnims,
    Version,
    VersionNumber,
    VillageUID,
    VirtualMoneyBags,
    WanderWithInformation,
    WaveWithInformation,
    Width,
    WifeLivingHere,
    WorkBuildingUID,
    XXXSectionEnd,
    XXXSectionStart,
    hero_title_object_def_name,
}

impl RawTngKeyIdentifier {
    fn parse(tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        use RawTngKeyIdentifier as I;
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        skip_whitespace(tokens)?;

        let ident_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
            location: None,
            kind: E::UnexpectedEOF,
        })?;

        if ident_token.kind != T::Identifier {
            return Err(RawTngParseError {
                location: Some(ident_token.location),
                kind: E::UnexpectedToken,
            });
        }

        Ok(match ident_token.text {
            "ActivateOnActivate" => I::ActivateOnActivate,
            "Active" => I::Active,
            "ActiveCreatureLimit" => I::ActiveCreatureLimit,
            "AllowRightStickRotation" => I::AllowRightStickRotation,
            "AllowRightStickZoom" => I::AllowRightStickZoom,
            "AllowZTarget" => I::AllowZTarget,
            "AllowedToFollowHero" => I::AllowedToFollowHero,
            "AlreadyRead" => I::AlreadyRead,
            "Angle" => I::Angle,
            "AnimationName" => I::AnimationName,
            "AtmosName" => I::AtmosName,
            "AugmentationDefNames" => I::AugmentationDefNames,
            "AutoGoBehind" => I::AutoGoBehind,
            "AutoGoBehindTime" => I::AutoGoBehindTime,
            "BestWitnessesAheadToDate" => I::BestWitnessesAheadToDate,
            "BooleanHusbandAppearances" => I::BooleanHusbandAppearances,
            "BoughtForAmount" => I::BoughtForAmount,
            "BribePool" => I::BribePool,
            "CageRadius" => I::CageRadius,
            "CameraTrackUID" => I::CameraTrackUID,
            "CanBeCourted" => I::CanBeCourted,
            "CanBeMarried" => I::CanBeMarried,
            "CanComeBetweenCameraAndHero" => I::CanComeBetweenCameraAndHero,
            "ChestOpen" => I::ChestOpen,
            "Colour" => I::Colour,
            "ContainerContents" => I::ContainerContents,
            "ContinueAIWithInformation" => I::ContinueAIWithInformation,
            "CoordAxisFwd" => I::CoordAxisFwd,
            "CoordAxisUp" => I::CoordAxisUp,
            "CoordBase" => I::CoordBase,
            "CourtingBlocked" => I::CourtingBlocked,
            "CreateTC" => I::CreateTC,
            "CreatureFamilies" => I::CreatureFamilies,
            "CurrentDressLevel" => I::CurrentDressLevel,
            "CurrentIsHeroCriminal" => I::CurrentIsHeroCriminal,
            "CutInto" => I::CutInto,
            "CutOutOf" => I::CutOutOf,
            "DayNextRentIsDue" => I::DayNextRentIsDue,
            "DeactivateAfterSetTime" => I::DeactivateAfterSetTime,
            "DefinitionType" => I::DefinitionType,
            "DisplayTime" => I::DisplayTime,
            "DivorcedHero" => I::DivorcedHero,
            "DoorTriggerType" => I::DoorTriggerType,
            "DoorType2" => I::DoorType2,
            "EnableCreatureAutoPlacing" => I::EnableCreatureAutoPlacing,
            "EnableFollowersEnemyProxy" => I::EnableFollowersEnemyProxy,
            "End" => I::End,
            "EndCTCAIScratchpad" => I::EndCTCAIScratchpad,
            "EndCTCActionUseBed" => I::EndCTCActionUseBed,
            "EndCTCActionUseReadable" => I::EndCTCActionUseReadable,
            "EndCTCActionUseScriptedHook" => I::EndCTCActionUseScriptedHook,
            "EndCTCActivationReceptorCreatureGenerator" => {
                I::EndCTCActivationReceptorCreatureGenerator
            }
            "EndCTCActivationReceptorDoor" => I::EndCTCActivationReceptorDoor,
            "EndCTCActivationTrigger" => I::EndCTCActivationTrigger,
            "EndCTCAtmosPlayer" => I::EndCTCAtmosPlayer,
            "EndCTCBoastingArea" => I::EndCTCBoastingArea,
            "EndCTCBuyableHouse" => I::EndCTCBuyableHouse,
            "EndCTCCameraPointFixedPoint" => I::EndCTCCameraPointFixedPoint,
            "EndCTCCameraPointGeneralCase" => I::EndCTCCameraPointGeneralCase,
            "EndCTCCameraPointScripted" => I::EndCTCCameraPointScripted,
            "EndCTCCameraPointScriptedSpline" => I::EndCTCCameraPointScriptedSpline,
            "EndCTCCameraPointTrack" => I::EndCTCCameraPointTrack,
            "EndCTCCarriedActionUseRead" => I::EndCTCCarriedActionUseRead,
            "EndCTCChest" => I::EndCTCChest,
            "EndCTCContainerRewardHero" => I::EndCTCContainerRewardHero,
            "EndCTCCreatureGenerator" => I::EndCTCCreatureGenerator,
            "EndCTCCreatureGeneratorCreator" => I::EndCTCCreatureGeneratorCreator,
            "EndCTCCreatureOpinionOfHero" => I::EndCTCCreatureOpinionOfHero,
            "EndCTCDCameraPoint" => I::EndCTCDCameraPoint,
            "EndCTCDNavigationSeed" => I::EndCTCDNavigationSeed,
            "EndCTCDParticleEmitter" => I::EndCTCDParticleEmitter,
            "EndCTCDRegionEntrance" => I::EndCTCDRegionEntrance,
            "EndCTCDRegionExit" => I::EndCTCDRegionExit,
            "EndCTCDiggingSpot" => I::EndCTCDiggingSpot,
            "EndCTCDoor" => I::EndCTCDoor,
            "EndCTCEditor" => I::EndCTCEditor,
            "EndCTCEnemy" => I::EndCTCEnemy,
            "EndCTCExplodingObject" => I::EndCTCExplodingObject,
            "EndCTCFishingSpot" => I::EndCTCFishingSpot,
            "EndCTCGuard" => I::EndCTCGuard,
            "EndCTCHero" => I::EndCTCHero,
            "EndCTCHeroCentreDoorMarker" => I::EndCTCHeroCentreDoorMarker,
            "EndCTCInfoDisplay" => I::EndCTCInfoDisplay,
            "EndCTCInventoryItem" => I::EndCTCInventoryItem,
            "EndCTCLight" => I::EndCTCLight,
            "EndCTCObjectAugmentations" => I::EndCTCObjectAugmentations,
            "EndCTCOwnedEntity" => I::EndCTCOwnedEntity,
            "EndCTCPhysicsLight" => I::EndCTCPhysicsLight,
            "EndCTCPhysicsNavigator" => I::EndCTCPhysicsNavigator,
            "EndCTCPhysicsStandard" => I::EndCTCPhysicsStandard,
            "EndCTCPreCalculatedNavigationRoute" => I::EndCTCPreCalculatedNavigationRoute,
            "EndCTCRandomAppearanceMorph" => I::EndCTCRandomAppearanceMorph,
            "EndCTCSearchableContainer" => I::EndCTCSearchableContainer,
            "EndCTCShapeManager" => I::EndCTCShapeManager,
            "EndCTCShop" => I::EndCTCShop,
            "EndCTCSpotLight" => I::EndCTCSpotLight,
            "EndCTCStealableItemLocation" => I::EndCTCStealableItemLocation,
            "EndCTCStockItem" => I::EndCTCStockItem,
            "EndCTCTalk" => I::EndCTCTalk,
            "EndCTCTargeted" => I::EndCTCTargeted,
            "EndCTCTeleporter" => I::EndCTCTeleporter,
            "EndCTCTrophy" => I::EndCTCTrophy,
            "EndCTCVillage" => I::EndCTCVillage,
            "EndCTCVillageMember" => I::EndCTCVillageMember,
            "EndCTCWallMount" => I::EndCTCWallMount,
            "EndCTCWife" => I::EndCTCWife,
            "EndPos" => I::EndPos,
            "EndThing" => I::EndThing,
            "EntranceConnectedToUID" => I::EntranceConnectedToUID,
            "EnvironmentDef" => I::EnvironmentDef,
            "FOV" => I::FOV,
            "FactionName" => I::FactionName,
            "FireDamage" => I::FireDamage,
            "Flicker" => I::Flicker,
            "ForSale" => I::ForSale,
            "ForceConfirmation" => I::ForceConfirmation,
            "ForcedAttitude" => I::ForcedAttitude,
            "FrameEnteredAttitudeHate" => I::FrameEnteredAttitudeHate,
            "FrameEnteredLoveWithHusbandPresentAtHome" => {
                I::FrameEnteredLoveWithHusbandPresentAtHome
            }
            "FrameGotMarriedToThePlayer" => I::FrameGotMarriedToThePlayer,
            "FrameLastAwareOfHusband" => I::FrameLastAwareOfHusband,
            "FrameLastBribeAdded" => I::FrameLastBribeAdded,
            "FrameLastConsideredGivingGift" => I::FrameLastConsideredGivingGift,
            "FrameLastCrimeSeen" => I::FrameLastCrimeSeen,
            "FrameLastCulledGiftsReceived" => I::FrameLastCulledGiftsReceived,
            "FrameLastEvaluatedGiftOpinion" => I::FrameLastEvaluatedGiftOpinion,
            "FrameLastEvaluatedLoveAttitude" => I::FrameLastEvaluatedLoveAttitude,
            "FrameLastGaveDivorceWarning" => I::FrameLastGaveDivorceWarning,
            "FrameLastGaveSexOffer" => I::FrameLastGaveSexOffer,
            "FrameLastReceivedApology" => I::FrameLastReceivedApology,
            "FrameLastReceivedNiceGift" => I::FrameLastReceivedNiceGift,
            "FrameLastReducedOpinion" => I::FrameLastReducedOpinion,
            "FramePendingCrimesAdded" => I::FramePendingCrimesAdded,
            "FramePlayerLastSeenByGuard" => I::FramePlayerLastSeenByGuard,
            "FrameToCheckAppearanceChanges" => I::FrameToCheckAppearanceChanges,
            "FrameToDecayNumberOfTimesHit" => I::FrameToDecayNumberOfTimesHit,
            "FramesAfterActivationToDeactivate" => I::FramesAfterActivationToDeactivate,
            "FriendsWithEverythingFlag" => I::FriendsWithEverythingFlag,
            "GameTextDefName" => I::GameTextDefName,
            "GenerationRadius" => I::GenerationRadius,
            "GiftGivingOpinionDistanceFromMax" => I::GiftGivingOpinionDistanceFromMax,
            "GiftGivingPriceValue" => I::GiftGivingPriceValue,
            "GiftToGiveDef" => I::GiftToGiveDef,
            "GreetedFlag" => I::GreetedFlag,
            "HasBeenInLoveWithPlayer" => I::HasBeenInLoveWithPlayer,
            "HasBeenInitiallyPopulated" => I::HasBeenInitiallyPopulated,
            "HasInformation" => I::HasInformation,
            "Health" => I::Health,
            "HeightOffset" => I::HeightOffset,
            "HeroIsSubject" => I::HeroIsSubject,
            "HeroOpinionEnemy" => I::HeroOpinionEnemy,
            "Hidden" => I::Hidden,
            "HiddenOnMiniMap" => I::HiddenOnMiniMap,
            "HomeBuildingUID" => I::HomeBuildingUID,
            "HouseDressingLevelLastCommentedOn" => I::HouseDressingLevelLastCommentedOn,
            "IndependantObject" => I::IndependantObject,
            "InitialPosX" => I::InitialPosX,
            "InitialPosY" => I::InitialPosY,
            "InitialPosZ" => I::InitialPosZ,
            "InnerRadius" => I::InnerRadius,
            "InteractedFlag" => I::InteractedFlag,
            "InventoryUID" => I::InventoryUID,
            "Inverted" => I::Inverted,
            "IsCoordBaseRelativeToParent" => I::IsCoordBaseRelativeToParent,
            "IsCoordsRelativeToMap" => I::IsCoordsRelativeToMap,
            "IsEnemyBecauseOfCrime" => I::IsEnemyBecauseOfCrime,
            "IsResidential" => I::IsResidential,
            "IsScripted" => I::IsScripted,
            "JustMarried" => I::JustMarried,
            "KeyCameras" => I::KeyCameras,
            "LastCrimeSeenSeverity" => I::LastCrimeSeenSeverity,
            "LastFatnessChangePoint" => I::LastFatnessChangePoint,
            "LastOpinionReactionFrame" => I::LastOpinionReactionFrame,
            "LastWeaponEquippedID" => I::LastWeaponEquippedID,
            "Limbo" => I::Limbo,
            "LinkedToUID1" => I::LinkedToUID1,
            "LinkedToUID2" => I::LinkedToUID2,
            "LockedInPlace" => I::LockedInPlace,
            "LookDirection" => I::LookDirection,
            "LookDirectionEnd" => I::LookDirectionEnd,
            "LookVector" => I::LookVector,
            "LoveAttitudeValue" => I::LoveAttitudeValue,
            "MaxDamage" => I::MaxDamage,
            "MessageRadius" => I::MessageRadius,
            "Mountable" => I::Mountable,
            "NavLayer0" => I::NavLayer0,
            "NavLayer1" => I::NavLayer1,
            "NavLayer2" => I::NavLayer2,
            "NavLayer3" => I::NavLayer3,
            "NavLayer4" => I::NavLayer4,
            "NavLayer5" => I::NavLayer5,
            "NavLayer6" => I::NavLayer6,
            "NavLayer7" => I::NavLayer7,
            "NavPosition0" => I::NavPosition0,
            "NavPosition1" => I::NavPosition1,
            "NavPosition2" => I::NavPosition2,
            "NavPosition3" => I::NavPosition3,
            "NavPosition4" => I::NavPosition4,
            "NavPosition5" => I::NavPosition5,
            "NavPosition6" => I::NavPosition6,
            "NavPosition7" => I::NavPosition7,
            "NeedsToChangeBrain" => I::NeedsToChangeBrain,
            "NewThing" => I::NewThing,
            "NumKeyCameras" => I::NumKeyCameras,
            "NumShapes" => I::NumShapes,
            "NumTriggers" => I::NumTriggers,
            "NumberOfStepsOnRoute" => I::NumberOfStepsOnRoute,
            "NumberOfTimesHit" => I::NumberOfTimesHit,
            "NumberOfTimesToSearch" => I::NumberOfTimesToSearch,
            "ObjectScale" => I::ObjectScale,
            "Open" => I::Open,
            "OuterRadius" => I::OuterRadius,
            "Overridden" => I::Overridden,
            "OverridingBrainName" => I::OverridingBrainName,
            "OwnedByHero" => I::OwnedByHero,
            "OwnedByPlayer" => I::OwnedByPlayer,
            "OwnerUID" => I::OwnerUID,
            "ParticleTypeName" => I::ParticleTypeName,
            "PermittedToRegionFollow" => I::PermittedToRegionFollow,
            "Player" => I::Player,
            "PositionX" => I::PositionX,
            "PositionY" => I::PositionY,
            "PositionZ" => I::PositionZ,
            "PrecCalculatedNavigationRouteVersion" => I::PrecCalculatedNavigationRouteVersion,
            "Price" => I::Price,
            "RHSetForwardX" => I::RHSetForwardX,
            "RHSetForwardY" => I::RHSetForwardY,
            "RHSetForwardZ" => I::RHSetForwardZ,
            "RHSetUpX" => I::RHSetUpX,
            "RHSetUpY" => I::RHSetUpY,
            "RHSetUpZ" => I::RHSetUpZ,
            "Radius" => I::Radius,
            "RadiusToBeWithin" => I::RadiusToBeWithin,
            "RadiusToTakeItemsBackTo" => I::RadiusToTakeItemsBackTo,
            "ReceivedWeddingRing" => I::ReceivedWeddingRing,
            "ReceptorUID" => I::ReceptorUID,
            "RegionFollowingOverriddenFromScript" => I::RegionFollowingOverriddenFromScript,
            "Rented" => I::Rented,
            "ReplacementObject" => I::ReplacementObject,
            "RespondingToFollowAndWait" => I::RespondingToFollowAndWait,
            "ReversedOnMiniMap" => I::ReversedOnMiniMap,
            "SavedInGame" => I::SavedInGame,
            "ScriptData" => I::ScriptData,
            "ScriptName" => I::ScriptName,
            "ScriptNameOfAllGeneratedCreatures" => I::ScriptNameOfAllGeneratedCreatures,
            "Seed" => I::Seed,
            "SelfTerminate" => I::SelfTerminate,
            "SelfTrigger" => I::SelfTrigger,
            "SelfTriggerRadius" => I::SelfTriggerRadius,
            "SelfTriggerResetInterval" => I::SelfTriggerResetInterval,
            "Shape" => I::Shape,
            "SoundName" => I::SoundName,
            "Start" => I::Start,
            "StartCTCAIScratchpad" => I::StartCTCAIScratchpad,
            "StartCTCActionUseBed" => I::StartCTCActionUseBed,
            "StartCTCActionUseReadable" => I::StartCTCActionUseReadable,
            "StartCTCActionUseScriptedHook" => I::StartCTCActionUseScriptedHook,
            "StartCTCActivationReceptorCreatureGenerator" => {
                I::StartCTCActivationReceptorCreatureGenerator
            }
            "StartCTCActivationReceptorDoor" => I::StartCTCActivationReceptorDoor,
            "StartCTCActivationTrigger" => I::StartCTCActivationTrigger,
            "StartCTCAtmosPlayer" => I::StartCTCAtmosPlayer,
            "StartCTCBoastingArea" => I::StartCTCBoastingArea,
            "StartCTCBuyableHouse" => I::StartCTCBuyableHouse,
            "StartCTCCameraPointFixedPoint" => I::StartCTCCameraPointFixedPoint,
            "StartCTCCameraPointGeneralCase" => I::StartCTCCameraPointGeneralCase,
            "StartCTCCameraPointScripted" => I::StartCTCCameraPointScripted,
            "StartCTCCameraPointScriptedSpline" => I::StartCTCCameraPointScriptedSpline,
            "StartCTCCameraPointTrack" => I::StartCTCCameraPointTrack,
            "StartCTCCarriedActionUseRead" => I::StartCTCCarriedActionUseRead,
            "StartCTCChest" => I::StartCTCChest,
            "StartCTCContainerRewardHero" => I::StartCTCContainerRewardHero,
            "StartCTCCreatureGenerator" => I::StartCTCCreatureGenerator,
            "StartCTCCreatureGeneratorCreator" => I::StartCTCCreatureGeneratorCreator,
            "StartCTCCreatureOpinionOfHero" => I::StartCTCCreatureOpinionOfHero,
            "StartCTCDCameraPoint" => I::StartCTCDCameraPoint,
            "StartCTCDNavigationSeed" => I::StartCTCDNavigationSeed,
            "StartCTCDParticleEmitter" => I::StartCTCDParticleEmitter,
            "StartCTCDRegionEntrance" => I::StartCTCDRegionEntrance,
            "StartCTCDRegionExit" => I::StartCTCDRegionExit,
            "StartCTCDiggingSpot" => I::StartCTCDiggingSpot,
            "StartCTCDoor" => I::StartCTCDoor,
            "StartCTCEditor" => I::StartCTCEditor,
            "StartCTCEnemy" => I::StartCTCEnemy,
            "StartCTCExplodingObject" => I::StartCTCExplodingObject,
            "StartCTCFishingSpot" => I::StartCTCFishingSpot,
            "StartCTCGuard" => I::StartCTCGuard,
            "StartCTCHero" => I::StartCTCHero,
            "StartCTCHeroCentreDoorMarker" => I::StartCTCHeroCentreDoorMarker,
            "StartCTCInfoDisplay" => I::StartCTCInfoDisplay,
            "StartCTCInventoryItem" => I::StartCTCInventoryItem,
            "StartCTCLight" => I::StartCTCLight,
            "StartCTCObjectAugmentations" => I::StartCTCObjectAugmentations,
            "StartCTCOwnedEntity" => I::StartCTCOwnedEntity,
            "StartCTCPhysicsLight" => I::StartCTCPhysicsLight,
            "StartCTCPhysicsNavigator" => I::StartCTCPhysicsNavigator,
            "StartCTCPhysicsStandard" => I::StartCTCPhysicsStandard,
            "StartCTCPreCalculatedNavigationRoute" => I::StartCTCPreCalculatedNavigationRoute,
            "StartCTCRandomAppearanceMorph" => I::StartCTCRandomAppearanceMorph,
            "StartCTCSearchableContainer" => I::StartCTCSearchableContainer,
            "StartCTCShapeManager" => I::StartCTCShapeManager,
            "StartCTCShop" => I::StartCTCShop,
            "StartCTCSpotLight" => I::StartCTCSpotLight,
            "StartCTCStealableItemLocation" => I::StartCTCStealableItemLocation,
            "StartCTCStockItem" => I::StartCTCStockItem,
            "StartCTCTalk" => I::StartCTCTalk,
            "StartCTCTargeted" => I::StartCTCTargeted,
            "StartCTCTeleporter" => I::StartCTCTeleporter,
            "StartCTCTrophy" => I::StartCTCTrophy,
            "StartCTCVillage" => I::StartCTCVillage,
            "StartCTCVillageMember" => I::StartCTCVillageMember,
            "StartCTCWallMount" => I::StartCTCWallMount,
            "StartCTCWife" => I::StartCTCWife,
            "StartPos" => I::StartPos,
            "Stealable" => I::Stealable,
            "StringLength" => I::StringLength,
            "SwitchableNavigationTCAdded" => I::SwitchableNavigationTCAdded,
            "Targetable" => I::Targetable,
            "TeleportToRegionEntrance" => I::TeleportToRegionEntrance,
            "Tension" => I::Tension,
            "TestAngleBeforeActivation" => I::TestAngleBeforeActivation,
            "TextTag" => I::TextTag,
            "TextTagBack" => I::TextTagBack,
            "ThingGamePersistent" => I::ThingGamePersistent,
            "ThingLevelPersistent" => I::ThingLevelPersistent,
            "ThingToCalculateRouteToUID" => I::ThingToCalculateRouteToUID,
            "ThingUID" => I::ThingUID,
            "TimeToChangeEnvironmentDef" => I::TimeToChangeEnvironmentDef,
            "TimeToPlay" => I::TimeToPlay,
            "ToleranceToBeingHitOverride" => I::ToleranceToBeingHitOverride,
            "TotalGenerationLimit" => I::TotalGenerationLimit,
            "TrackThing" => I::TrackThing,
            "TransitionTime" => I::TransitionTime,
            "TriggerOnActivate" => I::TriggerOnActivate,
            "TriggerRadius" => I::TriggerRadius,
            "TriggeredByThing" => I::TriggeredByThing,
            "TriggeredOnCreatureProximity" => I::TriggeredOnCreatureProximity,
            "TrophyID" => I::TrophyID,
            "UID" => I::UID,
            "Usable" => I::Usable,
            "UseableByHero" => I::UseableByHero,
            "UsingRelativeCoords" => I::UsingRelativeCoords,
            "UsingRelativeOrientation" => I::UsingRelativeOrientation,
            "ValidAnims" => I::ValidAnims,
            "Version" => I::Version,
            "VersionNumber" => I::VersionNumber,
            "VillageUID" => I::VillageUID,
            "VirtualMoneyBags" => I::VirtualMoneyBags,
            "WanderWithInformation" => I::WanderWithInformation,
            "WaveWithInformation" => I::WaveWithInformation,
            "Width" => I::Width,
            "WifeLivingHere" => I::WifeLivingHere,
            "WorkBuildingUID" => I::WorkBuildingUID,
            "XXXSectionEnd" => I::XXXSectionEnd,
            "XXXSectionStart" => I::XXXSectionStart,
            "hero_title_object_def_name" => I::hero_title_object_def_name,
            _ => {
                return Err(RawTngParseError {
                    location: Some(ident_token.location),
                    kind: E::UnrecognizedIdentifier,
                })
            }
        })
    }
}
