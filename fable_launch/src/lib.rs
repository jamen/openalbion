use std::io::Error;
use std::ffi::{OsStr,CString,c_void};
use std::iter::once;
use std::ptr::null_mut;
use std::mem;

use std::os::windows::ffi::OsStrExt;

use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::{DWORD,BOOL,FARPROC,LPVOID};
use winapi::shared::ntdef::{HANDLE,LPSTR,LPCSTR,LPWSTR};

use winapi::um::winnt::{MEM_COMMIT,MEM_RESERVE,PAGE_EXECUTE_READWRITE,PROCESS_ALL_ACCESS,MEM_RELEASE};
use winapi::um::winbase::INFINITE;
use winapi::um::winbase::CREATE_SUSPENDED;
use winapi::um::minwinbase::{PTHREAD_START_ROUTINE,LPTHREAD_START_ROUTINE};
use winapi::um::memoryapi::{VirtualAllocEx,VirtualFreeEx,WriteProcessMemory};
use winapi::um::processthreadsapi::{GetProcessId,OpenProcess,CreateProcessA,CreateRemoteThread,ResumeThread,STARTUPINFOA,PROCESS_INFORMATION};
use winapi::um::libloaderapi::{GetProcAddress,GetModuleHandleW};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::handleapi::CloseHandle;

// pub struct Cheat {
//     pub thread_handle: HANDLE
// }

pub fn inject_dll(process_id: &str, dll_path: &str) -> Result<(), u32> {
    let process_id = process_id.parse::<u32>().unwrap() as DWORD;

    println!("process_id {}", process_id);

    let process_handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, process_id) };

    let dll_path_str = CString::new(dll_path).unwrap();
    let dll_path_size = dll_path.len() + 1;

    println!("dll_path: {} {:?}", dll_path_size, dll_path);

    let dll_path_addr = unsafe {
        VirtualAllocEx(process_handle, null_mut(), dll_path_size, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE)
    };

    println!("dll_path_addr {:p}", dll_path_addr);

    let mut written: SIZE_T = 0;
    let written_ptr: *mut SIZE_T = &mut written as *mut SIZE_T;

    unsafe {
        WriteProcessMemory(process_handle, dll_path_addr, dll_path_str.as_ptr() as *const c_void, dll_path_size, written_ptr);
    };

    println!("bytes_written {}", written);

    let kernel32_dll_str = as_win_str("kernel32.dll");
    let kernel32_handle = unsafe { GetModuleHandleW(kernel32_dll_str) };

    println!("handle {:p}", kernel32_handle);

    let load_library_str = CString::new("LoadLibraryA").unwrap();
    let load_library_addr = unsafe { GetProcAddress(kernel32_handle, load_library_str.as_ptr()) };

    println!("load_library_addr {:p}", load_library_addr);

    let thread_handle = unsafe {
        CreateRemoteThread(
            process_handle,
            null_mut(),
            0,
            if load_library_addr.is_null() { None } else { Some(mem::transmute::<FARPROC, unsafe extern "system" fn(lpParameter: LPVOID) -> DWORD>(load_library_addr)) },
            dll_path_addr,
            0,
            null_mut()
        )
    };

    println!("thread_handle {:p}", thread_handle);

    unsafe {
        WaitForSingleObject(thread_handle, INFINITE);
        CloseHandle(thread_handle);
        // VirtualFreeEx(process_handle, dll_path_addr, dll_path_size, MEM_RELEASE);
    }

    let error_code = unsafe { GetLastError() };

    if error_code == 0 {
        Ok(())
    } else {
        Err(error_code)
    }
}

fn as_win_str(from: &str) -> *mut u16 {
    OsStr::new(from)
    .encode_wide()
    .chain(once(0))
    .collect::<Vec<u16>>()
    .as_mut_ptr()
}