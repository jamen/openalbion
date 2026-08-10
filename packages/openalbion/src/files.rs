use derive_more::{Display, Error};
use fable_data::{
    big::{AssetMetadata, BigReader, BigReaderError, ExtraMetadata, ReadAssetDataError},
    def::binary::{DefBinary, DefBody},
    def::names::Names,
    def::EngineDef,
    def::EngineGraphic,
    def::EngineLocalDetailGeneratorDef,
    def::EngineThemeDef,
    def::SkyDef,
    environment::{EnvironmentConfig, EnvironmentTheme},
    lev::{Lev, LevError},
    mesh::{Mesh, MeshError},
    tga::{Tga, TgaError},
    tng::Tng,
    wad::{ReadContentError, WadReader, WadReaderError},
    wld::Wld,
};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
};

pub struct Files {
    pub fable_directory: PathBuf,
    pub textures: BigReader<File>,
    pub graphics: BigReader<File>,
    /// Raw `lighting_colours.tga`. Consumed by the environment layer's integer texel
    /// fetch (AGENTS.md §3.1, step 2.1); not uploaded to the GPU — the original looks
    /// these up on the CPU.
    #[allow(dead_code)]
    pub lighting_lut_bytes: Vec<u8>,
    pub environment: Option<EnvironmentConfig>,
    pub engine_themes: HashMap<String, EngineThemeDef>,
    /// Every def that can be a thing's `DefinitionType` and draws something, keyed by
    /// instance name — what `.tng` placements resolve through. See [`Defs::read`].
    pub thing_graphics: HashMap<String, EngineGraphic>,
    /// `LOCAL_DETAIL_GENERATOR` defs by global entry index, which is how an `ENGINE_THEME`
    /// names one.
    local_detail_generators: HashMap<i32, EngineLocalDetailGeneratorDef>,
    engine_def: Option<EngineDef>,
    sky_def: Option<SkyDef>,
    /// `FinalAlbion.wld` — where each level sits in the world, and which levels see which.
    world: Option<Wld>,
}

/// Everything read out of `game.bin`, in one pass over its entries.
///
/// Each of these used to open and re-parse `names.bin` and `game.bin` for itself.
#[derive(Default)]
struct Defs {
    engine_themes: HashMap<String, EngineThemeDef>,
    thing_graphics: HashMap<String, EngineGraphic>,
    local_detail_generators: HashMap<i32, EngineLocalDetailGeneratorDef>,
    engine_def: Option<EngineDef>,
    sky_def: Option<SkyDef>,
}

impl Defs {
    /// Read every def type the engine needs, in one pass.
    ///
    /// A def entry's `def_name` is its *class* (`"ENGINE_THEME"`); the instance name is
    /// `file_name`. Themes and thing graphics are keyed by instance name because that is what
    /// a `.lev` palette and a `.tng` `DefinitionType` name them with; generators are keyed by
    /// global index because that is what an `ENGINE_THEME` references them with.
    ///
    /// Four def types carry a `Graphic` and between them cover every placed thing that draws:
    /// `OBJECT` (2,849 defs), `CREATURE` (517), `BUILDING` (321) and `MARKER` (57). Reading
    /// all four uniformly is what makes buildings work without a second code path.
    fn read(compiled: Option<&(Names, DefBinary)>) -> Defs {
        let mut defs = Defs::default();
        let Some((names, def_binary)) = compiled else {
            return defs;
        };

        for entry in def_binary.entries(names) {
            match &entry.record.body {
                DefBody::EngineThemeDef(def) => {
                    if let Some(name) = entry.file_name {
                        defs.engine_themes.insert(name.to_string(), def.clone());
                    }
                }
                DefBody::EngineLocalDetailGeneratorDef(def) => {
                    defs.local_detail_generators
                        .insert(entry.global_index as i32, def.clone());
                }
                DefBody::Engine(def) => defs.engine_def = Some(def.clone()),
                DefBody::SkyDef(def) => defs.sky_def = Some(def.clone()),
                body => {
                    let graphic = match body {
                        DefBody::ThingObjectDef(def) => &def.graphic,
                        DefBody::ThingBuildingDef(def) => &def.graphic,
                        DefBody::ThingMarkerDef(def) => &def.graphic,
                        DefBody::ThingCreatureDef(def) => &def.graphic,
                        _ => continue,
                    };
                    if let Some(name) = entry.file_name {
                        defs.thing_graphics.insert(name.to_string(), graphic.clone());
                    }
                }
            }
        }

        tracing::info!(
            "game.bin: {} ENGINE_THEME, {} thing graphics, {} LOCAL_DETAIL_GENERATOR",
            defs.engine_themes.len(),
            defs.thing_graphics.len(),
            defs.local_detail_generators.len(),
        );
        defs
    }
}

