use derive_more::{Display, Error};
use fable_data::{
    big::{AssetMetadata, BigReader, BigReaderError, ExtraMetadata, ReadAssetDataError},
    def::binary::{def_binary::DefBinary, names::Names},
    environment::{EnvironmentConfig, EnvironmentTheme},
    lev::{Lev, LevError},
    mesh::{Mesh, MeshError},
    tga::{Tga, TgaError},
    wad::{ReadContentError, WadReader, WadReaderError},
};
use std::{
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
                let names =
                    Names::load(&names_path).map_err(|e| format!("names.bin: {e:?}"))?;
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
                    tracing::warn!("Failed to load binary defs, falling back to environment.def: {bin_error}");

                    match Self::try_read_paths(&[
                        fable_directory.join("data/Defs/environment.def"),
                    ]) {
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

        Ok(Self {
            fable_directory: fable_directory.to_path_buf(),
            textures,
            graphics,
            lighting_lut_bytes,
            environment,
        })
    }

    /// Load and parse a level by name (e.g. "Witchwood") from `FinalAlbion.wad`.
    pub fn load_level(&self, name: &str) -> Result<Lev, LoadLevelError> {
        use LoadLevelError as E;

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
            mesh.materials.iter().map(|m| m.base_texture_id).collect::<Vec<_>>(),
        );

        // Mesh material texture ids generally resolve in textures.big, falling back to graphics.big.
        let mut textures = Vec::with_capacity(mesh.materials.len());
        for material in &mesh.materials {
            let tex_id = material.base_texture_id;
            if tex_id == 0 {
                continue;
            }
            let Some(tex_asset) = self.find_texture_asset(tex_id) else {
                tracing::debug!("Mesh {mesh_name}: tex_id={tex_id} has no texture asset");
                continue;
            };
            let tex_data = self
                .textures
                .read_asset_from_metadata(&tex_asset)
                .or_else(|_| self.graphics.read_asset_from_metadata(&tex_asset))
                .map_err(E::ReadAssetData)?;
            textures.push((tex_asset, tex_data));
        }

        Ok((mesh, textures))
    }
}

/// A mesh's resolved material textures: each asset's metadata paired with its raw bytes.
type MeshTextures = Vec<(AssetMetadata, Vec<u8>)>;
