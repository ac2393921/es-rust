use actix_web::{web, HttpResponse, Result};
use chrono::Utc;
use cqrs_es::mem_store::MemStore;
use cqrs_es::CqrsFramework;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{Product, ProductCommand, ProductError, ProductServices};
use crate::services::ProductViewRepository;

// ===== Application State =====

#[derive(Clone)]
pub struct AppState {
    pub cqrs: Arc<CqrsFramework<Product, MemStore<Product>>>,
    pub view_repo: ProductViewRepository,
}

impl AppState {
    pub fn new() -> Self {
        let store = MemStore::<Product>::default();
        let view_repo = ProductViewRepository::new();
        let queries: Vec<Box<dyn cqrs_es::Query<Product>>> = vec![Box::new(view_repo.clone())];
        let services = ProductServices;

        let cqrs = Arc::new(CqrsFramework::new(store, queries, services));

        Self { cqrs, view_repo }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// ===== Request/Response DTOs =====

#[derive(Debug, Deserialize)]
pub struct RegisterProductRequest {
    pub product_code: String,
    pub product_name: String,
    pub unit: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordInboundRequest {
    pub quantity: i32,
    pub supplier: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordOutboundRequest {
    pub quantity: i32,
    pub destination: String,
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub product_id: String,
    pub product_code: String,
    pub product_name: String,
    pub unit: String,
    pub current_stock: i32,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ===== Error Mapping =====

fn map_error(error: cqrs_es::AggregateError<ProductError>) -> HttpResponse {
    use cqrs_es::AggregateError;

    match error {
        AggregateError::UserError(e) => match e {
            ProductError::ProductAlreadyExists(msg) => {
                HttpResponse::Conflict().json(ErrorResponse { error: msg })
            }
            ProductError::ProductNotFound(msg) => {
                HttpResponse::NotFound().json(ErrorResponse { error: msg })
            }
            ProductError::InvalidQuantity(msg) => {
                HttpResponse::BadRequest().json(ErrorResponse { error: msg })
            }
            ProductError::InsufficientStock { current, requested } => HttpResponse::BadRequest()
                .json(ErrorResponse {
                    error: format!(
                        "Insufficient stock - current: {current}, requested: {requested}"
                    ),
                }),
        },
        AggregateError::DatabaseConnectionError(msg) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {msg}"),
            })
        }
        AggregateError::DeserializationError(msg) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Deserialization error: {msg}"),
            })
        }
        _ => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Internal server error".to_string(),
        }),
    }
}

// ===== API Handlers =====

/// POST /api/products - 商品登録
pub async fn register_product(
    state: web::Data<AppState>,
    req: web::Json<RegisterProductRequest>,
) -> Result<HttpResponse> {
    let product_id = Uuid::new_v4();
    let aggregate_id = product_id.to_string();

    let command = ProductCommand::RegisterProduct {
        product_id,
        product_code: req.product_code.clone(),
        product_name: req.product_name.clone(),
        unit: req.unit.clone(),
    };

    match state.cqrs.execute(&aggregate_id, command).await {
        Ok(_) => {
            if let Some(view) = state.view_repo.get(&aggregate_id) {
                let response = ProductResponse {
                    product_id: view.product_id.to_string(),
                    product_code: view.product_code,
                    product_name: view.product_name,
                    unit: view.unit,
                    current_stock: view.current_stock,
                };
                Ok(HttpResponse::Created().json(response))
            } else {
                Ok(HttpResponse::Created().json(ErrorResponse {
                    error: "Product created but view not found".to_string(),
                }))
            }
        }
        Err(e) => Ok(map_error(e)),
    }
}

/// GET /api/products/{id} - 商品照会
pub async fn get_product(
    state: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse> {
    let aggregate_id = id.into_inner();

    if let Some(view) = state.view_repo.get(&aggregate_id) {
        let response = ProductResponse {
            product_id: view.product_id.to_string(),
            product_code: view.product_code,
            product_name: view.product_name,
            unit: view.unit,
            current_stock: view.current_stock,
        };
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().json(ErrorResponse {
            error: "Product not found".to_string(),
        }))
    }
}

/// GET /api/products - 全商品照会
pub async fn get_all_products(state: web::Data<AppState>) -> Result<HttpResponse> {
    let views = state.view_repo.get_all();
    let responses: Vec<ProductResponse> = views
        .into_iter()
        .map(|view| ProductResponse {
            product_id: view.product_id.to_string(),
            product_code: view.product_code,
            product_name: view.product_name,
            unit: view.unit,
            current_stock: view.current_stock,
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

/// POST /api/products/{id}/inbound - 入庫記録
pub async fn record_inbound(
    state: web::Data<AppState>,
    id: web::Path<String>,
    req: web::Json<RecordInboundRequest>,
) -> Result<HttpResponse> {
    let aggregate_id = id.into_inner();
    let inbound_id = Uuid::new_v4();

    let command = ProductCommand::RecordInbound {
        inbound_id,
        quantity: req.quantity,
        inbound_date: Utc::now(),
        supplier: req.supplier.clone(),
    };

    match state.cqrs.execute(&aggregate_id, command).await {
        Ok(_) => {
            if let Some(view) = state.view_repo.get(&aggregate_id) {
                let response = ProductResponse {
                    product_id: view.product_id.to_string(),
                    product_code: view.product_code,
                    product_name: view.product_name,
                    unit: view.unit,
                    current_stock: view.current_stock,
                };
                Ok(HttpResponse::Ok().json(response))
            } else {
                Ok(HttpResponse::Ok().json(ErrorResponse {
                    error: "Inbound recorded but view not found".to_string(),
                }))
            }
        }
        Err(e) => Ok(map_error(e)),
    }
}

/// POST /api/products/{id}/outbound - 出庫記録
pub async fn record_outbound(
    state: web::Data<AppState>,
    id: web::Path<String>,
    req: web::Json<RecordOutboundRequest>,
) -> Result<HttpResponse> {
    let aggregate_id = id.into_inner();
    let outbound_id = Uuid::new_v4();

    let command = ProductCommand::RecordOutbound {
        outbound_id,
        quantity: req.quantity,
        outbound_date: Utc::now(),
        destination: req.destination.clone(),
    };

    match state.cqrs.execute(&aggregate_id, command).await {
        Ok(_) => {
            if let Some(view) = state.view_repo.get(&aggregate_id) {
                let response = ProductResponse {
                    product_id: view.product_id.to_string(),
                    product_code: view.product_code,
                    product_name: view.product_name,
                    unit: view.unit,
                    current_stock: view.current_stock,
                };
                Ok(HttpResponse::Ok().json(response))
            } else {
                Ok(HttpResponse::Ok().json(ErrorResponse {
                    error: "Outbound recorded but view not found".to_string(),
                }))
            }
        }
        Err(e) => Ok(map_error(e)),
    }
}
