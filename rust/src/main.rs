use std::env;
use std::ffi::CString;

// FFI declarations for Windows API functions
#[link(name = "kernel32")]
extern "system" {
    fn LoadLibraryA(lpFileName: *const i8) -> *mut std::ffi::c_void;
    fn GetProcAddress(hModule: *mut std::ffi::c_void, lpProcName: *const i8) -> *mut std::ffi::c_void;
    fn FreeLibrary(hModule: *mut std::ffi::c_void) -> i32;
    fn GetVolumeInformationA(
        lpRootPathName: *const i8,
        lpVolumeNameBuffer: *mut i8,
        nVolumeNameSize: u32,
        lpVolumeSerialNumber: *mut u32,
        lpMaximumComponentLength: *mut u32,
        lpFileSystemFlags: *mut u32,
        lpFileSystemNameBuffer: *mut i8,
        nFileSystemNameSize: u32,
    ) -> i32;
    fn CreateFileA(
        lpFileName: *const i8,
        dwDesiredAccess: u32,
        dwShareMode: u32,
        lpSecurityAttributes: *mut std::ffi::c_void,
        dwCreationDisposition: u32,
        dwFlagsAndAttributes: u32,
        hTemplateFile: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;
    fn WriteFile(
        hFile: *mut std::ffi::c_void,
        lpBuffer: *const std::ffi::c_void,
        nNumberOfBytesToWrite: u32,
        lpNumberOfBytesWritten: *mut u32,
        lpOverlapped: *mut std::ffi::c_void,
    ) -> i32;
    fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
    fn DeleteFileA(lpFileName: *const i8) -> i32;
    fn GetLastError() -> u32;
}

// Function pointer types matching the C signatures
type GetWriteProtectStatusFn = unsafe extern "system" fn(i8, *mut i32) -> i32;
type DisableReadOnlyFn = unsafe extern "system" fn(i8) -> i32;
type SectorWriteProtectFn = unsafe extern "system" fn(i8, *const u64, i32, *mut i32) -> i32;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Parse arguments - ensure exactly one valid argument or none
    let mut permanent = false;
    let mut temporary = false;
    let mut lunx = false;
    let mut clear_sectors = false;
    let mut apply = false;
    
    if args.len() > 2 || (args.len() == 2 && 
        args[1] != "--apply" && args[1] != "--temporary" && 
        args[1] != "--lunx" && args[1] != "--clear-sectors") {
        eprintln!("Usage: {} [--apply|--temporary|--lunx|--clear-sectors]", args[0]);
        std::process::exit(64);
    }
    
    if args.len() == 2 {
        permanent = args[1] == "--apply";
        temporary = args[1] == "--temporary";
        lunx = args[1] == "--lunx";
        clear_sectors = args[1] == "--clear-sectors";
        apply = true;
    }
    
    // Check that only one mode is specified (mutually exclusive)
    let modes_count = [permanent, temporary, lunx, clear_sectors].iter().filter(|&&x| x).count();
    if modes_count > 1 {
        eprintln!("Usage: {} [--apply|--temporary|--lunx|--clear-sectors]", args[0]);
        std::process::exit(64);
    }
    
    // Check volume label first
    check_volume_label();
    
    // Load the DLL - stub for now since we're building from scratch
    let dll_name = CString::new("uDiskDLL.dll").unwrap();
    let dll = unsafe { LoadLibraryA(dll_name.as_ptr()) };
    
    if dll.is_null() {
        eprintln!("LoadLibrary(uDiskDLL.dll) failed: {}", unsafe { GetLastError() });
        std::process::exit(3);
    }
    
    // Get function pointers - resolve all exports
    let get_status_fn = unsafe {
        let func_name = CString::new("GetWriteProtectStatusDLL").unwrap();
        let addr = GetProcAddress(dll, func_name.as_ptr());
        if addr.is_null() {
            eprintln!("Required Nexcopy exports were not found.");
            FreeLibrary(dll);
            std::process::exit(4);
        }
        std::mem::transmute::<*mut std::ffi::c_void, GetWriteProtectStatusFn>(addr)
    };
    
    let disable_readonly_fn = unsafe {
        let func_name = CString::new("DisableReadOnlyDLL").unwrap();
        let addr = GetProcAddress(dll, func_name.as_ptr());
        if addr.is_null() {
            eprintln!("Required Nexcopy exports were not found.");
            FreeLibrary(dll);
            std::process::exit(4);
        }
        std::mem::transmute::<*mut std::ffi::c_void, DisableReadOnlyFn>(addr)
    };
    
    let disable_temporary_fn = unsafe {
        let func_name = CString::new("DisableReadOnlyTemporarilyDLL").unwrap();
        let addr = GetProcAddress(dll, func_name.as_ptr());
        if addr.is_null() {
            eprintln!("Required Nexcopy exports were not found.");
            FreeLibrary(dll);
            std::process::exit(4);
        }
        std::mem::transmute::<*mut std::ffi::c_void, DisableReadOnlyFn>(addr)
    };
    
    let disable_lunx_fn = unsafe {
        let func_name = CString::new("DisableWriteProtectLunXDLL").unwrap();
        let addr = GetProcAddress(dll, func_name.as_ptr());
        if addr.is_null() {
            eprintln!("Required Nexcopy exports were not found.");
            FreeLibrary(dll);
            std::process::exit(4);
        }
        std::mem::transmute::<*mut std::ffi::c_void, DisableReadOnlyFn>(addr)
    };
    
    let sector_write_protect_fn = unsafe {
        let func_name = CString::new("SectorWriteProtectDLL").unwrap();
        let addr = GetProcAddress(dll, func_name.as_ptr());
        if addr.is_null() {
            eprintln!("Required Nexcopy exports were not found.");
            FreeLibrary(dll);
            std::process::exit(4);
        }
        std::mem::transmute::<*mut std::ffi::c_void, SectorWriteProtectFn>(addr)
    };
    
    // Probe first
    let mut status = -1;
    let probe_result = unsafe { get_status_fn('I' as i8, &mut status) };
    println!("GetWriteProtectStatusDLL returned {}; status={}", probe_result, status);
    
    // If no arguments, stop here
    if !apply {
        unsafe { FreeLibrary(dll); }
        println!("Probe only; no controller-setting change was requested.");
        std::process::exit(if probe_result != 0 { 0 } else { 5 });
    }
    
    // Check that probe succeeded before applying changes (C's refusal)
    if probe_result == 0 {
        eprintln!("Refusing write call because the Nexcopy status probe failed.");
        unsafe { FreeLibrary(dll); }
        std::process::exit(5);
    }
    
    // Apply changes based on argument
    let result = if clear_sectors {
        println!("Calling SectorWriteProtectDLL with an empty protection-range table ...");
        let mut detail: i32 = 0x7fffffff;
        let ret = unsafe { sector_write_protect_fn('I' as i8, std::ptr::null(), 0, &mut detail) };
        println!("SectorWriteProtectDLL returned {}; detail={}", ret, detail);
        ret
    } else if lunx {
        println!("Calling DisableWriteProtectLunXDLL for I: ...");
        let ret = unsafe { disable_lunx_fn('I' as i8) };
        println!("DisableWriteProtectLunXDLL returned {}", ret);
        ret
    } else if temporary {
        println!("Calling DisableReadOnlyTemporarilyDLL for I: ...");
        let ret = unsafe { disable_temporary_fn('I' as i8) };
        println!("DisableReadOnlyTemporarilyDLL returned {}", ret);
        ret
    } else {
        println!("Calling DisableReadOnlyDLL for I: ...");
        let ret = unsafe { disable_readonly_fn('I' as i8) };
        println!("DisableReadOnlyDLL returned {}", ret);
        ret
    };
    
    // Probe again
    status = -1;
    let probe_result = unsafe { get_status_fn('I' as i8, &mut status) };
    println!("Post-call GetWriteProtectStatusDLL returned {}; status={}", probe_result, status);
    
    // Test write operation (real)
    let write_test_passed = test_write('I');
    
    // Free library and return result
    unsafe { FreeLibrary(dll); }
    std::process::exit(if result != 0 {
        if write_test_passed { 0 } else { 7 }
    } else {
        6
    });
}

