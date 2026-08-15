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

    /// Every map in `level_name`'s region, positioned in world cells — what a level needs to
    /// load alongside itself rather than alone.
    ///
    /// Unions every region whose `ContainsMap` list names `level_name` (there is usually
    /// exactly one): the union of their `ContainsMap` entries load and populate
    /// (`populated: true`), the union of their `SeesMap` entries — minus anything already
    /// `ContainsMap` — are terrain-only fillers (`populated: false`, AGENTS.md §3.4). A named
    /// map with no matching `NewMap` entry is silently dropped rather than guessed at —
    /// logging that belongs to the caller (AGENTS.md §6.8), not this parser.
    ///
    /// `level_name` names no region — an isolated or debug-only level, or a `.wld` that
    /// doesn't cover it — falls back to `level_name` alone, at its own origin if the `.wld`
    /// places it or `(0, 0)` otherwise. That fallback is what keeps a single-level load
    /// identical to today's behaviour when there is nothing to join it to.
    pub fn maps_for_region_of(&self, level_name: &str) -> Vec<RegionMap> {
        let wanted = format!("{level_name}.lev").to_lowercase();
        let names_level = |path: &str| {
            path.to_lowercase()
                .rsplit(['\\', '/'])
                .next()
                .is_some_and(|file| file == wanted)
        };

        let mut contains: Vec<&str> = Vec::new();
        let mut sees: Vec<&str> = Vec::new();
        for region in &self.regions {
            if region.contains_maps.iter().any(|m| names_level(m)) {
                contains.extend(region.contains_maps.iter().map(String::as_str));
                sees.extend(region.sees_maps.iter().map(String::as_str));
            }
        }

        if contains.is_empty() {
            let origin = self
                .map_for_level(level_name)
                .map(|m| (m.map_x, m.map_y))
                .unwrap_or((0, 0));
            return vec![RegionMap {
                level_name: level_name.to_string(),
                origin,
                populated: true,
            }];
        }

        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut out = Vec::new();
        for (path, populated) in contains
            .into_iter()
            .map(|p| (p, true))
            .chain(sees.into_iter().map(|p| (p, false)))
        {
            let key = path.to_lowercase();
            if !seen.insert(key) {
                continue;
            }
            let Some(map) = self.maps.iter().find(|m| names_level_path(m, path)) else {
                continue;
            };
            let name = trailing_stem(path);
            out.push(RegionMap {
                level_name: name.to_string(),
                origin: (map.map_x, map.map_y),
                populated,
            });
        }
        out
    }

    /// **Every** map the `.wld` places, as one scene — the whole of Albion rather than one
    /// region.
    ///
    /// Nothing in the original engine does this: it streams a region at a time, which is what
    /// [`Self::maps_for_region_of`] models. This exists to answer how much of the world fits
    /// at once — the resident set a texture-streaming design has to budget for — and as a
    /// standing stress case for the renderer.
    ///
    /// A map is `populated` if **any** region lists it in `ContainsMap`; the rest are the
    /// terrain-only fillers every region sees. Ordered with the populated maps first, so a
    /// caller that gives up part way through still has the levels with things in them.
    pub fn all_maps(&self) -> Vec<RegionMap> {
        let mut contains: std::collections::HashSet<String> = std::collections::HashSet::new();
        for region in &self.regions {
            for path in &region.contains_maps {
                contains.insert(trailing_stem(path).to_lowercase());
            }
        }

        let mut out: Vec<RegionMap> = self
            .maps
            .iter()
            .map(|map| {
                let name = trailing_stem(&map.level_name);
                RegionMap {
                    level_name: name.to_string(),
                    origin: (map.map_x, map.map_y),
                    populated: contains.contains(&name.to_lowercase()),
                }
            })
            .collect();
        out.sort_by_key(|m| !m.populated);
        out
    }
}

