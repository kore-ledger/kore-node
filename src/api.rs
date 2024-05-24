// Copyright 2024 Kore Ledger
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
    keys::KeyPair,
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
    /// * `request` - Signed event request.
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error.
    /// * `NodeError::InvalidParameter` - Invalid request parameter.
    ///
    /// # Returns
    ///
    /// * `EventRequestResponse` - Id of request.
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
    /// * `request_id` - Event request identifier.
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error.
    /// * `NodeError::InvalidParameter` - Invalid request identifier.
    ///
    /// # Returns
    ///
    /// * `NodeSignedEventRequest` - Signed event request.
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

    /// Get an state of event request.
    /// The state of request is retrieved from the Kore API.
    ///
    /// # Arguments
    ///
    /// * `request_id` - Event request identifier.
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error
    /// * `NodeError::InvalidParameter` - Invalid request identifier.
    ///
    /// # Returns
    ///
    /// * `NodeKoreRequestState` - State of event request.
    ///
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

    /// Get approval events.
    /// Get the status of the approval events you want to obtain, among the 4 available:
    /// - Pending: events pending voting.
    /// - Obsolete: events to which the node has not taken any vote, but have become obsolete and can no longer take a vote.
    /// - RespondedAccepted: Events to which the node has voted in favor.
    /// - RespondedRejected: Events which the node has voted against.
    /// The get that obtains the events is performed.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters for retrieving approval events.
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error.
    /// * `NodeError::InvalidParameter` - Invalid request parameter.
    ///
    /// # Returns
    ///
    /// * `Vec<NodeApprovalEntity>` - Vector of approval event.
    ///
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

    /// Get approval event.
    /// Gets an approval event from ID.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of approval event.
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error.
    /// * `NodeError::InvalidParameter` - Invalid request parameter.
    ///
    /// # Returns
    ///
    /// * `NodeApprovalEntity` - Approval event.
    ///
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

    /// Voting of an approval event.
    /// Accepts or denies an approval event.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of approval event.
    /// * `response` - Event approval or denial.
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error.
    /// * `NodeError::InvalidParameter` - Invalid request parameter.
    ///
    /// # Returns
    ///
    /// * `NodeApprovalEntity` - Approval event updated with user voting.
    ///
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

    /// Get all allowed subjects and providers.
    /// Obtain all subjects and suppliers that have been previously permitted.
    ///
    /// # Arguments
    ///
    /// * `parameters` - Parameters for retrieving allowed subjects and providers.
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error.
    ///
    /// # Returns
    ///
    /// * `Vec<PreauthorizedSubjectsResponse>` - Vector of allowed subjects and providers.
    ///
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

    /// Preauthorize subject.
    /// Preauthorize subject to receive a copy of the ledger.
    ///
    /// # Arguments
    ///
    /// * `subject_id` - ID of subject.
    /// * `data` - Vec of providers.
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error.
    /// * `NodeError::InvalidParameter` - Invalid request parameter.
    ///
    /// # Returns
    ///
    /// * `String` - 'Ok' if everything went well.
    ///
    pub async fn add_preauthorize_subject(
        &self,
        subject_id: &str,
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
                &DigestIdentifier::from_str(subject_id).map_err(|_| {
                    NodeError::InvalidParameter(format!("Invalid digest identifier {}", subject_id))
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

    /// Generate keys in node and return public key.
    /// Generates a key pair on the node and returns the public key.
    ///
    /// # Arguments
    ///
    /// * `parameters` - Algorithm for generating cryptographic material.
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error.
    ///
    /// # Returns
    ///
    /// * `String` - 'Ok' if everything went well.
    ///
    pub async fn generate_public_key(&self, parameters: NodeKeys) -> Result<String, NodeError> {
        let derivator = KeyDerivator::from(parameters.algorithm.unwrap_or(KeyAlgorithms::Ed25519));

        match self.api.add_keys(derivator).await {
            Ok(pub_key) => Ok(pub_key.to_str()),
            Err(_) => Err(NodeError::InternalApi(
                "Failed to process request".to_owned(),
            )),
        }
    }

    /// Get subjects.
    /// Depending on the parameters you can obtain:
    /// - All the governances known to a node.
    /// - All the traceability subjects of a governance.
    /// - All traceability subjects of the node, including the governance and the subjects of the governance.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters for retrieving subjects.
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error.
    /// * `NodeError::InvalidParameter` - Invalid request parameter.
    ///
    /// # Returns
    ///
    /// * `Vec<NodeSubjectData>` - Vector of subjects.
    ///
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

    /// Get subject.
    /// Obtains the information of a traceability subject from its id.
    ///
    /// # Arguments
    ///
    /// * `subject_id` - Subject id.
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error.
    /// * `NodeError::InvalidParameter` - Invalid request parameter.
    ///
    /// # Returns
    ///
    /// * `NodeSubjectData` - Subject of traceability.
    ///
    pub async fn get_subject(&self, subject_id: &str) -> Result<NodeSubjectData, NodeError> {
        match self
            .api
            .get_subject(
                DigestIdentifier::from_str(subject_id)
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

    /// Get validation proof.
    /// Allows to obtain the validation test of the last event for a specified subject.
    ///
    /// # Arguments
    ///
    /// * `subject_id` - Subject id.
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error.
    /// * `NodeError::InvalidParameter` - Invalid request parameter.
    ///
    /// # Returns
    ///
    /// * `NodeProof` - Validation proof.
    ///
    pub async fn get_validation_proof(&self, subject_id: &str) -> Result<NodeProof, NodeError> {
        match self
            .api
            .get_validation_proof(
                DigestIdentifier::from_str(subject_id)
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

    /// Get events of subject.
    /// Get events of traceability subject.
    ///
    /// # Arguments
    ///
    /// * `subject_id` - Subject id.
    /// * `parameters` - Parameters for retrieving events of subject
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error.
    /// * `NodeError::InvalidParameter` - Invalid request parameter.
    ///
    /// # Returns
    ///
    /// * `Vec<NodeSigned<EventContentResponse>>` - Event vector of a traceability subject
    ///
    pub async fn get_events_of_subject(
        &self,
        subject_id: &str,
        parameters: PaginatorFromNumber,
    ) -> Result<Vec<NodeSigned<EventContentResponse>>, NodeError> {
        let value = self
            .api
            .get_events(
                DigestIdentifier::from_str(subject_id)
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

    /// Get event of subject.
    /// Get a specific event of traceability subject.
    ///
    /// # Arguments
    ///
    /// * `subject_id` - Subject id.
    /// * `sn` - Versión of subject.
    ///
    /// # Errors
    ///
    /// * `NodeError::InternalApi` - Internal API error.
    /// * `NodeError::InvalidParameter` - Invalid request parameter.
    ///
    /// # Returns
    ///
    /// * `NodeSigned<EventContentResponse>` - Event of a traceability subject
    ///
    pub async fn get_event_of_subject(
        &self,
        subject_id: &str,
        sn: u64,
    ) -> Result<NodeSigned<EventContentResponse>, NodeError> {
        let value = self
            .api
            .get_event(
                DigestIdentifier::from_str(subject_id)
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

    use crate::model::{AuthorizeSubject, NodeFactRequest, NodeSubjects, PaginatorFromString};
    use crate::model::{NodeEventRequest, NodeSignedEventRequest, NodeStartRequest};
    use crate::model::{NodeGetApprovals, PatchVote};
    use crate::model::{NodeKeys, PaginatorFromNumber};
    use crate::KoreApi;
    use kore_base::ApprovalState as BaseApprovalState;
    use serde_json::{json, Value};
    use std::time::Duration;
    use std::vec;
    use kore_base::RoutingNode;

    //////////////////////////////////////////////////////////////////////////////////////////
    /// Basic methods
    /// Methods that perform different actions that in combination achieve certain behaviors
    //////////////////////////////////////////////////////////////////////////////////////////
    /// Method that creates a governance and returns its identifier.
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

    /// Method that creates an approval event and performs the vote
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

    /// Method that approves a subject on a node and verifies that a copy of the ledger is created on the node.
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

    /// method that for a given subject checks 'number' events
    async fn check_event_events_of_subject(api: &KoreApi, gov_subject: &str, number: usize) {
        let mut res_vec;
        loop {
            res_vec = api
                .get_events_of_subject(
                    &gov_subject,
                    PaginatorFromNumber {
                        from: None,
                        quantity: None,
                    },
                )
                .await
                .unwrap();
            if res_vec.len() < number {
                tokio::time::sleep(Duration::from_millis(300)).await;
            } else {
                break;
            }
        }

        for n in 0..number {
            let res = api
                .get_event_of_subject(&gov_subject, n.try_into().unwrap())
                .await
                .unwrap();
            assert_eq!(res_vec[n].content.gov_version, res.content.gov_version);
            assert_eq!(res_vec[n].content.patch, res.content.patch);
            assert_eq!(
                res_vec[n].content.hash_prev_event,
                res.content.hash_prev_event
            );
        }
    }

    //////////////////////////////////////////////////////////////////////////////////////////
    /// Api aux methods
    /// Methods that combine several basic methods to achieve certain behaviors
    //////////////////////////////////////////////////////////////////////////////////////////
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
        let pub_key: String = api
            .generate_public_key(NodeKeys {
                algorithm: Some(crate::model::KeyAlgorithms::Ed25519),
            })
            .await
            .unwrap();

        assert!(!pub_key.is_empty());
    }

    async fn api_check_event_events_of_subject(api: &KoreApi, number: usize) {
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

        check_event_events_of_subject(&api, &gov_subject, number).await;
    }

    async fn api_get_validation_proof(api: &KoreApi) {
        let gov_subject = create_governance(&api).await;
        let res = api.get_validation_proof(&gov_subject).await.unwrap();

        assert_eq!(gov_subject, res.proof.subject_id);
        assert_eq!(0, res.proof.sn);
    }

    /// LevelDB Tests
    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_send_get_event_request() {
        let api = export_leveldb_api(101, vec![]);

        create_governance(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_approval_accept() {
        let api = export_leveldb_api(102, vec![]);

        api_approval_accept(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_approval_rejected() {
        let api = export_leveldb_api(103, vec![]);
        api_approval_rejected(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_preauthorize_subject() {
        let api_node1 = export_leveldb_api(104, vec![]);
        let peer_id_node1 = api_node1.api.peer_id().to_string();

        let api_node2 = export_leveldb_api(
            105,
            vec![RoutingNode {
                address: "/ip4/127.0.0.1/tcp/50104".to_owned(),
                peer_id: peer_id_node1
            }]
        );
        api_preauthorize_subject(&api_node1, &api_node2).await;
    }

    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_public_key() {
        let api = export_leveldb_api(106, vec![]);
        api_public_key(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_events_subject() {
        let api = export_leveldb_api(107, vec![]);
        api_check_event_events_of_subject(&api, 2).await;
    }

    #[tokio::test]
    #[cfg(feature = "leveldb")]
    async fn test_leveldb_api_validation_proof() {
        let api = export_leveldb_api(108, vec![]);
        api_get_validation_proof(&api).await;
    }


    /// Sqlite Tests
    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_sqlite_api_send_get_event_request() {
        let api = export_sqlite_api(201, vec![]);

        create_governance(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_sqlite_api_approval_accept() {
        let api = export_sqlite_api(202, vec![]);

        api_approval_accept(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_sqlite_api_approval_rejected() {
        let api = export_sqlite_api(203, vec![]);
        api_approval_rejected(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_sqlite_api_preauthorize_subject() {
        let api_node1 = export_sqlite_api(204, vec![]);
        let peer_id_node1 = api_node1.api.peer_id().to_string();

        let api_node2 = export_sqlite_api(
            205,
            vec![RoutingNode {
                address: vec!["/ip4/127.0.0.1/tcp/50204".to_owned()],
                peer_id: peer_id_node1
            }]
        );
        api_preauthorize_subject(&api_node1, &api_node2).await;
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_sqlite_api_public_key() {
        let api = export_sqlite_api(206, vec![]);
        api_public_key(&api).await;
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_sqlite_api_events_subject() {
        let api = export_sqlite_api(207, vec![]);
        api_check_event_events_of_subject(&api, 2).await;
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_sqlite_api_validation_proof() {
        let api = export_sqlite_api(208, vec![]);
        api_get_validation_proof(&api).await;
    }
}
