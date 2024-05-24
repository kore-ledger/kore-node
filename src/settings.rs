// Copyright 2024 Kore Ledger
// SPDX-License-Identifier: AGPL-3.0-or-later

use kore_base::Settings as BaseSettings;

use serde::Deserialize;

/// Database settings.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum DbSettings {
    /// Configuration for a LevelDB database.
    #[cfg(feature = "leveldb")]
    LevelDB(String),
    /// Configuration for a SQLite database.
    #[cfg(feature = "sqlite")]
    Sqlite(String),
    /// Configuration for a Cassandra database.
    #[cfg(feature = "cassandra")]
    Cassandra,
}

/// Specific settings for the node.
#[derive(Deserialize, Debug, Clone)]
pub struct KoreSettings {
    /// Settings from Kore Base.
    pub settings: BaseSettings,
    /// Database settings.
    pub db: DbSettings,
    /// Path for encryptep keys.
    #[serde(rename = "keysPath")]
    pub keys_path: String,
}

#[cfg(feature = "sqlite")]
impl Default for KoreSettings {
    fn default() -> Self {

        Self {
            settings: BaseSettings::default(),
            db: DbSettings::Sqlite("examples/sqlitedb/database".to_owned()),
            keys_path: "examples/keys".to_owned(),
        }
    }
}

#[cfg(feature = "leveldb")]
impl Default for KoreSettings {
    fn default() -> Self {
 Self {
            settings: BaseSettings::default(),
            db: DbSettings::LevelDB("examples/leveldb".to_owned()),
            keys_path: "examples/keys".to_owned(),
        }
    }
}