#[derive(Debug, Display, Error)]
pub enum LoadLevelError {
    OpenWad(io::Error),
    ReadWad(WadReaderError),
    #[display("level {_0:?} not found in FinalAlbion.wad")]
    NotFound(#[error(not(source))] String),
    ReadContent(ReadContentError),
    Parse(LevError),
}

#[derive(Debug, Display, Error)]
pub enum NewFilesError {
    OpenTextures(io::Error),
    LoadTextures(BigReaderError),
    OpenGraphics(io::Error),
    LoadGraphics(BigReaderError),
    ReadLightingLut(io::Error),
    ParseLightingLut(TgaError),
}

#[derive(Debug, Display, Error)]
pub enum ReadMeshError {
    #[display("Mesh not found")]
    NotFound,
    #[display("Decode mesh: {_0}")]
    Decode(MeshError),
    #[display("Read asset data: {_0}")]
    ReadAssetData(ReadAssetDataError),
}

impl Files {
    pub fn new(fable_directory: &Path) -> Result<Self, NewFilesError> {
        use NewFilesError as E;

        // Load textures.big
        let textures_path = fable_directory.join("data/graphics/pc/textures.big");
        let textures_file = File::open(&textures_path).map_err(E::OpenTextures)?;
        let textures = BigReader::new(textures_file).map_err(E::LoadTextures)?;

        // Load graphics.big
        let graphics_path = fable_directory.join("data/graphics/graphics.big");
        let graphics_file = File::open(&graphics_path).map_err(E::OpenGraphics)?;
        let graphics = BigReader::new(graphics_file).map_err(E::LoadGraphics)?;

        // Load lighting colours LUT
        // Try multiple possible paths
        let lighting_lut_bytes =
            Self::try_read_paths(
                &[fable_directory.join("data/LightingTable/lighting_colours.tga")],
            )
            .map_err(E::ReadLightingLut)?;

        // Validate it's a valid TGA
        Tga::parse(&lighting_lut_bytes).map_err(E::ParseLightingLut)?;

        tracing::info!(
            "Loaded lighting_colours.tga ({} bytes)",
            lighting_lut_bytes.len()
        );

        // The compiled defs, parsed once. Every def-backed lookup below reads this one
        // `DefBinary` — the loaders used to open and re-parse `names.bin` + `game.bin` per
        // call, which AGENTS.md §4 flagged and which a fourth caller would have made worse.
        let compiled_defs = Self::load_compiled_defs(fable_directory);

        // Load sky environment themes — prefer CompiledDefs/game.bin (retail binary format),
        // fall back to the debug-only text environment.def.
        let environment = {
            let from_binary = compiled_defs.as_ref().map(|(names, def_binary)| {
                EnvironmentConfig::from_binary_defs(def_binary, names, |id| {
                    textures
                        .bank("GBANK_MAIN_PC")
                        .and_then(|b| b.asset_by_id(id as u32))
                        .map(|a| a.symbol_name.to_string())
                })
            });

            match from_binary {
                Some(environment) => {
                    tracing::info!(
                        "Loaded environment themes from game.bin ({} themes)",
                        environment.themes.len()
                    );
                    Some(environment)
                }
                None => {
                    tracing::warn!("No compiled defs, falling back to environment.def");

                    match Self::try_read_paths(&[fable_directory.join("data/Defs/environment.def")])
                    {
                        Ok(env_bytes) => {
                            let env_str = String::from_utf8_lossy(&env_bytes);
                            match EnvironmentConfig::parse(&env_str) {
                                Ok(environment) => {
                                    tracing::info!(
                                        "Loaded environment.def ({} themes)",
                                        environment.themes.len()
                                    );
                                    Some(environment)
                                }
                                Err(error) => {
                                    tracing::warn!(
                                        "Failed to parse environment.def, sky disabled: {error}"
                                    );
                                    None
                                }
                            }
                        }
                        Err(_) => {
                            tracing::warn!("environment.def not found, sky disabled");
                            None
                        }
                    }
                }
            }
        };

        let defs = Defs::read(compiled_defs.as_ref());

        Ok(Self {
            fable_directory: fable_directory.to_path_buf(),
            textures,
            graphics,
            lighting_lut_bytes,
            environment,
            engine_themes: defs.engine_themes,
            thing_graphics: defs.thing_graphics,
            local_detail_generators: defs.local_detail_generators,
            engine_def: defs.engine_def,
            sky_def: defs.sky_def,
            world: Self::load_world(fable_directory),
        })
    }

