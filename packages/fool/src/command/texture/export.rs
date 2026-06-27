use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct TextureExportArgs {}

pub fn handler(_args: TextureExportArgs) -> anyhow::Result<()> {
    // TODO: Decode a texture asset (fable_data::texture / fable_data::tga) out of a .big bank
    // and write it as a PNG. The decode side already exists in fable-data; this needs the
    // big-asset -> TextureMetadata -> decoded-image -> PNG plumbing (and a `png` dependency).
    anyhow::bail!("`texture export` is not implemented yet")
}
