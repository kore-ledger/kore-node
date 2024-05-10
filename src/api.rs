// Copyright 2024 Antonio Estévez
// SPDX-License-Identifier: AGPL-3.0-or-later

//!
//! # API
//!
//! This module contains the Kore Node API.

use crate::{
    error::NodeError,
    model::{
        AuthorizeSubject, EventContentResponse, EventRequestResponse, KeyAlgorithms,
        NodeApprovalEntity, NodeEventRequest, NodeGetApprovals, NodeKeys, NodeKoreRequestState,
        NodeProof, NodeSigned, NodeSignedEventRequest, NodeSubjectData, NodeSubjects,
        PaginatorFromNumber, PaginatorFromString, PatchVote, PreauthorizedSubjectsResponse,
    },
};
use kore_base::{
    crypto::KeyPair,
    signature::{Signature as BaseSignature, Signed as BaseSigned},
    Api, ApprovalState, Derivable, DigestDerivator, DigestIdentifier,
    EventRequest as BaseEventRequest, KeyDerivator, KeyIdentifier,
};

use std::{collections::HashSet, convert::TryFrom, str::FromStr};

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
            .map_err(|_| NodeError::InternalApi("Failed to get request".to_owned()))?;
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

    pub async fn get_all_allowed_subjects_and_providers(
        &self,
        parameters: PaginatorFromString,
    ) -> Result<Vec<PreauthorizedSubjectsResponse>, NodeError> {
        match self
            .api
            .get_all_allowed_subjects_and_providers(parameters.from, parameters.quantity)
            .await
            .map(|x| Vec::from_iter(x.into_iter().map(PreauthorizedSubjectsResponse::from)))
        {
            Ok(result) => Ok(result),
            Err(_) => Err(NodeError::InternalApi(
                "Failed to process request".to_owned(),
            )),
        }
    }

    pub async fn add_preauthorize_subject(
        &self,
        id: &str,
        data: AuthorizeSubject,
    ) -> Result<String, NodeError> {
        let mut providers = HashSet::new();
        for provider in data.providers.iter() {
            let provider = match KeyIdentifier::from_str(provider) {
                Ok(provider) => provider,
                Err(_error) => {
                    return Err(NodeError::InvalidParameter(format!(
                        "Invalid key identifier {}",
                        provider
                    )))
                }
            };
            providers.insert(provider);
        }

        match self
            .api
            .add_preauthorize_subject(
                &DigestIdentifier::from_str(id).map_err(|_| {
                    NodeError::InvalidParameter(format!("Invalid digest identifier {}", id))
                })?,
                &providers,
            )
            .await
        {
            Ok(_) => Ok("Ok".to_owned()),
            Err(_) => Err(NodeError::InternalApi(
                "Failed to process request".to_owned(),
            )),
        }
    }

    // Testear.
    pub async fn generate_public_key(&self, parameters: NodeKeys) -> Result<String, NodeError> {
        let derivator = KeyDerivator::from(parameters.algorithm.unwrap_or(KeyAlgorithms::Ed25519));

        match self.api.add_keys(derivator).await {
            Ok(pub_key) => Ok(pub_key.to_str()),
            Err(_) => Err(NodeError::InternalApi(
                "Failed to process request".to_owned(),
            )),
        }
    }

    pub async fn get_subjects(
        &self,
        parameters: NodeSubjects,
    ) -> Result<Vec<NodeSubjectData>, NodeError> {
        enum SubjectType {
            All,
            Governances,
        }
        let subject_type = match &parameters.subject_type {
            Some(data) => match data.to_lowercase().as_str() {
                "all" => SubjectType::All,
                "governances" => {
                    if parameters.governanceid.is_some() {
                        return Err(NodeError::InvalidParameter(
                            "governanceid can not be specified with subject_type=governances"
                                .to_string(),
                        ));
                    }
                    SubjectType::Governances
                }
                other => {
                    return Err(NodeError::InvalidParameter(format!(
                        "unknow parameter {}",
                        other
                    )));
                }
            },
            None => SubjectType::All,
        };

        let data = match subject_type {
            SubjectType::All => {
                if let Some(data) = &parameters.governanceid {
                    self.api
                        .get_subjects_by_governance(
                            DigestIdentifier::from_str(data).map_err(|_| {
                                NodeError::InvalidParameter("governanceid".to_owned())
                            })?,
                            parameters.from,
                            parameters.quantity,
                        )
                        .await
                } else {
                    self.api
                        .get_subjects("".into(), parameters.from, parameters.quantity)
                        .await
                }
            }
            SubjectType::Governances => {
                self.api
                    .get_governances("".into(), parameters.from, parameters.quantity)
                    .await
            }
        }
        .map(|s| {
            s.into_iter()
                .map(NodeSubjectData::from)
                .collect::<Vec<NodeSubjectData>>()
        });

        match data {
            Ok(data) => Ok(data),
            Err(_) => Err(NodeError::InternalApi(
                "Failed to process request".to_owned(),
            )),
        }
    }

    pub async fn get_subject(&self, id: &str) -> Result<NodeSubjectData, NodeError> {
        match self
            .api
            .get_subject(
                DigestIdentifier::from_str(id)
                    .map_err(|_| NodeError::InvalidParameter("invalid subject_id".to_owned()))?,
            )
            .await
            .map(NodeSubjectData::from)
        {
            Ok(result) => Ok(result),
            Err(_) => Err(NodeError::InternalApi(
                "Failed to process request".to_owned(),
            )),
        }
    }

    pub async fn get_validation_proof(&self, id: &str) -> Result<NodeProof, NodeError> {
        match self
            .api
            .get_validation_proof(
                DigestIdentifier::from_str(id)
                    .map_err(|_| NodeError::InvalidParameter("invalid subject_id".to_owned()))?,
            )
            .await
        {
            Ok(value) => Ok(NodeProof::from(value)),
            Err(_) => Err(NodeError::InternalApi(
                "Failed to process request".to_owned(),
            )),
        }
    }

    pub async fn get_events_of_subject(
        &self,
        id: &str,
        parameters: PaginatorFromNumber,
    ) -> Result<Vec<NodeSigned<EventContentResponse>>, NodeError> {
        let value = self
            .api
            .get_events(
                DigestIdentifier::from_str(id)
                    .map_err(|_| NodeError::InvalidParameter("invalid subject_id".to_owned()))?,
                parameters.from,
                parameters.quantity,
            )
            .await
            .map(|vec| {
                vec.into_iter()
                    .map(NodeSigned::<EventContentResponse>::from)
                    .collect::<Vec<NodeSigned<EventContentResponse>>>()
            });
        match value {
            Ok(v) => Ok(v),
            Err(_) => Err(NodeError::InternalApi(
                "Failed to process request".to_owned(),
            )),
        }
    }

    pub async fn get_event_of_subject(
        &self,
        id: &str,
        sn: u64,
    ) -> Result<NodeSigned<EventContentResponse>, NodeError> {
        let value = self
            .api
            .get_event(
                DigestIdentifier::from_str(id)
                    .map_err(|_| NodeError::InvalidParameter("invalid subject_id".to_owned()))?,
                sn,
            )
            .await
            .map(NodeSigned::<EventContentResponse>::from);
        match value {
            Ok(v) => Ok(v),
            Err(_) => Err(NodeError::InternalApi(
                "Failed to process request".to_owned(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "leveldb")]
    use crate::node::tests::export_leveldb_api;

    #[cfg(feature = "sqlite")]
    use crate::node::tests::export_sqlite_api;

    use crate::model::NodeKeys;
    use crate::model::{AuthorizeSubject, NodeFactRequest, NodeSubjects, PaginatorFromString};
    use crate::model::{NodeEventRequest, NodeSignedEventRequest, NodeStartRequest};
    use crate::model::{NodeGetApprovals, PatchVote};
    use crate::KoreApi;
    use kore_base::ApprovalState as BaseApprovalState;
    use serde_json::{json, Value};
    use std::time::Duration;

    async fn create_governance(api: &KoreApi) -> String {
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
        status.subject_id.unwrap()
    }

    async fn create_approval_event_and_vote(
        api: &KoreApi,
        payload: Value,
        subject: &str,
        vote: PatchVote,
    ) {
        let _ = api
            .send_event_request(NodeSignedEventRequest {
                request: NodeEventRequest::Fact(NodeFactRequest {
                    subject_id: subject.to_owned(),
                    payload,
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
            .approval_request(&res_vec[0].id, vote.clone())
            .await
            .unwrap();

        match vote {
            PatchVote::RespondedAccepted => {
                assert_eq!(res.state, BaseApprovalState::RespondedAccepted)
            }
            PatchVote::RespondedRejected => {
                assert_eq!(res.state, BaseApprovalState::RespondedRejected)
            }
        };

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

    pub async fn preauthorized_and_ledger_copy(api_node2: &KoreApi, subject: &str) {
        let res = api_node2
            .add_preauthorize_subject(subject, AuthorizeSubject { providers: vec![] })
            .await
            .unwrap();
        assert_eq!(res, "Ok".to_owned());

        let res = api_node2
            .get_all_allowed_subjects_and_providers(PaginatorFromString {
                from: None,
                quantity: None,
            })
            .await
            .unwrap();
        assert_eq!(res[0].subject_id, subject);

        let mut res_vec;
        loop {
            res_vec = api_node2
                .get_subjects(NodeSubjects {
                    from: None,
                    governanceid: None,
                    subject_type: None,
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
        assert_eq!(res_vec[0].subject_id, subject);

        let res = api_node2.get_subject(subject).await.unwrap();
        assert_eq!(res.subject_id, res_vec[0].subject_id);
    }

    async fn api_approval_accept(api: &KoreApi) {
        let gov_subject = create_governance(&api).await;
        let payload = json!({
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
        });
        create_approval_event_and_vote(&api, payload, &gov_subject, PatchVote::RespondedAccepted)
            .await;
    }

    async fn api_approval_rejected(api: &KoreApi) {
        let gov_subject = create_governance(&api).await;
        let payload = json!({
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
        });
        create_approval_event_and_vote(&api, payload, &gov_subject, PatchVote::RespondedRejected)
            .await;
    }

    async fn api_preauthorize_subject(api_node1: &KoreApi, api_node2: &KoreApi) {
        let controller_id_node1 = api_node1.api.controller_id();
        let controller_id_node2 = api_node2.api.controller_id();

        let gov_subject = create_governance(&api_node1).await;
        let payload = json!({
            "Patch": {
                "data": [
                {
                    "op": "add",
                    "path": "/members/0",
                    "value": {
                    "id": controller_id_node1,
                    "name": "Node1"
                    }
                },
                {
                    "op": "add",
                    "path": "/members/1",
                    "value": {
                    "id": controller_id_node2,
                    "name": "Node2"
                    }
                },
                {
                    "op": "add",
                    "path": "/roles/1",
                    "value": {
                        "namespace": "",
                        "role": "WITNESS",
                        "schema": {
                            "ID": "governance"
                        },
                        "who": {
                            "NAME": "Node2"
                        }
                    }
                },
            ]
            }
        });
        create_approval_event_and_vote(
            &api_node1,
            payload,
            &gov_subject,
            PatchVote::RespondedAccepted,
        )
        .await;
        preauthorized_and_ledger_copy(&api_node2, &gov_subject).await;
    }

    async fn api_public_key(api: &KoreApi) {
        let pub_key = api
            .generate_public_key(NodeKeys {
                algorithm: Some(crate::model::KeyAlgorithms::Ed25519),
            })
            .await
            .unwrap();

        assert!(!pub_key.is_empty());
    }

    // LevelDB
    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_send_get_event_request() {
        let api = export_leveldb_api(101, &[]);

        create_governance(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_approval_accept() {
        let api = export_leveldb_api(102, &[]);

        api_approval_accept(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_approval_rejected() {
        let api = export_leveldb_api(103, &[]);
        api_approval_rejected(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_preauthorize_subject() {
        let api_node1 = export_leveldb_api(104, &[]);
        let peer_id_node1 = api_node1.api.peer_id().to_string();

        let api_node2 = export_leveldb_api(
            105,
            &[format!("/ip4/127.0.0.1/tcp/50104/p2p/{}", peer_id_node1)],
        );
        api_preauthorize_subject(&api_node1, &api_node2).await;
    }

    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_public_key() {
        let api = export_leveldb_api(106, &[]);
        api_public_key(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_events_subject() {}

    // sqlite
    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_sqlite_api_send_get_event_request() {
        let api = export_sqlite_api(201, &[]);

        create_governance(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_sqlite_api_approval_accept() {
        let api = export_sqlite_api(202, &[]);

        api_approval_accept(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_sqlite_api_approval_rejected() {
        let api = export_sqlite_api(203, &[]);
        api_approval_rejected(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_sqlite_api_preauthorize_subject() {
        let api_node1 = export_sqlite_api(204, &[]);
        let peer_id_node1 = api_node1.api.peer_id().to_string();

        let api_node2 = export_sqlite_api(
            205,
            &[format!("/ip4/127.0.0.1/tcp/50204/p2p/{}", peer_id_node1)],
        );
        api_preauthorize_subject(&api_node1, &api_node2).await;
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_sqlite_api_public_key() {
        let api = export_sqlite_api(206, &[]);
        api_public_key(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_sqlite_api_events_subject() {}
}

// get_pending_requests
// get_single_request
// get_governance_subjects

/*
GET: controller_id controller_id
PUT: controller_id controller_id
PUT: keys􏿿E5vSgznxGFnDdAg2iTx70SpoEeh0QF1_lEJmqyr1fEHw keys􏿿E5vSgznxGFnDdAg2iTx70SpoEeh0QF1_lEJmqyr1fEHw
GET_REQUEST: key kore_request􏿿JzAves76cI_UQGKIOM8nPNOuAV80Qc7hTgkMmWg-2RvE
GET: kore_request􏿿JzAves76cI_UQGKIOM8nPNOuAV80Qc7hTgkMmWg-2RvE kore_request􏿿JzAves76cI_UQGKIOM8nPNOuAV80Qc7hTgkMmWg-2RvE
GET ERROR EntryNotFound
GET: keys􏿿E5vSgznxGFnDdAg2iTx70SpoEeh0QF1_lEJmqyr1fEHw keys􏿿E5vSgznxGFnDdAg2iTx70SpoEeh0QF1_lEJmqyr1fEHw
SET_REQUEST: key kore_request􏿿JzAves76cI_UQGKIOM8nPNOuAV80Qc7hTgkMmWg-2RvE
PUT: kore_request􏿿JzAves76cI_UQGKIOM8nPNOuAV80Qc7hTgkMmWg-2RvE kore_request􏿿JzAves76cI_UQGKIOM8nPNOuAV80Qc7hTgkMmWg-2RvE
PUT: prevalidated_event􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY prevalidated_event􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY
GET_REQUEST: key kore_request􏿿JzAves76cI_UQGKIOM8nPNOuAV80Qc7hTgkMmWg-2RvE
GET: kore_request􏿿JzAves76cI_UQGKIOM8nPNOuAV80Qc7hTgkMmWg-2RvE kore_request􏿿JzAves76cI_UQGKIOM8nPNOuAV80Qc7hTgkMmWg-2RvE
GET: validation􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY validation􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY
PUT: validation􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY validation􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY
GET: subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY
GET: keys􏿿E5vSgznxGFnDdAg2iTx70SpoEeh0QF1_lEJmqyr1fEHw keys􏿿E5vSgznxGFnDdAg2iTx70SpoEeh0QF1_lEJmqyr1fEHw
PUT: governance_index􏿿􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY governance_index􏿿􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY
PUT: subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY
GET: signature􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY􏿿0000000000000000  signature􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY􏿿0000000000000000
PUT: signature􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY􏿿0000000000000000  signature􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY􏿿0000000000000000
PUT: event􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY􏿿0000000000000000  event􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY􏿿0000000000000000
SET_REQUEST: key kore_request􏿿JzAves76cI_UQGKIOM8nPNOuAV80Qc7hTgkMmWg-2RvE
    PUT: kore_request􏿿JzAves76cI_UQGKIOM8nPNOuAV80Qc7hTgkMmWg-2RvE kore_request􏿿JzAves76cI_UQGKIOM8nPNOuAV80Qc7hTgkMmWg-2RvE
GET: subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY
GET: event􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY􏿿0000000000000000  event􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY􏿿0000000000000000
GET: witness_signatures􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY witness_signatures􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY
PUT: witness_signatures􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY witness_signatures􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY
GET: subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY
GET: subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY
GET: subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY
GET: subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY
GET: subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY subject􏿿J2nB4BBaJezCNRPH2d4dirLNVNBRw8BqWI7ANjyMc9zY
GET_REQUEST: key kore_request􏿿JzAves76cI_UQGKIOM8nPNOuAV80Qc7hTgkMmWg-2RvE
GET: kore_request􏿿JzAves76cI_UQGKIOM8nPNOuAV80Qc7hTgkMmWg-2RvE kore_request􏿿JzAves76cI_UQGKIOM8nPNOuAV80Qc7hTgkMmWg-2RvE

GET: controller_id controller_id
PUT: controller_id controller_id
PUT: keys􏿿ER5YPH_8-iE7CPD1QPhfH_dJltrNG3d1W8lXlfizA4QI transfer
GET_REQUEST: key kore_request􏿿JHCA5mRLM-cQJTyYqJSNkn2rQ52CBmSirpH0uQI8TWzA
GET: kore_request􏿿JHCA5mRLM-cQJTyYqJSNkn2rQ52CBmSirpH0uQI8TWzA kore_request
GET ERROR EntryNotFound
GET: keys􏿿ER5YPH_8-iE7CPD1QPhfH_dJltrNG3d1W8lXlfizA4QI transfer
ELOTRO: EventCreationError { source: SubjectKeysNotFound("ER5YPH_8-iE7CPD1QPhfH_dJltrNG3d1W8lXlfizA4QI") }
thread 'api::tests::test_sqlite_api_approval_rejected' panicked at src/api.rs:485:14:
called `Result::unwrap()` on an `Err` value: InternalApi("Failed to process request")
*/
