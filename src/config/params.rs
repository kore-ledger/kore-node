use std::{time::Duration, vec};

use kore_base::{NodeSettings, NodeType, RoutingNode};
use serde::{Deserialize, Deserializer};

use crate::settings::{DbSettings, KoreSettings};

#[derive(Debug, Deserialize, Default)]
pub struct Params {
    kore: KoreParams,
}

impl From<Params> for KoreSettings {
    fn from(params: Params) -> Self {
		let tell = kore_base::TellConfig::default();
		let tell = tell.with_max_concurrent_streams(params.kore.network.tell.max_concurrent_streams);
		let tell = tell.with_message_timeout(params.kore.network.tell.message_timeout_secs);

		let routing = kore_base::RoutingConfig::new( params.kore.network.routing.boot_nodes);
		let routing = routing.with_dht_random_walk(params.kore.network.routing.dht_random_walk);
		let routing = routing.with_discovery_limit(params.kore.network.routing.discovery_only_if_under_num);
		let routing = routing.with_allow_non_globals_in_dht(params.kore.network.routing.allow_non_globals_in_dht);
		let routing = routing.with_allow_private_ip(params.kore.network.routing.allow_private_ip);
		let routing = routing.with_mdns(params.kore.network.routing.enable_mdns);
		let routing = routing.with_kademlia_disjoint_query_paths(params.kore.network.routing.kademlia_disjoint_query_paths);
		let routing = routing.with_kademlia_replication_factor(params.kore.network.routing.kademlia_replication_factor);

        Self {
            db: params.kore.db_path,
            keys_path: params.kore.keys_path,
            settings: kore_base::Settings {
                network: kore_base::NetworkConfig {
                    user_agent: params.kore.network.user_agent,
                    node_type: params.kore.network.node_type,
                    listen_addresses: params.kore.network.listen_addresses,
                    tell,
                    routing
                },
                node: NodeSettings {
					digest_derivator: kore_base::DigestDerivator::from(params.kore.node.digest_derivator),
					key_derivator: kore_base::KeyDerivator::from(params.kore.node.key_derivator),
					replication_factor: params.kore.node.replication_factor,
					secret_key: String::default(),
					smartcontracts_directory: params.kore.node.smartcontracts_directory,
					timeout: params.kore.node.timeout,
					passvotation: params.kore.node.passvotation
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
        }
    }
}

#[derive(Debug, Deserialize)]
struct TellParams {
    #[serde(default = "default_message_timeout_secs", deserialize_with = "deserialize_message_timeout_secs")]
    message_timeout_secs: Duration,
    #[serde(default = "default_max_concurrent_streams")]
    max_concurrent_streams: usize,
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
        }
    }
}

fn deserialize_boot_nodes<'de, D>(deserializer: D) -> Result<Vec<RoutingNode>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Vec<String> = Vec::deserialize(deserializer)?;
    Ok(v.into_iter()
        .map(|element| {
            if let Some(pos) = element.find("/p2p/") {
                // La parte antes de "/p2p/" (no incluye "/p2p/")
                let address = &element[..pos].to_owned();
                // La parte después de "/p2p/"
                let peer_id = &element[pos + 5..].to_owned();
                RoutingNode {
                    address: address.clone(),
                    peer_id: peer_id.clone(),
                }
            } else {
                RoutingNode {
                    address: String::default(),
                    peer_id: String::default(),
                }
            }
        })
        .collect())
}

fn default_true() -> bool {
    true
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

#[derive(Deserialize, Debug)]
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
            KeyDerivatorParams::Secp256k1 => kore_base::KeyDerivator::Secp256k1
        }
    }
}

/// Key derivators availables
#[derive(Deserialize, Debug)]
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
