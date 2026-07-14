use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = PathBuf::from("proto");
    let mut protos = Vec::new();
    collect_protos(&proto_root, &mut protos)?;
    protos.sort();

    for proto in &protos {
        println!("cargo:rerun-if-changed={}", proto.display());
    }

    prost_build::Config::new()
        .compile_protos(&protos, &[proto_root])?;
    Ok(())
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
