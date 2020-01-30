#![cfg(windows)]
#![allow(non_snake_case, unused_variables)]

mod hack;

use hack::HACK;

// use winapi::shared::ntdef::*;
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::um::*;

use processthreadsapi::*;
use winnt::*;
use winuser::*;

use std::ptr::null_mut;

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
    HACK.pid = GetCurrentProcessId();

    // Create a console for debug messages.
    consoleapi::AllocConsole();

    // Fable window search
    while HACK.hwnd.is_none() { EnumWindows(Some(find_fable_window), 0); }

    let hwnd = HACK.hwnd.unwrap();

    HACK.wnd_proc = Some(*(&GetWindowLongPtrA(hwnd, GWL_WNDPROC) as *const _ as *const WNDPROC));

    // SetWindowLongPtrA(hwnd, GWL_WNDPROC, *(&wnd_proc_hook as *const _ as *const i32));

    println!("hwnd {:?}", HACK.hwnd);

    // Hook
    hack::start().expect("Hook failed.");

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

//             MoveWindow(hwnd, 50, 50, 1280, 720, 1);

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