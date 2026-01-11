use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ===== Commands =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProductCommand {
    RegisterProduct {
        product_id: Uuid,
        product_code: String,
        product_name: String,
        unit: String,
    },
    RecordInbound {
        inbound_id: Uuid,
        quantity: i32,
        inbound_date: DateTime<Utc>,
        supplier: String,
    },
    RecordOutbound {
        outbound_id: Uuid,
        quantity: i32,
        outbound_date: DateTime<Utc>,
        destination: String,
    },
}
