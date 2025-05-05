fn main() {
    // This build script does nothing by default
    // In Windows CI, the workflow will add the skip_mpv feature flag
    println!("cargo:rerun-if-changed=build.rs");
}
