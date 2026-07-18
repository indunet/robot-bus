use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // libzmq uses Advapi32 security APIs on Windows. Recent Rust toolchains no
    // longer link advapi32 via std, so cdylib (maturin) builds fail with
    // LNK2019 for InitializeSecurityDescriptor / SetSecurityDescriptorDacl.
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("windows") {
        println!("cargo:rustc-link-lib=advapi32");
    }

    let proto_root = PathBuf::from("proto");
    let mut protos = Vec::new();
    collect_protos(&proto_root, &mut protos)?;
    // gRPC API protos (proto/robot_bus/grpc/) use tonic stubs, not plain prost.
    // Other robot_bus packages (e.g. action/) still go through prost.
    protos.retain(|p| !path_is_under(p, "proto/robot_bus/grpc"));
    protos.sort();

    for proto in &protos {
        println!("cargo:rerun-if-changed={}", proto.display());
    }

    // Foxglove schemas use google.protobuf.Timestamp / Duration; map to prost-types.
    prost_build::Config::new()
        .extern_path(".google.protobuf.Timestamp", "::prost_types::Timestamp")
        .extern_path(".google.protobuf.Duration", "::prost_types::Duration")
        .compile_protos(&protos, &[proto_root.clone()])?;

    #[cfg(feature = "grpc")]
    {
        let gateways = [
            PathBuf::from("proto/robot_bus/grpc/v1/message_gateway.proto"),
            PathBuf::from("proto/robot_bus/grpc/v1/service_gateway.proto"),
            PathBuf::from("proto/robot_bus/grpc/v1/action_gateway.proto"),
        ];
        for gateway in &gateways {
            println!("cargo:rerun-if-changed={}", gateway.display());
        }
        tonic_prost_build::configure()
            .build_server(true)
            .build_client(true)
            .compile_protos(&gateways, &[proto_root])?;
    }

    Ok(())
}

fn path_is_under(path: &std::path::Path, prefix: &str) -> bool {
    path.to_string_lossy().replace('\\', "/").contains(prefix)
}

fn collect_protos(
    dir: &std::path::Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_protos(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("proto") {
            out.push(path);
        }
    }
    Ok(())
}
