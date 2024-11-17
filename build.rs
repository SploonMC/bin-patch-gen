fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=RUST_TEST_THREADS=1");
}
