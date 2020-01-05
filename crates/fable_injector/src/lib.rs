#[macro_use]
extern crate arrayref;

// use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::*;
use winapi::um::*;

use processthreadsapi::*;
use psapi::*;

use std::convert::TryInto;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use std::ffi::CString;
use std::ffi::OsStr;
use std::iter::once;
use std::mem;

use std::process::Command;

pub struct Injector {
    pub process_handle: HANDLE,
    pub thread_handle: HANDLE
}

impl Injector {
    pub fn create_process(executable_path: &str) -> Result<Self, u32> {
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
        // let process_handle = unsafe {
        //     handleapi::CloseHandle(process_info.hProcess);
        //     OpenProcess(winnt::PROCESS_ALL_ACCESS, 0, process_info.dwProcessId)
        // };

        Ok(
            Injector {
                process_handle: process_handle,
                thread_handle: process_info.hThread,
            }
        )
    }

    pub fn get_loadlibraryw_address<T: AsRef<OsStr>>(injector_helper: T) -> Result<u32, u32> {
        match Command::new(&injector_helper).output() {
            Ok(output) => {
                let address_bytes: [u8; 4] = array_ref![&output.stdout, 0, 4].clone();
                Ok(u32::from_le_bytes(address_bytes))
            }
            Err(error) => {
                eprintln!("Recieved errror {:?}", error);
                Err(3)
            }
        }
    }

    pub fn inject_dll<T: AsRef<OsStr>>(&mut self, dll_path: T, injector_helper: T) -> Result<(), u32> {
        let dll_path = CString::new(dll_path.as_ref().to_str().unwrap()).unwrap();
        let dll_path_size = dll_path.to_bytes_with_nul().len();
        // let dll_path: Vec<u16> = CString::new(&dll_path).encode_wide().chain(once(0)).collect();
        // let dll_path_size = (dll_path.len() + 1) * mem::size_of::<u16>();

        let dll_path_in_remote = unsafe {
            memoryapi::VirtualAllocEx(
                self.process_handle,
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
                self.process_handle,
                dll_path_in_remote,
                dll_path.as_ptr() as LPVOID,
                dll_path_size,
                null_mut()
            )
        } == 0 {
            return Err(5)
        }

        let load_library_in_remote = Self::get_loadlibraryw_address(injector_helper)?;

        let remote_thread: HANDLE = unsafe {
            CreateRemoteThread(
                self.process_handle,
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

        unsafe { processthreadsapi::ResumeThread(self.thread_handle) };

        // NOTE: Maybe this always returns zero?

        // let mut remote_thread_exit_code = 0;

        // unsafe {
        //     GetExitCodeThread(remote_thread, &mut remote_thread_exit_code)
        // };

        // if remote_thread_exit_code == 0 {
        //     panic!("Remote thread exited with code {}.", remote_thread_exit_code);
        // }

        if !self.process_handle.is_null() {
            unsafe { handleapi::CloseHandle(self.process_handle) };
        }

        if !self.thread_handle.is_null() {
            unsafe { handleapi::CloseHandle(self.thread_handle) };
        }

        Ok(())
    }
}