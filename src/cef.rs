#![allow(clippy::all)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]
#![allow(unused)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]

include!(concat!(env!("OUT_DIR"), "/cef_bindings.rs"));

use std::{
    mem::MaybeUninit,
    os::{raw::c_void, windows::raw::HANDLE},
    ptr::null_mut,
    sync::atomic::{AtomicUsize, Ordering},
    usize,
};

use winapi::{
    shared::ntdef::LPCWSTR,
    um::{
        errhandlingapi::GetLastError,
        handleapi::CloseHandle,
        processthreadsapi::{CreateProcessW, ResumeThread, PROCESS_INFORMATION, STARTUPINFOW},
        synchapi::WaitForSingleObject,
        winbase::{CREATE_SUSPENDED, DEBUG_ONLY_THIS_PROCESS, INFINITE},
        winnt::LPCSTR,
        winuser::{MessageBoxA, MessageBoxW},
    },
};
