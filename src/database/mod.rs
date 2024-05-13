// Copyright 2024 Kore Ledger
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Database module.
//!
//! This module contains the diferent database implementations that can be used by the Kore Node.
//!
//! ## Database implementations
//!
//! The following database implementations are available:
//!
//! * [Leveldb](leveldb/index.html)
//! * [Sqlite](sqlite/index.html)
//! * [Cassandra](cassandra/index.html)
//!

#[cfg(feature = "cassandra")]
pub mod cassandra;
#[cfg(feature = "leveldb")]
pub mod leveldb;
#[cfg(feature = "sqlite")]
pub mod sqlite;
