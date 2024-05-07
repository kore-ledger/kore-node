// Copyright 2024 Antonio Estévez
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Event request.
//!

use crate::error::NodeError;

use kore_base::{
    identifier::{Derivable, DigestIdentifier, KeyIdentifier},
    request::{
        EOLRequest as BaseEOLRequest, FactRequest as BaseFactRequest,
        KoreRequest as BaseKoreRequest, RequestState, StartRequest as BaseStartRequest,
        TransferRequest as BaseTransferRequest,
    },
    signature::Signed as BaseSigned,
    ApprovalEntity as BaseApprovalEntity, ApprovalRequest as BaseApprovalRequest,
    ApprovalResponse as BaseApprovalResponse, ApprovalState as BaseApprovalState,
    EventRequest as BaseEventRequest, ValueWrapper,
};

use std::{fmt::Debug, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{NodeSignature, NodeSigned};

/// Event request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NodeEventRequest {
    Create(NodeStartRequest),
    Fact(NodeFactRequest),
    Transfer(NodeTransferRequest),
    EOL(NodeEOLRequest),
}

impl From<BaseEventRequest> for NodeEventRequest {
    fn from(request: BaseEventRequest) -> Self {
        match request {
            BaseEventRequest::Create(request) => Self::Create(request.into()),
            BaseEventRequest::Fact(request) => Self::Fact(request.into()),
            BaseEventRequest::Transfer(request) => Self::Transfer(request.into()),
            BaseEventRequest::EOL(request) => Self::EOL(request.into()),
        }
    }
}

impl TryFrom<NodeEventRequest> for BaseEventRequest {
    type Error = NodeError;
    fn try_from(request: NodeEventRequest) -> Result<Self, Self::Error> {
        match request {
            NodeEventRequest::Create(request) => {
                Ok(Self::Create(BaseStartRequest::try_from(request)?))
            }
            NodeEventRequest::Fact(request) => Ok(Self::Fact(request.try_into()?)),
            NodeEventRequest::Transfer(request) => Ok(Self::Transfer(request.try_into()?)),
            NodeEventRequest::EOL(request) => Ok(Self::EOL(request.try_into()?)),
        }
    }
}

/// Create request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeStartRequest {
    /// Governance identifier
    pub governance_id: String,
    /// Subject schema json identifier
    pub schema_id: String,
    /// Namespace to which the subject belongs
    pub namespace: String,
    /// Name of subject
    pub name: String,
    /// Public key of the subject
    pub public_key: Option<String>,
}

impl From<BaseStartRequest> for NodeStartRequest {
    fn from(request: BaseStartRequest) -> Self {
        Self {
            governance_id: request.governance_id.to_str(),
            schema_id: request.schema_id,
            namespace: request.namespace,
            name: request.name,
            public_key: Some(request.public_key.to_str()),
        }
    }
}

impl TryFrom<NodeStartRequest> for BaseStartRequest {
    type Error = NodeError;
    fn try_from(request: NodeStartRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            governance_id: DigestIdentifier::from_str(&request.governance_id).map_err(|_| {
                NodeError::InvalidParameter("Invalid governance identifier".to_string())
            })?,
            schema_id: request.schema_id,
            namespace: request.namespace,
            name: request.name,
            public_key: KeyIdentifier::from_str(&request.public_key.unwrap())
                .map_err(|_| NodeError::InvalidParameter("Invalid public key".to_string()))?,
        })
    }
}

/// Fact request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeFactRequest {
    /// Subject identifier
    pub subject_id: String,
    /// Changes to be applied to the subject
    pub payload: Value,
}

impl From<BaseFactRequest> for NodeFactRequest {
    fn from(request: BaseFactRequest) -> Self {
        Self {
            subject_id: request.subject_id.to_str(),
            payload: request.payload.0,
        }
    }
}

