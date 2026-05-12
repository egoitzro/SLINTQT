use std::env;

fn main() {
    slint_build::compile("ui/cute_main_window.slint").unwrap();
    
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let include_dir = std::path::Path::new(&crate_dir).join("include");
    std::fs::create_dir_all(&include_dir).unwrap();
    
    cbindgen::Builder::new()
      .with_src(std::path::Path::new(&crate_dir).join("src").join("lib_ffi.rs"))
      .with_src(std::path::Path::new(&crate_dir).join("src").join("ffi").join("mod.rs"))
      .with_language(cbindgen::Language::Cxx)
      .with_namespace("slint_qt")
      .with_include_guard("SLINT_QT_H")
      .generate()
      .expect("Unable to generate bindings")
      .write_to_file("include/slint_qt.h");
      
    println!("cargo:rerun-if-changed=src/ffi/mod.rs");
    println!("cargo:rerun-if-changed=src/lib_ffi.rs");
}
// Trigger rebuild