    /// `FinalAlbion.wld`, which places every level in the world.
    fn load_world(fable_directory: &Path) -> Option<Wld> {
        let path = fable_directory.join("data/Levels/FinalAlbion.wld");
        let text = std::fs::read_to_string(&path)
            .map_err(|error| tracing::warn!("{}: {error}", path.display()))
            .ok()?;
        match Wld::parse(&text) {
            Ok(world) => {
                tracing::info!("FinalAlbion.wld: {} maps", world.maps.len());
                Some(world)
            }
            Err(error) => {
                tracing::warn!("FinalAlbion.wld: {error:?}");
                None
            }
        }
    }

    /// `names.bin` + `game.bin`, or nothing if either is missing.
    fn load_compiled_defs(fable_directory: &Path) -> Option<(Names, DefBinary)> {
        let names_path = fable_directory.join("data/CompiledDefs/names.bin");
        let game_bin_path = fable_directory.join("data/CompiledDefs/game.bin");

        let names = match Names::load(&names_path) {
            Ok(names) => names,
            Err(error) => {
                tracing::warn!("names.bin not read, everything def-backed is disabled: {error:?}");
                return None;
            }
        };
        match DefBinary::load_with_names(&game_bin_path, &names) {
            Ok(def_binary) => Some((names, def_binary)),
            Err(error) => {
                tracing::warn!("game.bin not read, everything def-backed is disabled: {error:?}");
                None
            }
        }
    }

    /// An `ENGINE_THEME` def by its instance name, e.g. `"GROUND_GRASS"`.
    pub fn engine_theme_by_name(&self, name: &str) -> Option<&EngineThemeDef> {
        self.engine_themes.get(name)
    }

    /// A `LOCAL_DETAIL_GENERATOR` def by the global entry index a theme references it with.
    ///
    /// Index, not name — and that is worth stating, because the `.lev` theme palette's
    /// stored index is stale in retail data and has to be re-resolved by name (§3.4). This
    /// reference is not: `CEngineThemeDef::LocalDetailGeneratorDef` resolves directly for all
    /// 79 themes in retail `game.bin` that name a generator.
    pub fn local_detail_generator(&self, index: i32) -> Option<&EngineLocalDetailGeneratorDef> {
        self.local_detail_generators.get(&index)
    }

    /// The `ENGINE` def, which carries the engine-wide defaults local detail falls back to.
    pub fn engine_def(&self) -> Option<&EngineDef> {
        self.engine_def.as_ref()
    }

    /// A level's origin in world cells — `CEngineMap::WorldPosX`/`WorldPosY`, which
    /// `FinalAlbion.wld` calls `MapX`/`MapY`.
    ///
    /// Local detail needs it because its random draws are indexed by **world** cell, not by
    /// map cell: the same terrain at a different world position grows different foliage. In
    /// the shipped world every origin is a multiple of 32 — they are the cell coordinates of
    /// `CEngineWorldMap`'s 32×32 tile grid — so in practice this never shifts the pattern's
    /// phase, but it is threaded rather than assumed.
    ///
    /// `(0, 0)` when there is no `.wld`, or no entry for this level.
    pub fn level_origin(&self, level_name: &str) -> (i32, i32) {
        self.world
            .as_ref()
            .and_then(|wld| wld.map_for_level(level_name))
            .map(|map| (map.map_x, map.map_y))
            .unwrap_or_else(|| {
                tracing::debug!("No .wld entry for {level_name}, placing it at the world origin");
                (0, 0)
            })
    }

