#[cfg(feature = "bindgen")]
extern crate bindgen;
extern crate cmake;

use std::env;
use std::path::{PathBuf, Path};
use std::fs;

//In order to build 64 bit hawktracer:
// mkdir build
// cd build
// cmake .. -G "Visual Studio 15 2017 Win64" -T v141,host=x64
// cmake --build .
fn main() {
    build_project();
    let mut extra_include_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        extra_include_path.push("build");
        extra_include_path.push("lib");
        extra_include_path.push("include");

    #[cfg(feature = "generate_bindings")] {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let bindings = bindgen::Builder::default()
            .header("hawktracer/lib/include/hawktracer.h")
            .clang_arg("-I./hawktracer/lib/include/")
            .clang_arg(format!("-I{}", extra_include_path.display()))
            .blacklist_type("max_align_t")
            .generate()
            .expect("Unable to generate bindings");
        println!("Manifest dir: {:?}", manifest_dir);

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }

    #[cfg(not(feature = "generate_bindings"))] {
        copy_pregenerated_bindings();
    }


    let mut build_output_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    build_output_path.push("build");
    build_output_path.push("lib");
    let mut second_build_output_path = build_output_path.clone();
    second_build_output_path.push("RelWithDebInfo");
    let target = env::var("TARGET").unwrap();

    if target.contains("pc-windows") {    
        #[cfg(debug_assertions)] { 
            build_output_path.push("Debug");
        }

        #[cfg(not(debug_assertions))] {
            build_output_path.push("Release");
            if !Path::new(&build_output_path).exists() {
                //If debug = true is specified, then this generates RelWithDebInfo.
                if Path::new(&second_build_output_path).exists() {
                    build_output_path = second_build_output_path;
                }
            }
        }
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++"); 
    }

    println!("cargo:rustc-link-search=all={}", build_output_path.display());
    println!("cargo:rustc-link-lib=static=hawktracer");
}

fn build_project() {
    let configuration_type = {
        #[cfg(debug_assertions)] { 
            "Debug"
        }
        #[cfg(not(debug_assertions))] { 
            "Release"
        }
    };
    
    cmake::Config::new("hawktracer")
        .define("CMAKE_BUILD_TYPE", configuration_type)
        .define("BUILD_STATIC_LIB", "ON")
        .build_target("hawktracer")
        .build();
}


#[cfg(not(feature = "generate_bindings"))]
fn copy_pregenerated_bindings() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let crate_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    fs::copy(crate_path.join("pregenerated_bindings.rs"), out_path.join("bindings.rs"))
        .expect("Couldn't find pregenerated bindings!");
}
