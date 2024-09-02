use crate::{cef::*, define_hook, define_proxy};
use std::{os::raw::*, ptr::null_mut};

//# RENDERER PROCESS

static mut IS_MAIN: bool = false;

define_proxy! {
    extern "C" fn OnBrowserCreated(callme,
        self_: *mut _cef_render_process_handler_t,
        browser: *mut _cef_browser_t,
        extra_info: *mut _cef_dictionary_value_t,
    ) -> () {
        if !extra_info.is_null() {
            // ensure the main browser
            if (*extra_info).has_key.unwrap()(extra_info, &"main".into()) != 0 {
                IS_MAIN = true;
            }
        }
        callme(self_, browser, extra_info)
    }
}

define_proxy! {
    extern "C" fn OnContextCreated(callme,
        self_: *mut _cef_render_process_handler_t,
        browser: *mut _cef_browser_t,
        frame: *mut _cef_frame_t,
        context: *mut _cef_v8context_t,
    ) -> () {
        // get frame url
        let url = (*frame).get_url.unwrap()(frame).drop_string();
        // check target browser frame
        if IS_MAIN && url.starts_with("https://riot:") && url.ends_with("/index.html") {
            // run javascript
            let script = "console.log('hello from pengu-rust!')";
            (*frame).execute_java_script.unwrap()(frame, &script.into(), null_mut(), 1);
        }
        callme(self_, browser, frame, context)
    }
}

define_proxy! {
    extern "C" fn GetRenderProcessHandler(callme,
        self_: *mut _cef_app_t,
    ) -> *mut _cef_render_process_handler_t {
        let handler = callme(self_);

        OnContextCreated::proxy(&mut (*handler).on_context_created);
        OnBrowserCreated::proxy(&mut (*handler).on_browser_created);

        handler
    }
}

define_hook! {
    extern "C" fn CefExecuteProcess(callme,
        args: *const _cef_main_args_t,
        app: *mut cef_app_t,
        windows_sandbox_info: *mut ::std::os::raw::c_void,
    ) -> c_int {
        GetRenderProcessHandler::proxy(&mut (*app).get_render_process_handler);
        callme(args, app, windows_sandbox_info)
    }
}

pub fn init() {
    CefExecuteProcess::install("libcef.dll\0", "cef_execute_process\0");
}
