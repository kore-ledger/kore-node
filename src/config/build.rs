use std::env;

use crate::settings::KoreSettings;
use config::Config;

use super::params::Params;

pub fn build_config(env: bool, file: &str) -> KoreSettings {
    // Env configuration
    let mut params_env = Params::default();
    if env {
        params_env = Params::from_env();
    }

    // file configuration (json, yaml or toml)
    let mut params_file = Params::default();
    if !file.is_empty() {
        let mut config = Config::builder();

        config = config.add_source(config::File::with_name(file));

        let config = config
            .build()
            .map_err(|e| {
                println!("Error building config: {}", e);
            })
            .unwrap();

        params_file = config
            .try_deserialize()
            .map_err(|e| {
                println!("Error try deserialize config: {}", e);
            })
            .unwrap();
    }

    // Mix configurations.
    KoreSettings::from(params_env.mix_config(params_file))
}

pub fn build_password() -> String {
    env::var("KORE_PASSWORD").unwrap()
}

pub fn build_file_path() -> String {
    env::var("KORE_FILE_PATH").unwrap_or_default()
}


#[cfg(test)]
mod tests {
    use std::{num::NonZeroUsize, time::Duration};

    use crate::settings::DbSettings;
    use kore_base::{DigestDerivator, KeyDerivator, NodeType, RoutingNode};
    use serial_test::serial;
    use tempfile::TempDir;

    use super::build_config;

    #[test]
    #[serial]
    fn test_env_empty() {

    let config = build_config(true, "");

    assert_eq!(config.settings.network.port_reuse, false);
    assert_eq!(config.settings.network.user_agent, "kore-node");
    assert_eq!(config.settings.network.node_type, NodeType::Bootstrap);
    assert!(config.settings.network.listen_addresses.is_empty(),);
    assert!(config.settings.network.external_addresses.is_empty(),);
    assert_eq!(config.settings.node.key_derivator, KeyDerivator::Ed25519);
    assert_eq!(
        config.settings.node.digest_derivator,
        DigestDerivator::Blake3_256
    );
    assert_eq!(config.settings.node.replication_factor, 0.25f64);
    assert_eq!(config.settings.node.timeout, 3000);
    assert_eq!(config.settings.node.passvotation, 0);
    assert_eq!(config.settings.node.smartcontracts_directory, "./contracts");
    assert!(config.settings.network.routing.boot_nodes().is_empty(),);

    assert_eq!(config.settings.network.routing.get_dht_random_walk(), true);
    assert_eq!(
        config.settings.network.routing.get_discovery_limit(),
        std::u64::MAX
    );
    assert_eq!(
        config
            .settings
            .network
            .routing
            .get_allow_non_globals_in_dht(),
        false
    );
    assert_eq!(
        config.settings.network.routing.get_allow_private_ip(),
        false
    );
    assert_eq!(config.settings.network.routing.get_mdns(), true);
    assert_eq!(
        config
            .settings
            .network
            .routing
            .get_kademlia_disjoint_query_paths(),
        true
    );
    assert_eq!(
        config
            .settings
            .network
            .routing
            .get_kademlia_replication_factor(),
        None
    );
    assert_eq!(
        config.settings.network.routing.get_protocol_names(),
        vec!["/kore/tell/1.0.0", "/kore/reqres/1.0.0", "/kore/routing/1.0.0", "/ipfs/ping/1.0.0", "/ipfs/id/push/1.0.0", "/ipfs/id/id/1.0.0"]
    );
    assert_eq!(
        config.settings.network.tell.get_message_timeout(),
        Duration::from_secs(10)
    );
    assert_eq!(
        config.settings.network.tell.get_max_concurrent_streams(),
        100
    );

    #[cfg(feature = "leveldb")]
    assert_eq!(
        config.db,
        DbSettings::LevelDB("examples/leveldb".to_owned())
    );
    #[cfg(feature = "sqlite")]
    assert_eq!(
        config.db,
        DbSettings::Sqlite("examples/sqlitedb".to_owned())
    );
    assert_eq!(config.keys_path, "examples/keys".to_owned());
    assert_eq!(config.prometheus, "0.0.0.0:3050".to_owned());
    }

