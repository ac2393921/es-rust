use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use cqrs_es::{EventEnvelope, Query};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{Product, ProductEvent};

// ===== Read Model: ProductView =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductView {
    pub product_id: Uuid,
    pub product_code: String,
    pub product_name: String,
    pub unit: String,
    pub current_stock: i32,
}

// ===== ViewRepository: インメモリRead Model管理 =====

#[derive(Debug, Clone)]
pub struct ProductViewRepository {
    views: Arc<Mutex<HashMap<String, ProductView>>>,
}

impl ProductViewRepository {
    pub fn new() -> Self {
        Self {
            views: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, product_id: &str) -> Option<ProductView> {
        self.views.lock().unwrap().get(product_id).cloned()
    }

    pub fn get_all(&self) -> Vec<ProductView> {
        self.views.lock().unwrap().values().cloned().collect()
    }
}

impl Default for ProductViewRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Query<Product> for ProductViewRepository {
    async fn dispatch(&self, aggregate_id: &str, events: &[EventEnvelope<Product>]) {
        for envelope in events {
            let event = &envelope.payload;
            match event {
                ProductEvent::ProductRegistered {
                    product_id,
                    product_code,
                    product_name,
                    unit,
                    ..
                } => {
                    let view = ProductView {
                        product_id: *product_id,
                        product_code: product_code.clone(),
                        product_name: product_name.clone(),
                        unit: unit.clone(),
                        current_stock: 0,
                    };
                    self.views
                        .lock()
                        .unwrap()
                        .insert(aggregate_id.to_string(), view);
                }
                ProductEvent::InboundRecorded { quantity, .. } => {
                    if let Some(view) = self.views.lock().unwrap().get_mut(aggregate_id) {
                        view.current_stock += quantity;
                    }
                }
                ProductEvent::OutboundRecorded { quantity, .. } => {
                    if let Some(view) = self.views.lock().unwrap().get_mut(aggregate_id) {
                        view.current_stock -= quantity;
                    }
                }
            }
        }
    }
}

// ===== EventStore: PostgreSQL永続化 =====
// TODO: PostgreSQL EventStore実装は複雑なため、まずは基本機能を動かしてから実装する
// 当面はcqrs-es::MemStoreを使用する想定