impl TryFrom<NodeFactRequest> for BaseFactRequest {
    type Error = NodeError;
    fn try_from(request: NodeFactRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            subject_id: DigestIdentifier::from_str(&request.subject_id).map_err(|_| {
                NodeError::InvalidParameter("Invalid subject identifier".to_string())
            })?,
            payload: ValueWrapper(request.payload),
        })
    }
}

/// Transfer request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTransferRequest {
    /// Subject identifier
    pub subject_id: String,
    /// Public key of the new owner
    pub public_key: String,
}

impl From<BaseTransferRequest> for NodeTransferRequest {
    fn from(request: BaseTransferRequest) -> Self {
        Self {
            subject_id: request.subject_id.to_str(),
            public_key: request.public_key.to_str(),
        }
    }
}

impl TryFrom<NodeTransferRequest> for BaseTransferRequest {
    type Error = NodeError;
    fn try_from(request: NodeTransferRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            subject_id: DigestIdentifier::from_str(&request.subject_id).map_err(|_| {
                NodeError::InvalidParameter("Invalid subject identifier".to_string())
            })?,
            public_key: KeyIdentifier::from_str(&request.public_key)
                .map_err(|_| NodeError::InvalidParameter("Invalid public key".to_string()))?,
        })
    }
}

/// EOL request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEOLRequest {
    /// Subject identifier
    pub subject_id: String,
}

impl From<BaseEOLRequest> for NodeEOLRequest {
    fn from(request: BaseEOLRequest) -> Self {
        Self {
            subject_id: request.subject_id.to_str(),
        }
    }
}

impl TryFrom<NodeEOLRequest> for BaseEOLRequest {
    type Error = NodeError;
    fn try_from(request: NodeEOLRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            subject_id: DigestIdentifier::from_str(&request.subject_id).map_err(|_| {
                NodeError::InvalidParameter("Invalid subject identifier".to_string())
            })?,
        })
    }
}

/// Signed event request.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeSignedEventRequest {
    /// Event request
    pub request: NodeEventRequest,
    /// Signature
    pub signature: Option<NodeSignature>,
}

impl From<NodeSigned<BaseEventRequest>> for NodeSignedEventRequest {
    fn from(signed: NodeSigned<BaseEventRequest>) -> Self {
        Self {
            request: NodeEventRequest::from(signed.content),
            signature: Some(signed.signature),
        }
    }
}

impl TryFrom<NodeSignedEventRequest> for NodeSigned<kore_base::EventRequest> {
    type Error = NodeError;
    fn try_from(signed: NodeSignedEventRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            content: kore_base::EventRequest::try_from(signed.request)?,
            signature: signed
                .signature
                .ok_or_else(|| NodeError::InvalidParameter("Missing signature".to_string()))?,
        })
    }
}

impl From<BaseKoreRequest> for NodeSignedEventRequest {
    fn from(request: BaseKoreRequest) -> Self {
        let signed: NodeSigned<NodeEventRequest> = NodeSigned {
            content: request.event_request.content.into(),
            signature: request.event_request.signature.into(),
        };
        Self {
            request: signed.content,
            signature: Some(signed.signature),
        }
    }
}

