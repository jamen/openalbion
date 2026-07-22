use fable_data::def::binary::{def_binary::DefBinary, names::Names};
use std::path::Path;

fn main() {
    let data = std::env::args()
        .nth(1)
        .expect("usage: roundtrip_defs <Data dir> [bin name]");
    let bin = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "frontend.bin".to_string());
    let dir = Path::new(&data).join("CompiledDefs/Development");
    let names = Names::load(&dir.join("names.bin")).unwrap();
    let def_binary = DefBinary::load_with_names(&dir.join(&bin), &names).unwrap();

    let mut ok = 0usize;
    let mut bad = 0usize;
    for entry in def_binary.entries(&names) {
        let rec = entry.record;
        let size = rec.byte_size();
        let mut buf = vec![0u8; size];
        let mut out = &mut buf[..];
        match rec.serialize(&mut out) {
            Ok(()) => {
                let written = size - out.len();
                if buf[..written] == rec.raw_bytes[..] {
                    ok += 1;
                } else {
                    bad += 1;
                    if bad <= 5 {
                        println!(
                            "MISMATCH {} ({}): {} vs {} bytes",
                            entry.def_name.unwrap_or("?"),
                            entry.file_name.unwrap_or("?"),
                            written,
                            rec.raw_bytes.len()
                        );
                        for (i, (a, b)) in
                            buf[..written].iter().zip(rec.raw_bytes.iter()).enumerate()
                        {
                            if a != b {
                                println!("  first diff at byte {i}: wrote {a:02x} orig {b:02x}");
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                bad += 1;
                if bad <= 5 {
                    println!("SERIALIZE ERROR {}: {:?}", entry.def_name.unwrap_or("?"), e);
                }
            }
        }
    }
    println!("ok: {ok}, bad: {bad}");
}
