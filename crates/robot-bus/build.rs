use std::env;

fn main() {
    // libzmq uses Advapi32 security APIs on Windows. Recent Rust toolchains no
    // longer link advapi32 via std, so dependents fail to link
    // InitializeSecurityDescriptor / SetSecurityDescriptorDacl unless this
    // crate emits the link directive (it propagates to bins and cdylibs).
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("windows") {
        println!("cargo:rustc-link-lib=advapi32");
    }
}
