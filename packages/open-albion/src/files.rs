use derive_more::{Display, Error};
use fable_data::{
    big::{BigReader, BigReaderError, ReadAssetDataError},
    environment::{EnvironmentConfig, EnvironmentParseError, EnvironmentTheme},
    tga::{Tga, TgaError},
};
use std::{fs::File, io, path::Path};

pub struct Files {
    pub textures: BigReader<File>,
    pub lighting_lut_bytes: Vec<u8>,
    pub environment: EnvironmentConfig,
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

        // Load environment.def
        let env_bytes = Self::try_read_paths(&[fable_directory.join("data/Defs/environment.def")])
            .map_err(E::ReadEnvironmentDef)?;
        let env_str = String::from_utf8_lossy(&env_bytes);
        let environment = EnvironmentConfig::parse(&env_str).map_err(E::ParseEnvironmentDef)?;

        tracing::info!(
            "Loaded environment.def ({} themes)",
            environment.themes.len()
        );
        for (name, theme) in &environment.themes {
            tracing::debug!(
                "  Theme '{}': {} keyframes, textures: {:?}",
                name,
                theme.keyframes.len(),
                theme.sky_texture_names()
            );
        }

        Ok(Self {
            textures,
            lighting_lut_bytes,
            environment,
        })
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

    /// Get an environment theme by name.
    pub fn environment_theme(&self, name: &str) -> Option<&EnvironmentTheme> {
        self.environment.themes.get(name)
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