    /// Load and parse a level by name (e.g. "Witchwood") from `FinalAlbion.wad`.
    pub fn load_level(&self, name: &str) -> Result<Lev, LoadLevelError> {
        use LoadLevelError as E;

        let loose_path = self
            .fable_directory
            .join("data/Levels/FinalAlbion")
            .join(format!("{name}.lev"));
        if let Ok(bytes) = std::fs::read(&loose_path) {
            let lev = Lev::from_bytes(&bytes).map_err(E::Parse)?;
            tracing::info!(
                "Loaded level {} from {} ({}x{}, {} heightmap cells)",
                name,
                loose_path.display(),
                lev.header.width,
                lev.header.height,
                lev.heightmap_cells.len(),
            );
            return Ok(lev);
        }

        let wad_path = self.fable_directory.join("data/Levels/FinalAlbion.wad");
        let wad_file = BufReader::new(File::open(&wad_path).map_err(E::OpenWad)?);
        let mut wad = WadReader::new(wad_file).map_err(E::ReadWad)?;

        let suffix = format!("{name}.lev").to_lowercase();
        let asset = wad
            .asset_iter()
            .find(|a| a.path.to_lowercase().ends_with(&suffix))
            .cloned()
            .ok_or_else(|| E::NotFound(name.to_string()))?;

        let bytes = wad.read_content(&asset).map_err(E::ReadContent)?;
        let lev = Lev::from_bytes(&bytes).map_err(E::Parse)?;

        tracing::info!(
            "Loaded level {} ({}x{}, {} heightmap cells)",
            name,
            lev.header.width,
            lev.header.height,
            lev.heightmap_cells.len(),
        );

        Ok(lev)
    }

    fn try_read_paths(paths: &[std::path::PathBuf]) -> Result<Vec<u8>, io::Error> {
        for path in paths {
            match std::fs::read(path) {
                Ok(bytes) => return Ok(bytes),
                Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("None of the paths exist: {:?}", paths),
        ))
    }

    /// Get an environment theme by name, if environment data was loaded.
    pub fn environment_theme(&self, name: &str) -> Option<&EnvironmentTheme> {
        self.environment.as_ref()?.themes.get(name)
    }

    /// The `SKY` def, carrying sun/moon texture indices.
    ///
    /// Unused until sun/moon rendering is ported from `RenderSun`/`RenderMoon`
    /// (AGENTS.md step 4.2).
    #[allow(dead_code)]
    pub fn sky_def(&self) -> Option<&SkyDef> {
        self.sky_def.as_ref()
    }

    /// Read a texture asset by its numeric ID from the textures big.
    pub fn read_texture_by_id(
        &mut self,
        tex_id: u32,
    ) -> Result<(AssetMetadata, Vec<u8>), String> {
        let asset = self
            .find_texture_asset(tex_id)
            .ok_or_else(|| format!("texture id {tex_id} not found in textures.big"))?;
        let data = self
            .textures
            .read_asset_from_metadata(&asset)
            .map_err(|e| format!("read texture {tex_id}: {e}"))?;
        Ok((asset, data))
    }

    /// Read a sky texture by its symbol name (e.g., "GRAPHIC_ATMOSPHERIC_SKY_MIDNIGHT").
    pub fn read_sky_texture(
        &mut self,
        texture_name: &str,
    ) -> Result<(fable_data::big::AssetMetadata, Vec<u8>), ReadAssetDataError> {
        // Sky textures are in the GBANK_MAIN_PC bank
        self.textures.read_asset("GBANK_MAIN_PC", texture_name)
    }

    /// Find a texture-typed asset by id, preferring textures.big over graphics.big.
    fn find_texture_asset(&self, tex_id: u32) -> Option<AssetMetadata> {
        let is_texture = |a: &AssetMetadata| matches!(&a.extras, Some(ExtraMetadata::Texture(_)));
        let from = |reader: &BigReader<File>| {
            reader
                .bank_iter()
                .find_map(|b| b.asset_by_id(tex_id))
                .filter(|a| is_texture(a))
                .cloned()
        };
        from(&self.textures).or_else(|| from(&self.graphics))
    }

