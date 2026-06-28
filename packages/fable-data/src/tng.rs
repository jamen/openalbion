//! Parser for Fable's `.tng` "Things" text-format level-script files.
//!
//! A `.tng` lists `NewThing ... EndThing;` blocks (grouped into `XXXSectionStart/End` sections
//! in version-2 files), each carrying the thing's `DefinitionType`, position, orientation, and
//! components.

use crate::kv::{KvError, KvParser, KvStatement, KvValue};

#[derive(Debug, Clone, PartialEq)]
pub struct Tng {
    pub sections: Vec<TngSection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TngSection {
    pub name: String,
    pub things: Vec<TngThing>,
}

impl Tng {
    pub fn parse(input: &str) -> Result<Tng, KvError> {
        let mut parser = KvParser::new(input);
        let stmts = parser.parse_statements()?;

        let mut sections = Vec::new();
        let mut current_section: Option<TngSection> = None;

        for stmt in stmts {
            match stmt {
                KvStatement::Block { keyword: "XXXSectionStart", kind, body } => {
                    if let Some(sec) = current_section.take() {
                        sections.push(sec);
                    }
                    let mut things = Vec::new();
                    for body_stmt in body {
                        if let KvStatement::Block { keyword: "NewThing", kind, body } = body_stmt {
                            things.push(parse_thing(kind, &body));
                        }
                    }
                    current_section = Some(TngSection {
                        name: kind.to_string(),
                        things,
                    });
                }
                KvStatement::Block { keyword: "NewThing", kind, body } => {
                    // Top-level things (v1 format or no sections).
                    let thing = parse_thing(kind, &body);
                    if current_section.is_none() {
                        current_section = Some(TngSection {
                            name: String::new(),
                            things: Vec::new(),
                        });
                    }
                    current_section.as_mut().unwrap().things.push(thing);
                }
                KvStatement::Field("Version", KvValue::Number(_)) => {}
                _ => {}
            }
        }

        if let Some(sec) = current_section.take() {
            sections.push(sec);
        }

        Ok(Tng { sections })
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TngThing {
    pub thing_type: String,
    pub player: i32,
    pub uid: u64,
    pub definition_type: String,
    pub position: [f32; 3],
    pub forward: [f32; 3],
    pub up: [f32; 3],
    pub script_name: Option<String>,
    pub script_data: Option<String>,
    pub health: Option<f32>,
    pub object_scale: Option<f32>,
    pub game_persistent: bool,
    pub level_persistent: bool,
}

fn parse_thing(thing_type: &str, body: &[KvStatement<'_>]) -> TngThing {
    let mut thing = TngThing {
        thing_type: thing_type.to_string(),
        ..Default::default()
    };

    let mut in_physics = false;
    let mut in_editor = false;

    for stmt in body {
        match stmt {
            KvStatement::Field("Player", KvValue::Number(n)) => {
                thing.player = n.parse().unwrap_or(-1);
            }
            KvStatement::Field("UID", KvValue::Number(n)) => {
                thing.uid = n.parse().unwrap_or(0);
            }
            KvStatement::Field("DefinitionType", KvValue::String(s)) => {
                thing.definition_type = s.to_string();
            }
            KvStatement::Field("ScriptName", KvValue::Ident(s) | KvValue::String(s)) => {
                thing.script_name = Some(s.to_string());
            }
            KvStatement::Field("ScriptData", KvValue::String(s)) => {
                thing.script_data = Some(s.to_string());
            }
            KvStatement::Field("Health", KvValue::Number(n)) => {
                thing.health = n.parse().ok();
            }
            KvStatement::Field("ObjectScale", KvValue::Number(n)) => {
                thing.object_scale = n.parse().ok();
            }
            KvStatement::Field("ThingGamePersistent", KvValue::Bool(b)) => {
                thing.game_persistent = *b;
            }
            KvStatement::Field("ThingLevelPersistent", KvValue::Bool(b)) => {
                thing.level_persistent = *b;
            }
            KvStatement::Field("StartCTCPhysicsStandard", _) => {
                in_physics = true;
            }
            KvStatement::Field("EndCTCPhysicsStandard", _) => {
                in_physics = false;
            }
            KvStatement::Field("StartCTCEditor", _) => {
                in_editor = true;
            }
            KvStatement::Field("EndCTCEditor", _) => {
                in_editor = false;
            }
            KvStatement::Field(name, value) if in_physics => {
                match *name {
                    "PositionX" => {
                        thing.position[0] = value_as_f32(value);
                    }
                    "PositionY" => {
                        thing.position[1] = value_as_f32(value);
                    }
                    "PositionZ" => {
                        thing.position[2] = value_as_f32(value);
                    }
                    "RHSetForwardX" => {
                        thing.forward[0] = value_as_f32(value);
                    }
                    "RHSetForwardY" => {
                        thing.forward[1] = value_as_f32(value);
                    }
                    "RHSetForwardZ" => {
                        thing.forward[2] = value_as_f32(value);
                    }
                    "RHSetUpX" => {
                        thing.up[0] = value_as_f32(value);
                    }
                    "RHSetUpY" => {
                        thing.up[1] = value_as_f32(value);
                    }
                    "RHSetUpZ" => {
                        thing.up[2] = value_as_f32(value);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        let _ = in_editor;
    }

    thing
}

fn value_as_f32(v: &KvValue<'_>) -> f32 {
    match v {
        KvValue::Number(n) => n.parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_blank_tng() {
        let input = r#"Version 2;
XXXSectionStart NULL;
NewThing Marker;
Player 4;
UID 18446741874686301490;
DefinitionType "MARKER_BASIC";
ThingGamePersistent FALSE;
ThingLevelPersistent FALSE;
StartCTCPhysicsStandard;
PositionX 10.0;
PositionY 20.0;
PositionZ 30.0;
RHSetForwardX 1.0;
RHSetForwardY 0.0;
RHSetForwardZ 0.0;
RHSetUpX 0.0;
RHSetUpY 0.0;
RHSetUpZ 1.0;
EndCTCPhysicsStandard;
StartCTCEditor;
EndCTCEditor;
Health 1.0;
EndThing;
XXXSectionEnd;
"#;
        let tng = Tng::parse(input).unwrap();
        assert_eq!(tng.sections.len(), 1);
        assert_eq!(tng.sections[0].name, "NULL");
        assert_eq!(tng.sections[0].things.len(), 1);

        let t = &tng.sections[0].things[0];
        assert_eq!(t.definition_type, "MARKER_BASIC");
        assert_eq!(t.position, [10.0, 20.0, 30.0]);
        assert_eq!(t.forward, [1.0, 0.0, 0.0]);
        assert_eq!(t.up, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn parse_things_without_sections() {
        let input = r#"NewThing Object;
Player -1;
UID 123;
DefinitionType "OBJECT_FISHING_ROD";
EndThing;
NewThing Marker;
Player -1;
UID 456;
DefinitionType "MARKER_BASIC";
EndThing;
"#;
        let tng = Tng::parse(input).unwrap();
        assert_eq!(tng.sections.len(), 1);
        assert_eq!(tng.sections[0].things.len(), 2);
        assert_eq!(tng.sections[0].things[0].definition_type, "OBJECT_FISHING_ROD");
        assert_eq!(tng.sections[0].things[1].definition_type, "MARKER_BASIC");
    }
}
