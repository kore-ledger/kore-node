// Copyright 2024 Antonio Estévez
// SPDX-License-Identifier: AGPL-3.0-or-later

//!
//! # API
//!
//! This module contains the Kore Node API.

use crate::{
    error::NodeError,
    model::{EventRequest, EventRequestResponse, SignedEventRequest},
};
use kore_base::{
    crypto::KeyPair,
    signature::{Signature as BaseSignature, Signed as BaseSigned},
    Api, Derivable, DigestDerivator, DigestIdentifier, EventRequest as BaseEventRequest,
    KeyDerivator,
};

use std::{convert::TryFrom, str::FromStr};

/// Kore Node API.
pub struct KoreApi {
    api: Api,
    keys: KeyPair,
    digest_derivator: DigestDerivator,
    key_derivator: KeyDerivator,
}

/// Kore Node API implementation.
impl KoreApi {
    /// Create a new Kore Node API.
    pub fn new(
        api: Api,
        keys: KeyPair,
        digest_derivator: DigestDerivator,
        key_derivator: KeyDerivator,
    ) -> Self {
        Self {
            api,
            keys,
            digest_derivator,
            key_derivator,
        }
    }

    /// Send an event request.
    /// If the request is a create request and the public key is not provided, a new key pair is
    /// generated and the public key is added to the request.
    /// If the request is not signed, a signature is generated and added to the request.
    /// The request is then sent to the Kore API.
    /// The request identifier is returned.
    ///
    /// # Arguments
    ///
    /// * `request` - Signed event request
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error
    /// * `NodeError::InvalidParameter` - Invalid request parameter.
    ///
    pub async fn send_event_request(
        &self,
        mut request: SignedEventRequest,
    ) -> Result<EventRequestResponse, NodeError> {
        if let EventRequest::Create(create_request) = &mut request.request {
            if create_request.public_key.is_none() {
                let public_key = self
                    .api
                    .add_keys(self.key_derivator)
                    .await
                    .map_err(|_| NodeError::InternalApi("Failed to add keys".to_owned()))?;
                create_request.public_key = Some(public_key.to_str());
            }
        }

        let Ok(event_request) = BaseEventRequest::try_from(request.request) else {
            return Err(NodeError::InvalidParameter("event request".to_owned()));
        };

        let signature = match request.signature {
            Some(signature) => {
                let Ok(signature) = BaseSignature::try_from(signature) else {
                    return Err(NodeError::InvalidParameter("signature".to_owned()));
                };
                signature
            }
            None => BaseSignature::new(&event_request, &self.keys, self.digest_derivator)
                .map_err(|_| NodeError::InternalApi("Failed to create signature".to_owned()))?,
        };
        match self
            .api
            .external_request(BaseSigned {
                content: event_request,
                signature,
            })
            .await
        {
            Ok(id) => Ok(EventRequestResponse {
                request_id: id.to_str(),
            }),
            Err(_) => Err(NodeError::InternalApi(
                "Failed to process request".to_owned(),
            )),
        }
    }

    /// Get an event request.
    /// The request is retrieved from the Kore API.
    ///
    /// # Arguments
    ///
    /// * `request_id` - Event request identifier
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error
    /// * `NodeError::InvalidParameter` - Invalid request identifier.
    ///
    /// # Returns
    ///
    /// * `SignedEventRequest` - Signed event request
    ///
    pub async fn get_event_request(
        &self,
        request_id: &str,
    ) -> Result<SignedEventRequest, NodeError> {
        let result = self
            .api
            .get_request(
                DigestIdentifier::from_str(request_id)
                    .map_err(|_| NodeError::InvalidParameter("request identifier".to_owned()))?,
            )
            .await
            .map_err(|_| NodeError::InternalApi("Failed to get request".to_owned()))?;
        Ok(SignedEventRequest::from(result))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[cfg(feature = "leveldb")]
    use crate::node::{KoreNode,tests::create_leveldb_node};
    use crate::model::{StartRequest, EventRequest, SignedEventRequest};
    use tokio::signal;

    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api() {
        let node = create_leveldb_node();
        assert!(node.is_ok());
        let node = node.unwrap();
        node.bind_with_shutdown(signal::ctrl_c());
        let api = node.api().clone();
        node.run(|_| {}).await;
        assert!(api.send_event_request(SignedEventRequest {
            request: EventRequest::Create(StartRequest {
                governance_id: "".to_owned(),
                schema_id: "governance".to_owned(),
                namespace: "agro.wine".to_owned(),
                name: "wine_track".to_owned(),
                public_key: None,
            }),
            signature: None,
        }).await.is_ok());

    }
}