/// One map to load alongside a level, resolved from its region.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionMap {
    /// The level's bare name, e.g. `"BowerstoneBridge"` — what [`Files::load_level`] takes.
    pub level_name: String,
    /// The map's origin in world cells (`MapX`/`MapY`).
    pub origin: (i32, i32),
    /// `true` for a `ContainsMap` entry (loaded, walkable, populated from its `.tng` and
    /// local detail); `false` for a `SeesMap` filler (terrain only, AGENTS.md §3.4).
    pub populated: bool,
}

fn names_level_path(map: &WldMap, path: &str) -> bool {
    map.level_name.to_lowercase() == path.to_lowercase()
}

/// The bare level name from a `.wld` path like `FinalAlbion\BowerstoneBridge.lev`.
fn trailing_stem(path: &str) -> &str {
    let file = path.rsplit(['\\', '/']).next().unwrap_or(path);
    file.strip_suffix(".lev").unwrap_or(file)
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
        NewMap 3;\n\
        MapX 3232;\n\
        MapY 3616;\n\
        LevelName \"FinalAlbion\\BowerstoneBridge.lev\";\n\
        LevelScriptName \"BowerstoneBridge\";\n\
        MapUID 784595;\n\
        IsSea FALSE;\n\
        LoadedOnPlayerProximity TRUE;\n\
        EndMap;\n\
        NewMap 4;\n\
        MapX 3200;\n\
        MapY 3456;\n\
        LevelName \"FinalAlbion\\LookoutPoint_Filler_01.lev\";\n\
        LevelScriptName \"LookoutPoint_Filler_01\";\n\
        MapUID 111111;\n\
        IsSea FALSE;\n\
        LoadedOnPlayerProximity FALSE;\n\
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
        assert_eq!(wld.maps.len(), 4);
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

    #[test]
    fn resolves_a_level_s_region() {
        let wld = Wld::parse(WLD).unwrap();
        let mut maps = wld.maps_for_region_of("LookoutPoint");
        maps.sort_by(|a, b| a.level_name.cmp(&b.level_name));

        assert_eq!(
            maps,
            vec![
                RegionMap {
                    level_name: "BowerstoneBridge".to_string(),
                    origin: (3232, 3616),
                    populated: true,
                },
                RegionMap {
                    level_name: "LookoutPoint".to_string(),
                    origin: (3232, 3488),
                    populated: true,
                },
                RegionMap {
                    level_name: "LookoutPoint_Filler_01".to_string(),
                    origin: (3200, 3456),
                    populated: false,
                },
            ]
        );
    }

    /// Asking from either member of the same region gives the same set — the union is over
    /// every region that names the level, not just the first one found.
    #[test]
    fn region_lookup_is_symmetric_within_a_region() {
        let wld = Wld::parse(WLD).unwrap();
        let mut from_lookout = wld.maps_for_region_of("LookoutPoint");
        let mut from_bridge = wld.maps_for_region_of("BowerstoneBridge");
        from_lookout.sort_by(|a, b| a.level_name.cmp(&b.level_name));
        from_bridge.sort_by(|a, b| a.level_name.cmp(&b.level_name));
        assert_eq!(from_lookout, from_bridge);
    }

    /// A level no region names — the fixture's `PicnicArea` isn't in any `ContainsMap`/
    /// `SeesMap` list — falls back to itself alone, at its own `.wld` origin.
    #[test]
    fn falls_back_to_the_level_alone_when_no_region_names_it() {
        let wld = Wld::parse(WLD).unwrap();
        let maps = wld.maps_for_region_of("PicnicArea");
        assert_eq!(
            maps,
            vec![RegionMap {
                level_name: "PicnicArea".to_string(),
                origin: (3104, 3520),
                populated: true,
            }]
        );
    }

    /// And a level neither the maps nor the regions have ever heard of still returns
    /// something loadable, at the world origin, rather than an empty list.
    #[test]
    fn falls_back_to_the_world_origin_when_the_wld_has_no_entry_at_all() {
        let wld = Wld::parse(WLD).unwrap();
        let maps = wld.maps_for_region_of("Nowhere");
        assert_eq!(
            maps,
            vec![RegionMap {
                level_name: "Nowhere".to_string(),
                origin: (0, 0),
                populated: true,
            }]
        );
    }
}
