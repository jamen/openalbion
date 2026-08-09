use derive_more::{Display, Error};
use fable_data::{
    big::{AssetMetadata, BigReader, BigReaderError, ExtraMetadata, ReadAssetDataError},
    def::binary::{DefBinary, DefBody},
    def::names::Names,
    def::EngineThemeDef,
    def::SkyDef,
    environment::{EnvironmentConfig, EnvironmentTheme},
    lev::{Lev, LevError, ThemePalette},
    mesh::{Mesh, MeshError},
    object::ObjectDefs,
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
    pub lighting_lut_bytes: Vec<u8>,
    pub environment: Option<EnvironmentConfig>,
    pub engine_themes: HashMap<i32, EngineThemeDef>,
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

        let engine_themes = Self::load_engine_themes(fable_directory, &textures);

        Ok(Self {
            fable_directory: fable_directory.to_path_buf(),
            textures,
            graphics,
            lighting_lut_bytes,
            environment,
            engine_themes,
        })
    }

    fn load_engine_themes(
        fable_directory: &Path,
        _textures: &BigReader<File>,
    ) -> HashMap<i32, EngineThemeDef> {
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
                map.insert(entry.global_index as i32, def.clone());
            }
        }
        tracing::info!("Loaded {} engine theme defs from game.bin", map.len());

        // Debug: check if palette def_index values are CRCs into the names table
        let sample_names = [
            ("GROUND_GRASS_NO_LOCAL_DETAIL", 1909i32),
            ("GROUND_PATH_SAND", 1906i32),
            ("GROUND_ROCK_CLIFF", 1882i32),
            ("GROUND_BIGTREES", 1915i32),
        ];
        for (name, di) in &sample_names {
            let crc = *di as u32;
            if let Some(names_entry) = names.map.get(&crc) {
                tracing::info!(
                    "  CRC lookup: def_index={di} (0x{crc:08X}) → \"{}\", palette name=\"{}\", match={}",
                    names_entry.string,
                    name,
                    names_entry.string == *name,
                );
            } else {
                tracing::info!(
                    "  CRC lookup: def_index={di} (0x{crc:08X}) → <not found in names table>",
                );
            }
        }
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

    /// Load the LUT row indices from the ENVIRONMENT def in game.bin.
    /// Falls back to defaults (13-16) when the def fields are zero.
    pub fn load_lut_rows(&self) -> LutRows {
        let mut rows = LutRows::default();
        let names_path = self.fable_directory.join("data/CompiledDefs/names.bin");
        let game_bin_path = self.fable_directory.join("data/CompiledDefs/game.bin");
        let Ok(names) = Names::load(&names_path) else { return rows; };
        let Ok(def_binary) = DefBinary::load_with_names(&game_bin_path, &names) else { return rows; };
        for entry in def_binary.entries(&names) {
            if let DefBody::Environment(def) = &entry.record.body {
                if def.sky_gradient_top_lookup_row > 0 { rows.sky_gradient_top = def.sky_gradient_top_lookup_row as usize; }
                if def.sky_gradient_top_alpha_lookup_row > 0 { rows.sky_gradient_top_alpha = def.sky_gradient_top_alpha_lookup_row as usize; }
                if def.sky_gradient_bottom_lookup_row > 0 { rows.sky_gradient_bottom = def.sky_gradient_bottom_lookup_row as usize; }
                if def.sky_gradient_bottom_alpha_lookup_row > 0 { rows.sky_gradient_bottom_alpha = def.sky_gradient_bottom_alpha_lookup_row as usize; }
                break;
            }
        }
        rows
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

    pub fn resolve_terrain_themes(&self, palette: &ThemePalette) -> TerrainThemeBundle {
        let mut palette_to_layer = [0u16; 256];
        let mut texture_ids: Vec<i32> = Vec::new();
        let mut def_index_to_layer: HashMap<i32, u16> = HashMap::new();

        for (pal_idx, entry) in palette.entries.iter().enumerate() {
            if entry.def_index <= 0 || entry.name == "NO_THEME" || entry.name.is_empty() {
                continue;
            }
            if let Some(&layer) = def_index_to_layer.get(&entry.def_index) {
                palette_to_layer[pal_idx] = layer;
                continue;
            }
            let Some(theme) = self.engine_themes.get(&entry.def_index) else {
                tracing::debug!(
                    "Palette [{}] \"{}\" def_index={} not found in engine_themes ({} themes loaded)",
                    pal_idx, entry.name, entry.def_index, self.engine_themes.len(),
                );
                continue;
            };
            if theme.base_texture <= 0 {
                tracing::debug!(
                    "Engine theme \"{}\" (def_index={}) has no base_texture (id={})",
                    entry.name, entry.def_index, theme.base_texture,
                );
                continue;
            }
            let layer = texture_ids.len() as u16;
            texture_ids.push(theme.base_texture);
            def_index_to_layer.insert(entry.def_index, layer);
            palette_to_layer[pal_idx] = layer;
        }

        let theme_count = texture_ids.len();
        let used_slots = palette.entries.iter().filter(|e| e.name != "NO_THEME" && !e.name.is_empty()).count();
        tracing::info!(
            "Terrain themes: {} unique engine themes with textures ({} palette slots with names, {} total)",
            theme_count,
            used_slots,
            palette.entries.len(),
        );
        if theme_count > 0 {
            tracing::info!(
                "First texture IDs: {:?}",
                &texture_ids[..theme_count.min(5)],
            );
        }
        if used_slots > 0 {
            let sample: Vec<&str> = palette.entries.iter()
                .filter(|e| e.name != "NO_THEME" && !e.name.is_empty())
                .take(5)
                .map(|e| e.name.as_str())
                .collect();
            tracing::info!("Sample palette names: {:?}", sample);
            if !self.engine_themes.is_empty() {
                let sample_keys: Vec<i32> = self.engine_themes.keys().take(5).copied().collect();
                tracing::info!("Sample engine theme def_indices: {:?}", sample_keys);
            }
        }

        TerrainThemeBundle {
            palette_to_layer,
            texture_ids,
        }
    }

    /// Load OBJECT definitions from a text `objects.def` at `path`, returning a resolver that maps
    /// OBJECT def names to mesh symbols.
    ///
    /// NOTE: this is a temporary text-def bridge (used only when the user explicitly points at an
    /// `objects.def`, e.g. from the debug build). The engine's proper path is to resolve OBJECT
    /// defs from retail `CompiledDefs/game.bin` once the binary `OBJECT` def type is implemented —
    /// see the def-coverage work. The engine does not read text defs by default.
    pub fn load_object_defs(&self, path: &Path) -> Result<ObjectDefs, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("read {path:?}: {e}"))?;
        let text = String::from_utf8_lossy(&bytes);
        ObjectDefs::parse(&text).map_err(|e| format!("parse {path:?}: {e}"))
    }

    /// Read a mesh and its material textures from graphics.big.
    pub fn read_mesh(&mut self, mesh_name: &str) -> Result<(Mesh, MeshTextures), ReadMeshError> {
        use ReadMeshError as E;

        let asset = self
            .graphics
            .bank_iter()
            .find_map(|bank| {
                bank.asset_iter()
                    .find(|a| a.symbol_name == mesh_name)
                    .cloned()
            })
            .ok_or(E::NotFound)?;

        let mesh_data = self
            .graphics
            .read_asset_from_metadata(&asset)
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

/// LUT row indices from the ENVIRONMENT def.
#[derive(Debug, Clone, Copy)]
pub struct LutRows {
    pub sky_gradient_top: usize,
    pub sky_gradient_top_alpha: usize,
    pub sky_gradient_bottom: usize,
    pub sky_gradient_bottom_alpha: usize,
}

impl Default for LutRows {
    fn default() -> Self {
        Self {
            sky_gradient_top: 13,
            sky_gradient_top_alpha: 14,
            sky_gradient_bottom: 15,
            sky_gradient_bottom_alpha: 16,
        }
    }
}

pub struct TerrainThemeBundle {
    pub palette_to_layer: [u16; 256],
    pub texture_ids: Vec<i32>,
}
