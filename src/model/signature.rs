// Copyright 2024 Antonio Estévez
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Signature model.
//!

use std::str::FromStr;

use crate::error::NodeError;

use kore_base::{
    identifier::{Derivable, DigestIdentifier, KeyIdentifier, SignatureIdentifier},
    signature::{Signature as BaseSignature, Signed as BaseSigned},
    TimeStamp,
};

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use std::fmt::Debug;

/// Signature model.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeSignature {
    /// Public key of the issuer
    signer: String, // KeyIdentifier
    /// Timestamp at which the signature was made
    timestamp: u64,
    /// Signature value
    value: String, // SignatureIdentifier,
    /// Content hash
    content_hash: String,
}

impl From<BaseSignature> for NodeSignature {
    fn from(signature: BaseSignature) -> Self {
        Self {
            signer: signature.signer.to_str(),
            timestamp: signature.timestamp.0,
            value: signature.value.to_str(),
            content_hash: signature.content_hash.to_str(),
        }
    }
}

impl TryFrom<NodeSignature> for BaseSignature {
    type Error = NodeError;
    fn try_from(signature: NodeSignature) -> Result<Self, Self::Error> {
        Ok(Self {
            signer: KeyIdentifier::from_str(&signature.signer)
                .map_err(|_| NodeError::InvalidParameter("key identifier".to_owned()))?,
            timestamp: TimeStamp(signature.timestamp),
            value: SignatureIdentifier::from_str(&signature.value)
                .map_err(|_| NodeError::InvalidParameter("signature identifier".to_owned()))?,
            content_hash: DigestIdentifier::from_str(&signature.content_hash)
                .map_err(|_| NodeError::InvalidParameter("digest identifier".to_owned()))?,
        })
    }
}

/// Signed content.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeSigned<T>
where
    T: Clone + Debug,
{
    /// Content
    #[serde(flatten)]
    pub content: T,
    /// Signature
    pub signature: NodeSignature,
}

impl<C, T> From<BaseSigned<C>> for NodeSigned<T>
where
    C: BorshDeserialize + BorshSerialize + Clone + Debug,
    T: From<C> + Clone + Debug,
{
    fn from(signed: BaseSigned<C>) -> Self {
        Self {
            content: signed.content.into(),
            signature: signed.signature.into(),
        }
    }
}

impl<C, T> TryFrom<NodeSigned<T>> for BaseSigned<C>
where
    C: BorshDeserialize + BorshSerialize + Clone + Debug,
    T: Into<C> + Clone + Debug,
{
    type Error = NodeError;
    fn try_from(signed: NodeSigned<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            content: signed.content.into(),
            signature: signed.signature.try_into()?,
        })
    }
}
