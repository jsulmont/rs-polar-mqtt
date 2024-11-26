use std::env;
use std::path::PathBuf;

fn main() {
    // Configure CMake build (we use shared libraries).
    let dst = cmake::Config::new("cpp")
        .generator("Unix Makefiles")
        .build_target("all")
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("BUILD_SHARED_LIBS", "ON")
        .very_verbose(true)
        .build();

    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib64").display()
    );

    println!("cargo:rustc-link-lib=dylib=polar_mqtt_impl");
    println!("cargo:rustc-link-lib=dylib=polar_mqtt_bridge");

    // Link C++ standard library
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("cpp/bridge/include/mqtt_c.hpp")
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++17")
        .allowlist_file("cpp/bridge/include/mqtt_c.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Rebuild triggers
    println!("cargo:rerun-if-changed=cpp/impl");
    println!("cargo:rerun-if-changed=cpp/bridge");
    println!("cargo:rerun-if-changed=cpp/api");
    println!("cargo:rerun-if-changed=cpp/CMakeLists.txt");
}
