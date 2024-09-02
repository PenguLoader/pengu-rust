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

impl cef_string_t {
    pub fn new(s: &str) -> Self {
        let mut utf16: Vec<u16> = s.encode_utf16().collect();
        let len = utf16.len();
        // null terminated
        utf16.push(0);

        // allocate memory
        let ptr = Box::into_raw(utf16.into_boxed_slice()) as *mut u16;

        cef_string_t {
            str_: ptr,
            length: len,
            dtor: Some(Self::string_dtor),
        }
    }

    // custom destructor for the cef_string_t
    extern "C" fn string_dtor(str: *mut u16) {
        if !str.is_null() {
            unsafe {
                // recreate the Box and let it drop, freeing the memory
                let _ = Box::from_raw(str);
            }
        }
    }
}

impl Clone for cef_string_t {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = Box::into_raw(Box::from_raw(self.str_).clone());
            Self {
                str_: ptr,
                length: self.length,
                dtor: Some(Self::string_dtor),
            }
        }
    }
}

impl Drop for cef_string_t {
    fn drop(&mut self) {
        if let Some(dtor) = self.dtor {
            unsafe {
                dtor(self.str_);
                self.dtor = None;
            }
        }
    }
}
