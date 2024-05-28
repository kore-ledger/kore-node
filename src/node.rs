// Copyright 2024 Kore Ledger
// SPDX-License-Identifier: AGPL-3.0-or-later
use prometheus_client::registry::Registry;

#[cfg(feature = "leveldb")]
use std::path::Path;

use crate::{error::NodeError, prometheus::server::run_prometheus, settings::{DbSettings, KoreSettings}, utils::node_key_pair, KoreApi};

#[cfg(feature = "leveldb")]
use crate::database::leveldb::{LeveldbManager, open_db};
#[cfg(feature = "sqlite")]
use crate::database::sqlite::SqliteManager;

use kore_base::Node;

use futures::Future;
use tokio_util::sync::CancellationToken;
use async_trait::async_trait;

/// Kore node trait.
#[async_trait]
pub trait KoreNode {
    /// Get the Kore API.
    /// 
    /// # Returns
    /// 
    /// * `&KoreApi` - Kore API
    /// 
    fn api(&self) -> &KoreApi;
    /// Bind the node to the provided shutdown signal.
    /// 
    /// # Arguments
    /// 
    /// * `shutdown_signal` - Shutdown signal
    /// 
    fn bind_with_shutdown(&self, shutdown_signal: impl Future + Send + 'static);
}

/// Kore node with LevelDB database.
#[cfg(feature = "leveldb")]
pub struct LevelDBNode {
    /// Kore API.
    api: KoreApi,
    /// Cancellation token.
    cancellation: CancellationToken,
}

/// Implementation for `LevelDBNode`.
#[cfg(feature = "leveldb")]
impl LevelDBNode {
    /// Build a new `LevelDBNode`.
    /// 
    /// # Arguments
    /// 
    /// * `settings` - Kore settings
    /// * `password` - Password to encrypt/decrypt the key pair
    /// 
    /// # Returns
    /// 
    /// * `Result<Self, NodeError>` - `LevelDBNode`
    /// 
    pub fn build(settings: KoreSettings, password: &str) -> Result<Self, NodeError> {
        let key_pair = node_key_pair(&settings, password)?;
        let DbSettings::LevelDB(path) = settings.db;
        let db = open_db(Path::new(&path));
        let manager = LeveldbManager::new(db);
        
        let mut registry = <Registry>::default();

        let api= Node::build(settings.settings.clone(), key_pair.clone(), &mut registry, manager)
            .map_err(|_| {NodeError::InternalApi("Node build error".to_owned())})?;
        
        #[cfg(feature = "prometheus")]
        run_prometheus(registry, &settings.prometheus);
    
        let settings = settings.settings.node;
        Ok(Self {
            api: KoreApi::new(
                api,
                key_pair,
                settings.digest_derivator,
                settings.key_derivator,
            ),
            cancellation: CancellationToken::new(),
        })
    }
}

/// Implementation for `KoreNode` for `LevelDBNode`.
#[cfg(feature = "leveldb")]
#[async_trait]
impl KoreNode for LevelDBNode {

    /// Get the Kore API.
    /// 
    /// # Returns
    /// 
    /// * `&KoreApi` - Kore API
    /// 
    fn api(&self) -> &KoreApi {
        &self.api
    }

    /// Bind the node to the provided shutdown signal.
    /// 
    /// # Arguments
    /// 
    /// * `shutdown_signal` - Shutdown signal
    /// 
    fn bind_with_shutdown(&self, shutdown_signal: impl Future + Send + 'static) {
        let cancellation_token = self.cancellation.clone();
        tokio::spawn(async move {
            shutdown_signal.await;
            log::info!("Shutdown signal received");
            cancellation_token.cancel();
        });
    }
}

/// Kore node with SQLite database.
#[cfg(feature = "sqlite")]
pub struct SqliteNode {
    /// Kore API.
    api: KoreApi,
    /// Cancellation token.
    pub cancellation: CancellationToken,
}

/// Implementation for `SqliteNode`.
#[cfg(feature = "sqlite")]
impl SqliteNode {
    /// Build a new `SqliteNode`.
    /// 
    /// # Arguments
    /// 
    /// * `settings` - Kore settings
    /// * `password` - Password to encrypt/decrypt the key pair
    /// 
    /// # Returns
    /// 
    /// * `Result<Self, NodeError>` - `SqliteNode`
    /// 
    pub fn build(settings: KoreSettings, password: &str) -> Result<Self, NodeError> {
        let key_pair = node_key_pair(&settings, password)?;
        let DbSettings::Sqlite(path) = settings.db;
        let manager = SqliteManager::new(&path);
        
        let mut registry = <Registry>::default();

        let api= Node::build(settings.settings.clone(), key_pair.clone(), &mut registry, manager)
            .map_err(|_| NodeError::InternalApi("Node build error".to_owned()))?;

        #[cfg(feature = "prometheus")]
        run_prometheus(registry, &settings.prometheus);

        let settings = settings.settings.node;
        Ok(Self {
            api: KoreApi::new(
                api,
                key_pair,
                settings.digest_derivator,
                settings.key_derivator,
            ),
            cancellation: CancellationToken::new(),
        })
    }
}

/// Implementation for `KoreNode` for `SqliteNode`.
#[cfg(feature = "sqlite")]
#[async_trait]
impl KoreNode for SqliteNode {

