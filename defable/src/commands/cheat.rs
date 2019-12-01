use fable_launch::inject_dll;
use clap::{App, SubCommand, Arg, ArgMatches};

pub fn register<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("cheat")
        .about("Dll injection tool.")
        .arg(
            Arg::with_name("pid")
            .help("Fable's process id.")
            .short("p")
            .long("pid")
            .value_name("FABLE")
            .number_of_values(1)
        )
        .arg(
            Arg::with_name("dll")
            .help("Injected DLL path.")
            .short("d")
            .long("dll")
            .value_name("DLL")
        )
}

pub fn main<'a>(matches: &ArgMatches<'a>) {
    let pid = matches.value_of("pid").unwrap();
    let dll = matches.value_of("dll").unwrap();
    inject_dll(pid, dll).unwrap();
}