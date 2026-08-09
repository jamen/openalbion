//! Fable's `.wld` world-definition files.
//!
//! Same grammar as `.tng` (see [`crate::text`]), different vocabulary: a `.wld`
//! places every level on the world grid with `NewMap … EndMap`, and groups them
//! into named regions with `NewRegion … EndRegion`. A region lists its levels as
//! repeated `ContainsMap` (loaded, walkable) and `SeesMap` (visible only)
//! fields — the classification behind the filler levels in AGENTS.md §3.4.

use crate::text::{self, Statement, TextError, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Wld {
    /// Top-level fields before the first block, in file order and
    /// uninterpreted (`MapUIDCount`, `ThingManagerUIDCount`).
    pub header_fields: Vec<(String, WldValue)>,
    pub maps: Vec<WldMap>,
    pub regions: Vec<WldRegion>,
}

/// One `NewMap <n> … EndMap`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WldMap {
    pub map_number: u32,
    /// World-grid position, in the 32×32-cell tiles `CEngineWorldMap` indexes.
    pub map_x: i32,
    pub map_y: i32,
    /// The `.lev` path as written, e.g. `FinalAlbion\LookoutPoint.lev` — a
    /// literal backslash, not an escape.
    pub level_name: String,
    pub level_script_name: String,
    pub map_uid: u64,
    pub is_sea: bool,
    pub loaded_on_proximity: bool,
}

/// One `NewRegion <n> … EndRegion`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WldRegion {
    pub region_number: u32,
    pub region_name: String,
    /// The `REGION_*` def this region resolves to.
    pub region_def: String,
    pub display_name: String,
    pub appears_on_world_map: bool,
    /// Level paths the region owns: loaded and walkable.
    pub contains_maps: Vec<String>,
    /// Level paths the region can see but does not load — the fillers.
    pub sees_maps: Vec<String>,
}

/// An owned [`text::Value`], for the header fields this module does not type.
#[derive(Debug, Clone, PartialEq)]
pub enum WldValue {
    Number(String),
    Bool(bool),
    String(String),
    Symbol(String),
}

impl WldValue {
    fn from_text(value: &Value<'_>) -> Option<WldValue> {
        Some(match value {
            Value::Number(n) => WldValue::Number((*n).to_string()),
            Value::Bool(b) => WldValue::Bool(*b),
            Value::String(s) => WldValue::String((*s).to_string()),
            Value::Symbol(s) => WldValue::Symbol((*s).to_string()),
            // No `.wld` field is a constructor call; keeping the variant out
            // means nothing silently reads as an empty string.
            Value::Call(_) => return None,
        })
    }
}

impl Wld {
    pub fn parse(input: &str) -> Result<Wld, TextError> {
        let body = text::parse(input)?;

        let mut wld = Wld {
            header_fields: Vec::new(),
            maps: Vec::new(),
            regions: Vec::new(),
        };

        for statement in &body.statements {
            match &statement.value {
                Statement::Field(field) => {
                    if let Some(value) = WldValue::from_text(&field.value.value) {
                        wld.header_fields.push((field.path.to_string(), value));
                    }
                }
                Statement::Block(block) if block.keyword == "NewMap" => {
                    wld.maps.push(map(block));
                }
                Statement::Block(block) if block.keyword == "NewRegion" => {
                    wld.regions.push(region(block));
                }
                _ => {}
            }
        }

        Ok(wld)
    }

    /// The map placing `level_name`, matched case-insensitively on the trailing
    /// file name so callers can pass `"LookoutPoint"`.
    pub fn map_for_level(&self, level_name: &str) -> Option<&WldMap> {
        let wanted = format!("{level_name}.lev").to_lowercase();
        self.maps.iter().find(|m| {
            m.level_name
                .to_lowercase()
                .rsplit(['\\', '/'])
                .next()
                .is_some_and(|file| file == wanted)
        })
    }
}

fn map(block: &text::Block<'_>) -> WldMap {
    let body = &block.body;
    let field = |name: &str| body.field(name).map(|v| &v.value);

    WldMap {
        map_number: block.kind.and_then(|k| k.value.parse().ok()).unwrap_or(0),
        map_x: field("MapX").and_then(Value::as_i32).unwrap_or(0),
        map_y: field("MapY").and_then(Value::as_i32).unwrap_or(0),
        level_name: field("LevelName")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        level_script_name: field("LevelScriptName")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        map_uid: field("MapUID").and_then(Value::as_u64).unwrap_or(0),
        is_sea: field("IsSea").and_then(Value::as_bool).unwrap_or(false),
        loaded_on_proximity: field("LoadedOnPlayerProximity")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }
}

