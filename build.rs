extern crate bindgen;
extern crate rdkafka_sys;

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

fn apply_source_patch(
    build_output_dir: &str,
    source_file: &str,
    patch_file: &str,
) -> std::io::Result<()> {
    Command::new("patch")
        .arg(format!(
            "{}/{BGPSTREAM_VERSION}/{}",
            build_output_dir, source_file
        ))
        .arg(format!("vendor/{}", patch_file))
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

    // Map the rust auto generated build output directory for rdkafka-sys dependency
    let kafka_root = env::var("DEP_RDKAFKA_ROOT").unwrap();

    // Extract the bgpstream tar file, must be done before setting "libdir_path"
    extract_bgpstream(&build_output_dir)?;

    // Map the directory name where bgpstream has been extracted too
    let libdir_path = PathBuf::from(format!("{}/{BGPSTREAM_VERSION}", build_output_dir))
        // Canonicalize the path as `rustc-link-search` requires an absolute
        // path.
        .canonicalize()
        .expect("cannot canonicalize path");

    // Patch to fix pthread check that breaks on modern build tool chains
    apply_source_patch(
        &build_output_dir,
        "/m4/ax_pthread.m4",
        "remove-phread-check.patch",
    )?;

    // Patch the Kafka check to search for the library in the right location
    apply_source_patch(
        &build_output_dir,
        "/m4/check_rdkafka_version.m4",
        "change_include_path_rdkafka_check.patch",
    )?;

    // Patch the Kafka transport to search for the library in the right location
    apply_source_patch(
        &build_output_dir,
        "lib/transports/bs_transport_kafka.c",
        "change_include_path_rdkafka_transports.patch",
    )?;

    // Patch configure to fix duplicate warnings and apply syntax updates
    apply_source_patch(
        &build_output_dir,
        "configure.ac",
        "update_automake_configure_ac.patch",
    )?;

    // Run autogen since this tarball is originally from a git clone
    run_autogen(&build_output_dir)?;

    // Run configure and make via the autotools crate
    let mut conf = Config::new(&libdir_path);
    conf.enable_static()
        .disable_shared()
        .cflag(format!("-I{kafka_root}/src/"))
        .ldflag(format!("-L{kafka_root}/src/"))
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
