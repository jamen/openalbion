use anyhow::{Context, anyhow};
use clap::Parser;
use fable_data::fmp::FmpReader;
use std::{fs::File, path::PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct FmpInfoArgs {
    /// Input fmp file to inspect
    input: PathBuf,
}

pub fn handler(args: FmpInfoArgs) -> anyhow::Result<()> {
    let input_file = File::open(&args.input).context("Could not open fmp file")?;

    let reader = FmpReader::new(input_file).map_err(|e| anyhow!("Could not read fmp file: {e}"))?;

    let banks = reader.banks();

    println!("File:         {}", args.input.display());
    println!("Version:      {}", reader.version());
    println!("Content type: {}", reader.content_type());
    println!("Banks:        {}", banks.len());
    println!();
    println!(
        "{:<26} {:>4} {:>7} {:>10} {:>9} {:>6}",
        "BANK", "ID", "ASSETS", "POSITION", "LENGTH", "BLOCK"
    );

    for bank in banks {
        println!(
            "{:<26} {:>4} {:>7} {:>10} {:>9} {:>6}",
            bank.name, bank.id, bank.asset_count, bank.position, bank.length, bank.block_size
        );
    }

    let total: u64 = banks.iter().map(|bank| u64::from(bank.length)).sum();
    println!();
    println!("{} bank(s), {total} bytes of bank data", banks.len());

    Ok(())
}
