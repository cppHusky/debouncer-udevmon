fn main(){
    println!("cargo:rustc-link-search=/path/to/lib");
    let bindings=bindgen::Builder::default()
        .header("debouncer.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");
    let out_path=std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");
}
