use clap::{App,Arg};

// use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::*;
use winapi::um::*;

use processthreadsapi::*;

use std::ptr::null_mut;
use std::ffi::CString;
use std::mem;

fn main() -> Result<(), u32> {
    let matches = App::new("defable")
        .version("0.1.0")
        .about("Fable mod tool.")
        .author("Jamen Marz <me@jamen.dev>")
        .arg(
            Arg::with_name("fable")
            .required(true)
            .help("Fable's executable.")
        )
        .arg(
            Arg::with_name("dll")
            .required(false)
            .help("DLL thats injected.")
        )
        .get_matches();

    let executable_path = matches.value_of("fable").unwrap();
    let dll_path = matches.value_of("dll").unwrap();

    let executable_path = CString::new(executable_path).unwrap();

    let mut process_info: PROCESS_INFORMATION = Default::default();
    let mut startup_info: STARTUPINFOA = Default::default();

    startup_info.cb = mem::size_of::<STARTUPINFOA>() as u32;

    if unsafe {
        CreateProcessA(
            null_mut(),
            executable_path.as_ptr() as LPSTR,
            null_mut(),
            null_mut(),
            0,
            0, // winbase::CREATE_SUSPENDED,
            null_mut(),
            null_mut(),
            &mut startup_info,
            &mut process_info,
        )
    } == 0 {
        return Err(1)
    }

    let process_handle = process_info.hProcess;
    let thread_handle = process_info.hThread;

    let dll_path = CString::new(dll_path).unwrap();
    let dll_path_size = dll_path.to_bytes_with_nul().len();

    let dll_path_in_remote = unsafe {
        memoryapi::VirtualAllocEx(
            process_handle,
            null_mut(),
            dll_path_size,
            winnt::MEM_RESERVE | winnt::MEM_COMMIT,
            winnt::PAGE_EXECUTE_READWRITE
        )
    };

    if dll_path_in_remote.is_null() {
        return Err(4)
    }

    if unsafe {
        memoryapi::WriteProcessMemory(
            process_handle,
            dll_path_in_remote,
            dll_path.as_ptr() as LPVOID,
            dll_path_size,
            null_mut()
        )
    } == 0 {
        return Err(5)
    }

    let module_name = CString::new("kernel32.dll").unwrap();
    let proc_name = CString::new("LoadLibraryA").unwrap();
    let module_handle = unsafe { libloaderapi::GetModuleHandleA(module_name.as_ptr()) };

    let load_library_in_remote = unsafe { libloaderapi::GetProcAddress(module_handle, proc_name.as_ptr()) };

    let remote_thread: HANDLE = unsafe {
        CreateRemoteThread(
            process_handle,
            null_mut(),
            0,
            Some(*(&load_library_in_remote as *const _ as *const unsafe extern "system" fn(LPVOID) -> DWORD)),
            // Some(mem::transmute::<FARPROC, unsafe extern "system" fn(LPVOID) -> DWORD>(load_library_in_remote)),
            dll_path_in_remote,
            0,
            null_mut(),
        )
    };

    unsafe {
        synchapi::WaitForSingleObject(remote_thread, winbase::INFINITE)
    };

    // unsafe { processthreadsapi::ResumeThread(thread_handle) };

    // NOTE: Maybe this always returns zero?

    // let mut remote_thread_exit_code = 0;

    // unsafe {
    //     GetExitCodeThread(remote_thread, &mut remote_thread_exit_code)
    // };

    // if remote_thread_exit_code == 0 {
    //     panic!("Remote thread exited with code {}.", remote_thread_exit_code);
    // }

    if !process_handle.is_null() {
        unsafe { handleapi::CloseHandle(process_handle) };
    }

    if !thread_handle.is_null() {
        unsafe { handleapi::CloseHandle(thread_handle) };
    }

    Ok(())
}
