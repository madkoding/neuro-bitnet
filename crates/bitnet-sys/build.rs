//! Build script for bitnet-sys
//!
//! Compiles bitnet.cpp from the vendor directory and generates Rust bindings.
//! Falls back gracefully if compilation fails, allowing subprocess backend to be used.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Locate vendor/BitNet directory
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let bitnet_dir = manifest_dir.parent().unwrap().parent().unwrap().join("vendor/BitNet");
    
    if !bitnet_dir.exists() {
        println!("cargo:warning=BitNet submodule not found at {}. Run: git submodule update --init --recursive", bitnet_dir.display());
        println!("cargo:rustc-cfg=bitnet_sys_failed");
        return;
    }

    let llama_cpp_dir = bitnet_dir.join("3rdparty/llama.cpp");
    if !llama_cpp_dir.exists() {
        println!("cargo:warning=llama.cpp submodule not initialized. Run: cd vendor/BitNet && git submodule update --init --recursive");
        println!("cargo:rustc-cfg=bitnet_sys_failed");
        return;
    }

    // Check if we can build (requires cmake and proper setup)
    let cmake_check = Command::new("cmake").arg("--version").output();
    if cmake_check.is_err() {
        println!("cargo:warning=cmake not found. Native bindings require cmake to build bitnet.cpp");
        println!("cargo:rustc-cfg=bitnet_sys_failed");
        return;
    }

    // Check for the required bitnet header files
    let bitnet_header = bitnet_dir.join("include/bitnet-lut-kernels.h");
    if !bitnet_header.exists() {
        println!("cargo:warning=BitNet LUT kernels header not found. The BitNet submodule may need proper setup.");
        println!("cargo:warning=Native FFI bindings unavailable - using subprocess backend instead.");
        println!("cargo:rustc-cfg=bitnet_sys_failed");
        return;
    }

    // Build configuration
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    
    // Configure cmake
    let mut config = cmake::Config::new(&bitnet_dir);
    
    config
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("LLAMA_BUILD_TESTS", "OFF")
        .define("LLAMA_BUILD_EXAMPLES", "OFF")
        .define("LLAMA_BUILD_SERVER", "OFF");

    // Architecture-specific BitNet optimizations
    match target_arch.as_str() {
        "aarch64" => {
            config.define("BITNET_ARM_TL1", "ON");
            println!("cargo:rustc-cfg=bitnet_tl1");
        }
        "x86_64" => {
            config.define("BITNET_X86_TL2", "ON");
            println!("cargo:rustc-cfg=bitnet_tl2");
        }
        _ => {
            println!("cargo:warning=Unknown architecture '{}', using generic kernels", target_arch);
        }
    }

    // CUDA support
    #[cfg(feature = "cuda")]
    {
        config.define("GGML_CUDA", "ON");
        println!("cargo:rustc-cfg=bitnet_cuda");
    }

    // Build bitnet.cpp - this may fail, which is okay
    println!("cargo:warning=Attempting to build bitnet.cpp from source...");
    
    // Use catch_unwind equivalent by checking result
    let build_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        config.build_target("llama").build()
    }));

    let dst = match build_result {
        Ok(path) => path,
        Err(_) => {
            println!("cargo:warning=Failed to build bitnet.cpp - cmake/compilation error.");
            println!("cargo:warning=Native FFI bindings unavailable - using subprocess backend instead.");
            println!("cargo:rustc-cfg=bitnet_sys_failed");
            return;
        }
    };

    // Link paths
    let build_dir = dst.join("build");
    
    // Find and link libraries
    let lib_search_paths = [
        build_dir.join("src"),
        build_dir.join("ggml/src"),
        build_dir.clone(),
    ];

    for path in &lib_search_paths {
        if path.exists() {
            println!("cargo:rustc-link-search=native={}", path.display());
        }
    }

    // Link the libraries
    println!("cargo:rustc-link-lib=static=llama");
    println!("cargo:rustc-link-lib=static=ggml");
    println!("cargo:rustc-link-lib=static=ggml-base");
    println!("cargo:rustc-link-lib=static=ggml-cpu");
    
    // System libraries
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=m");
        println!("cargo:rustc-link-lib=pthread");
    }

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=c++");
        println!("cargo:rustc-link-lib=framework=Accelerate");
    }

    // Generate bindings with bindgen
    let llama_h = llama_cpp_dir.join("include/llama.h");
    let ggml_h = llama_cpp_dir.join("ggml/include/ggml.h");
    
    if !llama_h.exists() || !ggml_h.exists() {
        println!("cargo:warning=Header files not found. Expected: {}, {}", llama_h.display(), ggml_h.display());
        println!("cargo:rustc-cfg=bitnet_sys_failed");
        return;
    }

    let wrapper_h = manifest_dir.join("wrapper.h");
    
    let bindings = bindgen::Builder::default()
        .header(wrapper_h.to_string_lossy())
        .clang_arg(format!("-I{}", llama_cpp_dir.join("include").display()))
        .clang_arg(format!("-I{}", llama_cpp_dir.join("ggml/include").display()))
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++17")
        // Allow llama.cpp and ggml types/functions
        .allowlist_function("llama_.*")
        .allowlist_function("ggml_.*")
        .allowlist_type("llama_.*")
        .allowlist_type("ggml_.*")
        .allowlist_var("LLAMA_.*")
        .allowlist_var("GGML_.*")
        // Derive traits
        .derive_debug(true)
        .derive_default(true)
        .derive_copy(true)
        // Layout tests can be slow, skip in release
        .layout_tests(cfg!(debug_assertions))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");
}
