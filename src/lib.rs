mod browser;
mod cef;
mod renderer;
mod utils;

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
        browser::init();

    // renderer process
    } else if exe == "leagueclientuxrenderer.exe" {
        let args: Vec<String> = std::env::args().collect();
        // v8 renderer
        if args.iter().find(|a| *a == "--type=renderer").is_some() {
            renderer::init();
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
