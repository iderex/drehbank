//! The compiler that built this binary, recorded at build time.
//!
//! A measurement carries the machine it ran on, and the toolchain is part of
//! that. Nothing in the tree holds the version, and asking `rustc` on the path
//! at run time would answer about whatever is installed then rather than about
//! what produced this binary, so it is read here and baked in.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let compiler = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let version = Command::new(compiler)
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        // Never a guess. A toolchain field that could not be read says so, and
        // a reader of the record can tell that from a version that was read.
        .unwrap_or_else(|| "not read".to_string());
    println!("cargo:rustc-env=DREHBANK_TOOLCHAIN={version}");
}
