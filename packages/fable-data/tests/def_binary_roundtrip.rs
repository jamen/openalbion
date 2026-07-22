// Validation (ignored by default): round-trip the retail def binaries
// through parse → serialize → re-parse and check the container layout
// byte-for-byte.  cargo test -p fable-data --test def_binary_roundtrip -- --ignored --nocapture
use fable_data::def::binary::def_binary::DefBinary;
use fable_data::def::binary::names::Names;
use std::path::Path;

struct IndexEntry {
    compressed_offset: u32,
    cumulative_entry_count: u32,
}

fn u32_le(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

/// Split a serialized def binary into its container parts, asserting the
/// layout invariants the game's loader relies on.
fn check_layout(bytes: &[u8], label: &str) -> (usize, Vec<IndexEntry>, usize) {
    let entry_count = u32_le(bytes, 9) as usize;
    let index_at = 13 + entry_count * 12;
    let chunk_count = u32_le(bytes, index_at) as usize;
    let real_entries = chunk_count - 1;
    let entries_at = index_at + 8;
    let entries: Vec<IndexEntry> = (0..real_entries)
        .map(|i| IndexEntry {
            compressed_offset: u32_le(bytes, entries_at + i * 8),
            cumulative_entry_count: u32_le(bytes, entries_at + i * 8 + 4),
        })
        .collect();

    // Sentinel entry: both fields hold the total compressed data size.
    let sentinel_at = entries_at + real_entries * 8;
    let (s1, s2) = (u32_le(bytes, sentinel_at), u32_le(bytes, sentinel_at + 4));
    let data_size = (bytes.len() - sentinel_at - 8) as u32;
    assert_eq!(
        (s1, s2),
        (data_size, data_size),
        "{label}: sentinel must be (data_size, data_size)"
    );

    // Chunk offsets are relative to a data region that starts after the
    // sentinel; each must point at a zlib stream (78 01, level 1).
    let data_base = sentinel_at + 8;
    for (i, e) in entries.iter().enumerate() {
        let at = data_base + e.compressed_offset as usize;
        assert_eq!(
            &bytes[at..at + 2],
            &[0x78, 0x01],
            "{label}: chunk {i} does not start with a level-1 zlib header"
        );
    }
    (chunk_count, entries, data_base)
}

fn roundtrip(dir: &Path, names: &Names, bin_name: &str) {
    let original = std::fs::read(dir.join(bin_name)).unwrap();
    let parsed = DefBinary::load_with_names(&dir.join(bin_name), names).unwrap();
    let written = parsed.to_bytes();
    let label = bin_name;

    // Header (13 bytes) and name-ref table must be byte-identical.
    let entry_count = u32_le(&original, 9) as usize;
    let index_at = 13 + entry_count * 12;
    assert_eq!(
        &written[..index_at],
        &original[..index_at],
        "{label}: header + name refs differ"
    );

    let (orig_chunk_count, orig_entries, orig_base) = check_layout(&original, label);
    let (chunk_count, entries, base) = check_layout(&written, label);
    assert_eq!(
        chunk_count, orig_chunk_count,
        "{label}: chunk count changed"
    );
    assert_eq!(
        entries.len(),
        orig_entries.len(),
        "{label}: index entry count changed"
    );
    for (i, (a, b)) in entries.iter().zip(&orig_entries).enumerate() {
        assert_eq!(
            a.cumulative_entry_count, b.cumulative_entry_count,
            "{label}: entry {i} cumulative count changed"
        );
    }

    // Every chunk's decompressed content must be byte-identical.
    let mut chunk_count_checked = 0;
    for (i, (a, b)) in entries.iter().zip(&orig_entries).enumerate() {
        let end = |entries: &[IndexEntry], base: usize, len: usize, i: usize| {
            entries
                .get(i + 1)
                .map(|e| base + e.compressed_offset as usize)
                .unwrap_or(len)
        };
        let orig_blob = &original[orig_base + b.compressed_offset as usize
            ..end(&orig_entries, orig_base, original.len(), i)];
        let our_blob =
            &written[base + a.compressed_offset as usize..end(&entries, base, written.len(), i)];
        let orig_plain = miniz_oxide::inflate::decompress_to_vec_zlib(orig_blob).unwrap();
        let our_plain = miniz_oxide::inflate::decompress_to_vec_zlib(our_blob).unwrap();
        assert_eq!(orig_plain, our_plain, "{label}: chunk {i} content differs");
        chunk_count_checked += 1;
    }
    println!("{label}: {chunk_count_checked} chunks decompressed byte-identical");

    // Re-parse our output: every entry must round-trip byte-exact.
    let reparsed = DefBinary::from_bytes_with_names(&written, names).unwrap();
    let before: Vec<&[u8]> = parsed
        .entries(names)
        .map(|e| e.record.raw_bytes.as_slice())
        .collect();
    let after: Vec<&[u8]> = reparsed
        .entries(names)
        .map(|e| e.record.raw_bytes.as_slice())
        .collect();
    assert_eq!(before.len(), after.len(), "{label}: entry count changed");
    for (i, (a, b)) in before.iter().zip(&after).enumerate() {
        assert_eq!(a, b, "{label}: entry {i} bytes differ after re-parse");
    }
    println!("{label}: {} entries round-trip byte-exact", before.len());
}

#[test]
#[ignore]
fn roundtrip_retail_defs() {
    let dir = Path::new("/home/jamen/Fable/data/CompiledDefs");
    let names = Names::load(&dir.join("names.bin")).unwrap();
    for bin in ["frontend.bin", "script.bin", "game.bin"] {
        roundtrip(dir, &names, bin);
    }
}
