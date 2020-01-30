// use winapi::shared::basetsd::*;
// use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::um::*;

use processthreadsapi::*;
use libloaderapi::*;
// use winnt::*;
// use winuser::*;

use std::io::{Write,BufRead};

use super::{HackError,HACK};

pub fn start() -> Result<(), HackError> {
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();

    let mut stdout = std::io::stdout();

    loop {
        print!("> ");

        stdout.flush().expect("Failed to flush stdout.");

        let line = lines.next().unwrap().unwrap();

        match line.as_ref() {
            "ping" => ping()?,
            "unload" => unload()?,
            "exit" => exit()?,
            "" => println!("No command given."),
            _ => println!("Unknown command."),
        }
    }
}

fn ping() -> Result<(), HackError> {
    println!("pong");
    Ok(())
}

fn unload() -> Result<(), HackError> {
    unsafe {
        let moudle_handle = HACK.dll_handle.unwrap() as HMODULE;
        FreeLibraryAndExitThread(moudle_handle, 0);
    }
    Ok(())
}

fn exit() -> Result<(), HackError> {
    unsafe { ExitProcess(0) };
    Ok(())
}