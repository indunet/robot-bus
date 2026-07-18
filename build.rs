use std::env;

fn main() {
    // libzmq uses Advapi32 security APIs on Windows. Recent Rust toolchains no
    // longer link advapi32 via std, so cdylib (maturin) builds fail with
    // LNK2019 for InitializeSecurityDescriptor / SetSecurityDescriptorDacl.
    //
    // Protobuf / gRPC stubs are pre-generated (`just gen-rust`); this build
    // script does not invoke protoc.
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("windows") {
        println!("cargo:rustc-link-lib=advapi32");
    }
}
