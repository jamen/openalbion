use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `OPINION_SOURCE` — C++ `COpinionSourceDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct OpinionSourceDef {
        "ProducedOpinion" => pub produced_opinion: BTreeMap<Opinion, f32>,
        "AlwaysAwareOf" => pub always_aware_of: bool,
        "UnknownThreatSource" => pub unknown_threat_source: bool,
        "ReactNonstop" => pub react_nonstop: bool,
        "ReactToSelfPermitted" => pub react_to_self_permitted: bool,
        "ReactionFlagDefault" => pub reaction_flag_default: bool,
        "ReactionFlag" => pub reaction_flag: BTreeMap<OpinionDeedType, bool>,
        "UseNavLineOfSight" => pub use_nav_line_of_sight: bool = true,
        "WitnessesWillNotAttack" => pub witnesses_will_not_attack: bool,
        "WitnessesWillOverrideFactionEnemy" => pub witnesses_will_override_faction_enemy: bool,
        "BypassesPersonality" => pub bypasses_personality: bool,
        "RelativeEffect" => pub relative_effect: bool,
        "RestrictToFactionAndAllies" => pub restrict_to_faction_and_allies: i32,
        "RestrictToFactionIsReversed" => pub restrict_to_faction_is_reversed: bool,
        "BinaryReaction" => pub binary_reaction: bool,
        "BinaryReaction" => pub binary_reaction2: bool,
        "BinaryReaction" => pub binary_reaction3: bool,
        "BinaryReaction" => pub binary_reaction4: bool,
        "BinaryReaction" => pub binary_reaction5: bool,
        "BinaryReaction" => pub binary_reaction6: bool,
        "BinaryReaction" => pub binary_reaction7: bool,
        "BinaryReaction" => pub binary_reaction8: bool,
        "BinaryReaction" => pub binary_reaction9: bool,
        "BinaryReaction" => pub binary_reaction10: bool,
        "BinaryReaction" => pub binary_reaction11: bool,
        "BinaryReaction" => pub binary_reaction12: bool,
        "BinaryReaction" => pub binary_reaction13: bool,
        "BinaryReaction" => pub binary_reaction14: bool,
        "BinaryReaction" => pub binary_reaction15: bool,
        "BinaryReaction" => pub binary_reaction16: bool,
        "BinaryReaction" => pub binary_reaction17: bool,
        "BinaryReaction" => pub binary_reaction18: bool,
        "BinaryReaction" => pub binary_reaction19: bool,
        "BinaryReaction" => pub binary_reaction20: bool,
        "BinaryReaction" => pub binary_reaction21: bool,
        "BinaryReaction" => pub binary_reaction22: bool,
        "BinaryReaction" => pub binary_reaction23: bool,
        "BinaryReaction" => pub binary_reaction24: bool,
        "BinaryReaction" => pub binary_reaction25: bool,
        "BinaryReaction" => pub binary_reaction26: bool,
        "BinaryReaction" => pub binary_reaction27: bool,
        "BinaryReaction" => pub binary_reaction28: bool,
        "BinaryReaction" => pub binary_reaction29: bool,
        "BinaryReaction" => pub binary_reaction30: bool,
        "BinaryReaction" => pub binary_reaction31: bool,
        "BinaryReaction" => pub binary_reaction32: bool,
        "BinaryReaction" => pub binary_reaction33: bool,
        "BinaryReaction" => pub binary_reaction34: bool,
        "BinaryReaction" => pub binary_reaction35: bool,
        "BinaryReaction" => pub binary_reaction36: bool,
        "BinaryReaction" => pub binary_reaction37: bool,
        "BinaryReaction" => pub binary_reaction38: bool,
        "BinaryReaction" => pub binary_reaction39: bool,
        "BinaryReaction" => pub binary_reaction40: bool,
        "BinaryReaction" => pub binary_reaction41: bool,
        "BinaryReaction" => pub binary_reaction42: bool,
        "BinaryReaction" => pub binary_reaction43: bool,
        "BinaryReaction" => pub binary_reaction44: bool,
        "BinaryReaction" => pub binary_reaction45: bool,
        "BinaryReaction" => pub binary_reaction46: bool,
        "BinaryReaction" => pub binary_reaction47: bool,
        "BinaryReaction" => pub binary_reaction48: bool,
        "BinaryReaction" => pub binary_reaction49: bool,
        "BinaryReaction" => pub binary_reaction50: bool,
        "BinaryReaction" => pub binary_reaction51: bool,
        "BinaryReaction" => pub binary_reaction52: bool,
        "BinaryReaction" => pub binary_reaction53: bool,
        "BinaryReaction" => pub binary_reaction54: bool,
        "BinaryReaction" => pub binary_reaction55: bool,
        "BinaryReaction" => pub binary_reaction56: bool,
        "BinaryReaction" => pub binary_reaction57: bool,
        "BinaryReaction" => pub binary_reaction58: bool,
        "BinaryReaction" => pub binary_reaction59: bool,
        "BinaryReaction" => pub binary_reaction60: bool,
        "BinaryReaction" => pub binary_reaction61: bool,
        "BinaryReaction" => pub binary_reaction62: bool,
        "BinaryReaction" => pub binary_reaction63: bool,
        "BinaryReaction" => pub binary_reaction64: bool,
        "BinaryReaction" => pub binary_reaction65: bool,
        "BinaryReaction" => pub binary_reaction66: bool,
        "BinaryReaction" => pub binary_reaction67: bool,
        "BinaryReaction" => pub binary_reaction68: bool,
        "BinaryReaction" => pub binary_reaction69: bool,
        "BinaryReaction" => pub binary_reaction70: bool,
        "BinaryReaction" => pub binary_reaction71: bool,
        "BinaryReaction" => pub binary_reaction72: bool,
        "BinaryReaction" => pub binary_reaction73: bool,
        "BinaryReaction" => pub binary_reaction74: bool,
        "BinaryReaction" => pub binary_reaction75: bool,
        "BinaryReaction" => pub binary_reaction76: bool,
        "BinaryReaction" => pub binary_reaction77: bool,
        "BinaryReaction" => pub binary_reaction78: bool,
        "BinaryReaction" => pub binary_reaction79: bool,
        "BinaryOpinion" => pub binary_opinion: f32,
        "BinaryOpinion" => pub binary_opinion2: f32,
        "BinaryOpinion" => pub binary_opinion3: f32,
        "BinaryOpinion" => pub binary_opinion4: f32,
        "BinaryOpinion" => pub binary_opinion5: f32,
    }
}

impl OpinionSourceDef {
    /// The 79 `BinaryReaction` bools aren't def-file controls: the game
    /// derives them from [`Self::reaction_flag_default`] and the
    /// [`Self::reaction_flag`] map (verified against all 51 retail
    /// OPINION_SOURCE entries): index `i` is `map.get(i, default)` for
    /// `i >= 18`, always `false` below.
    pub fn derive_binary_reactions(&mut self) {
        let flags = [
            &mut self.binary_reaction, &mut self.binary_reaction2, &mut self.binary_reaction3,
            &mut self.binary_reaction4, &mut self.binary_reaction5, &mut self.binary_reaction6,
            &mut self.binary_reaction7, &mut self.binary_reaction8, &mut self.binary_reaction9,
            &mut self.binary_reaction10, &mut self.binary_reaction11, &mut self.binary_reaction12,
            &mut self.binary_reaction13, &mut self.binary_reaction14, &mut self.binary_reaction15,
            &mut self.binary_reaction16, &mut self.binary_reaction17, &mut self.binary_reaction18,
            &mut self.binary_reaction19, &mut self.binary_reaction20, &mut self.binary_reaction21,
            &mut self.binary_reaction22, &mut self.binary_reaction23, &mut self.binary_reaction24,
            &mut self.binary_reaction25, &mut self.binary_reaction26, &mut self.binary_reaction27,
            &mut self.binary_reaction28, &mut self.binary_reaction29, &mut self.binary_reaction30,
            &mut self.binary_reaction31, &mut self.binary_reaction32, &mut self.binary_reaction33,
            &mut self.binary_reaction34, &mut self.binary_reaction35, &mut self.binary_reaction36,
            &mut self.binary_reaction37, &mut self.binary_reaction38, &mut self.binary_reaction39,
            &mut self.binary_reaction40, &mut self.binary_reaction41, &mut self.binary_reaction42,
            &mut self.binary_reaction43, &mut self.binary_reaction44, &mut self.binary_reaction45,
            &mut self.binary_reaction46, &mut self.binary_reaction47, &mut self.binary_reaction48,
            &mut self.binary_reaction49, &mut self.binary_reaction50, &mut self.binary_reaction51,
            &mut self.binary_reaction52, &mut self.binary_reaction53, &mut self.binary_reaction54,
            &mut self.binary_reaction55, &mut self.binary_reaction56, &mut self.binary_reaction57,
            &mut self.binary_reaction58, &mut self.binary_reaction59, &mut self.binary_reaction60,
            &mut self.binary_reaction61, &mut self.binary_reaction62, &mut self.binary_reaction63,
            &mut self.binary_reaction64, &mut self.binary_reaction65, &mut self.binary_reaction66,
            &mut self.binary_reaction67, &mut self.binary_reaction68, &mut self.binary_reaction69,
            &mut self.binary_reaction70, &mut self.binary_reaction71, &mut self.binary_reaction72,
            &mut self.binary_reaction73, &mut self.binary_reaction74, &mut self.binary_reaction75,
            &mut self.binary_reaction76, &mut self.binary_reaction77, &mut self.binary_reaction78,
            &mut self.binary_reaction79,
        ];
        for (i, flag) in flags.into_iter().enumerate() {
            *flag = if i >= 18 {
                self.reaction_flag
                    .get(&OpinionDeedType(i as i32))
                    .copied()
                    .unwrap_or(self.reaction_flag_default)
            } else {
                false
            };
        }
    }

    /// The 5 `BinaryOpinion` floats aren't def-file controls: the game derives
    /// them from the [`Self::produced_opinion`] map (verified against retail
    /// OPINION_SOURCE entries): index `i` is the float for [`Opinion`]`(i)`,
    /// zero when that opinion is absent from the map.
    pub fn derive_binary_opinions(&mut self) {
        let opinions = [
            &mut self.binary_opinion, &mut self.binary_opinion2, &mut self.binary_opinion3,
            &mut self.binary_opinion4, &mut self.binary_opinion5,
        ];
        for (i, op) in opinions.into_iter().enumerate() {
            *op = self
                .produced_opinion
                .get(&Opinion(i as i32))
                .copied()
                .unwrap_or(0.0);
        }
    }
}
