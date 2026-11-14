use std::net::SocketAddr;
use std::time::Duration;

use template_dtnpm_cla::{ClaConfig, PeerEntry, TemplateCla, from_cla_address, to_cla_address};

#[test]
fn test_cla_config_validation() {
    let mut config = ClaConfig::default();
    assert!(config.validate().is_ok());

    config.max_frame_size = 10; // too small
    assert!(config.validate().is_err());

    config.max_frame_size = 1024;
    config.connect_timeout = Duration::from_secs(0); // zero timeout
    assert!(config.validate().is_err());
}

#[test]
fn test_cla_address_conversion_roundtrip() {
    let target = "127.0.0.1:4556";
    let cla_addr = to_cla_address(target);
    let extracted = from_cla_address(&cla_addr);
    assert_eq!(extracted.as_deref(), Some(target));
}

#[tokio::test]
async fn test_cla_init_parameters() {
    let config = ClaConfig {
        max_frame_size: 32_768,
        ..Default::default()
    };
    let cla = TemplateCla::new(config);
    let init = cla.cla_init();
    assert_eq!(init.max_bundle_size.map(|v| v.get()), Some(32_768));
    assert_eq!(init.lane_count.map(|v| v.get()), Some(1));
}

#[tokio::test]
async fn test_peer_registration_and_resolver() {
    let mut config = ClaConfig::default();
    let addr: SocketAddr = "127.0.0.1:8888".parse().expect("socket addr");
    config.peers.insert(
        "dtn://station-b/".to_string(),
        PeerEntry {
            node_id: "dtn://station-b/".to_string(),
            address: addr,
        },
    );

    let cla = TemplateCla::new(config);
    let node_id = "dtn://station-b/".parse().expect("node id");
    let resolved = cla.peer_resolver(&node_id).await;
    assert!(resolved.is_some());
    let extracted = from_cla_address(&resolved.expect("resolved"));
    assert_eq!(extracted.as_deref(), Some("127.0.0.1:8888"));

    let unknown_id = "dtn://unknown/".parse().expect("unknown node id");
    let unresolved = cla.peer_resolver(&unknown_id).await;
    assert!(unresolved.is_none());
}
