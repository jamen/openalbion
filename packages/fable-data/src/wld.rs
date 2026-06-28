//! Parser for Fable's `.wld` world-definition text files.
//!
//! A `.wld` lists `NewMap ... EndMap;` blocks, each placing a level file (`.lev`+`.tng`) at a
//! world-grid position.

use crate::kv::{KvError, KvParser, KvStatement, KvValue};

#[derive(Debug, Clone, PartialEq)]
pub struct Wld {
    pub header_stmts: Vec<(String, KvValueOwned)>,
    pub maps: Vec<WldMap>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WldMap {
    pub map_number: u32,
    pub map_x: i32,
    pub map_y: i32,
    pub level_name: String,
    pub level_script_name: String,
    pub map_uid: u64,
    pub is_sea: bool,
    pub loaded_on_proximity: bool,
}

impl Wld {
    pub fn parse(input: &str) -> Result<Wld, KvError> {
        let mut parser = KvParser::new(input);
        let stmts = parser.parse_statements()?;

        let mut header_stmts = Vec::new();
        let mut maps = Vec::new();

        for stmt in stmts {
            match stmt {
                KvStatement::Block { keyword: "NewMap", kind, body } => {
                    maps.push(parse_map(kind, &body));
                }
                KvStatement::Block { keyword: "NewRegion", kind, body } => {
                    // Regions parsed later; skip for now.
                    let _ = (kind, body);
                }
                KvStatement::Field(name, value) => {
                    header_stmts.push((name.to_string(), value.into_owned()));
                }
                _ => {}
            }
        }

        Ok(Wld { header_stmts, maps })
    }
}

fn parse_map(map_number_str: &str, body: &[KvStatement<'_>]) -> WldMap {
    let mut map = WldMap {
        map_number: map_number_str.parse().unwrap_or(0),
        map_x: 0,
        map_y: 0,
        level_name: String::new(),
        level_script_name: String::new(),
        map_uid: 0,
        is_sea: false,
        loaded_on_proximity: false,
    };

    for stmt in body {
        if let KvStatement::Field(name, value) = stmt {
            match *name {
                "MapX" => {
                    if let KvValue::Number(n) = value {
                        map.map_x = n.parse().unwrap_or(0);
                    }
                }
                "MapY" => {
                    if let KvValue::Number(n) = value {
                        map.map_y = n.parse().unwrap_or(0);
                    }
                }
                "LevelName" => {
                    if let KvValue::String(s) = value {
                        map.level_name = s.to_string();
                    }
                }
                "LevelScriptName" => {
                    if let KvValue::String(s) | KvValue::Ident(s) = value {
                        map.level_script_name = s.to_string();
                    }
                }
                "MapUID" => {
                    if let KvValue::Number(n) = value {
                        map.map_uid = n.parse().unwrap_or(0);
                    }
                }
                "IsSea" => {
                    if let KvValue::Bool(b) = value {
                        map.is_sea = *b;
                    }
                }
                "LoadedOnPlayerProximity" => {
                    if let KvValue::Bool(b) = value {
                        map.loaded_on_proximity = *b;
                    }
                }
                _ => {}
            }
        }
    }

    map
}

/// Owned variant of [`KvValue`] for storing in parsed structures.
#[derive(Debug, Clone, PartialEq)]
pub enum KvValueOwned {
    String(String),
    Number(String),
    Ident(String),
    Bool(bool),
}

impl<'a> KvValue<'a> {
    fn into_owned(self) -> KvValueOwned {
        match self {
            KvValue::String(s) => KvValueOwned::String(s.to_string()),
            KvValue::Number(s) => KvValueOwned::Number(s.to_string()),
            KvValue::Ident(s) => KvValueOwned::Ident(s.to_string()),
            KvValue::Bool(b) => KvValueOwned::Bool(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_wld() {
        let input = r#"MapUIDCount 72;
ThingManagerUIDCount 1;
NewMap 1;
MapX 3232;
MapY 3488;
LevelName "FinalAlbion\LookoutPoint.lev";
LevelScriptName "LookoutPoint";
MapUID 162441;
IsSea FALSE;
LoadedOnPlayerProximity TRUE;
EndMap;
NewMap 2;
MapX 3104;
MapY 3520;
LevelName "FinalAlbion\PicnicArea.lev";
LevelScriptName "PicnicArea";
MapUID 163625;
IsSea FALSE;
LoadedOnPlayerProximity TRUE;
EndMap;
"#;
        let wld = Wld::parse(input).unwrap();
        assert_eq!(wld.maps.len(), 2);
        assert_eq!(wld.maps[0].map_number, 1);
        assert_eq!(wld.maps[0].map_x, 3232);
        assert_eq!(wld.maps[0].level_name, "FinalAlbion\\LookoutPoint.lev");
        assert_eq!(wld.maps[0].is_sea, false);
        assert_eq!(wld.maps[0].loaded_on_proximity, true);

        assert_eq!(wld.maps[1].map_number, 2);
        assert_eq!(wld.maps[1].level_script_name, "PicnicArea");
    }
}
