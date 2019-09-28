use winapi::shared::windef::HWND;
use winapi::shared::minwindef::{BOOL,LPARAM};
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::um::winnt::{HANDLE,PROCESS_ALL_ACCESS};
use winapi::um::winuser::{ShowWindow,EnumWindows,GetWindowThreadProcessId,WNDENUMPROC};
use winapi::um::tlhelp32::{PROCESSENTRY32,CreateToolhelp32Snapshot,TH32CS_SNAPPROCESS,Process32First,Process32Next};
use winapi::um::processthreadsapi::{OpenProcess,GetProcessId};
use winapi::um::handleapi::CloseHandle;
use winapi::um::errhandlingapi::{SetLastError,GetLastError};
use std::ffi::CString;
use std::thread::sleep;
use std::time;
use std::ptr::{null,null_mut};
use std::mem;

struct EnumData {
    dw_process_id: u32,
    fable_hwnd: HWND,
}

extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let mut enum_data = &mut lparam as &mut EnumData;

    let mut dw_process_id: u32 = 0;

    unsafe { GetWindowThreadProcessId(hwnd, &mut dw_process_id) };

    if (enum_data.dw_process_id == dw_process_id) {
        enum_data.fable_hwnd = hwnd;

        SetLastError(ERROR_SUCCESS);

        return 0;
    }

    return 1;
}

fn input() {
    let mut entry: PROCESSENTRY32 = PROCESSENTRY32::default();
    entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

    let snapshot: HANDLE = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    let mut fable_hwnd: HANDLE = null_mut();

    if unsafe { Process32First(snapshot, &mut entry) == 1 } {
        while unsafe { Process32Next(snapshot, &mut entry) == 1 } {
            let szExeFile: [u8; 260] = unsafe { mem::transmute(entry.szExeFile) };
            let szExeFile: String = String::from_utf8(szExeFile[0..9].to_vec()).unwrap();

            if szExeFile == "Fable.exe" {
                let hProcess: HANDLE = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, entry.th32ProcessID) };

                let mut enum_data = EnumData {
                    dw_process_id: hProcess,
                    fable_hwnd: null_mut(),
                };

                if (EnumWindows(Some(enum_proc), &mut enum_data as LPARAM) == 1 && GetLastError() == ERROR_SUCCESS) {
                    fable_hwnd = enum_data.fable_hwnd;
                }

                unsafe { CloseHandle(hProcess) };

                break
            }
        }

        unsafe { CloseHandle(snapshot) };
    }
    // let fable_wnd: HWND = 0xff0b86 as HWND;

    println!("fable_hwnd {:?}", fable_hwnd);

    // unsafe { ShowWindow(fable_wnd, 9); }

    // let mut press_amount = 50;

    // while press_amount > 0 {
    //     unsafe { SendMessageA(fable_wnd, WM_KEYDOWN, 0x57, 1); } // press W

    //     let duration = time::Duration::from_millis(1000);

    //     sleep(duration);

    //     unsafe { SendMessageA(fable_wnd, WM_KEYUP, 0x57, 1); } // release W

    //     press_amount -= 1;
    // }
}