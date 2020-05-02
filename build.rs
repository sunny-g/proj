use bindgen;
#[cfg(all(
    not(target_os = "macos"),
    not(feature = "pkg_config"),
    feature = "bundled_proj",
    not(feature = "nobuild")
))]
use cmake;
#[cfg(all(
    feature = "pkg_config",
    not(feature = "bundled_proj"),
    not(feature = "nobuild")
))]
use pkg_config;
use std::env;
use std::path::PathBuf;

#[cfg(all(
    feature = "pkg_config",
    not(feature = "bundled_proj"),
    not(feature = "nobuild")
))]
const MINIMUM_PROJ_VERSION: &str = "7.0.1";

#[cfg(feature = "nobuild")]
fn main() {} // Skip the build script on docs.rs

// We sometimes need additional search paths, which we get using pkg-config
#[cfg(all(
    feature = "pkg_config",
    not(feature = "nobuild"),
    not(feature = "bundled_proj")
))]
fn main() {
    let pk = pkg_config::Config::new()
        .atleast_version(MINIMUM_PROJ_VERSION)
        .probe("proj")
        .expect("Your PROJ version may be too old. You need at least {:?}", MINIMUM_PROJ_VERSION);
    // Tell cargo to tell rustc to link the system proj
    // shared library.
    println!("cargo:rustc-link-search=native={:?}", pk.link_paths[0]);
    println!("cargo:rustc-link-lib=proj");
    let include_path = pk.include_paths[0].to_string_lossy();

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include_path))
        .trust_clang_mangling(false)
        .blacklist_type("max_align_t")
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

// Vanilla
#[cfg(all(
    not(feature = "pkg_config"),
    not(feature = "nobuild"),
    not(feature = "bundled_proj")
))]
fn main() {
    println!("cargo:rustc-link-lib=proj");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .trust_clang_mangling(false)
        .blacklist_type("max_align_t")
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

#[cfg(all(
    not(feature = "pkg-config"),
    not(feature = "nobuild"),
    feature = "bundled_proj"
))]
fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if &target_os != "linux" {
        panic!(
            "Currently the bundled_proj feature only supports Linux. Your target platform: {}",
            target_os
        );
    }

    // Build PROJ from the included submodule.
    // NOTE: The PROJ build expects Sqlite3 to be present on the system.
    let mut config = cmake::Config::new("PROJ");
    let proj = config.build();

    // Tell cargo to tell rustc where to look for PROJ.
    println!(
        "cargo:rustc-link-search=native={}",
        proj.join("lib").display()
    );
    // Tell cargo to tell rustc to link PROJ.
    println!("cargo:rustc-link-lib=dylib=proj");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::builder()
        .header(proj.join("include").join("proj.h").to_str().unwrap())
        .trust_clang_mangling(false)
        .blacklist_type("max_align_t")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
