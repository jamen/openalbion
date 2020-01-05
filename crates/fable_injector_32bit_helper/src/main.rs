use winapi::um::*;

use std::ffi::CString;
use std::io::{self, Write};

fn main() {
    let mut stdout = io::stdout();

    let module_name = CString::new("kernel32.dll").unwrap();
    let proc_name = CString::new("LoadLibraryA").unwrap();
    let module_handle = unsafe { libloaderapi::GetModuleHandleA(module_name.as_ptr()) };

    let load_library_addr = unsafe { libloaderapi::GetProcAddress(module_handle, proc_name.as_ptr()) };

    // println!("{:p}", load_library_addr);

    let load_library_addr_bytes = (load_library_addr as i32).to_le_bytes();

    stdout.write_all(&load_library_addr_bytes).unwrap();
}