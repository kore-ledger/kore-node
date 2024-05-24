use std::{time::Duration, vec};

use kore_base::{NodeSettings, NodeType, RoutingNode};
use serde::{Deserialize, Deserializer};

use crate::settings::{DbSettings, KoreSettings};

#[derive(Debug, Deserialize, Default)]
pub struct Params {
    kore: KoreParams,
}

impl Params {
    pub fn from_env() -> Self {
        Self {
            kore: KoreParams::from_env("KORE"),
        }
    }
}

impl From<Params> for KoreSettings {
    fn from(params: Params) -> Self {
        let tell = kore_base::TellConfig::new(
            params.kore.network.tell.message_timeout_secs,
            params.kore.network.tell.max_concurrent_streams,
        );

        let routing = kore_base::RoutingConfig::new(params.kore.network.routing.boot_nodes)
            .with_dht_random_walk(params.kore.network.routing.dht_random_walk)
            .with_discovery_limit(params.kore.network.routing.discovery_only_if_under_num)
            .with_allow_non_globals_in_dht(params.kore.network.routing.allow_non_globals_in_dht)
            .with_allow_private_ip(params.kore.network.routing.allow_private_ip)
            .with_mdns(params.kore.network.routing.enable_mdns)
            .with_kademlia_disjoint_query_paths(
                params.kore.network.routing.kademlia_disjoint_query_paths,
            )
            .with_kademlia_replication_factor(
                params.kore.network.routing.kademlia_replication_factor,
            )
            .set_all_protocols(params.kore.network.routing.protocol_names);

        Self {
            db: params.kore.db_path,
            keys_path: params.kore.keys_path,
            settings: kore_base::Settings {
                network: kore_base::NetworkConfig {
                    user_agent: params.kore.network.user_agent,
                    node_type: params.kore.network.node_type,
                    listen_addresses: params.kore.network.listen_addresses,
                    tell,
                    routing,
                    port_reuse: params.kore.network.port_reuse,
                },
                node: NodeSettings {
                    digest_derivator: kore_base::DigestDerivator::from(
                        params.kore.node.digest_derivator,
                    ),
                    key_derivator: kore_base::KeyDerivator::from(params.kore.node.key_derivator),
                    replication_factor: params.kore.node.replication_factor,
                    secret_key: String::default(),
                    smartcontracts_directory: params.kore.node.smartcontracts_directory,
                    timeout: params.kore.node.timeout,
                    passvotation: params.kore.node.passvotation,
                },
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct KoreParams {
    #[serde(default)]
    network: NetworkParams,
    #[serde(default)]
    node: NodeParams,
    #[serde(default = "default_db_path", deserialize_with = "deserialize_db_path")]
    db_path: DbSettings,
    #[serde(default = "default_keys_path")]
    keys_path: String,
}

impl KoreParams {
    fn from_env(parent: &str) -> Self {
        let mut config = config::Config::builder();
        config = config.add_source(config::Environment::with_prefix(&parent));

        let config = config
            .build()
            .map_err(|e| {
                println!("Error building config: {}", e);
            })
            .unwrap();

        let kore_params: KoreParams = config
            .try_deserialize()
            .map_err(|e| {
                println!("Error try deserialize config: {}", e);
            })
            .unwrap();

        Self {
            network: NetworkParams::from_env(&format!("{parent}_")),
            node: NodeParams::from_env(&format!("{parent}_")),
            db_path: kore_params.db_path,
            keys_path: kore_params.keys_path,
        }
    }
}

fn deserialize_db_path<'de, D>(deserializer: D) -> Result<DbSettings, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    #[cfg(feature = "leveldb")]
    return Ok(DbSettings::LevelDB(s));
    #[cfg(feature = "sqlite")]
    return Ok(DbSettings::Sqlite(s));
    #[cfg(feature = "cassandra")]
    return DbSettings::Cassandra;
}

impl Default for KoreParams {
    fn default() -> Self {
        Self {
            network: NetworkParams::default(),
            node: NodeParams::default(),
            db_path: default_db_path(),
            keys_path: default_keys_path(),
        }
    }
}

fn default_db_path() -> DbSettings {
    #[cfg(feature = "leveldb")]
    return DbSettings::LevelDB("examples/leveldb".to_owned());
    #[cfg(feature = "sqlite")]
    return DbSettings::Sqlite("examples/sqlitedb/database".to_owned());
    #[cfg(feature = "cassandra")]
    return DbSettings::Cassandra;
}

fn default_keys_path() -> String {
    "examples/keys".to_owned()
}

#[derive(Debug, Deserialize)]
struct NetworkParams {
    #[serde(default = "default_user_agent")]
    user_agent: String,
    #[serde(default = "default_node_type")]
    node_type: NodeType,
    #[serde(default)]
    listen_addresses: Vec<String>,
    #[serde(default)]
    tell: TellParams,
    #[serde(default)]
    routing: RoutingParams,
    #[serde(default)]
    port_reuse: bool,
}

impl NetworkParams {
    fn from_env(parent: &str) -> Self {
        let mut config = config::Config::builder();
        config = config.add_source(
            config::Environment::with_prefix(&format!("{parent}NETWORK"))
                .list_separator(",")
                .with_list_parse_key("listen_addresses")
                .try_parsing(true),
        );

        let config = config
            .build()
            .map_err(|e| {
                println!("Error building config: {}", e);
            })
            .unwrap();

        let network: NetworkParams = config
            .try_deserialize()
            .map_err(|e| {
                println!("Error try deserialize config: {}", e);
            })
            .unwrap();

        let parent = &format!("{parent}NETWORK_");
        Self {
            user_agent: network.user_agent,
            node_type: network.node_type,
            listen_addresses: network.listen_addresses,
            tell: TellParams::from_env(parent),
            routing: RoutingParams::from_env(parent),
            port_reuse: network.port_reuse,
        }
    }
}

fn default_user_agent() -> String {
    "kore-node".to_owned()
}

fn default_node_type() -> NodeType {
    NodeType::Bootstrap
}

impl Default for NetworkParams {
    fn default() -> Self {
        Self {
            user_agent: default_user_agent(),
            node_type: default_node_type(),
            listen_addresses: vec![],
            tell: TellParams::default(),
            routing: RoutingParams::default(),
            port_reuse: false,
        }
    }
}

#[derive(Debug, Deserialize)]
struct TellParams {
    #[serde(
        default = "default_message_timeout_secs",
        deserialize_with = "deserialize_message_timeout_secs"
    )]
    message_timeout_secs: Duration,
    #[serde(default = "default_max_concurrent_streams")]
    max_concurrent_streams: usize,
}

impl TellParams {
    fn from_env(parent: &str) -> Self {
        let mut config = config::Config::builder();
        config = config.add_source(config::Environment::with_prefix(&format!("{parent}TELL")));

        let config = config
            .build()
            .map_err(|e| {
                println!("Error building config: {}", e);
            })
            .unwrap();

        config
            .try_deserialize()
            .map_err(|e| {
                println!("Error try deserialize config: {}", e);
            })
            .unwrap()
    }
}

impl Default for TellParams {
    fn default() -> Self {
        Self {
            message_timeout_secs: default_message_timeout_secs(),
            max_concurrent_streams: default_max_concurrent_streams(),
        }
    }
}

fn deserialize_message_timeout_secs<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let u: u64 = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(u))
}