    /// Load the .tng (things) text file for a level by name.
    ///
    /// Prefers the debug build's loose .tng first; falls back to the wad.
    pub fn load_tng(&self, level_name: &str) -> Result<Tng, String> {
        let loose_path = self
            .fable_directory
            .join("data/Levels/FinalAlbion")
            .join(format!("{level_name}.tng"));
        if let Ok(bytes) = std::fs::read(&loose_path) {
            let text = String::from_utf8_lossy(&bytes);
            return Tng::parse(&text).map_err(|e| format!("parse tng: {e}"));
        }

        let wad_path = self.fable_directory.join("data/Levels/FinalAlbion.wad");
        let wad_file = std::io::BufReader::new(
            std::fs::File::open(&wad_path).map_err(|e| format!("open wad: {e}"))?,
        );
        let mut wad = WadReader::new(wad_file).map_err(|e| format!("read wad: {e}"))?;
        let suffix = format!("{level_name}.tng").to_lowercase();
        let asset = wad
            .asset_iter()
            .find(|a| a.path.to_lowercase().ends_with(&suffix))
            .cloned()
            .ok_or_else(|| format!("{level_name}.tng not found in wad"))?;
        let bytes = wad
            .read_content(&asset)
            .map_err(|e| format!("read tng: {e}"))?;
        let text = String::from_utf8_lossy(&bytes);
        Tng::parse(&text).map_err(|e| format!("parse tng: {e}"))
    }

    /// Read a mesh and its material textures from graphics.big, by symbol name.
    pub fn read_mesh(&mut self, mesh_name: &str) -> Result<(Mesh, MeshTextures), ReadMeshError> {
        let asset = self
            .graphics
            .bank_iter()
            .find_map(|bank| {
                bank.asset_iter()
                    .find(|a| a.symbol_name == mesh_name)
                    .cloned()
            })
            .ok_or(ReadMeshError::NotFound)?;
        self.read_mesh_asset(&asset)
    }

    /// Read a mesh by its `graphics.big` asset id — how a def's `Graphic.BankIndex` names one.
    pub fn read_mesh_by_id(&mut self, id: u32) -> Result<(Mesh, MeshTextures), ReadMeshError> {
        let asset = self
            .graphics
            .bank_iter()
            .find_map(|bank| bank.asset_by_id(id))
            .filter(|a| matches!(&a.extras, Some(ExtraMetadata::Mesh(_))))
            .cloned()
            .ok_or(ReadMeshError::NotFound)?;
        self.read_mesh_asset(&asset)
    }

    /// The mesh asset's symbol name, for logging a placement's provenance.
    pub fn mesh_name_by_id(&self, id: u32) -> Option<String> {
        self.graphics
            .bank_iter()
            .find_map(|bank| bank.asset_by_id(id))
            .map(|a| a.symbol_name.to_string())
    }

    fn read_mesh_asset(
        &mut self,
        asset: &AssetMetadata,
    ) -> Result<(Mesh, MeshTextures), ReadMeshError> {
        use ReadMeshError as E;

        let mesh_name = &asset.symbol_name;

        let mesh_data = self
            .graphics
            .read_asset_from_metadata(asset)
            .map_err(E::ReadAssetData)?;
        let mesh = Mesh::decode(&mesh_data).map_err(E::Decode)?;

        let mesh_extras = match &asset.extras {
            Some(ExtraMetadata::Mesh(extras)) => extras,
            _ => return Err(E::NotFound),
        };

        tracing::debug!(
            "Mesh {}: {} materials, texture_ids={:?}, base_texture_ids={:?}",
            mesh_name,
            mesh.materials.len(),
            mesh_extras.texture_ids,
            mesh.materials
                .iter()
                .map(|m| m.base_texture_id)
                .collect::<Vec<_>>(),
        );

        // Resolve each material's diffuse texture, keeping the result aligned 1:1 with
        // `mesh.materials` (None where the material has no/unresolvable texture) so the renderer
        // can index it by a primitive's material index. Texture ids generally resolve in
        // textures.big, falling back to graphics.big.
        let mut textures: MeshTextures = Vec::with_capacity(mesh.materials.len());
        for material in &mesh.materials {
            let tex_id = material.base_texture_id;
            let resolved = if tex_id == 0 {
                None
            } else if let Some(tex_asset) = self.find_texture_asset(tex_id) {
                let tex_data = self
                    .textures
                    .read_asset_from_metadata(&tex_asset)
                    .or_else(|_| self.graphics.read_asset_from_metadata(&tex_asset))
                    .map_err(E::ReadAssetData)?;
                Some((tex_asset, tex_data))
            } else {
                tracing::debug!("Mesh {mesh_name}: tex_id={tex_id} has no texture asset");
                None
            };
            textures.push(resolved);
        }

        Ok((mesh, textures))
    }
}

/// A mesh's resolved material textures, aligned 1:1 with `Mesh::materials`. Each entry is the
/// material's diffuse texture (metadata + raw bytes), or `None` if it has none.
type MeshTextures = Vec<Option<(AssetMetadata, Vec<u8>)>>;
