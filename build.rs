// extern crate cc;
// extern crate bindgen;

// use std::env;
// use std::path::PathBuf;

fn main() {
//     println!("cargo:rerun-if-changed=src/_c/wrapper.h");

//     cc::Build::new()
//         .file("src/_c/lexer.c")
//         .include("src/_c/")
//         .flag("-Wno-missing-braces")
//         .flag("-Wno-type-limits")
//         .flag("-Wno-unused-label")
//         .compile("regex");

//     let bindings = bindgen::Builder::default()
//         .header("src/_c/wrapper.h")
//         // Tell cargo to invalidate the built crate whenever any of the
//         // included header files changed.
//         .parse_callbacks(Box::new(bindgen::CargoCallbacks))
//         .generate()
//         .expect("Unable to generate bindings");

//     let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
//     bindings
//         .write_to_file(out_path.join("bindings.rs"))
//         .expect("Couldn't write bindings!");
}