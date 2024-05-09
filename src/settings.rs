// Copyright 2024 Antonio Estévez
// SPDX-License-Identifier: AGPL-3.0-or-later

use kore_base::Settings as BaseSettings;

use serde::Deserialize;

/// Database settings.
#[derive(Deserialize, Debug, Clone)]
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

impl Default for KoreSettings {
    fn default() -> Self {
        #[cfg(feature = "sqlite")]
        return Self {
            settings: BaseSettings::default(),
            db: DbSettings::Sqlite("examples/sqlitedb".to_owned()),
            keys_path: "examples/keys".to_owned(),
        };
        #[cfg(feature = "leveldb")]
        return Self {
            settings: BaseSettings::default(),
            db: DbSettings::LevelDB("examples/leveldb".to_owned()),
            keys_path: "examples/keys".to_owned(),
        };
    }
}