    /// Get the Kore API.
    /// 
    /// # Returns
    /// 
    /// * `&KoreApi` - Kore API
    /// 
    fn api(&self) -> &KoreApi {
        &self.api
    }

    /// Bind the node to the provided shutdown signal.
    /// 
    /// # Arguments
    /// 
    /// * `shutdown_signal` - Shutdown signal
    /// 
    fn bind_with_shutdown(&self, shutdown_signal: impl Future + Send + 'static) {
        let cancellation_token = self.cancellation.clone();
        tokio::spawn(async move {
            shutdown_signal.await;
            log::info!("Shutdown signal received");
            cancellation_token.cancel();
        });
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use kore_base::NetworkConfig;
    use kore_base::NodeType;
    use kore_base::RoutingNode;
    use tokio::signal;
    
    #[cfg(feature = "leveldb")]
    #[tokio::test]
    async fn test_leveldb_node() {
        let node = create_leveldb_node(100, vec![]);
        assert!(node.is_ok());
    }

    #[cfg(feature = "leveldb")]
    pub fn create_leveldb_node(node: u32, boot_nodes: Vec<RoutingNode>) -> Result<LevelDBNode, NodeError> {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join(format!("keys{}", node));
        let password = format!("password{}", node);
        let mut settings = KoreSettings::default();
        settings.prometheus = format!("127.0.0.1:3{}", node);
        settings.settings.network = NetworkConfig::new(NodeType::Bootstrap, vec![format!("/ip4/127.0.0.1/tcp/{}", 50000 + node)], boot_nodes, false);
        settings.db = DbSettings::LevelDB(path.to_str().unwrap().to_owned());
        settings.keys_path = path.to_str().unwrap().to_owned();
        LevelDBNode::build(settings, &password)
    }

    #[cfg(feature = "leveldb")]
    pub fn export_leveldb_api(node: u32, known_nodes: Vec<RoutingNode>) -> KoreApi {
        let node = create_leveldb_node(node, known_nodes);
        assert!(node.is_ok());
        let node = node.unwrap();
        node.bind_with_shutdown(signal::ctrl_c());
        node.api().clone()
    }
    


    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn test_sqlite_node() {
        let node = create_sqlite_node(200, vec![]);
        assert!(node.is_ok());
    }

    #[cfg(feature = "sqlite")]
    pub fn create_sqlite_node(node: u32, boot_nodes: Vec<RoutingNode>) -> Result<SqliteNode, NodeError> {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join(format!("keys{}", node));
        let password = format!("password{}", node);
        let mut settings = KoreSettings::default();
        settings.prometheus = format!("127.0.0.1:3{}", node);
        settings.settings.network = NetworkConfig::new(NodeType::Bootstrap, vec![format!("/ip4/127.0.0.1/tcp/{}", 50000 + node)], boot_nodes, false);

        settings.db = DbSettings::Sqlite(format!("{}/database",path.to_str().unwrap().to_owned()));
        settings.keys_path = path.to_str().unwrap().to_owned();
        SqliteNode::build(settings, &password)
    }

    #[cfg(feature = "sqlite")]
    pub fn export_sqlite_api(node: u32, boot_nodes: Vec<RoutingNode>) -> KoreApi {
        let node = create_sqlite_node(node, boot_nodes);
        assert!(node.is_ok());
        let node = node.unwrap();
        node.bind_with_shutdown(signal::ctrl_c());
        node.api().clone()
    }
}