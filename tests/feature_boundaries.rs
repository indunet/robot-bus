//! The same API assertions run for headless and embedded-console builds.
#![cfg(feature = "console-api")]
mod support;

#[test]
fn monitoring_works_without_assets_and_demo_flag_matches_build() {
    let _guard = support::lock_brokers();
    let mut config = support::ephemeral_robot_bus_config();
    config.console.enabled = true;
    // Even an explicitly enabled runtime flag cannot resurrect omitted code.
    config.console.tank_enabled = true;
    let broker = robot_bus::RobotBusBroker::start(config).unwrap();
    let base = format!("http://{}", broker.console_listen().unwrap());
    let status: serde_json::Value = ureq::get(&format!("{base}/api/v1/status"))
        .call()
        .unwrap()
        .into_json()
        .unwrap();
    assert!(status.is_object());
    let flags: serde_json::Value = ureq::get(&format!("{base}/api/v1/console"))
        .call()
        .unwrap()
        .into_json()
        .unwrap();
    assert_eq!(flags["tankEnabled"], cfg!(feature = "demo-tank"));
    assert_eq!(
        broker.discover.console_url.is_some(),
        cfg!(feature = "console")
    );
    let root = ureq::get(&base).call();
    #[cfg(feature = "console")]
    assert!(root.unwrap().into_string().unwrap().contains("<html"));
    #[cfg(not(feature = "console"))]
    assert!(matches!(root, Err(ureq::Error::Status(404, _))));
    #[cfg(not(feature = "demo-tank"))]
    assert!(matches!(
        ureq::post(&format!("{base}/api/v1/tank/session")).call(),
        Err(ureq::Error::Status(403, _))
    ));
    broker.stop().unwrap();
}

#[cfg(not(feature = "ws"))]
#[test]
fn monitoring_only_accepts_api_listen_and_cors_flags() {
    let args = [
        "--api-listen",
        "127.0.0.1:0",
        "--cors-origin",
        "http://localhost:3020",
    ]
    .map(str::to_string);
    let config = robot_bus::broker::parse_robot_bus_config(&args)
        .unwrap()
        .unwrap();
    assert_eq!(config.console.listen.to_string(), "127.0.0.1:0");
    assert_eq!(config.console.cors_origins, ["http://localhost:3020"]);
}