fn default_max_concurrent_streams() -> usize {
    100
}

fn default_message_timeout_secs() -> Duration {
    Duration::from_secs(10)
}

#[derive(Debug, Deserialize)]
struct RoutingParams {
    #[serde(default, deserialize_with = "deserialize_boot_nodes")]
    boot_nodes: Vec<RoutingNode>,
    #[serde(default = "default_true")]
    dht_random_walk: bool,
    #[serde(default = "default_discovery_only_if_under_num")]
    discovery_only_if_under_num: u64,
    #[serde(default)]
    allow_non_globals_in_dht: bool,
    #[serde(default)]
    allow_private_ip: bool,
    #[serde(default = "default_true")]
    enable_mdns: bool,
    #[serde(default = "default_true")]
    kademlia_disjoint_query_paths: bool,
    #[serde(default)]
    kademlia_replication_factor: usize,
    #[serde(default = "default_protocol_name")]
    protocol_names: Vec<String>,
}

impl RoutingParams {
    fn from_env(parent: &str) -> Self {
        let mut config = config::Config::builder();
        config = config.add_source(
            config::Environment::with_prefix(&format!("{parent}ROUTING"))
                .list_separator(",")
                .with_list_parse_key("protocol_names")
                .try_parsing(true),
        );

        let config = config
            .build()
            .map_err(|e| {
                println!("Error building config: {}", e);
            })
            .unwrap();

        config
            .try_deserialize()
            .map_err(|e| {
                println!("Error try deserialize config: {}", e);
            })
            .unwrap()
    }
}

