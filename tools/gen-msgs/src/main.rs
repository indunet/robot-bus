//! Generate on-disk (gitignored) prost / tonic stubs for robot-bus.
//!
//! Run from repo root via `scripts/generate_rust_msgs.py` / `just gen-rust`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("tools/gen-msgs must live at <repo>/tools/gen-msgs")?
        .to_path_buf();
    env::set_current_dir(&repo_root)?;

    let proto_root = PathBuf::from("proto");
    let mut protos = Vec::new();
    collect_protos(&proto_root, &mut protos)?;
    protos.retain(|p| !path_is_under(p, "proto/robot_bus_interface/grpc"));
    protos.sort();

    let msgs_out = PathBuf::from("src/msgs/generated");
    fs::create_dir_all(&msgs_out)?;
    clear_rs_files(&msgs_out)?;

    prost_build::Config::new()
        .out_dir(&msgs_out)
        .extern_path(".google.protobuf.Timestamp", "::prost_types::Timestamp")
        .extern_path(".google.protobuf.Duration", "::prost_types::Duration")
        .compile_protos(&protos, &[proto_root.clone()])?;

    let grpc_out = PathBuf::from("src/grpc/generated");
    fs::create_dir_all(&grpc_out)?;
    clear_rs_files(&grpc_out)?;

    let gateways = [
        PathBuf::from("proto/robot_bus_interface/grpc/v1/message_gateway.proto"),
        PathBuf::from("proto/robot_bus_interface/grpc/v1/service_gateway.proto"),
        PathBuf::from("proto/robot_bus_interface/grpc/v1/action_gateway.proto"),
    ];
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(&grpc_out)
        .compile_protos(&gateways, &[proto_root])?;

    println!(
        "generated {} msg stub(s) → {}/",
        count_rs(&msgs_out)?,
        msgs_out.display()
    );
    println!(
        "generated {} grpc stub(s) → {}/",
        count_rs(&grpc_out)?,
        grpc_out.display()
    );
    Ok(())
}

fn path_is_under(path: &Path, prefix: &str) -> bool {
    path.to_string_lossy().replace('\\', "/").contains(prefix)
}

fn collect_protos(
    dir: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
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

fn clear_rs_files(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn count_rs(dir: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let mut n = 0;
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            n += 1;
        }
    }
    Ok(n)
}
