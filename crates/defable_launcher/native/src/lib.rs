#[macro_use]
extern crate neon;
extern crate fable_cheat;
extern crate winapi;

use std::ffi::CString;
use std::ptr::null_mut;
use std::mem;

use neon::prelude::*;

use winapi::shared::basetsd::LONG_PTR;
use winapi::shared::minwindef::{BOOL,DWORD,LPARAM,LPDWORD};
use winapi::shared::windef::HWND;

use winapi::um::winbase::CREATE_SUSPENDED;

use winapi::um::processthreadsapi::{CreateProcessA,ResumeThread};
use winapi::um::processthreadsapi::{STARTUPINFOA,PROCESS_INFORMATION};

use winapi::um::winnt::{LPCSTR,LPSTR,PROCESS_ALL_ACCESS};

use winapi::um::winuser::{EnumWindows,GetWindowThreadProcessId,SetWindowLongPtrA,SetWindowPos,GetWindowLongPtrA};
use winapi::um::winuser::{GWL_STYLE,HWND_NOTOPMOST,SWP_FRAMECHANGED,SWP_SHOWWINDOW,WS_MINIMIZEBOX,WS_OVERLAPPEDWINDOW,WS_MAXIMIZEBOX,WS_CAPTION,WS_BORDER,WS_SIZEBOX};

#[derive(Debug)]
struct FableWindowSearch {
    process_id: DWORD,
    hwnd: HWND
}

fn launch_fable(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let fable_executable_original = cx.argument::<JsString>(0)?.value();
    let fable_executable = fable_executable_original.as_ptr() as LPSTR;

    let mut process_info: PROCESS_INFORMATION = Default::default();
    let mut startup_info: STARTUPINFOA = Default::default();

    startup_info.cb = mem::size_of::<STARTUPINFOA>() as u32;

    if unsafe {
        CreateProcessA(
            null_mut(),
            fable_executable,
            null_mut(),
            null_mut(),
            0,
            CREATE_SUSPENDED,
            null_mut(),
            null_mut(),
            &mut startup_info,
            &mut process_info,
        )
    } == 0 {
        panic!("Failed to execute the specified file.");
    }

    if unsafe { ResumeThread(process_info.hThread) } == 0 {
        panic!("Failed to resume the main thread.");
    }

    let mut fable_window_search = FableWindowSearch {
        process_id: process_info.dwProcessId,
        hwnd: null_mut(),
    };

    while fable_window_search.hwnd == null_mut() {
        unsafe { EnumWindows(Some(find_fable_window), &mut fable_window_search as *mut FableWindowSearch as LPARAM) };
    }

    println!("found window {:?}", fable_window_search);

    let styles = unsafe { GetWindowLongPtrA(fable_window_search.hwnd, GWL_STYLE as i32) as u32 };

    loop {
        if unsafe { SetWindowLongPtrA(fable_window_search.hwnd, GWL_STYLE, (styles |
         WS_OVERLAPPEDWINDOW | WS_BORDER) as LONG_PTR) } == 0 {
            panic!("SetWindowLongPtrA failed");
        }

        if unsafe {
            SetWindowPos(fable_window_search.hwnd, HWND_NOTOPMOST, 50, 50, 1980, 720, SWP_FRAMECHANGED | SWP_SHOWWINDOW)
        } == 0 {
            panic!("SetWindowPos failed");
        };
    }

    Ok(JsUndefined::new())
}

extern "system" fn find_fable_window(hwnd: HWND, search: LPARAM) -> BOOL {
    let mut search = unsafe { &mut *(search as *mut FableWindowSearch) };

    let mut process_id = 1 as DWORD;

    unsafe { GetWindowThreadProcessId(hwnd, &mut process_id as LPDWORD) };

    if process_id == search.process_id {
        search.hwnd = hwnd;
        0 as BOOL
    } else {
        1 as BOOL
    }
}

register_module!(mut cx, {
    cx.export_function("launch_fable", launch_fable)
});
