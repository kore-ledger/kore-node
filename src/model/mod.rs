// Copyright 2024 Antonio Estévez
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Data model for Kore Node.
//!
//! This module contains the data model for the Kore Node.
//!
//! ## Data model
//!
//! The data model is composed of the following elements:
//!

pub mod request;
pub mod signature;

pub use request::*;
pub use signature::*;
