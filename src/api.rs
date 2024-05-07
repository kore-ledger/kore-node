// Copyright 2024 Antonio Estévez
// SPDX-License-Identifier: AGPL-3.0-or-later

//!
//! # API
//!
//! This module contains the Kore Node API.

use crate::{
    error::NodeError,
    model::{
        EventRequestResponse, NodeApprovalEntity, NodeEventRequest, NodeGetApprovals,
        NodeKoreRequestState, NodeSignedEventRequest, PatchVote,
    },
};
use kore_base::{
    crypto::KeyPair,
    signature::{Signature as BaseSignature, Signed as BaseSigned},
    Api, ApprovalState, Derivable, DigestDerivator, DigestIdentifier,
    EventRequest as BaseEventRequest, KeyDerivator,
};

use std::{convert::TryFrom, str::FromStr};

/// Kore Node API.
#[derive(Clone)]
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
        mut request: NodeSignedEventRequest,
    ) -> Result<EventRequestResponse, NodeError> {
        if let NodeEventRequest::Create(create_request) = &mut request.request {
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
    ) -> Result<NodeSignedEventRequest, NodeError> {
        let result = self
            .api
            .get_request(
                DigestIdentifier::from_str(request_id)
                    .map_err(|_| NodeError::InvalidParameter("request identifier".to_owned()))?,
            )
            .await
            .map_err(|_| NodeError::InternalApi("Failed to get request".to_owned()))?;
        Ok(NodeSignedEventRequest::from(result))
    }

    pub async fn get_event_request_state(
        &self,
        request_id: &str,
    ) -> Result<NodeKoreRequestState, NodeError> {
        let result = self
            .api
            .get_request(
                DigestIdentifier::from_str(request_id)
                    .map_err(|_| NodeError::InvalidParameter("request identifier".to_owned()))?,
            )
            .await
            .map_err(|_| NodeError::InternalApi("Failed to get request".to_owned()))?;
        Ok(NodeKoreRequestState::from(result))
    }

    pub async fn get_approvals(
        &self,
        params: NodeGetApprovals,
    ) -> Result<Vec<NodeApprovalEntity>, NodeError> {
        let status = match params.status {
            None => None,
            Some(value) => match value.to_lowercase().as_str() {
                "pending" => Some(ApprovalState::Pending),
                "obsolete" => Some(ApprovalState::Obsolete),
                "responded_accepted" => Some(ApprovalState::RespondedAccepted),
                "responded_rejected" => Some(ApprovalState::RespondedRejected),
                other => {
                    return Err(NodeError::InvalidParameter(format!(
                        "Invalid ApprovalState: {}",
                        other
                    )));
                }
            },
        };

        match self
            .api
            .get_approvals(status, params.from, params.quantity)
            .await
            .map(|result| {
                result
                    .into_iter()
                    .map(NodeApprovalEntity::from)
                    .collect::<Vec<NodeApprovalEntity>>()
            }) {
            Ok(res) => Ok(res),
            Err(_) => Err(NodeError::InternalApi(
                "Failed to process request".to_owned(),
            )),
        }
    }

    pub async fn get_approval_id(&self, id: &str) -> Result<NodeApprovalEntity, NodeError> {
        let result = self
            .api
            .get_approval(DigestIdentifier::from_str(id).map_err(|_| {
                NodeError::InvalidParameter("approval request identifier".to_owned())
            })?)
            .await
            .map_err(|e| {
                println!("{:?}", e);
                NodeError::InternalApi("Failed to get request".to_owned())
            })?;
        Ok(NodeApprovalEntity::from(result))
    }

    pub async fn approval_request(
        &self,
        id: &str,
        response: PatchVote,
    ) -> Result<NodeApprovalEntity, NodeError> {
        let acceptance = match response {
            PatchVote::RespondedAccepted => true,
            PatchVote::RespondedRejected => false,
        };

        match self
            .api
            .approval_request(
                DigestIdentifier::from_str(id)
                    .map_err(|_| NodeError::InvalidParameter("request identifier".to_owned()))?,
                acceptance,
            )
            .await
            .map(NodeApprovalEntity::from)
        {
            Ok(result) => Ok(result),
            Err(_) => Err(NodeError::InternalApi(
                "Failed to process request".to_owned(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::NodeFactRequest;
    #[cfg(feature = "leveldb")]
    use crate::model::{NodeEventRequest, NodeSignedEventRequest, NodeStartRequest};
    use crate::model::{NodeGetApprovals, PatchVote};
    use crate::node::tests::export_leveldb_api;
    use kore_base::ApprovalState as BaseApprovalState;
    use serde_json::json;
    use std::time::Duration;

    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_send_get_event_request() {
        let api = export_leveldb_api();

        let res = api
            .send_event_request(NodeSignedEventRequest {
                request: NodeEventRequest::Create(NodeStartRequest {
                    governance_id: "".to_owned(),
                    schema_id: "governance".to_owned(),
                    namespace: "agro.wine".to_owned(),
                    name: "wine_track".to_owned(),
                    public_key: None,
                }),
                signature: None,
            })
            .await
            .unwrap();
        assert!(api.get_event_request(&res.request_id).await.is_ok());
    }

    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_approval_accept() {
        let api = export_leveldb_api();
        let res = api
            .send_event_request(NodeSignedEventRequest {
                request: NodeEventRequest::Create(NodeStartRequest {
                    governance_id: "".to_owned(),
                    schema_id: "governance".to_owned(),
                    namespace: "agro.wine".to_owned(),
                    name: "wine_track".to_owned(),
                    public_key: None,
                }),
                signature: None,
            })
            .await
            .unwrap();

        let mut status;
        loop {
            status = api.get_event_request_state(&res.request_id).await.unwrap();
            match status.success {
                Some(val) => {
                    assert!(val);
                    break;
                }
                None => {
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
            }
        }

        let gov_subject = status.subject_id.unwrap();
        println!("subj: {}", gov_subject);

        let _ = api
            .send_event_request(NodeSignedEventRequest {
                request: NodeEventRequest::Fact(NodeFactRequest {
                    subject_id: gov_subject.clone(),
                    payload: json!({
                        "Patch": {
                            "data": [
                            {
                                "op": "add",
                                "path": "/members/0",
                                "value": {
                                "id": "EnyisBz0lX9sRvvV0H-BXTrVtARjUa0YDHzaxFHWH-N4",
                                "name": "Test1"
                                }
                            }
                        ]
                        }
                    }),
                }),
                signature: None,
            })
            .await
            .unwrap();

        let mut res_vec;
        loop {
            res_vec = api
                .get_approvals(NodeGetApprovals {
                    status: Some("pending".to_owned()),
                    from: None,
                    quantity: None,
                })
                .await
                .unwrap();
            if res_vec.is_empty() {
                tokio::time::sleep(Duration::from_millis(300)).await;
            } else {
                break;
            }
        }
        assert_eq!(res_vec.len(), 1);
        let res = api.get_approval_id(&res_vec[0].id).await.unwrap();
        assert_eq!(res.id, res_vec[0].id);

        let res = api
            .approval_request(&res_vec[0].id, PatchVote::RespondedAccepted)
            .await
            .unwrap();
        assert_eq!(res.state, BaseApprovalState::RespondedAccepted);

        res_vec = api
            .get_approvals(NodeGetApprovals {
                status: Some("pending".to_owned()),
                from: None,
                quantity: None,
            })
            .await
            .unwrap();
        assert!(res_vec.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_approval_rejected() {
        let api = export_leveldb_api();
        let res = api
            .send_event_request(NodeSignedEventRequest {
                request: NodeEventRequest::Create(NodeStartRequest {
                    governance_id: "".to_owned(),
                    schema_id: "governance".to_owned(),
                    namespace: "agro.wine".to_owned(),
                    name: "wine_track".to_owned(),
                    public_key: None,
                }),
                signature: None,
            })
            .await
            .unwrap();

        let mut status;
        loop {
            status = api.get_event_request_state(&res.request_id).await.unwrap();
            match status.success {
                Some(val) => {
                    assert!(val);
                    break;
                }
                None => {
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
            }
        }

        let gov_subject = status.subject_id.unwrap();
        println!("subj: {}", gov_subject);

        let _ = api
            .send_event_request(NodeSignedEventRequest {
                request: NodeEventRequest::Fact(NodeFactRequest {
                    subject_id: gov_subject.clone(),
                    payload: json!({
                        "Patch": {
                            "data": [
                            {
                                "op": "add",
                                "path": "/members/0",
                                "value": {
                                "id": "EnyisBz0lX9sRvvV0H-BXTrVtARjUa0YDHzaxFHWH-N4",
                                "name": "Test1"
                                }
                            }
                        ]
                        }
                    }),
                }),
                signature: None,
            })
            .await
            .unwrap();

        let mut res_vec;
        loop {
            res_vec = api
                .get_approvals(NodeGetApprovals {
                    status: Some("pending".to_owned()),
                    from: None,
                    quantity: None,
                })
                .await
                .unwrap();
            if res_vec.is_empty() {
                tokio::time::sleep(Duration::from_millis(300)).await;
            } else {
                break;
            }
        }
        assert_eq!(res_vec.len(), 1);
        let res = api.get_approval_id(&res_vec[0].id).await.unwrap();
        assert_eq!(res.id, res_vec[0].id);

        let res = api
            .approval_request(&res_vec[0].id, PatchVote::RespondedRejected)
            .await
            .unwrap();
        assert_eq!(res.state, BaseApprovalState::RespondedRejected);

        res_vec = api
            .get_approvals(NodeGetApprovals {
                status: Some("pending".to_owned()),
                from: None,
                quantity: None,
            })
            .await
            .unwrap();
        assert!(res_vec.is_empty());
    }
}