/// Event request response
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventRequestResponse {
    /// Event request identifier
    pub request_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeKoreRequest {
    /// The identifier of the request.
    pub id: String,
    /// The identifier of the subject associated with the request, if any.
    pub subject_id: Option<String>,
    /// The sequence number of the request, if any.
    pub sn: Option<u64>,
    /// The event request associated with the request.
    pub event_request: NodeSignedEventRequest,
    /// The state of the request.
    pub state: RequestState,
    /// The success status of the request, if any.
    pub success: Option<bool>,
}

impl From<BaseKoreRequest> for NodeKoreRequest {
    fn from(request: BaseKoreRequest) -> Self {
        let event_request = NodeSignedEventRequest::from(request.clone());

        Self {
            id: request.id.to_str().to_string(),
            subject_id: Some(request.subject_id.unwrap_or_default().to_string()),
            sn: request.sn,
            event_request,
            state: request.state,
            success: request.success,
        }
    }
}

impl TryFrom<NodeKoreRequest> for BaseKoreRequest {
    type Error = NodeError;

    fn try_from(request: NodeKoreRequest) -> Result<Self, Self::Error> {
        type SignedType = NodeSigned<kore_base::EventRequest>;
        type BaseSignedType = BaseSigned<kore_base::EventRequest>;

        Ok(Self {
            id: DigestIdentifier::from_str(&request.id)
                .map_err(|_| NodeError::InvalidParameter("Invalid id identifier".to_string()))?,
            subject_id: Some(
                DigestIdentifier::from_str(&request.subject_id.unwrap_or_default()).map_err(
                    |_| NodeError::InvalidParameter("Invalid subject identifier".to_string()),
                )?,
            ),
            sn: request.sn,
            event_request: BaseSignedType::try_from(SignedType::try_from(request.event_request)?)?,
            state: request.state,
            success: request.success,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeKoreRequestState {
    /// Request identifier
    pub id: String,
    /// Subject identifier
    pub subject_id: Option<String>,
    /// Current sequence number of the subject
    pub sn: Option<u64>,
    /// Current status of the request
    pub state: RequestState,
    /// Value that says if the request has been successful
    pub success: Option<bool>,
}

impl From<BaseKoreRequest> for NodeKoreRequestState {
    fn from(value: BaseKoreRequest) -> Self {
        Self {
            id: value.id.to_str(),
            subject_id: value.subject_id.map(|id| id.to_str()),
            sn: value.sn,
            state: value.state,
            success: value.success,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeGetApprovals {
    /// Status of approvals
    pub status: Option<String>,
    /// Request for approval from which the query is made (being excluded)
    pub from: Option<String>,
    /// Number of entries
    pub quantity: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeApprovalRequest {
    // Evaluation Request
    /// Signature of the event request
    pub event_request: NodeSignedEventRequest,
    /// Current sequence number of the subject
    pub sn: u64,
    /// Governance version
    pub gov_version: u64,
    // Evaluation Response
    /// Changes to be applied to the subject
    pub patch: Value,
    /// Hash of the state
    pub state_hash: String,
    /// Previous event hash
    pub hash_prev_event: String,
}

impl From<BaseApprovalRequest> for NodeApprovalRequest {
    fn from(value: BaseApprovalRequest) -> Self {
        Self {
            event_request: NodeSignedEventRequest::from(NodeSigned::from(value.event_request)),
            sn: value.sn,
            gov_version: value.gov_version,
            patch: value.patch.0,
            state_hash: value.state_hash.to_str(),
            hash_prev_event: value.hash_prev_event.to_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeApprovalEntity {
    /// Approval request identifier
    pub id: String,
    /// Signature of the request for approval
    pub request: NodeSigned<NodeApprovalRequest>,
    /// Signature of the petition by approvers
    pub reponse: Option<NodeSigned<NodeApprovalResponse>>,
    /// Current status of the request
    pub state: BaseApprovalState,
}

impl From<BaseApprovalEntity> for NodeApprovalEntity {
    fn from(value: BaseApprovalEntity) -> Self {
        Self {
            id: value.id.to_str(),
            request: NodeSigned::from(value.request),
            reponse: value.response.map(NodeSigned::from),
            state: value.state,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeApprovalResponse {
    /// Hash of the request for approval
    pub appr_req_hash: String,
    /// Value specifying if it has been approved
    pub approved: bool,
}

impl From<BaseApprovalResponse> for NodeApprovalResponse {
    fn from(value: BaseApprovalResponse) -> Self {
        Self {
            appr_req_hash: value.appr_req_hash.to_str(),
            approved: value.approved,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum PatchVote{
    /// Vote to accept a particular request
    RespondedAccepted,
    /// Vote to reject a particular request
    RespondedRejected,
}
