#[macro_use]
extern crate neon;
extern crate winapi;

use std::ffi::OsStr;
use std::ffi::CString;
use std::ptr::null_mut;
use std::mem;
use std::path::Path;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;

use neon::prelude::*;

use winapi::shared::minwindef::{DWORD,LPDWORD,LPVOID,FARPROC};
use winapi::shared::basetsd::SIZE_T;

use winapi::um::winbase::INFINITE;
use winapi::um::minwinbase::LPTHREAD_START_ROUTINE;

use winapi::um::winnt::{LPCSTR,LPSTR};
use winapi::um::winnt::{MEM_RESERVE,MEM_COMMIT,PAGE_EXECUTE_READWRITE};
use winapi::um::winnt::PROCESS_ALL_ACCESS;
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION,PROCESS_CREATE_THREAD,PROCESS_VM_OPERATION,PROCESS_VM_WRITE};

use winapi::um::winbase::CREATE_SUSPENDED;

use winapi::um::processthreadsapi::{CreateProcessA,ResumeThread,CreateRemoteThread,OpenProcess,GetExitCodeThread};
use winapi::um::processthreadsapi::{STARTUPINFOA,PROCESS_INFORMATION};

use winapi::um::memoryapi::{VirtualAllocEx,WriteProcessMemory};

use winapi::um::libloaderapi::{GetProcAddress,GetModuleHandleA};

use winapi::um::synchapi::WaitForSingleObject;

use winapi::um::errhandlingapi::GetLastError;

fn launch_fable(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // Arguments
    let fable_executable_str = cx.argument::<JsString>(0)?.value();
    let fable_executable = fable_executable_str.as_ptr() as LPSTR;

    let dll_path_str = cx.argument::<JsString>(1)?.value();
    let dll_path = CString::new(dll_path_str).unwrap();
    let dll_path_size = dll_path.to_bytes_with_nul().len();

    // Returned values
    let mut process_info: PROCESS_INFORMATION = Default::default();
    let mut startup_info: STARTUPINFOA = Default::default();

    startup_info.cb = mem::size_of::<STARTUPINFOA>() as u32;

    // Create Fable process
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

    let hprocess = process_info.hProcess;

    // Allocate area for DLL in process's memory.
    let dll_path_in_remote = unsafe {
        VirtualAllocEx(
            hprocess,
            null_mut(),
            dll_path_size,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_EXECUTE_READWRITE
        )
    };

    if dll_path_in_remote.is_null() {
        panic!("Failed to allocate memory in remote process.");
    }

    // Write DLL to process's memory.
    if unsafe {
        WriteProcessMemory(
            hprocess,
            dll_path_in_remote,
            dll_path.as_ptr() as LPVOID,
            dll_path_size,
            null_mut(),
        )
    } == 0 {
        panic!("Failed to write DLL to remote process's memory.");
    };

    println!("Wrote");

    // Get LoadLibrary's address for any process.
    let load_library_address = unsafe {
        GetProcAddress(
            GetModuleHandleA(CString::new("kernel32.dll").unwrap().as_ptr() as LPCSTR),
            CString::new("LoadLibraryA").unwrap().as_ptr() as LPCSTR
        )
    };

    if load_library_address.is_null() {
        panic!("Could not find LoadLibraryA in kernel32.dll");
    }

    println!("LoadLibraryA {:?}", load_library_address);

    // Call LoadLibrary in remote process.
    let remote_thread = unsafe {
        CreateRemoteThread(
            hprocess,
            null_mut(),
            0,
            Some(mem::transmute::<FARPROC, unsafe extern "system" fn(lpThreadParameter: LPVOID) -> DWORD>(load_library_address)),
            dll_path_in_remote as LPVOID,
            0,
            null_mut()
        )
    };

    unsafe { WaitForSingleObject(remote_thread, INFINITE) };

    let mut exit_code: DWORD = 0;

    unsafe { GetExitCodeThread(remote_thread, &mut exit_code); }

    if exit_code == 0 {
        println!("Failed with {}", unsafe { GetLastError() });
        panic!("Failed to CreateRemoteThread");
    }

    println!("Injected (exit code {})", exit_code);

    // Resume Fable
    if unsafe { ResumeThread(process_info.hThread) } == 0 {
        panic!("Failed to resume the main thread.")
    };

    println!("Resumed");

    Ok(JsUndefined::new())
}

register_module!(mut cx, {
    cx.export_function("launch_fable", launch_fable)
});