fn region(block: &text::Block<'_>) -> WldRegion {
    let body = &block.body;
    let field = |name: &str| body.field(name).map(|v| &v.value);

    // `ContainsMap` / `SeesMap` repeat, so they are collected rather than
    // looked up — a region owns up to a dozen of each.
    let collect = |name: &str| {
        body.fields()
            .filter(|f| f.path.as_name() == Some(name))
            .filter_map(|f| f.value.value.as_str())
            .map(str::to_string)
            .collect()
    };

    WldRegion {
        region_number: block.kind.and_then(|k| k.value.parse().ok()).unwrap_or(0),
        region_name: field("RegionName")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        region_def: field("RegionDef")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        display_name: field("NewDisplayName")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        appears_on_world_map: body.flags().any(|f| f == "AppearOnWorldMap"),
        contains_maps: collect("ContainsMap"),
        sees_maps: collect("SeesMap"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WLD: &str = "START_INITIAL_QUESTS;\n\
        Q_SunnyvaleMaster;\n\
        END_INITIAL_QUESTS;\n\
        \n\
        MapUIDCount 72;\n\
        ThingManagerUIDCount 1;\n\
        NewMap 1;\n\
        MapX 3232;\n\
        MapY 3488;\n\
        LevelName \"FinalAlbion\\LookoutPoint.lev\";\n\
        LevelScriptName \"LookoutPoint\";\n\
        MapUID 162441;\n\
        IsSea FALSE;\n\
        LoadedOnPlayerProximity TRUE;\n\
        EndMap;\n\
        NewMap 2;\n\
        MapX 3104;\n\
        MapY 3520;\n\
        LevelName \"FinalAlbion\\PicnicArea.lev\";\n\
        LevelScriptName \"PicnicArea\";\n\
        MapUID 163625;\n\
        IsSea FALSE;\n\
        LoadedOnPlayerProximity TRUE;\n\
        EndMap;\n\
        NewRegion 1;\n\
        RegionName \"LookoutPoint\";\n\
        NewDisplayName \"TXT_REGION_LOOKOUT_POINT\";\n\
        RegionDef \"REGION_LOOKOUT_POINT\";\n\
        AppearOnWorldMap;\n\
        MiniMapGraphic MINIMAP_LOOKOUTPOINT;\n\
        MiniMapRegionExitTextOffsetX[HeroGuildComplexInside] 0.0;\n\
        MiniMapRegionExitTextOffsetY[PicnicArea] 20.0;\n\
        ContainsMap \"FinalAlbion\\BowerstoneBridge.lev\";\n\
        ContainsMap \"FinalAlbion\\LookoutPoint.lev\";\n\
        SeesMap \"FinalAlbion\\LookoutPoint_Filler_01.lev\";\n\
        EndRegion;\n";

    #[test]
    fn parses_maps() {
        let wld = Wld::parse(WLD).unwrap();
        assert_eq!(wld.maps.len(), 2);
        assert_eq!(wld.maps[0].map_number, 1);
        assert_eq!(wld.maps[0].map_x, 3232);
        assert_eq!(wld.maps[0].map_y, 3488);
        // The backslash is a literal byte, not an escape.
        assert_eq!(wld.maps[0].level_name, "FinalAlbion\\LookoutPoint.lev");
        assert_eq!(wld.maps[0].map_uid, 162441);
        assert!(!wld.maps[0].is_sea);
        assert!(wld.maps[0].loaded_on_proximity);
        assert_eq!(wld.maps[1].level_script_name, "PicnicArea");
    }

    #[test]
    fn parses_regions_with_repeated_and_indexed_fields() {
        let wld = Wld::parse(WLD).unwrap();
        let region = &wld.regions[0];
        assert_eq!(region.region_number, 1);
        assert_eq!(region.region_name, "LookoutPoint");
        assert_eq!(region.region_def, "REGION_LOOKOUT_POINT");
        assert_eq!(region.display_name, "TXT_REGION_LOOKOUT_POINT");
        assert!(region.appears_on_world_map);
        assert_eq!(region.contains_maps.len(), 2);
        assert_eq!(region.sees_maps, vec!["FinalAlbion\\LookoutPoint_Filler_01.lev"]);
    }

    #[test]
    fn header_fields_stop_at_the_first_block() {
        let wld = Wld::parse(WLD).unwrap();
        assert_eq!(
            wld.header_fields,
            vec![
                ("MapUIDCount".to_string(), WldValue::Number("72".into())),
                ("ThingManagerUIDCount".to_string(), WldValue::Number("1".into())),
            ]
        );
    }

    #[test]
    fn finds_a_map_by_level_name() {
        let wld = Wld::parse(WLD).unwrap();
        assert_eq!(wld.map_for_level("LookoutPoint").unwrap().map_number, 1);
        assert_eq!(wld.map_for_level("picnicarea").unwrap().map_number, 2);
        assert!(wld.map_for_level("Nowhere").is_none());
    }
}
