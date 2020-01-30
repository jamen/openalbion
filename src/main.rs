use clap::{App,Arg,ArgMatches};

// use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::*;
use winapi::um::*;

use winapi::shared::winerror::*;

use processthreadsapi::*;
use tlhelp32::*;

use std::io::{Error,ErrorKind};
use std::ptr::null_mut;
use std::ffi::{CString,CStr};
use std::mem;
use std::path::Path;

fn remote_call(process_handle: HANDLE, proc_name: &str, thread_param: impl AsRef<[u8]>) -> Result<u32, Error> {
    let thread_param_bytes = thread_param.as_ref();

    let dll_path_in_remote = unsafe {
        memoryapi::VirtualAllocEx(
            process_handle,
            null_mut(),
            thread_param_bytes.len(),
            winnt::MEM_RESERVE | winnt::MEM_COMMIT,
            winnt::PAGE_EXECUTE_READWRITE
        )
    };

    if dll_path_in_remote.is_null() {
        return Err(Error::new(ErrorKind::AddrNotAvailable, "Failed to allocate memory for remote parameter."))
    }

    if unsafe {
        memoryapi::WriteProcessMemory(
            process_handle,
            dll_path_in_remote,
            thread_param_bytes.as_ptr() as LPVOID,
            thread_param_bytes.len(),
            null_mut()
        )
    } == 0 {
        return Err(Error::new(ErrorKind::WriteZero, "Failed to write the remote parameter."))
    }

    let module_name = CString::new("kernel32.dll").unwrap();
    let proc_name = CString::new(proc_name).unwrap();
    let module_handle = unsafe { libloaderapi::GetModuleHandleA(module_name.as_ptr()) };

    let remote_proc = unsafe { libloaderapi::GetProcAddress(module_handle, proc_name.as_ptr()) };

    let remote_thread: HANDLE = unsafe {
        CreateRemoteThread(
            process_handle,
            null_mut(),
            0,
            Some(*(&remote_proc as *const _ as *const unsafe extern "system" fn(LPVOID) -> DWORD)),
            dll_path_in_remote,
            0,
            null_mut(),
        )
    };

    unsafe {
        synchapi::WaitForSingleObject(remote_thread, winbase::INFINITE)
    };

    // NOTE: Stackoverflow comment says "GetExitCodeThread(t1, &retVal) and returned 4294967295 (retVal being a DWORD). The actual return value in the thread was -1. I just figured out the ints rebounded to negatives. Sigh... – Sefu Aug 17 '11 at 23:05"
    // TODO: Verify the return value somehow?

    let mut ret_val = 0;

    if unsafe { GetExitCodeThread(remote_thread, &mut ret_val) } == 0 {
        return Err(Error::new(ErrorKind::Other, "Failed to get remote thread's exit code."))
    }

    Ok(ret_val)
}

fn main() -> Result<(), u32> {
    let matches = parse_cli();

    let dll = matches.value_of("dll").unwrap_or("defable_hack.dll");

    // Create a process and use its PID, or use a PID supplied by command line, or find a PID by an executable path.
    let pid: u32 =
        match matches.value_of("exe") {
            Some(exe) => {
                create_process(&exe).expect("Failed to create process.")
            }
            None => {
                match matches.value_of("pid") {
                    Some(pid_arg) => {
                        pid_arg.parse::<u32>().expect("Invalid process ID.")
                    }
                    None => {
                        let exe = matches.value_of("find").unwrap_or("Fable.exe");
                        find_pid(exe).expect("Failed to find Fable.")
                    }
                }
            }
        };

    attach_hack(pid, &dll).expect("Failed to attach hack");

    Ok(())
}

fn parse_cli<'a>() -> ArgMatches<'a> {
    App::new("defable")
        .version("0.1.0")
        .about("Fable mod tool.")
        .author("Jamen Marz <me@jamen.dev>")
        .arg(
            Arg::with_name("exe")
            .long("exe")
            .help("Path to Fable's executable.")
            .conflicts_with_all(&["pid", "find"])
            .required(false)
            .takes_value(true)
        )
        .arg(
            Arg::with_name("pid")
            .long("pid")
            .help("Attach to Fable process by PID.")
            .conflicts_with_all(&["exe", "find"])
            .required(false)
            .takes_value(true)
        )
        .arg(
            Arg::with_name("find")
            .long("find")
            .help("Attempts to find ")
            .conflicts_with_all(&["exe", "pid"])
            .required(false)
            .takes_value(true)
            .default_value("Fable.exe")
        )
        .arg(
            Arg::with_name("dll")
            .long("dll")
            .required(false)
            .help("Path to DLL hack.")
            .takes_value(true)
        )
        .get_matches()
}

