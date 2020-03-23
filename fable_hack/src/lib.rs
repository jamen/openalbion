//! To use this:
//!
//! ```txt
//! defable --find
//! ```

#![cfg(windows)]
#![allow(non_snake_case, unused_variables)]

use winapi::shared::d3d9::*;
use winapi::shared::d3d9types::*;
use winapi::shared::minwindef::*;
// use winapi::shared::ntdef::*;
use winapi::shared::windef::*;
use winapi::um::*;

use processthreadsapi::*;
use libloaderapi::*;
use winnt::*;
use winuser::*;
use memoryapi::*;
use wingdi::*;

use std::ptr::null_mut;
use std::io::{BufRead,Write};
use std::mem;
use std::ffi::CStr;
use std::os::raw::c_char;

pub struct Hack {
    pub dll_handle: Option<HINSTANCE>,
    pub pid: u32,
    pub hwnd: Option<HWND>,
}

pub static mut HACK: Hack = Hack {
    dll_handle: None,
    pid: 0,
    hwnd: None,
};

// struct GSystemManager {}

// static CFGetSystemManager: fn() -> GSystemManager = unsafe { *(0x009a4ec0 as *const fn() -> GSystemManager) };

#[no_mangle]
unsafe extern "system" fn DllMain(dll_handle: HINSTANCE, fdv_reason: DWORD, lpv_reserved: LPVOID) -> BOOL {
    HACK.dll_handle = Some(dll_handle);

    match fdv_reason {
        DLL_PROCESS_ATTACH => {
            CreateThread(null_mut(), 0, Some(init), null_mut(), 0, null_mut());
        },
        DLL_PROCESS_DETACH => {
            wincon::FreeConsole();
        },
        _ => {}
    }

    1
}

unsafe extern "system" fn init(lpThreadParameter: LPVOID) -> DWORD {
    //
    // Do some basic stuff
    //

    consoleapi::AllocConsole();

    HACK.pid = GetCurrentProcessId();

    //
    // Search or wait for Fable's window
    //

    while HACK.hwnd.is_none() { EnumWindows(Some(find_fable_window), 0); }

    let hwnd = HACK.hwnd.unwrap();

    println!("hwnd {:?}", HACK.hwnd);

    //
    // Make the thread handle commands.
    //

    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();

    let mut stdout = std::io::stdout();

    loop {
        print!("> ");

        stdout.flush().expect("Failed to flush stdout.");

        let line = lines.next().unwrap().unwrap();

        match line.as_ref() {
            // ...
            "" => println!("No command given."),
            _ => println!("Unknown command."),
        }
    }

    0
}

// Fable window search callback
unsafe extern "system" fn find_fable_window(hwnd: HWND, _: LPARAM) -> BOOL {
    let mut pid = 1 as DWORD;

    GetWindowThreadProcessId(hwnd, &mut pid as LPDWORD);

    if pid == HACK.pid && GetWindow(hwnd, GW_OWNER) == 0 as HWND {
        let mut window_text_dest: [u8; 256] = [0; 256];
        let window_text_len = GetWindowTextA(hwnd, window_text_dest.as_mut_ptr() as LPSTR, window_text_dest.len() as i32);
        let window_text = std::str::from_utf8(&window_text_dest[..window_text_len as usize]).unwrap();

        let mut class_name_dest: [u8; 256] = [0; 256];
        let class_name_len = GetClassNameA(hwnd, class_name_dest.as_mut_ptr() as LPSTR, class_name_dest.len() as i32);
        let class_name = std::str::from_utf8(&class_name_dest[..class_name_len as usize]).unwrap();

        if class_name == "Fable - The Lost Chapters " && window_text == "Fable - The Lost Chapters " {
            HACK.hwnd = Some(hwnd);
            0
        } else {
            1
        }
    } else {
        1
    }
}