//! Reproducible build: embed schema version.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let version = env!("CARGO_PKG_VERSION");
    println!("cargo:rustc-env=ETK_CLI_VERSION={}", version);
}