impl Default for RoutingParams {
    fn default() -> Self {
        Self {
            boot_nodes: vec![],
            dht_random_walk: default_true(),
            discovery_only_if_under_num: default_discovery_only_if_under_num(),
            allow_non_globals_in_dht: false,
            allow_private_ip: false,
            enable_mdns: default_true(),
            kademlia_disjoint_query_paths: default_true(),
            kademlia_replication_factor: 0,
            protocol_names: default_protocol_name(),
        }
    }
}

fn deserialize_boot_nodes<'de, D>(deserializer: D) -> Result<Vec<RoutingNode>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    let v: Vec<&str> = s.split(',').collect();

    Ok(v.into_iter()
        .map(|element| {
            if let Some(pos) = element.find("/p2p/") {
                // La parte antes de "/p2p/" (no incluye "/p2p/")
                let address = &element[..pos].to_owned();
                // La parte después de "/p2p/"
                let peer_id = &element[pos + 5..].to_owned();
                RoutingNode {
                    address: address.split('_').map(|e| e.to_owned()).collect(),
                    peer_id: peer_id.clone(),
                }
            } else {
                RoutingNode {
                    address: vec![],
                    peer_id: String::default(),
                }
            }
        })
        .collect())
}

fn default_true() -> bool {
    true
}

fn default_protocol_name() -> Vec<String> {
    vec!["/kore/routing/1.0.0".to_owned()]
}

fn default_discovery_only_if_under_num() -> u64 {
    std::u64::MAX
}

#[derive(Debug, Deserialize)]
struct NodeParams {
    #[serde(default)]
    key_derivator: KeyDerivatorParams,
    #[serde(default)]
    digest_derivator: DigestDerivatorParams,
    #[serde(default = "default_replication_factor")]
    replication_factor: f64,
    #[serde(default = "default_timeout")]
    timeout: u32,
    #[serde(default)]
    passvotation: u8,
    #[serde(default = "default_smartcontracts_directory")]
    smartcontracts_directory: String,
}

impl NodeParams {
    fn from_env(parent: &str) -> Self {
        let mut config = config::Config::builder();
        config = config.add_source(config::Environment::with_prefix(&format!("{parent}NODE")));

        let config = config
            .build()
            .map_err(|e| {
                println!("Error building config: {}", e);
            })
            .unwrap();

        config
            .try_deserialize()
            .map_err(|e| {
                println!("Error try deserialize config: {}", e);
            })
            .unwrap()
    }
}

impl Default for NodeParams {
    fn default() -> Self {
        Self {
            key_derivator: KeyDerivatorParams::default(),
            digest_derivator: DigestDerivatorParams::default(),
            replication_factor: default_replication_factor(),
            timeout: default_timeout(),
            passvotation: 0,
            smartcontracts_directory: default_smartcontracts_directory(),
        }
    }
}

fn default_replication_factor() -> f64 {
    0.25f64
}

fn default_timeout() -> u32 {
    3000u32
}

fn default_smartcontracts_directory() -> String {
    "./contracts".to_owned()
}

