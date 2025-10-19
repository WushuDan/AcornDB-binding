use std::{env, path::PathBuf};

fn main() {
    // Where is acorn.h?
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let header = root.join("..").join("c").join("acorn.h");

    println!("cargo:rerun-if-changed={}", header.display());

    let bindings = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("bindgen");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(&out)
        .expect("write bindings");

    // Link args: expect loader to find the shim at runtime; allow env override for build/test.
    // Only link if ACORN_SHIM_DIR is explicitly set
    if env::var("ACORN_SHIM_DIR").is_ok() {
        if let Ok(dir) = env::var("ACORN_SHIM_DIR") {
            println!("cargo:rustc-link-search=native={}", dir);
        }

        let libname = "acornshim";
        println!("cargo:rustc-link-lib=dylib={}", libname);
    }
}
