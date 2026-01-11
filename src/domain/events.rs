use chrono::{DateTime, Utc};
use cqrs_es::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ===== Events =====

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProductEvent {
    ProductRegistered {
        product_id: Uuid,
        product_code: String,
        product_name: String,
        unit: String,
        timestamp: DateTime<Utc>,
    },
    InboundRecorded {
        inbound_id: Uuid,
        quantity: i32,
        inbound_date: DateTime<Utc>,
        supplier: String,
    },
    OutboundRecorded {
        outbound_id: Uuid,
        quantity: i32,
        outbound_date: DateTime<Utc>,
        destination: String,
    },
}

impl DomainEvent for ProductEvent {
    fn event_type(&self) -> String {
        match self {
            ProductEvent::ProductRegistered { .. } => "ProductRegistered".to_string(),
            ProductEvent::InboundRecorded { .. } => "InboundRecorded".to_string(),
            ProductEvent::OutboundRecorded { .. } => "OutboundRecorded".to_string(),
        }
    }

    fn event_version(&self) -> String {
        "1.0".to_string()
    }
}

// ===== Errors =====

#[derive(Debug, thiserror::Error)]
pub enum ProductError {
    #[error("Product already exists: {0}")]
    ProductAlreadyExists(String),

    #[error("Product not found: {0}")]
    ProductNotFound(String),

    #[error("Invalid quantity: {0}")]
    InvalidQuantity(String),

    #[error("Insufficient stock - current: {current}, requested: {requested}")]
    InsufficientStock { current: i32, requested: i32 },
}
