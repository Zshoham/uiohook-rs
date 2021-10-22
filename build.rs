fn main() {
    if cfg!(feature = "reserve") && (cfg!(target_os = "windows") || cfg!(target_os = "macos")) {
        println!("cargo:rustc-cfg=use_reserved")
    }
}
