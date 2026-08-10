use derive_more::{Display, Error};
use fable_data::{
    big::{AssetMetadata, BigReader, BigReaderError, ExtraMetadata, ReadAssetDataError},
    def::binary::{DefBinary, DefBody},
    def::names::Names,
    def::EngineGraphic,
    def::EngineThemeDef,
    def::SkyDef,
    environment::{EnvironmentConfig, EnvironmentTheme},
    lev::{Lev, LevError},
    mesh::{Mesh, MeshError},
    tga::{Tga, TgaError},
    tng::Tng,
    wad::{ReadContentError, WadReader, WadReaderError},
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
    /// instance name — what `.tng` placements resolve through. See [`Self::load_thing_graphics`].
    pub thing_graphics: HashMap<String, EngineGraphic>,
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

        // Load sky environment themes — prefer CompiledDefs/game.bin (retail binary format),
        // fall back to the debug-only text environment.def.
        let environment = {
            let names_path = fable_directory.join("data/CompiledDefs/names.bin");
            let game_bin_path = fable_directory.join("data/CompiledDefs/game.bin");

            let from_binary = (|| -> Result<EnvironmentConfig, String> {
                let names = Names::load(&names_path).map_err(|e| format!("names.bin: {e:?}"))?;
                let def_binary = DefBinary::load_with_names(&game_bin_path, &names)
                    .map_err(|e| format!("game.bin: {e:?}"))?;

                Ok(EnvironmentConfig::from_binary_defs(
                    &def_binary,
                    &names,
                    |id| {
                        textures
                            .bank("GBANK_MAIN_PC")
                            .and_then(|b| b.asset_by_id(id as u32))
                            .map(|a| a.symbol_name.to_string())
                    },
                ))
            })();

            match from_binary {
                Ok(environment) => {
                    tracing::info!(
                        "Loaded environment themes from game.bin ({} themes)",
                        environment.themes.len()
                    );
                    Some(environment)
                }
                Err(bin_error) => {
                    tracing::warn!(
                        "Failed to load binary defs, falling back to environment.def: {bin_error}"
                    );

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

        let engine_themes = Self::load_engine_themes(fable_directory);
        let thing_graphics = Self::load_thing_graphics(fable_directory);

        Ok(Self {
            fable_directory: fable_directory.to_path_buf(),
            textures,
            graphics,
            lighting_lut_bytes,
            environment,
            engine_themes,
            thing_graphics,
        })
    }

    /// Load every `ENGINE_THEME` def from `game.bin`, keyed by its instance name.
    ///
    /// Name, not index: `CMap::LoadFromFile` resolves a `.lev`'s theme palette through
    /// `GetDefGlobalIndexFromName` (`fablelib/map.cpp:2561`), and the index stored in the
    /// palette is stale in retail data. Note that a def entry's `def_name` is its *class*
    /// (`"ENGINE_THEME"`); the instance name is `file_name`.
    fn load_engine_themes(fable_directory: &Path) -> HashMap<String, EngineThemeDef> {
        let names_path = fable_directory.join("data/CompiledDefs/names.bin");
        let game_bin_path = fable_directory.join("data/CompiledDefs/game.bin");

        let names = match Names::load(&names_path) {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!("names.bin not found, terrain textures disabled: {e:?}");
                return HashMap::new();
            }
        };
        let def_binary = match DefBinary::load_with_names(&game_bin_path, &names) {
            Ok(db) => db,
            Err(e) => {
                tracing::warn!("game.bin not found, terrain textures disabled: {e:?}");
                return HashMap::new();
            }
        };

        let mut map = HashMap::new();
        for entry in def_binary.entries(&names) {
            if let DefBody::EngineThemeDef(def) = &entry.record.body {
                if let Some(name) = entry.file_name {
                    map.insert(name.to_string(), def.clone());
                }
            }
        }
        tracing::info!("Loaded {} ENGINE_THEME defs from game.bin", map.len());
        map
    }

    /// An `ENGINE_THEME` def by its instance name, e.g. `"GROUND_GRASS"`.
    pub fn engine_theme_by_name(&self, name: &str) -> Option<&EngineThemeDef> {
        self.engine_themes.get(name)
    }

    /// Every def a `.tng` thing can name that carries a `Graphic`, keyed by instance name.
    ///
    /// Four def types have one, and between them they cover every placed thing that draws:
    /// `OBJECT` (2,849 defs), `CREATURE` (517), `BUILDING` (321) and `MARKER` (57). Reading
    /// all four uniformly is what makes buildings work without a second code path.
    ///
    /// `Graphic.BankIndex` is an asset id in `graphics.big` — not an index into anything,
    /// and not a symbol name. Across Witchwood, LookoutPoint and Arena every non-zero index
    /// resolves to a mesh-typed asset, which is why the text `objects.def` bridge this
    /// replaced is no longer needed.
    fn load_thing_graphics(fable_directory: &Path) -> HashMap<String, EngineGraphic> {
        let names_path = fable_directory.join("data/CompiledDefs/names.bin");
        let game_bin_path = fable_directory.join("data/CompiledDefs/game.bin");

        let names = match Names::load(&names_path) {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!("names.bin not found, level things disabled: {e:?}");
                return HashMap::new();
            }
        };
        let def_binary = match DefBinary::load_with_names(&game_bin_path, &names) {
            Ok(db) => db,
            Err(e) => {
                tracing::warn!("game.bin not found, level things disabled: {e:?}");
                return HashMap::new();
            }
        };

        let mut map = HashMap::new();
        for entry in def_binary.entries(&names) {
            let graphic = match &entry.record.body {
                DefBody::ThingObjectDef(def) => &def.graphic,
                DefBody::ThingBuildingDef(def) => &def.graphic,
                DefBody::ThingMarkerDef(def) => &def.graphic,
                DefBody::ThingCreatureDef(def) => &def.graphic,
                _ => continue,
            };
            if let Some(name) = entry.file_name {
                map.insert(name.to_string(), graphic.clone());
            }
        }
        tracing::info!("Loaded {} thing graphics from game.bin", map.len());
        map
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

    /// Load the SKY def from game.bin containing sun/moon texture indices.
    ///
    /// Unused until sun/moon rendering is ported from `RenderSun`/`RenderMoon`
    /// (AGENTS.md step 4.2).
    #[allow(dead_code)]
    pub fn load_sky_def(&self) -> Option<SkyDef> {
        let names_path = self.fable_directory.join("data/CompiledDefs/names.bin");
        let game_bin_path = self.fable_directory.join("data/CompiledDefs/game.bin");

        let names = Names::load(&names_path).ok()?;
        let def_binary = DefBinary::load_with_names(&game_bin_path, &names).ok()?;

        for entry in def_binary.entries(&names) {
            if let DefBody::SkyDef(def) = &entry.record.body {
                return Some(def.clone());
            }
        }
        None
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