    #[test]
    #[serial]
    fn test_env_full() {
        std::env::set_var("KORE_NETWORK_TELL_MESSAGE_TIMEOUT_SECS", "58");
        std::env::set_var("KORE_NETWORK_TELL_MAX_CONCURRENT_STREAMS", "166");

        std::env::set_var("KORE_NETWORK_ROUTING_BOOT_NODES", "/ip4/172.17.0.1/tcp/50000_/ip4/127.0.0.1/tcp/60001/p2p/12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B,/ip4/11.11.0.11/tcp/10000_/ip4/12.22.33.44/tcp/55511/p2p/12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze");
        std::env::set_var("KORE_NETWORK_ROUTING_DHT_RANDOM_WALK", "false");
        std::env::set_var("KORE_NETWORK_ROUTING_DISCOVERY_ONLY_IF_UNDER_NUM", "55");
        std::env::set_var("KORE_NETWORK_ROUTING_ALLOW_NON_GLOBALS_IN_DHT", "true");
        std::env::set_var("KORE_NETWORK_ROUTING_ALLOW_PRIVATE_IP", "true");
        std::env::set_var("KORE_NETWORK_ROUTING_ENABLE_MDNS", "false");
        std::env::set_var(
            "KORE_NETWORK_ROUTING_KADEMLIA_DISJOINT_QUERY_PATHS",
            "false",
        );
        std::env::set_var("KORE_NETWORK_ROUTING_KADEMLIA_REPLICATION_FACTOR", "30");
        std::env::set_var(
            "KORE_NETWORK_ROUTING_PROTOCOL_NAMES",
            "/kore/routing/2.2.2,/kore/routing/1.1.1",
        );
        std::env::set_var("KORE_NODE_KEY_DERIVATOR", "Secp256k1");
        std::env::set_var("KORE_NODE_DIGEST_DERIVATOR", "Blake3_512");
        std::env::set_var("KORE_NODE_REPLICATION_FACTOR", "0.555");
        std::env::set_var("KORE_NODE_TIMEOUT", "30");
        std::env::set_var("KORE_NODE_PASSVOTATION", "50");
        std::env::set_var("KORE_NODE_SMARTCONTRACTS_DIRECTORY", "./fake_route");
        std::env::set_var("KORE_NETWORK_PORT_REUSE", "true");
        std::env::set_var("KORE_NETWORK_USER_AGENT", "Kore2.0");
        std::env::set_var("KORE_NETWORK_NODE_TYPE", "Addressable");
        std::env::set_var(
            "KORE_NETWORK_LISTEN_ADDRESSES",
            "/ip4/127.0.0.1/tcp/50000,/ip4/127.0.0.1/tcp/50001,/ip4/127.0.0.1/tcp/50002",
        );
        std::env::set_var(
            "KORE_NETWORK_EXTERNAL_ADDRESSES",
            "/ip4/90.1.0.60/tcp/50000,/ip4/90.1.0.61/tcp/50000",
        );
        std::env::set_var("KORE_DB_PATH", "./fake/db/path");
        std::env::set_var("KORE_KEYS_PATH", "./fake/keys/path");
        std::env::set_var("KORE_PROMETHEUS", "10.0.0.0:3030");

        let config = build_config(true, "");

        let boot_nodes = vec![
            RoutingNode {
                address: vec![
                    "/ip4/172.17.0.1/tcp/50000".to_owned(),
                    "/ip4/127.0.0.1/tcp/60001".to_owned(),
                ],
                peer_id: "12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B".to_owned(),
            },
            RoutingNode {
                address: vec![
                    "/ip4/11.11.0.11/tcp/10000".to_owned(),
                    "/ip4/12.22.33.44/tcp/55511".to_owned(),
                ],
                peer_id: "12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze".to_owned(),
            },
        ];

        assert_eq!(config.settings.network.port_reuse, true);
        assert_eq!(config.settings.network.user_agent, "Kore2.0");
        assert_eq!(config.settings.network.node_type, NodeType::Addressable);
        assert_eq!(
            config.settings.network.listen_addresses,
            vec![
                "/ip4/127.0.0.1/tcp/50000".to_owned(),
                "/ip4/127.0.0.1/tcp/50001".to_owned(),
                "/ip4/127.0.0.1/tcp/50002".to_owned()
            ]
        );
        assert_eq!(
            config.settings.network.external_addresses,
            vec![
                "/ip4/90.1.0.60/tcp/50000".to_owned(),
                "/ip4/90.1.0.61/tcp/50000".to_owned(),
            ]
        );
        assert_eq!(config.settings.node.key_derivator, KeyDerivator::Secp256k1);
        assert_eq!(
            config.settings.node.digest_derivator,
            DigestDerivator::Blake3_512
        );
        assert_eq!(config.settings.node.replication_factor, 0.555f64);
        assert_eq!(config.settings.node.timeout, 30);
        assert_eq!(config.settings.node.passvotation, 50);
        assert_eq!(
            config.settings.node.smartcontracts_directory,
            "./fake_route"
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].peer_id,
            boot_nodes[0].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].address,
            boot_nodes[0].address
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].peer_id,
            boot_nodes[1].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].address,
            boot_nodes[1].address
        );

        assert_eq!(config.settings.network.routing.get_dht_random_walk(), false);
        assert_eq!(config.settings.network.routing.get_discovery_limit(), 55);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_allow_non_globals_in_dht(),
            true
        );
        assert_eq!(config.settings.network.routing.get_allow_private_ip(), true);
        assert_eq!(config.settings.network.routing.get_mdns(), false);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_disjoint_query_paths(),
            false
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_replication_factor(),
            Some(NonZeroUsize::new(30).unwrap())
        );
        assert_eq!(
            config.settings.network.routing.get_protocol_names(),
            vec![
                "/kore/routing/2.2.2".to_owned(),
                "/kore/routing/1.1.1".to_owned()
            ]
        );
        assert_eq!(
            config.settings.network.tell.get_message_timeout(),
            Duration::from_secs(58)
        );
        assert_eq!(
            config.settings.network.tell.get_max_concurrent_streams(),
            166
        );

        #[cfg(feature = "leveldb")]
        assert_eq!(config.db, DbSettings::LevelDB("./fake/db/path".to_owned()));
        #[cfg(feature = "sqlite")]
        assert_eq!(config.db, DbSettings::Sqlite("./fake/db/path".to_owned()));
        assert_eq!(config.keys_path, "./fake/keys/path".to_owned());
        assert_eq!(config.prometheus, "10.0.0.0:3030".to_owned());

        std::env::remove_var("KORE_NETWORK_TELL_MESSAGE_TIMEOUT_SECS");
        std::env::remove_var("KORE_NETWORK_TELL_MAX_CONCURRENT_STREAMS");
        std::env::remove_var("KORE_NETWORK_ROUTING_BOOT_NODES");
        std::env::remove_var("KORE_NETWORK_ROUTING_DHT_RANDOM_WALK");
        std::env::remove_var("KORE_NETWORK_ROUTING_DISCOVERY_ONLY_IF_UNDER_NUM");
        std::env::remove_var("KORE_NETWORK_ROUTING_ALLOW_NON_GLOBALS_IN_DHT");
        std::env::remove_var("KORE_NETWORK_ROUTING_ALLOW_PRIVATE_IP");
        std::env::remove_var("KORE_NETWORK_ROUTING_ENABLE_MDNS");
        std::env::remove_var("KORE_NETWORK_ROUTING_KADEMLIA_DISJOINT_QUERY_PATHS");
        std::env::remove_var("KORE_NETWORK_ROUTING_KADEMLIA_REPLICATION_FACTOR");
        std::env::remove_var("KORE_NETWORK_ROUTING_PROTOCOL_NAMES");
        std::env::remove_var("KORE_DB_PATH");
        std::env::remove_var("KORE_KEYS_PATH");
        std::env::remove_var("KORE_NETWORK_PORT_REUSE");
        std::env::remove_var("KORE_NETWORK_USER_AGENT");
        std::env::remove_var("KORE_NETWORK_NODE_TYPE");
        std::env::remove_var("KORE_NETWORK_LISTEN_ADDRESSES");
        std::env::remove_var("KORE_NETWORK_EXTERNAL_ADDRESSES");
        std::env::remove_var("KORE_NODE_KEY_DERIVATOR");
        std::env::remove_var("KORE_NODE_DIGEST_DERIVATOR");
        std::env::remove_var("KORE_NODE_REPLICATION_FACTOR");
        std::env::remove_var("KORE_NODE_TIMEOUT");
        std::env::remove_var("KORE_NODE_PASSVOTATION");
        std::env::remove_var("KORE_NODE_SMARTCONTRACTS_DIRECTORY");
        std::env::remove_var("KORE_PROMETHEUS");
    }

    #[test]
    fn test_json_empty_file() {
        let content = r#"
            {
            "kore": {}
          }"#;
        let temp_dir = TempDir::new().unwrap();
        let temp_file_path = temp_dir.path().join("config.json");
        std::fs::write(&temp_file_path, content.to_string().as_bytes()).unwrap();

        let config = build_config(false, temp_file_path.to_str().unwrap());

        assert_eq!(config.settings.network.port_reuse, false);
        assert_eq!(config.settings.network.user_agent, "kore-node");
        assert_eq!(config.settings.network.node_type, NodeType::Bootstrap);
        assert!(config.settings.network.listen_addresses.is_empty(),);
        assert!(config.settings.network.external_addresses.is_empty(),);
        assert_eq!(config.settings.node.key_derivator, KeyDerivator::Ed25519);
        assert_eq!(
            config.settings.node.digest_derivator,
            DigestDerivator::Blake3_256
        );
        assert_eq!(config.settings.node.replication_factor, 0.25f64);
        assert_eq!(config.settings.node.timeout, 3000);
        assert_eq!(config.settings.node.passvotation, 0);
        assert_eq!(config.settings.node.smartcontracts_directory, "./contracts");
        assert!(config.settings.network.routing.boot_nodes().is_empty(),);

        assert_eq!(config.settings.network.routing.get_dht_random_walk(), true);
        assert_eq!(
            config.settings.network.routing.get_discovery_limit(),
            std::u64::MAX
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_allow_non_globals_in_dht(),
            false
        );
        assert_eq!(
            config.settings.network.routing.get_allow_private_ip(),
            false
        );
        assert_eq!(config.settings.network.routing.get_mdns(), true);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_disjoint_query_paths(),
            true
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_replication_factor(),
            None
        );
        assert_eq!(
            config.settings.network.routing.get_protocol_names(),
            vec!["/kore/tell/1.0.0", "/kore/reqres/1.0.0", "/kore/routing/1.0.0", "/ipfs/ping/1.0.0", "/ipfs/id/push/1.0.0", "/ipfs/id/id/1.0.0"]
        );
        assert_eq!(
            config.settings.network.tell.get_message_timeout(),
            Duration::from_secs(10)
        );
        assert_eq!(
            config.settings.network.tell.get_max_concurrent_streams(),
            100
        );

        #[cfg(feature = "leveldb")]
        assert_eq!(
            config.db,
            DbSettings::LevelDB("examples/leveldb".to_owned())
        );
        #[cfg(feature = "sqlite")]
        assert_eq!(
            config.db,
            DbSettings::Sqlite("examples/sqlitedb".to_owned())
        );
        assert_eq!(config.keys_path, "examples/keys".to_owned());
        assert_eq!(config.prometheus, "0.0.0.0:3050".to_owned());
    }

    #[test]
    fn test_json_full_file() {
        let content = r#"
            {
            "kore": {
              "network": {
                  "user_agent": "Kore2.0",
                  "node_type": "Addressable",
                  "listen_addresses": ["/ip4/127.0.0.1/tcp/50000","/ip4/127.0.0.1/tcp/50001","/ip4/127.0.0.1/tcp/50002"],
                  "external_addresses": ["/ip4/90.1.0.60/tcp/50000", "/ip4/90.1.0.61/tcp/50000"],
                  "tell": {
                    "message_timeout_secs": 58,
                    "max_concurrent_streams": 166
                  },
                  "routing": {
                    "boot_nodes": ["/ip4/172.17.0.1/tcp/50000_/ip4/127.0.0.1/tcp/60001/p2p/12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B","/ip4/11.11.0.11/tcp/10000_/ip4/12.22.33.44/tcp/55511/p2p/12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze"],
                    "dht_random_walk": false,
                    "discovery_only_if_under_num": 55,
                    "allow_non_globals_in_dht": true,
                    "allow_private_ip": true,
                    "enable_mdns": false,
                    "kademlia_disjoint_query_paths": false,
                    "kademlia_replication_factor": 30,
                    "protocol_names": ["/kore/routing/2.2.2","/kore/routing/1.1.1"]
                  },
                  "port_reuse": true
              },
              "node": {
                "key_derivator": "Secp256k1",
                "digest_derivator": "Blake3_512",
                "replication_factor": 0.555,
                "timeout": 30,
                "passvotation": 50,
                "smartcontracts_directory": "./fake_route"
              },
              "db_path": "./fake/db/path",
              "keys_path": "./fake/keys/path",
              "prometheus": "10.0.0.0:3030"
            }
          }"#;
        let temp_dir = TempDir::new().unwrap();
        let temp_file_path = temp_dir.path().join("config.json");
        std::fs::write(&temp_file_path, content.to_string().as_bytes()).unwrap();

        let config = build_config(false, temp_file_path.to_str().unwrap());

        let boot_nodes = vec![
            RoutingNode {
                address: vec![
                    "/ip4/172.17.0.1/tcp/50000".to_owned(),
                    "/ip4/127.0.0.1/tcp/60001".to_owned(),
                ],
                peer_id: "12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B".to_owned(),
            },
            RoutingNode {
                address: vec![
                    "/ip4/11.11.0.11/tcp/10000".to_owned(),
                    "/ip4/12.22.33.44/tcp/55511".to_owned(),
                ],
                peer_id: "12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze".to_owned(),
            },
        ];

        assert_eq!(config.settings.network.port_reuse, true);
        assert_eq!(config.settings.network.user_agent, "Kore2.0");
        assert_eq!(config.settings.network.node_type, NodeType::Addressable);
        assert_eq!(
            config.settings.network.listen_addresses,
            vec![
                "/ip4/127.0.0.1/tcp/50000".to_owned(),
                "/ip4/127.0.0.1/tcp/50001".to_owned(),
                "/ip4/127.0.0.1/tcp/50002".to_owned()
            ]
        );
        assert_eq!(
            config.settings.network.external_addresses,
            vec![
                "/ip4/90.1.0.60/tcp/50000".to_owned(),
                "/ip4/90.1.0.61/tcp/50000".to_owned(),
            ]
        );
        assert_eq!(config.settings.node.key_derivator, KeyDerivator::Secp256k1);
        assert_eq!(
            config.settings.node.digest_derivator,
            DigestDerivator::Blake3_512
        );
        assert_eq!(config.settings.node.replication_factor, 0.555f64);
        assert_eq!(config.settings.node.timeout, 30);
        assert_eq!(config.settings.node.passvotation, 50);
        assert_eq!(
            config.settings.node.smartcontracts_directory,
            "./fake_route"
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].peer_id,
            boot_nodes[0].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].address,
            boot_nodes[0].address
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].peer_id,
            boot_nodes[1].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].address,
            boot_nodes[1].address
        );

        assert_eq!(config.settings.network.routing.get_dht_random_walk(), false);
        assert_eq!(config.settings.network.routing.get_discovery_limit(), 55);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_allow_non_globals_in_dht(),
            true
        );
        assert_eq!(config.settings.network.routing.get_allow_private_ip(), true);
        assert_eq!(config.settings.network.routing.get_mdns(), false);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_disjoint_query_paths(),
            false
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_replication_factor(),
            Some(NonZeroUsize::new(30).unwrap())
        );
        assert_eq!(
            config.settings.network.routing.get_protocol_names(),
            vec![
                "/kore/routing/2.2.2".to_owned(),
                "/kore/routing/1.1.1".to_owned()
            ]
        );
        assert_eq!(
            config.settings.network.tell.get_message_timeout(),
            Duration::from_secs(58)
        );
        assert_eq!(
            config.settings.network.tell.get_max_concurrent_streams(),
            166
        );

        #[cfg(feature = "leveldb")]
        assert_eq!(config.db, DbSettings::LevelDB("./fake/db/path".to_owned()));
        #[cfg(feature = "sqlite")]
        assert_eq!(config.db, DbSettings::Sqlite("./fake/db/path".to_owned()));
        assert_eq!(config.keys_path, "./fake/keys/path".to_owned());
        assert_eq!(config.prometheus, "10.0.0.0:3030".to_owned());
    }

    #[test]
    fn test_json_partial_file() {
        let content = r#"
        {
            "kore": {
              "network": {
                  "tell": {
                    "message_timeout_secs": 58,
                    "max_concurrent_streams": 166
                  },
                  "routing": {
                    "boot_nodes": ["/ip4/172.17.0.1/tcp/50000_/ip4/127.0.0.1/tcp/60001/p2p/12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B","/ip4/11.11.0.11/tcp/10000_/ip4/12.22.33.44/tcp/55511/p2p/12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze"],
                    "dht_random_walk": false,
                    "discovery_only_if_under_num": 55,
                    "allow_non_globals_in_dht": true,
                    "allow_private_ip": true
                  },
                  "port_reuse": true
              },
              "node": {
                "replication_factor": 0.555,
                "timeout": 30,
                "passvotation": 50,
                "smartcontracts_directory": "./fake_route"
              }
            }
          }"#;
        let temp_dir = TempDir::new().unwrap();
        let temp_file_path = temp_dir.path().join("config.json");
        std::fs::write(&temp_file_path, content.to_string().as_bytes()).unwrap();

        let config = build_config(false, temp_file_path.to_str().unwrap());

        let boot_nodes = vec![
            RoutingNode {
                address: vec![
                    "/ip4/172.17.0.1/tcp/50000".to_owned(),
                    "/ip4/127.0.0.1/tcp/60001".to_owned(),
                ],
                peer_id: "12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B".to_owned(),
            },
            RoutingNode {
                address: vec![
                    "/ip4/11.11.0.11/tcp/10000".to_owned(),
                    "/ip4/12.22.33.44/tcp/55511".to_owned(),
                ],
                peer_id: "12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze".to_owned(),
            },
        ];

        assert_eq!(config.settings.network.port_reuse, true);
        assert_eq!(config.settings.network.user_agent, "kore-node");
        assert_eq!(config.settings.network.node_type, NodeType::Bootstrap);
        assert!(config.settings.network.listen_addresses.is_empty(),);
        assert!(config.settings.network.external_addresses.is_empty(),);
        assert_eq!(config.settings.node.key_derivator, KeyDerivator::Ed25519);
        assert_eq!(
            config.settings.node.digest_derivator,
            DigestDerivator::Blake3_256
        );
        assert_eq!(config.settings.node.replication_factor, 0.555f64);
        assert_eq!(config.settings.node.timeout, 30);
        assert_eq!(config.settings.node.passvotation, 50);
        assert_eq!(
            config.settings.node.smartcontracts_directory,
            "./fake_route"
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].peer_id,
            boot_nodes[0].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].address,
            boot_nodes[0].address
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].peer_id,
            boot_nodes[1].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].address,
            boot_nodes[1].address
        );

        assert_eq!(config.settings.network.routing.get_dht_random_walk(), false);
        assert_eq!(config.settings.network.routing.get_discovery_limit(), 55);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_allow_non_globals_in_dht(),
            true
        );
        assert_eq!(config.settings.network.routing.get_allow_private_ip(), true);
        assert_eq!(config.settings.network.routing.get_mdns(), true);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_disjoint_query_paths(),
            true
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_replication_factor(),
            None
        );
        assert_eq!(
            config.settings.network.routing.get_protocol_names(),
            vec!["/kore/tell/1.0.0", "/kore/reqres/1.0.0", "/kore/routing/1.0.0", "/ipfs/ping/1.0.0", "/ipfs/id/push/1.0.0", "/ipfs/id/id/1.0.0"]
        );
        assert_eq!(
            config.settings.network.tell.get_message_timeout(),
            Duration::from_secs(58)
        );
        assert_eq!(
            config.settings.network.tell.get_max_concurrent_streams(),
            166
        );

        #[cfg(feature = "leveldb")]
        assert_eq!(
            config.db,
            DbSettings::LevelDB("examples/leveldb".to_owned())
        );
        #[cfg(feature = "sqlite")]
        assert_eq!(
            config.db,
            DbSettings::Sqlite("examples/sqlitedb".to_owned())
        );
        assert_eq!(config.keys_path, "examples/keys".to_owned());
        assert_eq!(config.prometheus, "0.0.0.0:3050".to_owned());
    }

    #[test]
    #[serial]
    fn test_json_mix_env() {
        let content = r#"
        {
            "kore": {
              "network": {
                "external_addresses": ["/ip4/90.1.0.60/tcp/50000", "/ip4/90.1.0.61/tcp/50000"],
                "user_agent": "Kore3.0",
                  "tell": {
                    "message_timeout_secs": 58,
                    "max_concurrent_streams": 166
                  },
                  "routing": {
                    "boot_nodes": ["/ip4/172.17.0.1/tcp/50000_/ip4/127.0.0.1/tcp/60001/p2p/12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B","/ip4/11.11.0.11/tcp/10000_/ip4/12.22.33.44/tcp/55511/p2p/12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze"],
                    "dht_random_walk": false,
                    "discovery_only_if_under_num": 55,
                    "allow_non_globals_in_dht": true,
                    "allow_private_ip": true
                  },
                  "port_reuse": true
              },
              "node": {
                "replication_factor": 0.555,
                "timeout": 30,
                "passvotation": 50,
                "smartcontracts_directory": "./fake_route"
              }
            }
          }"#;
        std::env::set_var("KORE_NETWORK_USER_AGENT", "Kore2.0");
        std::env::set_var("KORE_NETWORK_NODE_TYPE", "Addressable");
        std::env::set_var(
            "KORE_NETWORK_LISTEN_ADDRESSES",
            "/ip4/127.0.0.1/tcp/50000,/ip4/127.0.0.1/tcp/50001,/ip4/127.0.0.1/tcp/50002",
        );
        std::env::set_var("KORE_NETWORK_ROUTING_ENABLE_MDNS", "false");
        std::env::set_var(
            "KORE_NETWORK_ROUTING_KADEMLIA_DISJOINT_QUERY_PATHS",
            "false",
        );
        std::env::set_var("KORE_NETWORK_ROUTING_KADEMLIA_REPLICATION_FACTOR", "30");
        std::env::set_var(
            "KORE_NETWORK_ROUTING_PROTOCOL_NAMES",
            "/kore/routing/2.2.2,/kore/routing/1.1.1",
        );
        std::env::set_var("KORE_NODE_KEY_DERIVATOR", "Secp256k1");
        std::env::set_var("KORE_NODE_DIGEST_DERIVATOR", "Blake3_512");
        std::env::set_var("KORE_DB_PATH", "./fake/db/path");
        std::env::set_var("KORE_KEYS_PATH", "./fake/keys/path");
        std::env::set_var("KORE_PROMETHEUS", "10.0.0.0:3030");

        let temp_dir = TempDir::new().unwrap();
        let temp_file_path = temp_dir.path().join("config.json");
        std::fs::write(&temp_file_path, content.to_string().as_bytes()).unwrap();

        let config = build_config(true, temp_file_path.to_str().unwrap());

        let boot_nodes = vec![
            RoutingNode {
                address: vec![
                    "/ip4/172.17.0.1/tcp/50000".to_owned(),
                    "/ip4/127.0.0.1/tcp/60001".to_owned(),
                ],
                peer_id: "12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B".to_owned(),
            },
            RoutingNode {
                address: vec![
                    "/ip4/11.11.0.11/tcp/10000".to_owned(),
                    "/ip4/12.22.33.44/tcp/55511".to_owned(),
                ],
                peer_id: "12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze".to_owned(),
            },
        ];

        assert_eq!(config.settings.network.port_reuse, true);
        assert_eq!(config.settings.network.user_agent, "Kore3.0");
        assert_eq!(config.settings.network.node_type, NodeType::Addressable);
        assert_eq!(
            config.settings.network.listen_addresses,
            vec![
                "/ip4/127.0.0.1/tcp/50000".to_owned(),
                "/ip4/127.0.0.1/tcp/50001".to_owned(),
                "/ip4/127.0.0.1/tcp/50002".to_owned()
            ]
        );
        assert_eq!(
            config.settings.network.external_addresses,
            vec![
                "/ip4/90.1.0.60/tcp/50000".to_owned(),
                "/ip4/90.1.0.61/tcp/50000".to_owned(),
            ]
        );
        assert_eq!(config.settings.node.key_derivator, KeyDerivator::Secp256k1);
        assert_eq!(
            config.settings.node.digest_derivator,
            DigestDerivator::Blake3_512
        );
        assert_eq!(config.settings.node.replication_factor, 0.555f64);
        assert_eq!(config.settings.node.timeout, 30);
        assert_eq!(config.settings.node.passvotation, 50);
        assert_eq!(
            config.settings.node.smartcontracts_directory,
            "./fake_route"
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].peer_id,
            boot_nodes[0].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].address,
            boot_nodes[0].address
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].peer_id,
            boot_nodes[1].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].address,
            boot_nodes[1].address
        );

        assert_eq!(config.settings.network.routing.get_dht_random_walk(), false);
        assert_eq!(config.settings.network.routing.get_discovery_limit(), 55);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_allow_non_globals_in_dht(),
            true
        );
        assert_eq!(config.settings.network.routing.get_allow_private_ip(), true);
        assert_eq!(config.settings.network.routing.get_mdns(), false);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_disjoint_query_paths(),
            false
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_replication_factor(),
            Some(NonZeroUsize::new(30).unwrap())
        );
        assert_eq!(
            config.settings.network.routing.get_protocol_names(),
            vec![
                "/kore/routing/2.2.2".to_owned(),
                "/kore/routing/1.1.1".to_owned()
            ]
        );
        assert_eq!(
            config.settings.network.tell.get_message_timeout(),
            Duration::from_secs(58)
        );
        assert_eq!(
            config.settings.network.tell.get_max_concurrent_streams(),
            166
        );

        #[cfg(feature = "leveldb")]
        assert_eq!(config.db, DbSettings::LevelDB("./fake/db/path".to_owned()));
        #[cfg(feature = "sqlite")]
        assert_eq!(config.db, DbSettings::Sqlite("./fake/db/path".to_owned()));
        assert_eq!(config.keys_path, "./fake/keys/path".to_owned());
        assert_eq!(config.prometheus, "10.0.0.0:3030".to_owned());

        std::env::remove_var("KORE_NETWORK_USER_AGENT");
        std::env::remove_var("KORE_NETWORK_NODE_TYPE");
        std::env::remove_var("KORE_NETWORK_LISTEN_ADDRESSES");
        std::env::remove_var("KORE_NETWORK_ROUTING_ENABLE_MDNS");
        std::env::remove_var("KORE_NETWORK_ROUTING_KADEMLIA_DISJOINT_QUERY_PATHS");
        std::env::remove_var("KORE_NETWORK_ROUTING_KADEMLIA_REPLICATION_FACTOR");
        std::env::remove_var("KORE_NETWORK_ROUTING_PROTOCOL_NAMES");
        std::env::remove_var("KORE_NODE_KEY_DERIVATOR");
        std::env::remove_var("KORE_NODE_DIGEST_DERIVATOR");
        std::env::remove_var("KORE_DB_PATH");
        std::env::remove_var("KORE_KEYS_PATH");
        std::env::remove_var("KORE_PROMETHEUS");
    }

    #[test]
    fn test_yaml_empty_file() {
        let content = r#"
            kore: {}
        "#;
        let temp_dir = TempDir::new().unwrap();
        let temp_file_path = temp_dir.path().join("config.yaml");
        std::fs::write(&temp_file_path, content.to_string().as_bytes()).unwrap();

        let config = build_config(false, temp_file_path.to_str().unwrap());

        assert_eq!(config.settings.network.port_reuse, false);
        assert_eq!(config.settings.network.user_agent, "kore-node");
        assert_eq!(config.settings.network.node_type, NodeType::Bootstrap);
        assert!(config.settings.network.listen_addresses.is_empty(),);
        assert!(config.settings.network.external_addresses.is_empty(),);
        assert_eq!(config.settings.node.key_derivator, KeyDerivator::Ed25519);
        assert_eq!(
            config.settings.node.digest_derivator,
            DigestDerivator::Blake3_256
        );
        assert_eq!(config.settings.node.replication_factor, 0.25f64);
        assert_eq!(config.settings.node.timeout, 3000);
        assert_eq!(config.settings.node.passvotation, 0);
        assert_eq!(config.settings.node.smartcontracts_directory, "./contracts");
        assert!(config.settings.network.routing.boot_nodes().is_empty(),);

        assert_eq!(config.settings.network.routing.get_dht_random_walk(), true);
        assert_eq!(
            config.settings.network.routing.get_discovery_limit(),
            std::u64::MAX
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_allow_non_globals_in_dht(),
            false
        );
        assert_eq!(
            config.settings.network.routing.get_allow_private_ip(),
            false
        );
        assert_eq!(config.settings.network.routing.get_mdns(), true);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_disjoint_query_paths(),
            true
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_replication_factor(),
            None
        );
        assert_eq!(
            config.settings.network.routing.get_protocol_names(),
            vec!["/kore/tell/1.0.0", "/kore/reqres/1.0.0", "/kore/routing/1.0.0", "/ipfs/ping/1.0.0", "/ipfs/id/push/1.0.0", "/ipfs/id/id/1.0.0"]
        );
        assert_eq!(
            config.settings.network.tell.get_message_timeout(),
            Duration::from_secs(10)
        );
        assert_eq!(
            config.settings.network.tell.get_max_concurrent_streams(),
            100
        );

        #[cfg(feature = "leveldb")]
        assert_eq!(
            config.db,
            DbSettings::LevelDB("examples/leveldb".to_owned())
        );
        #[cfg(feature = "sqlite")]
        assert_eq!(
            config.db,
            DbSettings::Sqlite("examples/sqlitedb".to_owned())
        );
        assert_eq!(config.keys_path, "examples/keys".to_owned());
        assert_eq!(config.prometheus, "0.0.0.0:3050".to_owned());
    }

    #[test]
    fn test_yaml_full_file() {
        let content = r#"
        kore:
            network:
                user_agent: "Kore2.0"
                node_type: "Addressable"
                listen_addresses:
                - "/ip4/127.0.0.1/tcp/50000"
                - "/ip4/127.0.0.1/tcp/50001"
                - "/ip4/127.0.0.1/tcp/50002"
                external_addresses:
                - "/ip4/90.1.0.60/tcp/50000"
                - "/ip4/90.1.0.61/tcp/50000"
                tell:
                    message_timeout_secs: 58
                    max_concurrent_streams: 166
                routing:
                    boot_nodes:
                    - "/ip4/172.17.0.1/tcp/50000_/ip4/127.0.0.1/tcp/60001/p2p/12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B"
                    - "/ip4/11.11.0.11/tcp/10000_/ip4/12.22.33.44/tcp/55511/p2p/12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze"
                    dht_random_walk: false
                    discovery_only_if_under_num: 55
                    allow_non_globals_in_dht: true
                    allow_private_ip: true
                    enable_mdns: false
                    kademlia_disjoint_query_paths: false
                    kademlia_replication_factor: 30
                    protocol_names:
                    - "/kore/routing/2.2.2"
                    - "/kore/routing/1.1.1"
                port_reuse: true
            node:
                key_derivator: "Secp256k1"
                digest_derivator: "Blake3_512"
                replication_factor: 0.555
                timeout: 30
                passvotation: 50
                smartcontracts_directory: "./fake_route"
            db_path: "./fake/db/path"
            keys_path: "./fake/keys/path"
            prometheus: "10.0.0.0:3030"
          "#;
        let temp_dir = TempDir::new().unwrap();
        let temp_file_path = temp_dir.path().join("config.yaml");
        std::fs::write(&temp_file_path, content.to_string().as_bytes()).unwrap();

        let config = build_config(false, temp_file_path.to_str().unwrap());

        let boot_nodes = vec![
            RoutingNode {
                address: vec![
                    "/ip4/172.17.0.1/tcp/50000".to_owned(),
                    "/ip4/127.0.0.1/tcp/60001".to_owned(),
                ],
                peer_id: "12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B".to_owned(),
            },
            RoutingNode {
                address: vec![
                    "/ip4/11.11.0.11/tcp/10000".to_owned(),
                    "/ip4/12.22.33.44/tcp/55511".to_owned(),
                ],
                peer_id: "12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze".to_owned(),
            },
        ];

        assert_eq!(config.settings.network.port_reuse, true);
        assert_eq!(config.settings.network.user_agent, "Kore2.0");
        assert_eq!(config.settings.network.node_type, NodeType::Addressable);
        assert_eq!(
            config.settings.network.listen_addresses,
            vec![
                "/ip4/127.0.0.1/tcp/50000".to_owned(),
                "/ip4/127.0.0.1/tcp/50001".to_owned(),
                "/ip4/127.0.0.1/tcp/50002".to_owned()
            ]
        );
        assert_eq!(
            config.settings.network.external_addresses,
            vec![
                "/ip4/90.1.0.60/tcp/50000".to_owned(),
                "/ip4/90.1.0.61/tcp/50000".to_owned(),
            ]
        );
        assert_eq!(config.settings.node.key_derivator, KeyDerivator::Secp256k1);
        assert_eq!(
            config.settings.node.digest_derivator,
            DigestDerivator::Blake3_512
        );
        assert_eq!(config.settings.node.replication_factor, 0.555f64);
        assert_eq!(config.settings.node.timeout, 30);
        assert_eq!(config.settings.node.passvotation, 50);
        assert_eq!(
            config.settings.node.smartcontracts_directory,
            "./fake_route"
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].peer_id,
            boot_nodes[0].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].address,
            boot_nodes[0].address
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].peer_id,
            boot_nodes[1].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].address,
            boot_nodes[1].address
        );

        assert_eq!(config.settings.network.routing.get_dht_random_walk(), false);
        assert_eq!(config.settings.network.routing.get_discovery_limit(), 55);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_allow_non_globals_in_dht(),
            true
        );
        assert_eq!(config.settings.network.routing.get_allow_private_ip(), true);
        assert_eq!(config.settings.network.routing.get_mdns(), false);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_disjoint_query_paths(),
            false
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_replication_factor(),
            Some(NonZeroUsize::new(30).unwrap())
        );
        assert_eq!(
            config.settings.network.routing.get_protocol_names(),
            vec![
                "/kore/routing/2.2.2".to_owned(),
                "/kore/routing/1.1.1".to_owned()
            ]
        );
        assert_eq!(
            config.settings.network.tell.get_message_timeout(),
            Duration::from_secs(58)
        );
        assert_eq!(
            config.settings.network.tell.get_max_concurrent_streams(),
            166
        );

        #[cfg(feature = "leveldb")]
        assert_eq!(config.db, DbSettings::LevelDB("./fake/db/path".to_owned()));
        #[cfg(feature = "sqlite")]
        assert_eq!(config.db, DbSettings::Sqlite("./fake/db/path".to_owned()));
        assert_eq!(config.keys_path, "./fake/keys/path".to_owned());
        assert_eq!(config.prometheus, "10.0.0.0:3030".to_owned());
    }

    #[test]
    fn test_yaml_partial_file() {
        let content = r#"
        kore:
            network:
                tell:
                    message_timeout_secs: 58
                    max_concurrent_streams: 166
                routing:
                    boot_nodes:
                    - "/ip4/172.17.0.1/tcp/50000_/ip4/127.0.0.1/tcp/60001/p2p/12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B"
                    - "/ip4/11.11.0.11/tcp/10000_/ip4/12.22.33.44/tcp/55511/p2p/12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze"
                    dht_random_walk: false
                    discovery_only_if_under_num: 55
                    allow_non_globals_in_dht: true
                    allow_private_ip: true
                port_reuse: true
            node:
                replication_factor: 0.555
                timeout: 30
                passvotation: 50
                smartcontracts_directory: "./fake_route"      
        "#;
        let temp_dir = TempDir::new().unwrap();
        let temp_file_path = temp_dir.path().join("config.yaml");
        std::fs::write(&temp_file_path, content.to_string().as_bytes()).unwrap();

        let config = build_config(false, temp_file_path.to_str().unwrap());

        let boot_nodes = vec![
            RoutingNode {
                address: vec![
                    "/ip4/172.17.0.1/tcp/50000".to_owned(),
                    "/ip4/127.0.0.1/tcp/60001".to_owned(),
                ],
                peer_id: "12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B".to_owned(),
            },
            RoutingNode {
                address: vec![
                    "/ip4/11.11.0.11/tcp/10000".to_owned(),
                    "/ip4/12.22.33.44/tcp/55511".to_owned(),
                ],
                peer_id: "12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze".to_owned(),
            },
        ];

        assert_eq!(config.settings.network.port_reuse, true);
        assert_eq!(config.settings.network.user_agent, "kore-node");
        assert_eq!(config.settings.network.node_type, NodeType::Bootstrap);
        assert!(config.settings.network.listen_addresses.is_empty(),);
        assert!(config.settings.network.external_addresses.is_empty(),);
        assert_eq!(config.settings.node.key_derivator, KeyDerivator::Ed25519);
        assert_eq!(
            config.settings.node.digest_derivator,
            DigestDerivator::Blake3_256
        );
        assert_eq!(config.settings.node.replication_factor, 0.555f64);
        assert_eq!(config.settings.node.timeout, 30);
        assert_eq!(config.settings.node.passvotation, 50);
        assert_eq!(
            config.settings.node.smartcontracts_directory,
            "./fake_route"
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].peer_id,
            boot_nodes[0].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].address,
            boot_nodes[0].address
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].peer_id,
            boot_nodes[1].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].address,
            boot_nodes[1].address
        );

        assert_eq!(config.settings.network.routing.get_dht_random_walk(), false);
        assert_eq!(config.settings.network.routing.get_discovery_limit(), 55);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_allow_non_globals_in_dht(),
            true
        );
        assert_eq!(config.settings.network.routing.get_allow_private_ip(), true);
        assert_eq!(config.settings.network.routing.get_mdns(), true);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_disjoint_query_paths(),
            true
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_replication_factor(),
            None
        );
        assert_eq!(
            config.settings.network.routing.get_protocol_names(),
            vec!["/kore/tell/1.0.0", "/kore/reqres/1.0.0", "/kore/routing/1.0.0", "/ipfs/ping/1.0.0", "/ipfs/id/push/1.0.0", "/ipfs/id/id/1.0.0"]
        );
        assert_eq!(
            config.settings.network.tell.get_message_timeout(),
            Duration::from_secs(58)
        );
        assert_eq!(
            config.settings.network.tell.get_max_concurrent_streams(),
            166
        );

        #[cfg(feature = "leveldb")]
        assert_eq!(
            config.db,
            DbSettings::LevelDB("examples/leveldb".to_owned())
        );
        #[cfg(feature = "sqlite")]
        assert_eq!(
            config.db,
            DbSettings::Sqlite("examples/sqlitedb".to_owned())
        );
        assert_eq!(config.keys_path, "examples/keys".to_owned());
        assert_eq!(config.prometheus, "0.0.0.0:3050".to_owned());
    }

    #[test]
    #[serial]
    fn test_yaml_mix_env() {
        let content = r#"
        kore:
            network:
                user_agent: "Kore3.0"
                external_addresses:
                - "/ip4/90.1.0.60/tcp/50000"
                - "/ip4/90.1.0.61/tcp/50000"
                tell:
                    message_timeout_secs: 58
                    max_concurrent_streams: 166
                routing:
                    boot_nodes:
                    - "/ip4/172.17.0.1/tcp/50000_/ip4/127.0.0.1/tcp/60001/p2p/12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B"
                    - "/ip4/11.11.0.11/tcp/10000_/ip4/12.22.33.44/tcp/55511/p2p/12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze"
                    dht_random_walk: false
                    discovery_only_if_under_num: 55
                    allow_non_globals_in_dht: true
                    allow_private_ip: true
                port_reuse: true
            node:
                replication_factor: 0.555
                timeout: 30
                passvotation: 50
                smartcontracts_directory: "./fake_route"
          "#;
        std::env::set_var("KORE_NETWORK_USER_AGENT", "Kore2.0");
        std::env::set_var("KORE_NETWORK_NODE_TYPE", "Addressable");
        std::env::set_var(
            "KORE_NETWORK_LISTEN_ADDRESSES",
            "/ip4/127.0.0.1/tcp/50000,/ip4/127.0.0.1/tcp/50001,/ip4/127.0.0.1/tcp/50002",
        );
        std::env::set_var("KORE_NETWORK_ROUTING_ENABLE_MDNS", "false");
        std::env::set_var(
            "KORE_NETWORK_ROUTING_KADEMLIA_DISJOINT_QUERY_PATHS",
            "false",
        );
        std::env::set_var("KORE_NETWORK_ROUTING_KADEMLIA_REPLICATION_FACTOR", "30");
        std::env::set_var(
            "KORE_NETWORK_ROUTING_PROTOCOL_NAMES",
            "/kore/routing/2.2.2,/kore/routing/1.1.1",
        );
        std::env::set_var("KORE_NODE_KEY_DERIVATOR", "Secp256k1");
        std::env::set_var("KORE_NODE_DIGEST_DERIVATOR", "Blake3_512");
        std::env::set_var("KORE_DB_PATH", "./fake/db/path");
        std::env::set_var("KORE_KEYS_PATH", "./fake/keys/path");
        std::env::set_var("KORE_PROMETHEUS", "10.0.0.0:3030");

        let temp_dir = TempDir::new().unwrap();
        let temp_file_path = temp_dir.path().join("config.yaml");
        std::fs::write(&temp_file_path, content.to_string().as_bytes()).unwrap();

        let config = build_config(true, temp_file_path.to_str().unwrap());

        let boot_nodes = vec![
            RoutingNode {
                address: vec![
                    "/ip4/172.17.0.1/tcp/50000".to_owned(),
                    "/ip4/127.0.0.1/tcp/60001".to_owned(),
                ],
                peer_id: "12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B".to_owned(),
            },
            RoutingNode {
                address: vec![
                    "/ip4/11.11.0.11/tcp/10000".to_owned(),
                    "/ip4/12.22.33.44/tcp/55511".to_owned(),
                ],
                peer_id: "12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze".to_owned(),
            },
        ];

        assert_eq!(config.settings.network.port_reuse, true);
        assert_eq!(config.settings.network.user_agent, "Kore3.0");
        assert_eq!(config.settings.network.node_type, NodeType::Addressable);
        assert_eq!(
            config.settings.network.listen_addresses,
            vec![
                "/ip4/127.0.0.1/tcp/50000".to_owned(),
                "/ip4/127.0.0.1/tcp/50001".to_owned(),
                "/ip4/127.0.0.1/tcp/50002".to_owned()
            ]
        );
        assert_eq!(
            config.settings.network.external_addresses,
            vec![
                "/ip4/90.1.0.60/tcp/50000".to_owned(),
                "/ip4/90.1.0.61/tcp/50000".to_owned(),
            ]
        );
        assert_eq!(config.settings.node.key_derivator, KeyDerivator::Secp256k1);
        assert_eq!(
            config.settings.node.digest_derivator,
            DigestDerivator::Blake3_512
        );
        assert_eq!(config.settings.node.replication_factor, 0.555f64);
        assert_eq!(config.settings.node.timeout, 30);
        assert_eq!(config.settings.node.passvotation, 50);
        assert_eq!(
            config.settings.node.smartcontracts_directory,
            "./fake_route"
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].peer_id,
            boot_nodes[0].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].address,
            boot_nodes[0].address
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].peer_id,
            boot_nodes[1].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].address,
            boot_nodes[1].address
        );

        assert_eq!(config.settings.network.routing.get_dht_random_walk(), false);
        assert_eq!(config.settings.network.routing.get_discovery_limit(), 55);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_allow_non_globals_in_dht(),
            true
        );
        assert_eq!(config.settings.network.routing.get_allow_private_ip(), true);
        assert_eq!(config.settings.network.routing.get_mdns(), false);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_disjoint_query_paths(),
            false
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_replication_factor(),
            Some(NonZeroUsize::new(30).unwrap())
        );
        assert_eq!(
            config.settings.network.routing.get_protocol_names(),
            vec![
                "/kore/routing/2.2.2".to_owned(),
                "/kore/routing/1.1.1".to_owned()
            ]
        );
        assert_eq!(
            config.settings.network.tell.get_message_timeout(),
            Duration::from_secs(58)
        );
        assert_eq!(
            config.settings.network.tell.get_max_concurrent_streams(),
            166
        );

        #[cfg(feature = "leveldb")]
        assert_eq!(config.db, DbSettings::LevelDB("./fake/db/path".to_owned()));
        #[cfg(feature = "sqlite")]
        assert_eq!(config.db, DbSettings::Sqlite("./fake/db/path".to_owned()));
        assert_eq!(config.keys_path, "./fake/keys/path".to_owned());
        assert_eq!(config.prometheus, "10.0.0.0:3030".to_owned());

        std::env::remove_var("KORE_PROMETHEUS");
        std::env::remove_var("KORE_NETWORK_USER_AGENT");
        std::env::remove_var("KORE_NETWORK_NODE_TYPE");
        std::env::remove_var("KORE_NETWORK_LISTEN_ADDRESSES");
        std::env::remove_var("KORE_NETWORK_ROUTING_ENABLE_MDNS");
        std::env::remove_var("KORE_NETWORK_ROUTING_KADEMLIA_DISJOINT_QUERY_PATHS");
        std::env::remove_var("KORE_NETWORK_ROUTING_KADEMLIA_REPLICATION_FACTOR");
        std::env::remove_var("KORE_NETWORK_ROUTING_PROTOCOL_NAMES");
        std::env::remove_var("KORE_NODE_KEY_DERIVATOR");
        std::env::remove_var("KORE_NODE_DIGEST_DERIVATOR");
        std::env::remove_var("KORE_DB_PATH");
        std::env::remove_var("KORE_KEYS_PATH");
    }


    #[test]
    fn test_toml_empty_file() {
        let content = r#"
        [kore]
        "#;
        let temp_dir = TempDir::new().unwrap();
        let temp_file_path = temp_dir.path().join("config.toml");
        std::fs::write(&temp_file_path, content.to_string().as_bytes()).unwrap();

        let config = build_config(false, temp_file_path.to_str().unwrap());

        assert_eq!(config.settings.network.port_reuse, false);
        assert_eq!(config.settings.network.user_agent, "kore-node");
        assert_eq!(config.settings.network.node_type, NodeType::Bootstrap);
        assert!(config.settings.network.listen_addresses.is_empty(),);
        assert!(config.settings.network.external_addresses.is_empty(),);
        assert_eq!(config.settings.node.key_derivator, KeyDerivator::Ed25519);
        assert_eq!(
            config.settings.node.digest_derivator,
            DigestDerivator::Blake3_256
        );
        assert_eq!(config.settings.node.replication_factor, 0.25f64);
        assert_eq!(config.settings.node.timeout, 3000);
        assert_eq!(config.settings.node.passvotation, 0);
        assert_eq!(config.settings.node.smartcontracts_directory, "./contracts");
        assert!(config.settings.network.routing.boot_nodes().is_empty(),);

        assert_eq!(config.settings.network.routing.get_dht_random_walk(), true);
        assert_eq!(
            config.settings.network.routing.get_discovery_limit(),
            std::u64::MAX
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_allow_non_globals_in_dht(),
            false
        );
        assert_eq!(
            config.settings.network.routing.get_allow_private_ip(),
            false
        );
        assert_eq!(config.settings.network.routing.get_mdns(), true);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_disjoint_query_paths(),
            true
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_replication_factor(),
            None
        );
        assert_eq!(
            config.settings.network.routing.get_protocol_names(),
            vec!["/kore/tell/1.0.0", "/kore/reqres/1.0.0", "/kore/routing/1.0.0", "/ipfs/ping/1.0.0", "/ipfs/id/push/1.0.0", "/ipfs/id/id/1.0.0"]
        );
        assert_eq!(
            config.settings.network.tell.get_message_timeout(),
            Duration::from_secs(10)
        );
        assert_eq!(
            config.settings.network.tell.get_max_concurrent_streams(),
            100
        );

        #[cfg(feature = "leveldb")]
        assert_eq!(
            config.db,
            DbSettings::LevelDB("examples/leveldb".to_owned())
        );
        #[cfg(feature = "sqlite")]
        assert_eq!(
            config.db,
            DbSettings::Sqlite("examples/sqlitedb".to_owned())
        );
        assert_eq!(config.keys_path, "examples/keys".to_owned());
        assert_eq!(config.prometheus, "0.0.0.0:3050".to_owned());
    }

    #[test]
    fn test_toml_full_file() {
        let content = r#"
        [kore.network]
        user_agent = "Kore2.0"
        node_type = "Addressable"
        port_reuse = true
        listen_addresses = ["/ip4/127.0.0.1/tcp/50000","/ip4/127.0.0.1/tcp/50001","/ip4/127.0.0.1/tcp/50002"]
        external_addresses = ["/ip4/90.1.0.60/tcp/50000","/ip4/90.1.0.61/tcp/50000"]
        
        [kore.network.tell]
        message_timeout_secs = 58
        max_concurrent_streams = 166
        
        [kore.network.routing]
        boot_nodes = ["/ip4/172.17.0.1/tcp/50000_/ip4/127.0.0.1/tcp/60001/p2p/12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B", "/ip4/11.11.0.11/tcp/10000_/ip4/12.22.33.44/tcp/55511/p2p/12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze"]
        dht_random_walk = false
        discovery_only_if_under_num = 55
        allow_non_globals_in_dht = true
        allow_private_ip = true
        enable_mdns = false
        kademlia_disjoint_query_paths = false
        kademlia_replication_factor = 30
        protocol_names = ["/kore/routing/2.2.2", "/kore/routing/1.1.1"]
        
        [kore.node]
        key_derivator = "Secp256k1"
        digest_derivator = "Blake3_512"
        replication_factor = 0.555
        timeout = 30
        passvotation = 50
        smartcontracts_directory = "./fake_route"
        
        [kore]
        db_path = "./fake/db/path"
        keys_path = "./fake/keys/path"    
        prometheus = "10.0.0.0:3030"    
        "#;
        let temp_dir = TempDir::new().unwrap();
        let temp_file_path = temp_dir.path().join("config.toml");
        std::fs::write(&temp_file_path, content.to_string().as_bytes()).unwrap();

        let config = build_config(false, temp_file_path.to_str().unwrap());

        let boot_nodes = vec![
            RoutingNode {
                address: vec![
                    "/ip4/172.17.0.1/tcp/50000".to_owned(),
                    "/ip4/127.0.0.1/tcp/60001".to_owned(),
                ],
                peer_id: "12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B".to_owned(),
            },
            RoutingNode {
                address: vec![
                    "/ip4/11.11.0.11/tcp/10000".to_owned(),
                    "/ip4/12.22.33.44/tcp/55511".to_owned(),
                ],
                peer_id: "12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze".to_owned(),
            },
        ];

        assert_eq!(config.settings.network.port_reuse, true);
        assert_eq!(config.settings.network.user_agent, "Kore2.0");
        assert_eq!(config.settings.network.node_type, NodeType::Addressable);
        assert_eq!(
            config.settings.network.listen_addresses,
            vec![
                "/ip4/127.0.0.1/tcp/50000".to_owned(),
                "/ip4/127.0.0.1/tcp/50001".to_owned(),
                "/ip4/127.0.0.1/tcp/50002".to_owned()
            ]
        );
        assert_eq!(
            config.settings.network.external_addresses,
            vec![
                "/ip4/90.1.0.60/tcp/50000".to_owned(),
                "/ip4/90.1.0.61/tcp/50000".to_owned(),
            ]
        );
        assert_eq!(config.settings.node.key_derivator, KeyDerivator::Secp256k1);
        assert_eq!(
            config.settings.node.digest_derivator,
            DigestDerivator::Blake3_512
        );
        assert_eq!(config.settings.node.replication_factor, 0.555f64);
        assert_eq!(config.settings.node.timeout, 30);
        assert_eq!(config.settings.node.passvotation, 50);
        assert_eq!(
            config.settings.node.smartcontracts_directory,
            "./fake_route"
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].peer_id,
            boot_nodes[0].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].address,
            boot_nodes[0].address
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].peer_id,
            boot_nodes[1].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].address,
            boot_nodes[1].address
        );

        assert_eq!(config.settings.network.routing.get_dht_random_walk(), false);
        assert_eq!(config.settings.network.routing.get_discovery_limit(), 55);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_allow_non_globals_in_dht(),
            true
        );
        assert_eq!(config.settings.network.routing.get_allow_private_ip(), true);
        assert_eq!(config.settings.network.routing.get_mdns(), false);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_disjoint_query_paths(),
            false
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_replication_factor(),
            Some(NonZeroUsize::new(30).unwrap())
        );
        assert_eq!(
            config.settings.network.routing.get_protocol_names(),
            vec![
                "/kore/routing/2.2.2".to_owned(),
                "/kore/routing/1.1.1".to_owned()
            ]
        );
        assert_eq!(
            config.settings.network.tell.get_message_timeout(),
            Duration::from_secs(58)
        );
        assert_eq!(
            config.settings.network.tell.get_max_concurrent_streams(),
            166
        );

        #[cfg(feature = "leveldb")]
        assert_eq!(config.db, DbSettings::LevelDB("./fake/db/path".to_owned()));
        #[cfg(feature = "sqlite")]
        assert_eq!(config.db, DbSettings::Sqlite("./fake/db/path".to_owned()));
        assert_eq!(config.keys_path, "./fake/keys/path".to_owned());
        assert_eq!(config.prometheus, "10.0.0.0:3030".to_owned());
    }

    #[test]
    fn test_toml_partial_file() {
        let content = r#"
        [kore.network.tell]
        message_timeout_secs = 58
        max_concurrent_streams = 166
        
        [kore.network.routing]
        boot_nodes = ["/ip4/172.17.0.1/tcp/50000_/ip4/127.0.0.1/tcp/60001/p2p/12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B", "/ip4/11.11.0.11/tcp/10000_/ip4/12.22.33.44/tcp/55511/p2p/12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze"]
        dht_random_walk = false
        discovery_only_if_under_num = 55
        allow_non_globals_in_dht = true
        allow_private_ip = true
        
        [kore.network]
        port_reuse = true
        
        [kore.node]
        replication_factor = 0.555
        timeout = 30
        passvotation = 50
        smartcontracts_directory = "./fake_route"        
        "#;
        let temp_dir = TempDir::new().unwrap();
        let temp_file_path = temp_dir.path().join("config.toml");
        std::fs::write(&temp_file_path, content.to_string().as_bytes()).unwrap();

        let config = build_config(false, temp_file_path.to_str().unwrap());

        let boot_nodes = vec![
            RoutingNode {
                address: vec![
                    "/ip4/172.17.0.1/tcp/50000".to_owned(),
                    "/ip4/127.0.0.1/tcp/60001".to_owned(),
                ],
                peer_id: "12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B".to_owned(),
            },
            RoutingNode {
                address: vec![
                    "/ip4/11.11.0.11/tcp/10000".to_owned(),
                    "/ip4/12.22.33.44/tcp/55511".to_owned(),
                ],
                peer_id: "12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze".to_owned(),
            },
        ];

        assert_eq!(config.settings.network.port_reuse, true);
        assert_eq!(config.settings.network.user_agent, "kore-node");
        assert_eq!(config.settings.network.node_type, NodeType::Bootstrap);
        assert!(config.settings.network.listen_addresses.is_empty(),);
        assert!(config.settings.network.external_addresses.is_empty(),);
        assert_eq!(config.settings.node.key_derivator, KeyDerivator::Ed25519);
        assert_eq!(
            config.settings.node.digest_derivator,
            DigestDerivator::Blake3_256
        );
        assert_eq!(config.settings.node.replication_factor, 0.555f64);
        assert_eq!(config.settings.node.timeout, 30);
        assert_eq!(config.settings.node.passvotation, 50);
        assert_eq!(
            config.settings.node.smartcontracts_directory,
            "./fake_route"
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].peer_id,
            boot_nodes[0].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].address,
            boot_nodes[0].address
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].peer_id,
            boot_nodes[1].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].address,
            boot_nodes[1].address
        );

        assert_eq!(config.settings.network.routing.get_dht_random_walk(), false);
        assert_eq!(config.settings.network.routing.get_discovery_limit(), 55);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_allow_non_globals_in_dht(),
            true
        );
        assert_eq!(config.settings.network.routing.get_allow_private_ip(), true);
        assert_eq!(config.settings.network.routing.get_mdns(), true);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_disjoint_query_paths(),
            true
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_replication_factor(),
            None
        );
        assert_eq!(
            config.settings.network.routing.get_protocol_names(),
            vec!["/kore/tell/1.0.0", "/kore/reqres/1.0.0", "/kore/routing/1.0.0", "/ipfs/ping/1.0.0", "/ipfs/id/push/1.0.0", "/ipfs/id/id/1.0.0"]
        );
        assert_eq!(
            config.settings.network.tell.get_message_timeout(),
            Duration::from_secs(58)
        );
        assert_eq!(
            config.settings.network.tell.get_max_concurrent_streams(),
            166
        );

        #[cfg(feature = "leveldb")]
        assert_eq!(
            config.db,
            DbSettings::LevelDB("examples/leveldb".to_owned())
        );
        #[cfg(feature = "sqlite")]
        assert_eq!(
            config.db,
            DbSettings::Sqlite("examples/sqlitedb".to_owned())
        );
        assert_eq!(config.keys_path, "examples/keys".to_owned());
        assert_eq!(config.prometheus, "0.0.0.0:3050".to_owned());
    }

    #[test]
    #[serial]
    fn test_toml_mix_env() {
        let content = r#"
        [kore.network]
        user_agent = "Kore3.0"
        port_reuse = true
        external_addresses = ["/ip4/90.1.0.60/tcp/50000","/ip4/90.1.0.61/tcp/50000"]
        
        [kore.network.tell]
        message_timeout_secs = 58
        max_concurrent_streams = 166
        
        [kore.network.routing]
        boot_nodes = ["/ip4/172.17.0.1/tcp/50000_/ip4/127.0.0.1/tcp/60001/p2p/12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B", "/ip4/11.11.0.11/tcp/10000_/ip4/12.22.33.44/tcp/55511/p2p/12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze"]
        dht_random_walk = false
        discovery_only_if_under_num = 55
        allow_non_globals_in_dht = true
        allow_private_ip = true
        
        [kore.node]
        replication_factor = 0.555
        timeout = 30
        passvotation = 50
        smartcontracts_directory = "./fake_route"        
        "#;
        std::env::set_var("KORE_NETWORK_USER_AGENT", "Kore2.0");
        std::env::set_var("KORE_NETWORK_NODE_TYPE", "Addressable");
        std::env::set_var(
            "KORE_NETWORK_LISTEN_ADDRESSES",
            "/ip4/127.0.0.1/tcp/50000,/ip4/127.0.0.1/tcp/50001,/ip4/127.0.0.1/tcp/50002",
        );
        std::env::set_var("KORE_NETWORK_ROUTING_ENABLE_MDNS", "false");
        std::env::set_var(
            "KORE_NETWORK_ROUTING_KADEMLIA_DISJOINT_QUERY_PATHS",
            "false",
        );
        std::env::set_var("KORE_NETWORK_ROUTING_KADEMLIA_REPLICATION_FACTOR", "30");
        std::env::set_var(
            "KORE_NETWORK_ROUTING_PROTOCOL_NAMES",
            "/kore/routing/2.2.2,/kore/routing/1.1.1",
        );
        std::env::set_var("KORE_NODE_KEY_DERIVATOR", "Secp256k1");
        std::env::set_var("KORE_NODE_DIGEST_DERIVATOR", "Blake3_512");
        std::env::set_var("KORE_DB_PATH", "./fake/db/path");
        std::env::set_var("KORE_KEYS_PATH", "./fake/keys/path");
        std::env::set_var("KORE_PROMETHEUS", "10.0.0.0:3030");

        let temp_dir = TempDir::new().unwrap();
        let temp_file_path = temp_dir.path().join("config.toml");
        std::fs::write(&temp_file_path, content.to_string().as_bytes()).unwrap();

        let config = build_config(true, temp_file_path.to_str().unwrap());

        let boot_nodes = vec![
            RoutingNode {
                address: vec![
                    "/ip4/172.17.0.1/tcp/50000".to_owned(),
                    "/ip4/127.0.0.1/tcp/60001".to_owned(),
                ],
                peer_id: "12D3KooWLXexpg81PjdjnrhmHUxN7U5EtfXJgr9cahei1SJ9Ub3B".to_owned(),
            },
            RoutingNode {
                address: vec![
                    "/ip4/11.11.0.11/tcp/10000".to_owned(),
                    "/ip4/12.22.33.44/tcp/55511".to_owned(),
                ],
                peer_id: "12D3KooWRS3QVwqBtNp7rUCG4SF3nBrinQqJYC1N5qc1Wdr4jrze".to_owned(),
            },
        ];

        assert_eq!(config.settings.network.port_reuse, true);
        assert_eq!(config.settings.network.user_agent, "Kore3.0");
        assert_eq!(config.settings.network.node_type, NodeType::Addressable);
        assert_eq!(
            config.settings.network.listen_addresses,
            vec![
                "/ip4/127.0.0.1/tcp/50000".to_owned(),
                "/ip4/127.0.0.1/tcp/50001".to_owned(),
                "/ip4/127.0.0.1/tcp/50002".to_owned()
            ]
        );
        assert_eq!(
            config.settings.network.external_addresses,
            vec![
                "/ip4/90.1.0.60/tcp/50000".to_owned(),
                "/ip4/90.1.0.61/tcp/50000".to_owned(),
            ]
        );
        assert_eq!(config.settings.node.key_derivator, KeyDerivator::Secp256k1);
        assert_eq!(
            config.settings.node.digest_derivator,
            DigestDerivator::Blake3_512
        );
        assert_eq!(config.settings.node.replication_factor, 0.555f64);
        assert_eq!(config.settings.node.timeout, 30);
        assert_eq!(config.settings.node.passvotation, 50);
        assert_eq!(
            config.settings.node.smartcontracts_directory,
            "./fake_route"
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].peer_id,
            boot_nodes[0].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[0].address,
            boot_nodes[0].address
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].peer_id,
            boot_nodes[1].peer_id
        );
        assert_eq!(
            config.settings.network.routing.boot_nodes()[1].address,
            boot_nodes[1].address
        );

        assert_eq!(config.settings.network.routing.get_dht_random_walk(), false);
        assert_eq!(config.settings.network.routing.get_discovery_limit(), 55);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_allow_non_globals_in_dht(),
            true
        );
        assert_eq!(config.settings.network.routing.get_allow_private_ip(), true);
        assert_eq!(config.settings.network.routing.get_mdns(), false);
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_disjoint_query_paths(),
            false
        );
        assert_eq!(
            config
                .settings
                .network
                .routing
                .get_kademlia_replication_factor(),
            Some(NonZeroUsize::new(30).unwrap())
        );
        assert_eq!(
            config.settings.network.routing.get_protocol_names(),
            vec![
                "/kore/routing/2.2.2".to_owned(),
                "/kore/routing/1.1.1".to_owned()
            ]
        );
        assert_eq!(
            config.settings.network.tell.get_message_timeout(),
            Duration::from_secs(58)
        );
        assert_eq!(
            config.settings.network.tell.get_max_concurrent_streams(),
            166
        );

        #[cfg(feature = "leveldb")]
        assert_eq!(config.db, DbSettings::LevelDB("./fake/db/path".to_owned()));
        #[cfg(feature = "sqlite")]
        assert_eq!(config.db, DbSettings::Sqlite("./fake/db/path".to_owned()));
        assert_eq!(config.keys_path, "./fake/keys/path".to_owned());
        assert_eq!(config.prometheus, "10.0.0.0:3030".to_owned());

        std::env::remove_var("KORE_PROMETHEUS");
        std::env::remove_var("KORE_NETWORK_USER_AGENT");
        std::env::remove_var("KORE_NETWORK_NODE_TYPE");
        std::env::remove_var("KORE_NETWORK_LISTEN_ADDRESSES");
        std::env::remove_var("KORE_NETWORK_ROUTING_ENABLE_MDNS");
        std::env::remove_var("KORE_NETWORK_ROUTING_KADEMLIA_DISJOINT_QUERY_PATHS");
        std::env::remove_var("KORE_NETWORK_ROUTING_KADEMLIA_REPLICATION_FACTOR");
        std::env::remove_var("KORE_NETWORK_ROUTING_PROTOCOL_NAMES");
        std::env::remove_var("KORE_NODE_KEY_DERIVATOR");
        std::env::remove_var("KORE_NODE_DIGEST_DERIVATOR");
        std::env::remove_var("KORE_DB_PATH");
        std::env::remove_var("KORE_KEYS_PATH");
    }
}
