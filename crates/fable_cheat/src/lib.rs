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

// use winapi::shared::basetsd::LONG_PTR;
// use winapi::shared::minwindef::{BOOL,DWORD,LPARAM,LPDWORD,WPARAM,LRESULT};
// use winapi::shared::windef::HWND;

// use winapi::um::winnt::{LPCSTR,LPSTR,PROCESS_ALL_ACCESS};

// use winapi::um::winuser::{EnumWindows,GetWindowThreadProcessId,GetWindowLongA,SetWindowPos,GetWindowLongPtrA,SetWindowLongPtrA,DefWindowProcA};
// use winapi::um::winuser::{WNDPROC};
// use winapi::um::winuser::{GWL_STYLE,GWL_WNDPROC,HWND_NOTOPMOST,SWP_FRAMECHANGED,SWP_SHOWWINDOW,WS_MINIMIZEBOX,WS_OVERLAPPEDWINDOW,WS_MAXIMIZEBOX,WS_CAPTION,WS_BORDER,WS_SIZEBOX};

// #[derive(Debug)]
// struct FableWindowSearch {
//     process_id: DWORD,
//     hwnd: HWND
// }

// static mut fable_wnd_proc: WNDPROC = None;

#[no_mangle]
extern "system" fn DllMain(dll_handle: HINSTANCE, fdv_reason: DWORD, lpv_reserved: LPVOID) -> BOOL {
    match fdv_reason {
        DLL_PROCESS_ATTACH => {
            unsafe { AllocConsole() };

            // Fable window search

            // let mut fable_window_search = FableWindowSearch {
            //     process_id: process_info.dwProcessId,
            //     hwnd: null_mut(),
            // };

            // while fable_window_search.hwnd == null_mut() {
            //     unsafe { EnumWindows(Some(find_fable_window), &mut fable_window_search as *mut FableWindowSearch as LPARAM) };
            // }

            // println!("found window {:?}", fable_window_search);

            // // error[E0133]: use of mutable static is unsafe and requires unsafe function or block
            // // note: mutable statics can be mutated by multiple threads: aliasing violations or data races will cause undefined
            // unsafe {
            //     fable_wnd_proc = mem::transmute::<LONG_PTR, WNDPROC>(GetWindowLongPtrA(fable_window_search.hwnd, GWL_WNDPROC));
            // }

            // unsafe {
            //     SetWindowLongPtrA(fable_window_search.hwnd, GWL_WNDPROC, mem::transmute::<WNDPROC, LONG_PTR>(Some(wnd_proc_hook)));
            // }

            // Alloc console in thread

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

// Fable window search callbacks

// extern "system" fn find_fable_window(hwnd: HWND, search: LPARAM) -> BOOL {
//     let mut search = unsafe { &mut *(search as *mut FableWindowSearch) };

//     let mut process_id = 1 as DWORD;

//     unsafe { GetWindowThreadProcessId(hwnd, &mut process_id as LPDWORD) };

//     if process_id == search.process_id {
//         search.hwnd = hwnd;
//         0 as BOOL
//     } else {
//         1 as BOOL
//     }
// }

// unsafe extern "system" fn wnd_proc_hook(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
//     match fable_wnd_proc {
//         Some(wnd_proc) => {
//             println!("forwarded wndproc");
//             wnd_proc(hwnd, msg, wparam, lparam)
//         }
//         None => {
//             println!("system wndproc");
//             DefWindowProcA(hwnd, msg, wparam, lparam)
//         }
//     }
// }

// Alloc console in thread callback

// extern "system" fn init(lpThreadParameter: LPVOID) -> DWORD {
//     unsafe { AllocConsole() };
//     0
// }