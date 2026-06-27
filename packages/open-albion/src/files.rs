use derive_more::{Display, Error};
use fable_data::{
    big::{BigReader, BigReaderError, ReadAssetDataError},
    environment::{EnvironmentConfig, EnvironmentParseError, EnvironmentTheme},
    lev::{Lev, LevError},
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
    ReadLightingLut(io::Error),
    ParseLightingLut(TgaError),
    ReadEnvironmentDef(io::Error),
    ParseEnvironmentDef(EnvironmentParseError),
}

impl Files {
    pub fn new(fable_directory: &Path) -> Result<Self, NewFilesError> {
        use NewFilesError as E;

        // Load textures.big
        let textures_path = fable_directory.join("data/graphics/pc/textures.big");
        let textures_file = File::open(&textures_path).map_err(E::OpenTextures)?;
        let textures = BigReader::new(textures_file).map_err(E::LoadTextures)?;

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

        // Load environment.def (optional — the sky is incomplete and the file isn't always
        // present; the terrain renders without it).
        let environment = match Self::try_read_paths(&[
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
                        tracing::warn!("Failed to parse environment.def, sky disabled: {error}");
                        None
                    }
                }
            }
            Err(_) => {
                tracing::warn!("environment.def not found, sky disabled");
                None
            }
        };

        Ok(Self {
            fable_directory: fable_directory.to_path_buf(),
            textures,
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
}
