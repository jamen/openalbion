// use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::*;
use winapi::um::*;

use winapi::shared::winerror::*;

use processthreadsapi::*;
use tlhelp32::*;

use std::ptr::null_mut;
use std::ffi::{CString,CStr};
use std::mem;
use std::path::Path;
use std::num::ParseIntError;

use clap::{App,Arg};

#[derive(Debug)]
pub struct Inject<'a> {
    pub target: InjectTarget<'a>,
    pub dll: Option<&'a str>,
}

#[derive(Debug)]
pub enum InjectTarget<'a> {
    Create(&'a str),
    Pid(&'a str),
    Find(&'a str),
}

#[derive(Debug)]
pub enum InjectError {
    ParseIntError,
    CreateProcessError,
    InvalidSnapshotHandle,
    ProcessNotFound,
    RemoteAllocError,
    RemoteWriteError,
    RemoteExitCodeError,
}

impl From<ParseIntError> for InjectError {
    fn from(_: ParseIntError) -> Self {
        InjectError::ParseIntError
    }
}

impl Inject<'_> {
    pub fn start(&self) -> Result<u32, InjectError> {

        let pid =
            match self.target {
                InjectTarget::Create(exe) => Self::create_process(&exe)?,
                InjectTarget::Find(exe) => Self::find_pid(exe)?,
                InjectTarget::Pid(pid_str) => pid_str.parse::<u32>()?,
            };

        let dll = self.dll.unwrap_or("defable_hack.dll");

        Self::inject(pid, &dll)?;

        Ok(pid)
    }

    pub fn create_process(exe: &str) -> Result<u32, InjectError> {
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
            return Err(InjectError::CreateProcessError)
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

    pub fn find_pid(exe: impl AsRef<Path>) -> Result<u32, InjectError> {
        let exe = exe.as_ref();
        let exe_name = exe.file_name().unwrap().to_str().unwrap();
        let snapshot_handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };

        if snapshot_handle == handleapi::INVALID_HANDLE_VALUE {
            return Err(InjectError::InvalidSnapshotHandle)
        }

        let mut pid = 0;
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
                pid = process_entry.th32ProcessID;
                break
            }

            if unsafe { errhandlingapi::GetLastError() } == ERROR_NO_MORE_FILES {
                break
            }
        }

        if pid == 0 {
            return Err(InjectError::ProcessNotFound)
        }

        Ok(pid)
    }

    pub fn remote_call(process_handle: HANDLE, proc_name: &str, thread_param: impl AsRef<[u8]>) -> Result<u32, InjectError> {
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
            return Err(InjectError::RemoteAllocError)
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
            return Err(InjectError::RemoteWriteError)
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
            return Err(InjectError::RemoteExitCodeError)
        }

        Ok(ret_val)
    }

    pub fn inject(pid: u32, dll: &str) -> Result<(), InjectError> {
        let process_handle = unsafe { OpenProcess(winnt::PROCESS_ALL_ACCESS, 0, pid) };

        let dll_path = CString::new(dll).unwrap();
        let dll_path_bytes = dll_path.to_bytes_with_nul();

        Self::remote_call(process_handle, "LoadLibraryA", &dll_path_bytes)?;

        Ok(())
    }
}

fn main() -> Result<(), u32> {
    let matches =
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
        .get_matches();

    let method =
        if let Some(value) = matches.value_of("create") { InjectTarget::Create(value) }
        else if let Some(value) = matches.value_of("pid") { InjectTarget::Pid(value) }
        else if let Some(value) = matches.value_of("find") { InjectTarget::Find(value) }
        else { InjectTarget::Find("Fable.exe") };

    let injector = Inject {
        target: method,
        dll: matches.value_of("dll"),
    };

    injector.start().unwrap();

    Ok(())
}