//! Validates the texture layer against real game data: every texture asset in `textures.big`.
//!
//! Skips gracefully if the game data isn't present. Set `FABLE_DATA` to override the data dir
//! (defaults to `~/Fable/data`).
//!
//! This is AGENTS.md §6.9's *provenance* layer: it pins the measurements §3.12 derives from,
//! so a wrong reading of the format tag or the mip layout fails a test rather than showing up
//! as noise on the screen.

use fable_data::big::{BigReader, ExtraMetadata, TextureMetadata};
use fable_data::texture::{BcnEncoding, bcn_block_bytes, bcn_encoding_from_dxt};
use std::{collections::BTreeMap, fs::File, path::PathBuf};

fn fable_data_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("FABLE_DATA") {
        return Some(PathBuf::from(dir));
    }
    let home = std::env::var("HOME").ok()?;
    let dir = PathBuf::from(home).join("Fable/data");
    dir.is_dir().then_some(dir)
}

fn textures_big() -> Option<BigReader<File>> {
    let path = fable_data_dir()?.join("graphics/pc/textures.big");
    if !path.is_file() {
        eprintln!("skipping: {path:?} not found");
        return None;
    }
    Some(BigReader::new(File::open(&path).expect("open textures.big")).expect("read textures.big"))
}

fn texture_assets(reader: &mut BigReader<File>) -> Vec<(String, String, TextureMetadata)> {
    reader
        .bank_iter()
        .flat_map(|bank| {
            let bank_name = bank.metadata().name.to_string();
            bank.asset_iter()
                .filter_map(move |asset| match &asset.extras {
                    Some(ExtraMetadata::Texture(x)) => {
                        Some((bank_name.clone(), asset.symbol_name.to_string(), x.clone()))
                    }
                    _ => None,
                })
        })
        .collect()
}

/// Size of one mip level, transcribed from `CTextureManager::CalculateTextureSize`
/// (`bbblibrary/lib_texture_manager_2.cpp:1976`): DXT levels clamp to a 4-pixel minimum in
/// each dimension. AGENTS.md §3.12.
fn level_bytes(width: u32, height: u32, depth: u32, encoding: BcnEncoding) -> u64 {
    let blocks = |n: u32| n.max(4).div_ceil(4) as u64;
    blocks(width) * blocks(height) * depth.max(1) as u64 * bcn_block_bytes(encoding) as u64
}

/// The `dxt_compression` tag is not uniformly a BCn tag: three values are block compressed and
/// the rest are not, whatever their D3D ordinals suggest. Pins the table in §3.12.
#[test]
fn dxt_tags_are_only_31_32_35() {
    let Some(mut reader) = textures_big() else {
        return;
    };
    let assets = texture_assets(&mut reader);
    assert!(!assets.is_empty(), "no texture assets in textures.big");

    let mut by_tag: BTreeMap<u16, usize> = BTreeMap::new();
    for (_, _, x) in &assets {
        *by_tag.entry(x.dxt_compression).or_default() += 1;
    }
    println!("dxt tag histogram over {} assets: {by_tag:?}", assets.len());

    // Exactly these five tags occur. 31/32/35 are DXT1/3/5; 1 is uncompressed 32bpp and 24 is
    // D3DFMT_X1R5G5B5, neither of which we decode.
    let tags: Vec<u16> = by_tag.keys().copied().collect();
    assert_eq!(tags, vec![1, 24, 31, 32, 35]);

    // `BcnEncoding` is bcndecode's and derives neither PartialEq nor Debug, so match rather
    // than compare.
    assert!(matches!(bcn_encoding_from_dxt(31), Some(BcnEncoding::Bc1)));
    assert!(matches!(bcn_encoding_from_dxt(32), Some(BcnEncoding::Bc2)));
    assert!(matches!(bcn_encoding_from_dxt(35), Some(BcnEncoding::Bc3)));

    // The two tags that are not block compressed must be declined, not decoded as BC1.
    assert!(bcn_encoding_from_dxt(1).is_none());
    assert!(bcn_encoding_from_dxt(24).is_none());

    // And the aliases that were once mapped here match nothing, so they stay unmapped.
    for absent in [3u16, 5, 33, 34] {
        assert!(!by_tag.contains_key(&absent), "tag {absent} does occur");
        assert!(bcn_encoding_from_dxt(absent).is_none());
    }
}

/// `top_mip_map_size` is level 0 at the encoding's block size — the check that the tag → encoding
/// mapping is right, rather than merely self-consistent.
#[test]
fn top_mip_size_matches_the_encoding() {
    let Some(mut reader) = textures_big() else {
        return;
    };

    let mut checked = 0usize;
    let mut mismatched = Vec::new();
    for (_, symbol, x) in texture_assets(&mut reader) {
        let Some(encoding) = bcn_encoding_from_dxt(x.dxt_compression) else {
            continue;
        };
        // Animated assets pack every frame into the asset; the header's size is per frame.
        if x.frame_count > 1 || x.width == 0 || x.height == 0 {
            continue;
        }
        let expected = level_bytes(x.width as u32, x.height as u32, x.depth as u32, encoding);
        checked += 1;
        if x.top_mip_map_size as u64 != expected {
            mismatched.push((symbol, x.width, x.height, x.depth, x.top_mip_map_size, expected));
        }
    }

    assert!(checked > 5000, "expected most of the archive, checked {checked}");
    assert!(
        mismatched.is_empty(),
        "{} of {checked} assets disagree with their declared encoding: {:?}",
        mismatched.len(),
        &mismatched[..mismatched.len().min(8)],
    );
}

/// `depth > 1` is a volume texture, and `top_mip_map_size` then covers every slice — which is
/// what `top_mip_size_matches_the_encoding` verifies by multiplying by depth. Two textures are
/// volumes, each with a `_PC` twin in a second bank, and none is a 2D image, so the 2D upload
/// path must decline them rather than treat slice 0's dimensions as the whole asset.
#[test]
fn volume_textures_are_exactly_the_four_we_know() {
    let Some(mut reader) = textures_big() else {
        return;
    };

    let mut volumes: Vec<_> = texture_assets(&mut reader)
        .into_iter()
        .filter(|(_, _, x)| x.depth > 1)
        .map(|(_, symbol, x)| (symbol, x.width, x.height, x.depth))
        .collect();
    volumes.sort();

    assert_eq!(
        volumes,
        vec![
            ("MIST_ALPHA".to_string(), 64, 64, 64),
            ("MIST_ALPHA_PC".to_string(), 64, 64, 64),
            ("WEATHER_RAIN".to_string(), 32, 32, 8),
            ("WEATHER_RAIN_PC".to_string(), 32, 32, 8),
        ]
    );
}
