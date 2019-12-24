#[macro_use]
extern crate neon;
extern crate winapi;

use std::ffi::CString;
use std::ptr::null_mut;
use std::mem;
use std::path::Path;

use neon::prelude::*;

use winapi::shared::minwindef::{DWORD,LPDWORD,LPVOID,FARPROC};

use winapi::um::minwinbase::LPTHREAD_START_ROUTINE;

use winapi::um::winnt::{LPCSTR,LPSTR};
use winapi::um::winnt::{MEM_RESERVE,MEM_COMMIT,PAGE_EXECUTE_READWRITE};
use winapi::um::winnt::PROCESS_ALL_ACCESS;

use winapi::um::winbase::CREATE_SUSPENDED;

use winapi::um::processthreadsapi::{CreateProcessA,ResumeThread,CreateRemoteThread,OpenProcess};
use winapi::um::processthreadsapi::{STARTUPINFOA,PROCESS_INFORMATION};

use winapi::um::memoryapi::{VirtualAllocEx,WriteProcessMemory};
use winapi::um::libloaderapi::{GetProcAddress,GetModuleHandleA};

fn launch_fable(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let fable_executable_original = cx.argument::<JsString>(0)?.value();
    let fable_executable = fable_executable_original.as_ptr() as LPSTR;

    let dll_path_original = CString::new("C:\\Users\\Jamen\\repos\\defable\\target\\debug\\fable_cheat.dll").unwrap();
    let dll_path = dll_path_original.as_ptr() as LPCSTR;

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
        panic!("Failed to resume the main thread.")
    };

    let hprocess = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, process_info.dwProcessId) };

    // Put DLL path in Fable's memory.

    let dll_path_in_remote = unsafe {
        VirtualAllocEx(hprocess, null_mut(), dll_path_original.to_bytes_with_nul().len(), MEM_RESERVE | MEM_COMMIT, PAGE_EXECUTE_READWRITE)
    };

    println!("dll_path_in_remote {:?}", dll_path_in_remote);

    unsafe {
        WriteProcessMemory(hprocess, dll_path_in_remote, dll_path as LPVOID, dll_path_original.to_bytes_with_nul().len(), null_mut())
    };

    println!("wrote memory");

    let load_library_in_remote = unsafe {
        GetProcAddress(
            GetModuleHandleA(CString::new("kernel32.dll").unwrap().as_ptr() as LPCSTR),
            CString::new("LoadLibraryA").unwrap().as_ptr() as LPCSTR
        )
    };

    println!("load_library_in_remote {:?}", load_library_in_remote);

    std::thread::sleep(std::time::Duration::from_millis(30000));

    let remote_thread = unsafe {
        CreateRemoteThread(
            hprocess,
            null_mut(),
            0,
            Some(*(&load_library_in_remote as *const _ as *const unsafe extern "system" fn(LPVOID) -> DWORD)),
            dll_path_in_remote as LPVOID,
            0,
            null_mut()
        )
    };

    println!("remote_thread {:?}", remote_thread);

    Ok(JsUndefined::new())
}

register_module!(mut cx, {
    cx.export_function("launch_fable", launch_fable)
});