#[derive(Deserialize, Debug, PartialEq)]
enum KeyDerivatorParams {
    /// The Ed25519 key derivator.
    Ed25519,
    /// The Secp256k1 key derivator.
    Secp256k1,
}

impl From<KeyDerivatorParams> for kore_base::KeyDerivator {
    fn from(val: KeyDerivatorParams) -> Self {
        match val {
            KeyDerivatorParams::Ed25519 => kore_base::KeyDerivator::Ed25519,
            KeyDerivatorParams::Secp256k1 => kore_base::KeyDerivator::Secp256k1,
        }
    }
}

/// Key derivators availables
#[derive(Deserialize, Debug, PartialEq)]
pub enum DigestDerivatorParams {
    Blake3_256,
    Blake3_512,
    SHA2_256,
    SHA2_512,
    SHA3_256,
    SHA3_512,
}

impl From<DigestDerivatorParams> for kore_base::DigestDerivator {
    fn from(val: DigestDerivatorParams) -> Self {
        match val {
            DigestDerivatorParams::Blake3_256 => kore_base::DigestDerivator::Blake3_256,
            DigestDerivatorParams::Blake3_512 => kore_base::DigestDerivator::Blake3_512,
            DigestDerivatorParams::SHA2_256 => kore_base::DigestDerivator::SHA2_256,
            DigestDerivatorParams::SHA2_512 => kore_base::DigestDerivator::SHA2_512,
            DigestDerivatorParams::SHA3_256 => kore_base::DigestDerivator::SHA3_256,
            DigestDerivatorParams::SHA3_512 => kore_base::DigestDerivator::SHA3_512,
        }
    }
}

impl Default for KeyDerivatorParams {
    fn default() -> Self {
        Self::Ed25519
    }
}