fn find_pid(exe: impl AsRef<Path>) -> Option<u32> {
    let exe = exe.as_ref();
    let exe_name = exe.file_name().unwrap().to_str().unwrap();
    let snapshot_handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };

    if snapshot_handle == handleapi::INVALID_HANDLE_VALUE {
        return None
    }

    let mut process_entry = PROCESSENTRY32::default();
    process_entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

    loop {
        if unsafe { Process32Next(snapshot_handle, &mut process_entry) } == 0 {
            break
        }

        let target_exe_name = unsafe { CStr::from_ptr(process_entry.szExeFile.as_ptr()) };
        let target_exe_name = target_exe_name.to_owned();

        if exe_name.as_bytes() == target_exe_name.as_bytes() {
            // TODO: Sanity check with QueryFullProcessImageNameA?
            return Some(process_entry.th32ProcessID)
        }

        if unsafe { errhandlingapi::GetLastError() } == ERROR_NO_MORE_FILES {
            break
        }
    }

    return None
}

fn create_process(exe: &str) -> Result<u32, u32> {
    let executable_path = CString::new(exe).unwrap();
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

    // unsafe { processthreadsapi::ResumeThread(thread_handle) };

    if !process_info.hProcess.is_null() {
        unsafe { handleapi::CloseHandle(process_info.hProcess) };
    }

    if !process_info.hThread.is_null() {
        unsafe { handleapi::CloseHandle(process_info.hThread) };
    }

    Ok(process_info.dwProcessId)
}

#[repr(C)]
struct FreeLibraryAndExitThreadArgs {
    module_handle: HMODULE,
    exit_code: u32,
}

fn detach_loaded_hack(process_handle: HANDLE, pid: u32, dll: &str) -> Result<(), u32> {
    // Use CreateToolhelp32Snapshot to get cheat module then remote call FreeLibraryAndExitThread.

    let short_dll = if dll.len() > MAX_PATH { &dll[..MAX_PATH] } else { dll };
    let snapshot_handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid) };
    let mut remote_module_handle = None;
    let mut module_entry = MODULEENTRY32::default();
    module_entry.dwSize = mem::size_of::<MODULEENTRY32>() as u32;


    loop {
        if unsafe { Module32Next(snapshot_handle, &mut module_entry) } == 0 {
            break
        }

        let target_module_path = unsafe { CStr::from_ptr(module_entry.szExePath.as_ptr()) };
        let target_module_path = target_module_path.to_owned();

        if short_dll.as_bytes() == target_module_path.as_bytes() {
            // TODO: Sanity check with QueryFullProcessImageNameA?
            remote_module_handle = Some(module_entry.hModule);
            break
        }

        if unsafe { errhandlingapi::GetLastError() } == ERROR_NO_MORE_FILES {
            break
        }
    }

    match remote_module_handle {
        Some(remote_module_handle) => {
            // TODO: Verify the return value of the thread.

            let free_and_exit_args = FreeLibraryAndExitThreadArgs {
                module_handle: remote_module_handle as HMODULE,
                exit_code: 0,
            };
            let free_and_exit_args_bytes = unsafe { mem::transmute::<FreeLibraryAndExitThreadArgs, [u8; 8]>(free_and_exit_args) };
            let r = remote_call(process_handle, "FreeLibraryAndExitThread", &free_and_exit_args_bytes).expect("Failed to remote call FreeLibraryAndExitThread.");

            println!("freelibrary return {}", r);
        }
        None => {
            // Module was not loaded or failed to match.
        }
    }

    Ok(())
}

fn attach_hack(pid: u32, dll: &str) -> Result<(), u32> {
    let process_handle = unsafe { OpenProcess(winnt::PROCESS_ALL_ACCESS, 0, pid) };

    detach_loaded_hack(process_handle, pid, &dll).expect("Failed to unload the hack thats running.");

    let dll_path = CString::new(dll).unwrap();
    let dll_path_bytes = dll_path.to_bytes_with_nul();

    remote_call(process_handle, "LoadLibraryA", &dll_path_bytes).expect("Failed to remote call LoadLibraryA");

    Ok(())
}