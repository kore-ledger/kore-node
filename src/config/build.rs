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

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use super::build_config;

    #[test]
    fn test_env() {
        build_config(true, "");
    }

    #[test]
    fn test_json() {
        let content = r#"
            {
            "kore": {
              "network": {
                  "user_agent": "pepe",
                  "node_type": "Bootstrap",
                  "listen_addresses": [""],
                  "tell": {
                    "message_timeout_secs": 1,
                    "max_concurrent_streams": 10
                  },
                  "routing": {
                    "boot_nodes": "",
                    "dht_random_walk": true,
                    "discovery_only_if_under_num": 100,
                    "allow_non_globals_in_dht": true,
                    "allow_private_ip": true,
                    "enable_mdns": true,
                    "kademlia_disjoint_query_paths": true,
                    "kademlia_replication_factor": 1,
                    "protocol_names": [""]
                  },
                  "port_reuse": false
              },
              "node": {
                "key_derivator": "Ed25519",
                "digest_derivator": "Blake3_256",
                "replication_factor": 0,
                "timeout": 0,
                "passvotation": 1,
                "smartcontracts_directory": "./contracts"
              },
              "db_path": "./db",
              "keys_path": "./keys"
            }
          }"#;
        let temp_dir = TempDir::new().unwrap();
        let temp_file_path = temp_dir.path().join("config.json");
        std::fs::write(&temp_file_path, content.to_string().as_bytes()).unwrap();

        build_config(false, temp_file_path.to_str().unwrap());
    }

    #[test]
    fn yaml() {
        let content = r#"
        kore:
            network:
                user_agent: ""
                node_type: "Bootstrap"
                listen_addresses:
                - ""
                tell:
                    message_timeout_secs: 1
                    max_concurrent_streams: 10
                routing:
                    boot_nodes: ""
                    dht_random_walk: true
                    discovery_only_if_under_num: 100
                    allow_non_globals_in_dht: true
                    allow_private_ip: true
                    enable_mdns: true
                    kademlia_disjoint_query_paths: true
                    kademlia_replication_factor: 1
                    protocol_names:
                    - ""
                port_reuse: false
            node:
                key_derivator: "Ed25519"
                digest_derivator: "Blake3_256"
                replication_factor: 0
                timeout: 0
                passvotation: 1
                smartcontracts_directory: "./contracts"
            db_path: "./db"
            keys_path: "./keys"
        "#;
        let temp_dir = TempDir::new().unwrap();
        let temp_file_path = temp_dir.path().join("config.yaml");
        std::fs::write(&temp_file_path, content.to_string().as_bytes()).unwrap();

        build_config(false, temp_file_path.to_str().unwrap());
    }

    #[test]
    fn toml() {
        let content = r#"
        [kore.network]
        user_agent = ""
        node_type = "Bootstrap"
        listen_addresses = [""]
        port_reuse = false

        [kore.network.tell]
        message_timeout_secs = 1
        max_concurrent_streams = 10

        [kore.network.routing]
        boot_nodes = ""
        dht_random_walk = true
        discovery_only_if_under_num = 100
        allow_non_globals_in_dht = true
        allow_private_ip = true
        enable_mdns = true
        kademlia_disjoint_query_paths = true
        kademlia_replication_factor = 1
        protocol_names = [""]

        [kore.node]
        key_derivator = "Ed25519"
        digest_derivator = "Blake3_256"
        replication_factor = 0
        timeout = 0
        passvotation = 1
        smartcontracts_directory = "./contracts"

        [kore]
        db_path = "./db"
        keys_path = "./keys"
        "#;
        let temp_dir = TempDir::new().unwrap();
        let temp_file_path = temp_dir.path().join("config.toml");
        std::fs::write(&temp_file_path, content.to_string().as_bytes()).unwrap();

        build_config(false, temp_file_path.to_str().unwrap());
    }
}
