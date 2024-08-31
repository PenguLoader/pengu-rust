use crate::{cef::*, define_hook, define_proxy};
use std::os::raw::*;

//# BROWSER PROCESS ENTRY

define_proxy! {
    extern "C" fn OnBeforeCommandLineProcessing(callme,
        self_: *mut _cef_app_t,
        process_type: *const cef_string_t,
        command_line: *mut _cef_command_line_t,
    ) -> () {
        _ = msgbox::create("pengu.rs", "processing command line", msgbox::IconType::Info);
        callme(self_, process_type, command_line)
    }
}

define_hook! {
    extern "C" fn CefCreateBrowser(callme,
        window_info: *const cef_window_info_t,
        client: *mut _cef_client_t,
        url: *const cef_string_t,
        settings: *const _cef_browser_settings_t,
        extra_info: *mut _cef_dictionary_value_t,
        request_context: *mut _cef_request_context_t
    ) -> c_int {
        _ = msgbox::create("pengu.rs", "creating browser", msgbox::IconType::Info);
        callme(window_info, client, url, settings, extra_info, request_context)
    }
}

define_hook! {
    extern "C" fn CefInitialize(callme,
        args: *const cef_main_args_t,
        settings: *const cef_settings_t,
        app: *mut cef_app_t,
        windows_sandbox_info: *mut c_void
    ) -> c_int {
        _ = msgbox::create("pengu.rs", "initializing cef", msgbox::IconType::Info);

        // hook method
        OnBeforeCommandLineProcessing::proxy(&mut (*app).on_before_command_line_processing);

        callme(args, settings, app, windows_sandbox_info)
    }
}

pub fn init() {
    CefInitialize::install("libcef.dll\0", "cef_initialize\0");
    CefCreateBrowser::install("libcef.dll\0", "cef_browser_host_create_browser\0");
}
