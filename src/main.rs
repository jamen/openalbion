use std::io::{Write,Seek,SeekFrom};
use std::fs::OpenOptions;
use std::path::Path;

use fable_data::{Stb,StbEntry,Decode};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let stb_path = args.get(0).expect("First argument must be an stb path.");

    let mut stb_file = OpenOptions::new().read(true).write(true).open(stb_path).unwrap();
    let stb = Stb::decode(&mut stb_file).unwrap();

    if args.len() <= 1 { return println!("{:#?}", stb); }

    let entry_name = args.get(1).expect("Second argument must be an entry name.");
    let mut entry: Option<&StbEntry> = None;

    for candidate_entry in &stb.entries {
        let candidate_name = Path::new(&candidate_entry.name_1).file_stem().unwrap().to_str().unwrap();
        if candidate_name == entry_name {
            entry = Some(candidate_entry);
            break
        }
    }

    let entry = entry.expect("Failed to find entry.");

    if args.len() <= 2 { return println!("{:#?}", entry); }

    let target_name = args.get(2).expect("Third argument must be the target name.");
    let mut target_pos: u32 = stb.header.entries_offset + 12;

    for candidate_entry in &stb.entries {
        let candidate_name = Path::new(&candidate_entry.name_1).file_stem().unwrap().to_str().unwrap();

        if candidate_name == target_name {
            break
        }

        target_pos +=
            44 +
            candidate_entry.name_1.len() as u32 +
            candidate_entry.name_2.len() as u32 +
            match candidate_entry.extras { Some(_) => 16, None => 0 };
    }

    let mut new_offsets: [u8; 8] = [0; 8];

    new_offsets[..4].copy_from_slice(&entry.length.to_le_bytes());
    new_offsets[4..].copy_from_slice(&entry.offset.to_le_bytes());

    stb_file.seek(SeekFrom::Start((target_pos + 12) as u64)).unwrap();
    stb_file.write(&new_offsets).unwrap();

    println!("Entry location written.");
}
