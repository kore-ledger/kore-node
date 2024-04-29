// Copyright 2024 Antonio Estévez
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use crate::{
    error::NodeError,
    settings::{DbSettings, KoreSettings},
    KoreApi,
    utils::node_key_pair,
};

#[cfg(feature = "leveldb")]
use crate::database::leveldb::{LeveldbCollection, LeveldbManager, open_db};
#[cfg(feature = "sqlite")]
use crate::database::sqlite::{SqliteCollection, SqliteManager};

use kore_base::{Node, Notification};

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
    /// Run the node.
    /// 
    /// # Arguments
    /// 
    /// * `notifications_handler` - Notifications handler
    /// 
    async fn run<H>(self, notifications_handler: H)
    where
        H: Fn(Notification) + Send;
}

/// Kore node with LevelDB database.
#[cfg(feature = "leveldb")]
#[derive(Clone)]
pub struct LevelDBNode {
    /// Kore API.
    api: KoreApi,
    /// Node base.
    node_base: Node<LeveldbManager, LeveldbCollection>,
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
        let (node_base, api) = Node::build(settings.settings.clone(), key_pair.clone(), manager)
            .map_err(|_| NodeError::InternalApi("Node build error".to_owned()))?;
        let settings = settings.settings.node;
        Ok(Self {
            api: KoreApi::new(
                api,
                key_pair,
                settings.digest_derivator,
                settings.key_derivator,
            ),
            node_base,
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

    /// Run the node.
    /// 
    /// # Arguments
    /// 
    /// * `notifications_handler` - Notifications handler
    /// 
    async fn run<H>(self, notifications_handler: H)
    where
        H: Fn(Notification) + Send,
    {
        self.node_base
            .handle_notifications(notifications_handler)
            .await;
        self.cancellation.cancel();
        log::info!("Stopped");
    }
}

/// Kore node with SQLite database.
#[cfg(feature = "sqlite")]
pub struct SqliteNode {
    /// Kore API.
    api: KoreApi,
    /// Node base.
    node_base: Node<SqliteManager, SqliteCollection>,
    /// Cancellation token.
    cancellation: CancellationToken,
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
        let (node_base, api) = Node::build(settings.settings.clone(), key_pair.clone(), manager)
            .map_err(|_| NodeError::InternalApi("Node build error".to_owned()))?;
        let settings = settings.settings.node;
        Ok(Self {
            api: KoreApi::new(
                api,
                key_pair,
                settings.digest_derivator,
                settings.key_derivator,
            ),
            node_base,
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

    /// Run the node.
    /// 
    /// # Arguments
    /// 
    /// * `notifications_handler` - Notifications handler
    /// 
    async fn run<H>(self, notifications_handler: H)
    where
        H: Fn(Notification) + Send,
    {
        self.node_base
            .handle_notifications(notifications_handler)
            .await;
        self.cancellation.cancel();
        log::info!("Stopped");
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use tokio::signal;

    #[cfg(feature = "leveldb")]
    #[tokio::test]
    async fn test_leveldb_node() {
        let node = create_leveldb_node();
        assert!(node.is_ok());
        let node = node.unwrap();
        node.bind_with_shutdown(signal::ctrl_c());
        node.run(|_| {}).await;
    }

    #[cfg(feature = "leveldb")]
    pub fn create_leveldb_node() -> Result<LevelDBNode, NodeError> {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("keys");
        let password = "password";
        let mut settings = KoreSettings::default();
        settings.db = DbSettings::LevelDB(path.to_str().unwrap().to_owned());
        settings.keys_path = path.to_str().unwrap().to_owned();
        LevelDBNode::build(settings, &password)
    }


    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn test_sqlite_node() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("keys");
        let password = "password";
        let mut settings = KoreSettings::default();
        settings.db = DbSettings::Sqlite(":memory:".to_owned());
        settings.keys_path = path.to_str().unwrap().to_owned();
        let node = SqliteNode::build(settings, &password);
        assert!(node.is_ok());
        let node = node.unwrap();
        node.bind_with_shutdown(signal::ctrl_c());
        node.run(|_| {}).await;
    }

}