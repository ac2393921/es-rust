use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cqrs_es::Aggregate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::commands::ProductCommand;
use super::events::{ProductError, ProductEvent};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Product {
    pub product_id: Option<Uuid>,
    pub product_code: String,
    pub product_name: String,
    pub unit: String,
    pub current_stock: i32,
    pub created_at: Option<DateTime<Utc>>,
}

/// 商品ドメインのサービス（依存性注入用）
#[derive(Debug, Clone, Default)]
pub struct ProductServices {
    /// テスト用の固定時刻（Noneの場合はUtc::now()を使用）
    fixed_time: Option<DateTime<Utc>>,
}

impl ProductServices {
    /// 本番用インスタンスを作成（現在時刻を使用）
    pub fn new() -> Self {
        Self { fixed_time: None }
    }

    /// テスト用インスタンスを作成（固定時刻を使用）
    pub fn with_fixed_time(time: DateTime<Utc>) -> Self {
        Self {
            fixed_time: Some(time),
        }
    }

    /// 現在時刻を取得（固定時刻が設定されていればそれを返す）
    pub fn now(&self) -> DateTime<Utc> {
        self.fixed_time.unwrap_or_else(Utc::now)
    }
}

#[async_trait]
impl Aggregate for Product {
    type Command = ProductCommand;
    type Event = ProductEvent;
    type Error = ProductError;
    type Services = ProductServices;

    fn aggregate_type() -> String {
        "Product".to_string()
    }

    async fn handle(
        &self,
        command: Self::Command,
        services: &Self::Services,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        use ProductCommand::*;
        use ProductEvent::*;

        match command {
            RegisterProduct {
                product_id,
                product_code,
                product_name,
                unit,
            } => {
                // ビジネスルール: 既に登録済みの場合はエラー
                if self.product_id.is_some() {
                    return Err(ProductError::ProductAlreadyExists(product_id.to_string()));
                }

                Ok(vec![ProductRegistered {
                    product_id,
                    product_code,
                    product_name,
                    unit,
                    timestamp: services.now(),
                }])
            }

            RecordInbound {
                inbound_id,
                quantity,
                inbound_date,
                supplier,
            } => {
                // ビジネスルール: 商品が未登録の場合はエラー
                if self.product_id.is_none() {
                    return Err(ProductError::ProductNotFound(
                        "Product not registered".to_string(),
                    ));
                }

                // ビジネスルール: 数量は正の整数のみ
                if quantity <= 0 {
                    return Err(ProductError::InvalidQuantity(format!(
                        "Quantity must be positive, got: {quantity}"
                    )));
                }

                Ok(vec![InboundRecorded {
                    inbound_id,
                    quantity,
                    inbound_date,
                    supplier,
                }])
            }

            RecordOutbound {
                outbound_id,
                quantity,
                outbound_date,
                destination,
            } => {
                // ビジネスルール: 商品が未登録の場合はエラー
                if self.product_id.is_none() {
                    return Err(ProductError::ProductNotFound(
                        "Product not registered".to_string(),
                    ));
                }

                // ビジネスルール: 数量は正の整数のみ
                if quantity <= 0 {
                    return Err(ProductError::InvalidQuantity(format!(
                        "Quantity must be positive, got: {quantity}"
                    )));
                }

                // ビジネスルール: 在庫不足チェック
                if self.current_stock < quantity {
                    return Err(ProductError::InsufficientStock {
                        current: self.current_stock,
                        requested: quantity,
                    });
                }

                Ok(vec![OutboundRecorded {
                    outbound_id,
                    quantity,
                    outbound_date,
                    destination,
                }])
            }
        }
    }

    fn apply(&mut self, event: Self::Event) {
        use ProductEvent::*;

        match event {
            ProductRegistered {
                product_id,
                product_code,
                product_name,
                unit,
                timestamp,
            } => {
                self.product_id = Some(product_id);
                self.product_code = product_code;
                self.product_name = product_name;
                self.unit = unit;
                self.current_stock = 0;
                self.created_at = Some(timestamp);
            }

            InboundRecorded { quantity, .. } => {
                self.current_stock += quantity;
            }

            OutboundRecorded { quantity, .. } => {
                self.current_stock -= quantity;
            }
        }
    }
}
