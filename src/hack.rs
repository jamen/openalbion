#![cfg(windows)]
#![allow(non_snake_case, unused_variables)]

mod hack {
    pub mod cli;
}

// use winapi::shared::ntdef::*;
use winapi::shared::basetsd::*;
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::um::*;

use processthreadsapi::*;
use winnt::*;
use winuser::*;

use std::os::windows::ffi::OsStringExt;
use std::ffi::OsString;
use std::ffi::CString;
use std::ptr::null_mut;

static mut FABLE_WND_PROC: Option<WNDPROC> = None;

#[derive(Debug)]
struct FableWindowSearch {
    process_id: DWORD,
    hwnd: Option<HWND>,
}

static mut DLL_HANDLE: Option<HINSTANCE> = None;

#[no_mangle]
extern "system" fn DllMain(dll_handle: HINSTANCE, fdv_reason: DWORD, lpv_reserved: LPVOID) -> BOOL {
    unsafe {
        DLL_HANDLE = Some(dll_handle);
    }

    match fdv_reason {
        DLL_PROCESS_ATTACH => {
            unsafe {
                CreateThread(null_mut(), 0, Some(init), null_mut(), 0, null_mut())
            };
        },
        DLL_PROCESS_DETACH => {
            unsafe { wincon::FreeConsole() };
        },
        _ => {}
    }
    1
}

extern "system" fn init(lpThreadParameter: LPVOID) -> DWORD {
    // Create a console for debug messages.
    unsafe { consoleapi::AllocConsole() };

    // Fable window search
    let process_id = unsafe { GetCurrentProcessId() };

    let mut fable_window_search = FableWindowSearch {
        process_id: process_id,
        hwnd: None,
    };

    while fable_window_search.hwnd.is_none() {
        unsafe {
            EnumWindows(Some(find_fable_window), &mut fable_window_search as *mut FableWindowSearch as LPARAM)
        };
    }

    let hwnd = fable_window_search.hwnd.unwrap();

    println!("{:#?}", fable_window_search);

    // error[E0133]: use of mutable static is unsafe and requires unsafe function or block
    // note: mutable statics can be mutated by multiple threads: aliasing violations or data races will cause undefined
    unsafe {
        FABLE_WND_PROC = Some(*(&GetWindowLongPtrA(hwnd, GWL_WNDPROC) as *const _ as *const WNDPROC));
    }

    // unsafe {
    //     SetWindowLongPtrA(hwnd, GWL_WNDPROC, *(&wnd_proc_hook as *const _ as *const i32));
    // }

    // AllocConsole prompt

    let module_handle = unsafe { DLL_HANDLE.unwrap() as HMODULE };

    hack::cli::start(module_handle).expect("Failed to start CLI.");

    0
}

// Fable window search callbacks

extern "system" fn find_fable_window(hwnd: HWND, search: LPARAM) -> BOOL {
    let mut search = unsafe { &mut *(search as *mut FableWindowSearch) };

    let mut process_id = 1 as DWORD;

    unsafe { GetWindowThreadProcessId(hwnd, &mut process_id as LPDWORD) };

    if process_id == search.process_id && unsafe { GetWindow(hwnd, GW_OWNER) } == 0 as HWND {
        let mut window_text_dest: [u8; 256] = [0; 256];
        let window_text_len = unsafe { GetWindowTextA(hwnd, window_text_dest.as_mut_ptr() as LPSTR, window_text_dest.len() as i32) };
        let window_text = std::str::from_utf8(&window_text_dest[..window_text_len as usize]).unwrap();

        let mut class_name_dest: [u8; 256] = [0; 256];
        let class_name_len = unsafe { GetClassNameA(hwnd, class_name_dest.as_mut_ptr() as LPSTR, class_name_dest.len() as i32) };
        let class_name = std::str::from_utf8(&class_name_dest[..class_name_len as usize]).unwrap();

        if class_name == "Fable - The Lost Chapters " && window_text == "Fable - The Lost Chapters " {
            search.hwnd = Some(hwnd);
            0
        } else {
            1
        }
    } else {
        1
    }
}

// Dxwnd wnd proc hook for reference:
// https://github.com/old-games/DXWnd-OG-build/blob/master/src/dxhook.cpp#L160-L309

// Borderless Gaming reference:
// https://github.com/Codeusa/Borderless-Gaming/blob/3677f913918d000f67177892252cec2cc67cd42a/BorderlessGaming.Logic/Windows/Manipulation.cs#L34

// unsafe extern "system" fn wnd_proc_hook(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
//     match msg {
//         WM_ACTIVATE => {
//             0
//         },
//         WM_SIZE => {
//             let mut rect: RECT = Default::default();

//             unsafe { MoveWindow(hwnd, 50, 50, 1280, 720, 1) };

//             0
//         },
//         _ => {
//             match FABLE_WND_PROC {
//                 Some(wnd_proc) => {
//                     CallWindowProcA(wnd_proc, hwnd, msg, wparam, lparam)
//                 },
//                 None => {
//                 }
//             }
//         }
//     }
//     DefWindowProcA(hwnd, msg, wparam, lparam)
// }