impl Default for DigestDerivatorParams {
    fn default() -> Self {
        Self::Blake3_256
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use kore_base::{NodeType, RoutingNode};
    use serial_test::serial;

    use crate::{config::params::{
        DigestDerivatorParams, KeyDerivatorParams, KoreParams, NetworkParams, NodeParams, Params, RoutingParams
    }, settings::DbSettings};

    use super::TellParams;


    #[test]
    #[serial]
    fn test_from_env_tell_default() {
        let tell = TellParams::from_env("KORE_NETWORK_");

        assert_eq!(tell.message_timeout_secs, Duration::from_secs(10));
        assert_eq!(tell.max_concurrent_streams, 100);
    }

    #[test]
    #[serial]
    fn test_from_env_routing_default() {
        let routing = RoutingParams::from_env("KORE_NETWORK_");
        println!("{:?}", routing.boot_nodes);
        assert!(routing.boot_nodes.is_empty());

        assert_eq!(routing.dht_random_walk, true);
        assert_eq!(routing.discovery_only_if_under_num, std::u64::MAX);
        assert_eq!(routing.allow_non_globals_in_dht, false);
        assert_eq!(routing.allow_private_ip, false);
        assert_eq!(routing.enable_mdns, true);
        assert_eq!(routing.kademlia_disjoint_query_paths, true);
        assert_eq!(routing.kademlia_replication_factor, 0);
        assert_eq!(routing.protocol_names[0], "/kore/routing/1.0.0".to_owned());
    }

    #[test]
    #[serial]
    fn test_from_env_node_default() {
        let node: NodeParams = NodeParams::from_env("KORE_");

        assert_eq!(node.key_derivator, KeyDerivatorParams::Ed25519);
        assert_eq!(node.digest_derivator, DigestDerivatorParams::Blake3_256);
        assert_eq!(node.replication_factor, 0.25f64);
        assert_eq!(node.timeout, 3000u32);
        assert_eq!(node.passvotation, 0);
        assert_eq!(node.smartcontracts_directory, "./contracts");
    }

    #[test]
    #[serial]
    fn test_from_env_network_default() {
        let network = NetworkParams::from_env("KORE_");

        assert_eq!(network.port_reuse, false);
        assert_eq!(network.user_agent, "kore-node");
        assert_eq!(network.node_type, NodeType::Bootstrap);
        assert!(network.listen_addresses.is_empty());
    }

    #[test]
    #[serial]
    fn test_from_env_kore_params_default() {
        let kore = KoreParams::from_env("KORE");

        #[cfg(feature = "leveldb")]
        assert_eq!(kore.db_path, return DbSettings::LevelDB("examples/leveldb".to_owned()););
        #[cfg(feature = "sqlite")]
        assert_eq!(kore.db_path, DbSettings::Sqlite("examples/sqlitedb/database".to_owned()));
        assert_eq!(kore.keys_path, "examples/keys".to_owned());
    }

    #[test]
    #[serial]
    fn test_from_env_tell_values() {
        std::env::set_var("KORE_NETWORK_TELL_MESSAGE_TIMEOUT_SECS", "55");
        std::env::set_var("KORE_NETWORK_TELL_MAX_CONCURRENT_STREAMS", "166");

        let tell = TellParams::from_env("KORE_NETWORK_");

        assert_eq!(tell.message_timeout_secs, Duration::from_secs(55));
        assert_eq!(tell.max_concurrent_streams, 166);
    }

    #[test]
    #[serial]
    fn test_from_env_routing_values() {
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
        std::env::set_var("KORE_NETWORK_ROUTINGPORT_REUSE", "true");

        let routing = RoutingParams::from_env("KORE_NETWORK_");
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

        assert_eq!(routing.boot_nodes[0].peer_id, boot_nodes[0].peer_id);
        assert_eq!(routing.boot_nodes[0].address, boot_nodes[0].address);
        assert_eq!(routing.boot_nodes[1].peer_id, boot_nodes[1].peer_id);
        assert_eq!(routing.boot_nodes[1].address, boot_nodes[1].address);

        assert_eq!(routing.dht_random_walk, false);
        assert_eq!(routing.discovery_only_if_under_num, 55);
        assert_eq!(routing.allow_non_globals_in_dht, true);
        assert_eq!(routing.allow_private_ip, true);
        assert_eq!(routing.enable_mdns, false);
        assert_eq!(routing.kademlia_disjoint_query_paths, false);
        assert_eq!(routing.kademlia_replication_factor, 30);
        assert_eq!(
            routing.protocol_names,
            vec![
                "/kore/routing/2.2.2".to_owned(),
                "/kore/routing/1.1.1".to_owned()
            ]
        );
    }

    #[test]
    #[serial]
    fn test_from_env_node_values() {
        std::env::set_var("KORE_NODE_KEY_DERIVATOR", "Secp256k1");
        std::env::set_var("KORE_NODE_DIGEST_DERIVATOR", "Blake3_512");
        std::env::set_var("KORE_NODE_REPLICATION_FACTOR", "0.555");
        std::env::set_var("KORE_NODE_TIMEOUT", "30");
        std::env::set_var("KORE_NODE_PASSVOTATION", "50");
        std::env::set_var("KORE_NODE_SMARTCONTRACTS_DIRECTORY", "./fake_route");

        let node = NodeParams::from_env("KORE_");

        assert_eq!(node.key_derivator, KeyDerivatorParams::Secp256k1);
        assert_eq!(node.digest_derivator, DigestDerivatorParams::Blake3_512);
        assert_eq!(node.replication_factor, 0.555f64);
        assert_eq!(node.timeout, 30);
        assert_eq!(node.passvotation, 50);
        assert_eq!(node.smartcontracts_directory, "./fake_route");
    }



    #[test]
    #[serial]
    fn test_from_env_network_values() {
        std::env::set_var("KORE_NETWORK_PORT_REUSE", "true");
        std::env::set_var("KORE_NETWORK_USER_AGENT", "Kore2.0");
        std::env::set_var("KORE_NETWORK_NODE_TYPE", "Addressable");
        std::env::set_var(
            "KORE_NETWORK_LISTEN_ADDRESSES",
            "/ip4/127.0.0.1/tcp/50000,/ip4/127.0.0.1/tcp/50001,/ip4/127.0.0.1/tcp/50002",
        );
        let network = NetworkParams::from_env("KORE_");

        assert_eq!(network.port_reuse, true);
        assert_eq!(network.user_agent, "Kore2.0");
        assert_eq!(network.node_type, NodeType::Addressable);
        assert_eq!(
            network.listen_addresses,
            vec![
                "/ip4/127.0.0.1/tcp/50000".to_owned(),
                "/ip4/127.0.0.1/tcp/50001".to_owned(),
                "/ip4/127.0.0.1/tcp/50002".to_owned()
            ]
        );
    }



    #[test]
    #[serial]
    fn test_from_env_kore_params_value() {
        std::env::set_var("KORE_DB_PATH", "./fake/db/path");
        std::env::set_var("KORE_KEYS_PATH", "./fake/keys/path");

        let kore = KoreParams::from_env("KORE");

        #[cfg(feature = "leveldb")]
        assert_eq!(kore.db_path, DbSettings::LevelDB("./fake/db/path".to_owned()));
        #[cfg(feature = "sqlite")]
        assert_eq!(kore.db_path, DbSettings::Sqlite("./fake/db/path".to_owned()));
        assert_eq!(kore.keys_path, "./fake/keys/path".to_owned());
    }


    #[test]
    #[serial]
    fn test_from_env_params_value() {
        std::env::set_var("KORE_NETWORK_TELL_MESSAGE_TIMEOUT_SECS", "55");
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
        std::env::set_var("KORE_NETWORK_ROUTINGPORT_REUSE", "true");
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


        let params = Params::from_env();
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

        assert_eq!(params.kore.network.port_reuse, true);
        assert_eq!(params.kore.network.user_agent, "Kore2.0");
        assert_eq!(params.kore.network.node_type, NodeType::Addressable);
        assert_eq!(
            params.kore.network.listen_addresses,
            vec![
                "/ip4/127.0.0.1/tcp/50000".to_owned(),
                "/ip4/127.0.0.1/tcp/50001".to_owned(),
                "/ip4/127.0.0.1/tcp/50002".to_owned()
            ]
        );
        assert_eq!(params.kore.node.key_derivator, KeyDerivatorParams::Secp256k1);
        assert_eq!(params.kore.node.digest_derivator, DigestDerivatorParams::Blake3_512);
        assert_eq!(params.kore.node.replication_factor, 0.555f64);
        assert_eq!(params.kore.node.timeout, 30);
        assert_eq!(params.kore.node.passvotation, 50);
        assert_eq!(params.kore.node.smartcontracts_directory, "./fake_route");
        assert_eq!(params.kore.network.routing.boot_nodes[0].peer_id, boot_nodes[0].peer_id);
        assert_eq!(params.kore.network.routing.boot_nodes[0].address, boot_nodes[0].address);
        assert_eq!(params.kore.network.routing.boot_nodes[1].peer_id, boot_nodes[1].peer_id);
        assert_eq!(params.kore.network.routing.boot_nodes[1].address, boot_nodes[1].address);

        assert_eq!(params.kore.network.routing.dht_random_walk, false);
        assert_eq!(params.kore.network.routing.discovery_only_if_under_num, 55);
        assert_eq!(params.kore.network.routing.allow_non_globals_in_dht, true);
        assert_eq!(params.kore.network.routing.allow_private_ip, true);
        assert_eq!(params.kore.network.routing.enable_mdns, false);
        assert_eq!(params.kore.network.routing.kademlia_disjoint_query_paths, false);
        assert_eq!(params.kore.network.routing.kademlia_replication_factor, 30);
        assert_eq!(
            params.kore.network.routing.protocol_names,
            vec![
                "/kore/routing/2.2.2".to_owned(),
                "/kore/routing/1.1.1".to_owned()
            ]
        );
        assert_eq!(params.kore.network.tell.message_timeout_secs, Duration::from_secs(55));
        assert_eq!(params.kore.network.tell.max_concurrent_streams, 166);
    }

}
