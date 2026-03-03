fn main() {
    // Embed the .ico into the Windows executable so Explorer, the taskbar,
    // and the Alt+Tab switcher all show the correct icon.
    // CARGO_CFG_TARGET_OS reflects the *target* OS, which is correct when
    // cross-compiling for Windows from Linux.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().expect("Failed to compile Windows resources");
    }

    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=src/icons/app.svg");
}
