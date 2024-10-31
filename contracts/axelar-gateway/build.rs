fn main() {
    // trick taken from:
    // https://github.com/rust-lang/cargo/issues/6583#issuecomment-1259871885
    if let Ok(val) = std::env::var("GATEWAY_CONTRACT_TEST_VERSION") {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", val);
    }
    println!("cargo:rerun-if-env-changed=GATEWAY_CONTRACT_TEST_VERSION");
}
