// Copyright 2024 Kore Ledger
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Kore Node errors.
//!
//! This module contains the different errors that can be returned by the Kore Node.
//!  

use thiserror::Error;

/// Kore Node errors.
#[derive(Error, Debug, Clone)]
pub enum NodeError {
    /// Invalid parameter.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    /// API error
    #[error("API error: {0}")]
    InternalApi(String),
    /// Database error
    #[error("Database error: {0}")]
    Database(String),
    /// Keys Error
    #[error("Keys error: {0}")]
    Keys(String),
}
