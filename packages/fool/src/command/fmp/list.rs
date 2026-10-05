use anyhow::{Context, anyhow};
use clap::Parser;
use fable_data::big::{AssetMetadata, ExtraMetadata};
use fable_data::fmp::FmpReader;
use std::{fs::File, path::PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct FmpListArgs {
    /// Input fmp file to list
    input: PathBuf,

    /// Only list records in this bank (e.g. GameBINEntries)
    #[arg(long)]
    bank: Option<String>,

    /// Decode and print a preview of text-bank payloads
    #[arg(long)]
    text: bool,
}

pub fn handler(args: FmpListArgs) -> anyhow::Result<()> {
    let input_file = File::open(&args.input).context("Could not open fmp file")?;

    let mut reader = FmpReader::new(input_file).map_err(|e| anyhow!("Could not read fmp file: {e}"))?;

    println!("File:         {}", args.input.display());
    println!("Version:      {}  Content type: {}", reader.version(), reader.content_type());
    println!();

    // Snapshot the bank table; `bank_records`/`read_payload` need `&mut`.
    let banks: Vec<_> = reader.banks().to_vec();

    for bank in &banks {
        if let Some(filter) = &args.bank {
            if !bank.name.eq_ignore_ascii_case(filter) {
                continue;
            }
        }

        if bank.asset_count == 0 {
            continue;
        }

        match reader.bank_records(bank) {
            Ok(records) => {
                println!("{} ({} record(s))", bank.name, records.len());
                for record in &records {
                    println!("  {}", describe(&mut reader, bank.name.as_ref(), record, args.text)?);
                }
            }
            Err(e) => println!("{} ({} record(s), unreadable layout: {e})", bank.name, bank.asset_count),
        }
    }

    Ok(())
}

/// One printed line: the symbol name plus whatever this bank's metadata lets us say about it.
fn describe(
    reader: &mut FmpReader,
    bank: &str,
    record: &AssetMetadata,
    decode_text: bool,
) -> anyhow::Result<String> {
    let name = record.symbol_name.as_ref();
    let detail = match &record.extras {
        Some(ExtraMetadata::Unknown(bytes)) if bank.ends_with("BINEntries") => {
            let def = bytes
                .split(|b| *b == 0)
                .next()
                .map(|b| String::from_utf8_lossy(b).into_owned())
                .unwrap_or_default();
            format!("def={def}")
        }
        Some(ExtraMetadata::Texture(tex)) => format!("texture {}x{}", tex.width, tex.height),
        Some(ExtraMetadata::Mesh(_)) => "mesh".to_string(),
        _ if bank.eq_ignore_ascii_case("text") => {
            if decode_text {
                let payload = reader
                    .read_payload(record)
                    .map_err(|e| anyhow!("read {} payload: {e}", record.symbol_name))?;
                format!("\"{}\"", preview_utf16(&payload))
            } else {
                String::new()
            }
        }
        _ => String::new(),
    };

    Ok(format!(
        "{name:<48} {detail:<28} {:>9} B",
        record.size
    ))
}

/// A Fable text payload is UTF-16LE, NUL-terminated.
fn preview_utf16(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|u| *u != 0)
        .collect();

    let mut text = String::from_utf16_lossy(&units);
    let max = 80;
    if text.chars().count() > max {
        text = text.chars().take(max).collect::<String>() + "...";
    }
    text
}
