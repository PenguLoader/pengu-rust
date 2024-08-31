use std::{
    ffi::{c_void, OsString}, mem::{transmute, MaybeUninit}, os::windows::ffi::OsStringExt, ptr::null_mut
};
use winapi::{
    shared::ntdef::HANDLE,
    um::{
        errhandlingapi::GetLastError,
        handleapi::CloseHandle,
        libloaderapi::{GetModuleFileNameW, GetModuleHandleExA},
        memoryapi::{VirtualAllocEx, WriteProcessMemory},
        processthreadsapi::*,
        synchapi::WaitForSingleObject,
        winbase::*,
        winnt::{MEM_COMMIT, PAGE_READWRITE},
    },
};

use crate::define_hook;

#[link(name = "ntdll")]
extern "system" {
    fn NtQueryInformationProcess(handle: isize, n: u32, hdbg: *mut isize, _: u32, _: isize) -> i32;
    fn NtRemoveProcessDebug(h: isize, hdbg: isize) -> i32;
    fn NtClose(hdbg: isize) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn LoadLibraryW();
}

unsafe fn get_this_dll_path(path_buf: &mut [u16]) -> usize {
    let mut module = null_mut();
    let ptr = get_this_dll_path as *const c_void;
    GetModuleHandleExA(4 | 2, transmute(ptr), &mut module);
    return GetModuleFileNameW(module, path_buf.as_mut_ptr(), path_buf.len() as u32) as usize;
}

unsafe fn inject_this_dll(proc: HANDLE) {
    let mut path_buf = [0 as u16; 2048];
    let path_len = get_this_dll_path(&mut path_buf);

    let path_size = (path_len + 1) * size_of::<u16>();
    let path_addr = VirtualAllocEx(proc, null_mut(), path_size, MEM_COMMIT, PAGE_READWRITE);
    WriteProcessMemory(
        proc,
        path_addr,
        transmute(path_buf.as_mut_ptr()),
        path_size,
        null_mut(),
    );

    let loader = CreateRemoteThread(
        proc,
        null_mut(),
        0,
        transmute(LoadLibraryW as *const c_void),
        path_addr,
        0,
        null_mut(),
    );

    WaitForSingleObject(loader, INFINITE);
    CloseHandle(loader);
}

#[no_mangle]
pub unsafe extern "system" fn BootstrapEntry(
    _hwnd: isize,
    _hinst: isize,
    command_line: *mut u16,
    _nshow: i32,
) -> i32 {
    let mut si = MaybeUninit::<STARTUPINFOW>::zeroed();
    let mut pi = MaybeUninit::<PROCESS_INFORMATION>::zeroed();

    (*si.as_mut_ptr()).cb = size_of::<STARTUPINFOW>() as u32;

    let result = CreateProcessW(
        null_mut(),
        command_line,
        null_mut(),
        null_mut(),
        0,
        CREATE_SUSPENDED | DEBUG_ONLY_THIS_PROCESS,
        null_mut(),
        null_mut(),
        si.as_mut_ptr(),
        pi.as_mut_ptr(),
    );

    if result == 0 {
        msgbox::create(
            "Pengu Bootstrapper",
            &format!(
                "Failed to create LeagueClientUx process, last error: {}",
                GetLastError()
            ),
            msgbox::IconType::Error,
        )
        .unwrap();
        -1
    } else {
        let mut hdbg: isize = 0;
        let pi2 = *pi.as_mut_ptr();

        if NtQueryInformationProcess(
            pi2.hProcess as isize,
            30,
            &mut hdbg,
            size_of::<isize>() as u32,
            0,
        ) >= 0
        {
            NtRemoveProcessDebug(pi2.hProcess as isize, hdbg);
            NtClose(hdbg);
        }

        inject_this_dll(pi2.hProcess);

        ResumeThread(pi2.hThread);
        WaitForSingleObject(pi2.hProcess, INFINITE);

        CloseHandle(pi2.hThread);
        CloseHandle(pi2.hProcess);
        0
    }
}

define_hook! {
    extern "system" fn CreateProcessW_Hook(callme,
        lpApplicationName: *const u16,
        lpCommandLine: *const u16,
        lpProcessAttributes: isize,
        lpThreadAttributes: isize,
        bInheritHandles: i32,
        dwCreationFlags: u32,
        lpEnvironment: isize,
        lpCurrentDirectory: *const u16,
        lpStartupInfo: LPSTARTUPINFOW,
        lpProcessInformation: LPPROCESS_INFORMATION,
    ) -> i32 {
        let length = (0..).take_while(|&i| *lpCommandLine.offset(i) != 0).count();
        let slice = std::slice::from_raw_parts(lpCommandLine, length);
        let command_line: OsString = OsStringExt::from_wide(slice);
        let command_line = command_line.to_string_lossy().into_owned().to_lowercase();

        let is_renderer = command_line.contains("leagueclientuxrender.exe")
            && command_line.contains("--type=renderer");

        let mut dwCreationFlags: u32 = dwCreationFlags;
        if is_renderer {
            dwCreationFlags |= CREATE_SUSPENDED;
        }

        let ret = callme(lpApplicationName, lpCommandLine, lpProcessAttributes, lpThreadAttributes,
            bInheritHandles, dwCreationFlags, lpEnvironment, lpCurrentDirectory, lpStartupInfo, lpProcessInformation);

        if ret != 0 && is_renderer {
            inject_this_dll((*lpProcessInformation).hProcess);
            ResumeThread((*lpProcessInformation).hThread);
        }

        return ret;
    }
}

/// Called when DLL is loaded in process.
fn main() {
    // get exe name
    let exe = std::env::current_exe()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_lowercase();

    // browser process
    if exe == "leagueclientux.exe" {
        super::browser::init();
        CreateProcessW_Hook::install("kernel32.dll\0", "CreateProcessW\0");

    // renderer process
    } else if exe == "leagueclientuxrender.exe" {
        let args: Vec<String> = std::env::args().collect();
        // v8 renderer
        if args.iter().find(|a| *a == "--type=renderer").is_some() {
            super::renderer::init();
        }
    }
}

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: isize, reason: u32, _reserved: isize) -> u32 {
    if reason == 1 {
        main();
    }
    return 1;
}