// Copyright 2024 Antonio Estévez
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Event request.
//!

use crate::error::NodeError;

use kore_base::{
    identifier::{Derivable, DigestIdentifier, KeyIdentifier},
    request::{
        EOLRequest as BaseEOLRequest, FactRequest as BaseFactRequest, KoreRequest, RequestState,
        StartRequest as BaseStartRequest, TransferRequest as BaseTransferRequest,
    },
    EventRequest as BaseEventRequest, ValueWrapper,
};

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Signature, Signed};

/// Event request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EventRequest {
    Create(StartRequest),
    Fact(FactRequest),
    Transfer(TransferRequest),
    EOL(EOLRequest),
}

impl From<BaseEventRequest> for EventRequest {
    fn from(request: BaseEventRequest) -> Self {
        match request {
            BaseEventRequest::Create(request) => Self::Create(request.into()),
            BaseEventRequest::Fact(request) => Self::Fact(request.into()),
            BaseEventRequest::Transfer(request) => Self::Transfer(request.into()),
            BaseEventRequest::EOL(request) => Self::EOL(request.into()),
        }
    }
}

impl TryFrom<EventRequest> for BaseEventRequest {
    type Error = NodeError;
    fn try_from(request: EventRequest) -> Result<Self, Self::Error> {
        match request {
            EventRequest::Create(request) => Ok(Self::Create(request.try_into()?)),
            EventRequest::Fact(request) => Ok(Self::Fact(request.try_into()?)),
            EventRequest::Transfer(request) => Ok(Self::Transfer(request.try_into()?)),
            EventRequest::EOL(request) => Ok(Self::EOL(request.try_into()?)),
        }
    }
}

/// Create request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StartRequest {
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

impl From<BaseStartRequest> for StartRequest {
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

impl TryFrom<StartRequest> for BaseStartRequest {
    type Error = NodeError;
    fn try_from(request: StartRequest) -> Result<Self, Self::Error> {
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
pub struct FactRequest {
    /// Subject identifier
    pub subject_id: String,
    /// Changes to be applied to the subject
    pub payload: Value,
}

impl From<BaseFactRequest> for FactRequest {
    fn from(request: BaseFactRequest) -> Self {
        Self {
            subject_id: request.subject_id.to_str(),
            payload: request.payload.0,
        }
    }
}

impl TryFrom<FactRequest> for BaseFactRequest {
    type Error = NodeError;
    fn try_from(request: FactRequest) -> Result<Self, Self::Error> {
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
pub struct TransferRequest {
    /// Subject identifier
    pub subject_id: String,
    /// Public key of the new owner
    pub public_key: String,
}

impl From<BaseTransferRequest> for TransferRequest {
    fn from(request: BaseTransferRequest) -> Self {
        Self {
            subject_id: request.subject_id.to_str(),
            public_key: request.public_key.to_str(),
        }
    }
}

impl TryFrom<TransferRequest> for BaseTransferRequest {
    type Error = NodeError;
    fn try_from(request: TransferRequest) -> Result<Self, Self::Error> {
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
pub struct EOLRequest {
    /// Subject identifier
    pub subject_id: String,
}

impl From<BaseEOLRequest> for EOLRequest {
    fn from(request: BaseEOLRequest) -> Self {
        Self {
            subject_id: request.subject_id.to_str(),
        }
    }
}

impl TryFrom<EOLRequest> for BaseEOLRequest {
    type Error = NodeError;
    fn try_from(request: EOLRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            subject_id: DigestIdentifier::from_str(&request.subject_id).map_err(|_| {
                NodeError::InvalidParameter("Invalid subject identifier".to_string())
            })?,
        })
    }
}

/// Signed event request.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignedEventRequest {
    /// Event request
    pub request: EventRequest,
    /// Signature
    pub signature: Option<Signature>,
}

impl From<Signed<BaseEventRequest>> for SignedEventRequest {
    fn from(signed: Signed<BaseEventRequest>) -> Self {
        Self {
            request: EventRequest::from(signed.content),
            signature: Some(signed.signature),
        }
    }
}

impl TryFrom<SignedEventRequest> for Signed<kore_base::EventRequest> {
    type Error = NodeError;
    fn try_from(signed: SignedEventRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            content: kore_base::EventRequest::try_from(signed.request)?,
            signature: signed
                .signature
                .ok_or_else(|| NodeError::InvalidParameter("Missing signature".to_string()))?,
        })
    }
}

impl From<KoreRequest> for SignedEventRequest {
    fn from(request: KoreRequest) -> Self {
        let signed: Signed<EventRequest> = Signed {
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
pub struct KoreRequestState {
    /// The identifier of the request.
    pub id: String,
    /// The identifier of the subject associated with the request, if any.
    pub subject_id: Option<String>,
    /// The sequence number of the request, if any.
    pub sn: Option<u64>,
    /// The event request associated with the request.
    pub event_request: SignedEventRequest,
    /// The state of the request.
    pub state: RequestState,
    /// The success status of the request, if any.
    pub success: Option<bool>,
}
