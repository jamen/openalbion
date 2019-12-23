#![cfg(windows)]
#![allow(non_snake_case, unused_variables)]

use std::ptr::null_mut;
use std::mem;
use std::ffi::CString;
use winapi::shared::minwindef::TRUE;

use winapi::um::processthreadsapi::{CreateThread,ExitProcess};
use winapi::shared::minwindef::{HINSTANCE,DWORD,LPVOID,BOOL};
use winapi::um::consoleapi::AllocConsole;
use winapi::um::winnt::{DLL_PROCESS_ATTACH,DLL_PROCESS_DETACH};
use winapi::um::fileapi::{CreateFileA,CREATE_NEW};
use winapi::um::winnt::{GENERIC_ALL,FILE_ATTRIBUTE_NORMAL};

#[no_mangle]
extern "system" fn DllMain(
    dll_handle: HINSTANCE,
    fdv_reason: DWORD,
    lpv_reserved: LPVOID
) -> BOOL {
    match fdv_reason {
        DLL_PROCESS_ATTACH => {
            unsafe { AllocConsole() };
            // unsafe { CreateThread(null_mut(), 0, Some(init), null_mut(), 0, null_mut()) };

            // let file_name = CString::new("C:\\Users\\Jamen\\test.txt").unwrap();
            // unsafe { CreateFileA(file_name.as_ptr(), GENERIC_ALL, 0, null_mut(), CREATE_NEW, FILE_ATTRIBUTE_NORMAL, null_mut()) };

            // unsafe { ExitProcess(0) };
        },
        DLL_PROCESS_DETACH => {},
        _ => {}
    }
    TRUE
}

// extern "system" fn init(lpThreadParameter: LPVOID) -> DWORD {
//     unsafe { AllocConsole() };
//     0
// }