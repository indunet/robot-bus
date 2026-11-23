use std::env;
use std::path::Path;

fn main() {
    // ROS 2 bridge uses system typesupport via AMENT_PREFIX_PATH (same as rclrs).
    if let Ok(ament) = env::var("AMENT_PREFIX_PATH") {
        for prefix in ament.split(':').filter(|p| !p.is_empty()) {
            println!(
                "cargo:rustc-link-search=native={}",
                Path::new(prefix).join("lib").display()
            );
        }
    }
}
