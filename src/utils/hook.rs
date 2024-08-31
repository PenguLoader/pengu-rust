use libc::c_void;
use std::mem::transmute;
use std::ptr::null_mut;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use winapi::um::{memoryapi::VirtualProtect, winnt::PAGE_EXECUTE_READWRITE};

const OPCODE_SIZE: usize = 12;

fn gen_code(addr: *mut c_void) -> Vec<u8> {
    let mut opcodes: Vec<u8> = vec![0; OPCODE_SIZE];
    // movabs rax [addr]
    opcodes[0] = 0x48;
    opcodes[1] = 0xB8;

    unsafe {
        let ptr_addr = opcodes.as_mut_ptr() as isize + 2;
        libc::memcpy(transmute(ptr_addr), transmute(&addr), size_of::<isize>());
    }

    // push rax
    opcodes[10] = 0x50;
    // ret
    opcodes[11] = 0xC3;

    opcodes
}

fn memcpy_safe(dst: *mut c_void, src: *const c_void, size: usize) -> bool {
    unsafe {
        let mut op = 0;
        let mut success = VirtualProtect(transmute(dst), size, PAGE_EXECUTE_READWRITE, &mut op);
        if success != 0 {
            libc::memcpy(dst, src, size);
            success = VirtualProtect(transmute(dst), size, op, &mut op);
        }
        return success != 0;
    }
}

pub struct SwapGuard {
    pub func: *mut c_void,
    pub backup: *mut u8,
    pub size: usize,
}

impl SwapGuard {
    pub fn new(func: *mut c_void, code: *const u8, size: usize) -> Self {
        unsafe {
            let backup = libc::calloc(1, size) as *mut u8;

            libc::memcpy(backup as *mut c_void, func, size);
            memcpy_safe(func, code as *mut c_void, size);

            Self { func, backup, size }
        }
    }

    pub fn swap(&self) -> SwapGuard {
        SwapGuard::new(self.func, self.backup, self.size)
    }
}

impl Drop for SwapGuard {
    fn drop(&mut self) {
        unsafe {
            memcpy_safe(self.func, self.backup as *const c_void, self.size);
            libc::free(self.backup as *mut c_void);
        }
    }
}

pub struct Hook {
    pub hook: *mut c_void,
    pub func: *mut c_void,
    pub backup: Option<SwapGuard>,
}

impl Hook {
    pub fn new(hook: *mut c_void) -> Self {
        Self {
            hook,
            func: null_mut(),
            backup: None,
        }
    }

    pub fn install(&mut self, lib: &str, func: &str) {
        unsafe {
            let lib = GetModuleHandleA(lib.as_ptr() as *const i8);
            let proc = GetProcAddress(lib, func.as_ptr() as *const i8);
            self.func = proc as *mut c_void;

            let opcodes = gen_code(self.hook);
            self.backup = Some(SwapGuard::new(self.func, opcodes.as_ptr(), opcodes.len()));
        }
    }

    pub fn uninstall(&mut self) {
        self.backup = None;
    }
}

// unsafe impl Sync for Hook {}
// unsafe impl Send for Hook {}

#[macro_export]
macro_rules! define_hook {
    { extern $conv:literal fn $name:ident ($self:ident, $($param_name:ident: $param_type:ty),* $(,)?) -> $ret_type:ty $body:block } => {
        #[allow(non_snake_case)]
        mod $name {
            use crate::utils::hook::Hook;
            use super::*;

            type Func = extern $conv fn ($($param_name: $param_type),*) ->  $ret_type;

            static MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
            static mut HOOK: Option<Hook> = None;

            pub fn install(lib: &str, func: &str) {
                unsafe {
                    let mut hook = Hook::new(std::mem::transmute(hook_fn as *mut libc::c_void));
                    hook.install(lib, func);
                    HOOK = Some(hook);
                }
            }

            extern $conv fn call($($param_name: $param_type),*) -> $ret_type {
                unsafe {
                    let _guard = MUTEX.try_lock().unwrap();
                    if let Some(hook) = HOOK.as_ref() {
                        if let Some(backup) = hook.backup.as_ref() {
                            let _swap = backup.swap();
                            {
                                let func: Func = std::mem::transmute(hook.func);
                                return func($($param_name),*);
                            }
                        }
                    }
                    panic!();
                }
            }

            extern $conv fn hook_fn($($param_name: $param_type),*) -> $ret_type {
                unsafe {
                    hook_fn2(call, $($param_name),*)
                }
            }

            unsafe extern $conv fn hook_fn2($self: Func, $($param_name: $param_type),*) -> $ret_type $body
        }
    };
}

#[macro_export]
macro_rules! define_proxy {
    { extern $conv:literal fn $name:ident ($self:ident, $($param_name:ident: $param_type:ty),* $(,)?) -> $ret_type:ty $body:block } => {
        #[allow(non_snake_case)]
        mod $name {
            use super::*;

            type Func = unsafe extern $conv fn ($($param_name: $param_type),*) ->  $ret_type;
            static mut FUNC: isize = 0;

            pub fn proxy(orig: &mut Option<Func>) {
                unsafe {
                    FUNC = std::mem::transmute(*orig);
                    *orig = Some(hook_fn);
                }
            }

            extern $conv fn hook_fn($($param_name: $param_type),*) -> $ret_type {
                unsafe {
                    let call: Func = std::mem::transmute(FUNC);
                    hook_fn2(call, $($param_name),*)
                }
            }

            #[allow(non_snake_case)]
            unsafe extern $conv fn hook_fn2($self: Func, $($param_name: $param_type),*) -> $ret_type $body
        }
    };
}
