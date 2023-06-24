extern crate bindgen;

use autotools::Config;
use std::env;
use std::path::PathBuf;
use std::process::Command;

// Upstream version and name of the tarball file in the vendor directory
static BGPSTREAM_VERSION: &str = "libbgpstream-2.2.0";

// Extract bgpstream source to the auto generated build output directory
fn extract_bgpstream(build_output_dir: &str) -> std::io::Result<()> {
    Command::new("tar")
        .arg("-xf")
        .arg(format!("vendor/{BGPSTREAM_VERSION}.tgz"))
        .arg("-C")
        .arg(build_output_dir)
        .status()
        .unwrap();

    Ok(())
}

// Patch bgpstream to remove the check for pthreads as its broken on modern toolchains
fn patch_remove_pthread(build_output_dir: &str) -> std::io::Result<()> {
    Command::new("patch")
        .arg(format!(
            "{}/{BGPSTREAM_VERSION}/m4/ax_pthread.m4",
            build_output_dir
        ))
        .arg(format!("vendor/remove-phread-check.patch"))
        .status()
        .unwrap();

    Ok(())
}

// Remove duplicate "AM_INIT_AUTOMAKE" command in the configure input file
fn remove_duplicate_automake_options(build_output_dir: &str) -> std::io::Result<()> {
    let sed_matcher = r#"/^AM_INIT_AUTOMAKE$/d"#;
    Command::new("sed")
        .arg("-i")
        .arg(sed_matcher)
        .arg(format!(
            "{}/{BGPSTREAM_VERSION}/configure.ac",
            build_output_dir
        ))
        .status()
        .unwrap();

    Ok(())
}

// Since the tar file is originally from a git clone, we must run autogen.sh first
fn run_autogen(build_output_dir: &str) -> std::io::Result<()> {
    Command::new("sh")
        .current_dir(format!("{}/{BGPSTREAM_VERSION}", build_output_dir))
        .arg("autogen.sh")
        .status()
        .unwrap();
    Ok(())
}

fn main() -> std::io::Result<()> {
    // Map the Rust auto generated build output directory to a friendly name
    let build_output_dir = env::var("OUT_DIR").unwrap();

    // Extract the bgpstream tar file, must be done before setting "libdir_path"
    extract_bgpstream(&build_output_dir)?;

    // Map the directory name where bgpstream has been extracted too
    let libdir_path = PathBuf::from(format!("{}/{BGPSTREAM_VERSION}", build_output_dir))
        // Canonicalize the path as `rustc-link-search` requires an absolute
        // path.
        .canonicalize()
        .expect("cannot canonicalize path");

    // Setup the build
    patch_remove_pthread(&build_output_dir)?;
    remove_duplicate_automake_options(&build_output_dir)?;
    run_autogen(&build_output_dir)?;

    // Run configure and make via the autotools crate
    let mut conf = Config::new(&libdir_path);
    conf.enable_static()
        .disable_shared()
        .without("kafka", None)
        .insource(true);
    conf.build();

    // Map the directory where bgpstream's library's are located
    let bgpstream_libdir = format!("{build_output_dir}/{BGPSTREAM_VERSION}/lib");

    // Generate the bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // Add additional directories needed to locate all headers
        .clang_arg(format!("-I{bgpstream_libdir}/"))
        .clang_arg(format!("-I{bgpstream_libdir}/utils"))
        .generate_comments(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings out to file
    bindings
        .write_to_file(PathBuf::from(build_output_dir).join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Regenerate if changed
    println!("cargo:rerun-if-changed=wrapper.h");
    // Name of the library
    println!("cargo:rustc-link-lib=static=bgpstream");
    // Add in additional search path
    println!("cargo:rustc-link-search=native={bgpstream_libdir}");
    // Tell rustc to link in the dynamic library called wandio
    println!("cargo:rustc-link-lib=wandio");
    Ok(())
}