fn check_volume_label() {
    let root_path = "I:\\".to_string();
    let mut volume_name = [0u8; 260];
    let mut serial_number = 0u32;
    let mut max_component_length = 0u32;
    let mut file_system_flags = 0u32;
    let mut file_system_name = [0u8; 260];
    
    let root_cstring = CString::new(root_path).unwrap();
    let volume_name_ptr = volume_name.as_mut_ptr() as *mut i8;
    let file_system_name_ptr = file_system_name.as_mut_ptr() as *mut i8;
    
    unsafe {
        let result = GetVolumeInformationA(
            root_cstring.as_ptr(),
            volume_name_ptr,
            260,
            &mut serial_number,
            &mut max_component_length,
            &mut file_system_flags,
            file_system_name_ptr,
            260,
        );
        
        if result == 0 {
            eprintln!("GetVolumeInformation(I:\\) failed: {}", GetLastError());
            std::process::exit(2);
        }
        
        // Convert volume name to string for comparison
        let volume_str = std::ffi::CStr::from_ptr(volume_name_ptr).to_string_lossy();
        println!("Volume I:\\ label={} filesystem={}", volume_str, 
                 std::ffi::CStr::from_ptr(file_system_name_ptr).to_string_lossy());
        
        // Case insensitive comparison
        if volume_str.to_uppercase() != "CVN7291B".to_uppercase() {
            eprintln!("Refusing: expected volume label CVN7291B.");
            std::process::exit(2);
        }
    }
}

fn test_write(drive: char) -> bool {
    let path = format!("{}:\\__nexcopy_write_test.tmp", drive);
    let c_path = CString::new(path).unwrap();
    let payload = b"Nexcopy write test\r\n";
    unsafe {
        DeleteFileA(c_path.as_ptr());
        let file = CreateFileA(c_path.as_ptr(), 0x4000_0000, 0, std::ptr::null_mut(), 2, 0x80, std::ptr::null_mut());
        if file as isize == -1 {
            println!("Write test: FAILED opening file (Win32 error {})", GetLastError());
            return false;
        }
        let mut written: u32 = 0;
        let ok = WriteFile(file, payload.as_ptr() as *const _, payload.len() as u32, &mut written, std::ptr::null_mut()) != 0
                 && written as usize == payload.len();
        let error = if ok { 0 } else { GetLastError() };
        CloseHandle(file);
        if !ok {
            println!("Write test: FAILED writing file (Win32 error {})", error);
            DeleteFileA(c_path.as_ptr());
            return false;
        }
        println!("Write test: SUCCEEDED ({} bytes)", written);
        println!("Delete test: {}", if DeleteFileA(c_path.as_ptr()) != 0 { "SUCCEEDED" } else { "FAILED" });
    }
    true
}