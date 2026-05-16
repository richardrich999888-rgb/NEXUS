// VECTRA Build Script
// Embeds watermark information into the compiled binary

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Generate build watermark
    let build_id = generate_build_id();
    let git_commit = get_git_commit();
    let build_timestamp = get_timestamp();
    let rustc_version = get_rustc_version();
    
    // Export as environment variables for embedding
    println!("cargo:rustc-env=VECTRA_BUILD_ID={}", build_id);
    println!("cargo:rustc-env=VECTRA_GIT_COMMIT={}", git_commit);
    println!("cargo:rustc-env=VECTRA_BUILD_TIMESTAMP={}", build_timestamp);
    println!("cargo:rustc-env=VECTRA_RUSTC_VERSION={}", rustc_version);
    println!("cargo:rustc-env=VECTRA_ORG_FINGERPRINT=SYNTRIASS_LABS_PVT_LTD");
    
    // Create watermark string
    let watermark = format!(
        "SYNTRIASS_VECTRA_{}_{}_{}_{}",
        build_id, git_commit, build_timestamp, rustc_version
    );
    
    println!("cargo:rustc-env=VECTRA_WATERMARK={}", watermark);
    println!("cargo:warning=Build watermark: {}", watermark);
}

fn generate_build_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let mut hasher = DefaultHasher::new();
    timestamp.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn get_git_commit() -> String {
    Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

fn get_rustc_version() -> String {
    Command::new("rustc")
        .args(&["--version"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|s| {
            s.split_whitespace()
                .nth(1)
                .map(|v| v.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}
