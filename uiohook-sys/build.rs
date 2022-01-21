use std::{env, path::PathBuf};
use bindgen::EnumVariation;

use cmake;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.c");
    println!("cargo:rerun-if-changed=wrapper.h");

    let uihook_dst = cmake::build("libuiohook");
    cc::Build::new().file("wrapper.c").compile("wrapper");

    println!(
        "cargo:rustc-link-search=native={}",
        uihook_dst.join("lib").display()
    );
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=static=uiohook");
    println!("cargo:rustc-link-lib=static=wrapper");
    println!("cargo:include={}", uihook_dst.join("include").display());
    println!("cargo:lib={}", uihook_dst.join("lib").display());
    println!("cargo:root={}", uihook_dst.display());

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_default(true)
        .derive_debug(true)
        .rustfmt_bindings(true)
        .default_enum_style(EnumVariation::Rust { non_exhaustive: false })
        .generate()
        .expect("Unable to generate bindings.");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Could not save bindings.")
}
