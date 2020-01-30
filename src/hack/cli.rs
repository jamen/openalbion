// use winapi::shared::basetsd::*;
// use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::um::*;

use processthreadsapi::*;
use libloaderapi::*;
// use winnt::*;
// use winuser::*;

use std::io::{Write,BufRead};

pub fn start(module_handle: HMODULE) -> Result<(), u32> {
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();

    let mut stdout = std::io::stdout();

    loop {
        print!("> ");

        stdout.flush().expect("Failed to flush stdout.");

        let line = lines.next().unwrap().unwrap();

        match line.as_ref() {
            "ping" => ping()?,
            "unload" => unload(module_handle)?,
            "exit" => exit()?,
            "" => println!("No command given."),
            _ => println!("Unknown command."),
        }
    }
}

fn ping() -> Result<(), u32> {
    println!("pong");
    Ok(())
}

fn unload(module_handle: HMODULE) -> Result<(), u32> {
    unsafe { FreeLibraryAndExitThread(module_handle, 0) };
    Ok(())
}

fn exit() -> Result<(), u32> {
    unsafe { ExitProcess(0) };
    Ok(())
}