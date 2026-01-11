// PostgresEventStoreとの統合例
// 注意: これは実際に動作させるにはPostgreSQLが必要

use cqrs_es::persist::PersistedEventStore;
use cqrs_es::CqrsFramework;
use postgres_es::PostgresCqrs;
use sqlx::PgPool;

use crate::{Product, ProductServices};

/// PostgresEventStoreの初期化例
pub async fn create_cqrs_framework(
    database_url: &str,
) -> Result<CqrsFramework<Product, PersistedEventStore<Product, PgPool>>, Box<dyn std::error::Error>> {
    // PostgreSQL接続プールの作成
    let pool = PgPool::connect(database_url).await?;

    // Event Storeの作成
    // cqrs-es 0.4.xでは、PostgresCqrsを使ってCqrsFrameworkを作成する
    let (cqrs, _) = postgres_es::postgres_cqrs(
        pool,
        vec![/* Query processors here */],
        ProductServices::default(),
    );

    Ok(cqrs)
}

/// スキーマ作成SQL（参考）
pub const SCHEMA_SQL: &str = r#"
-- Event Store用のテーブル
CREATE TABLE events (
    aggregate_type VARCHAR(255) NOT NULL,
    aggregate_id VARCHAR(255) NOT NULL,
    sequence BIGINT NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    event_version VARCHAR(20) NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (aggregate_type, aggregate_id, sequence)
);

CREATE INDEX idx_events_aggregate ON events(aggregate_type, aggregate_id);
CREATE INDEX idx_events_timestamp ON events(timestamp);

-- Snapshot用のテーブル（将来的な最適化）
CREATE TABLE snapshots (
    aggregate_type VARCHAR(255) NOT NULL,
    aggregate_id VARCHAR(255) NOT NULL,
    last_sequence BIGINT NOT NULL,
    payload JSONB NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (aggregate_type, aggregate_id)
);
"#;

/*
使用例:

use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 環境変数から接続文字列を取得
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/inventory_system".to_string());

    // CqrsFrameworkの初期化
    let cqrs = create_cqrs_framework(&database_url)
        .await
        .expect("Failed to create CQRS framework");

    // actix-webのApp Dataとして登録
    let cqrs_data = web::Data::new(std::sync::Arc::new(cqrs));

    HttpServer::new(move || {
        App::new()
            .app_data(cqrs_data.clone())
            .route("/api/products", web::post().to(register_product))
            // 他のルート...
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn register_product(
    cqrs: web::Data<std::sync::Arc<CqrsFramework<Product, _>>>,
    req: web::Json<RegisterProductRequest>,
) -> Result<HttpResponse, Error> {
    let product_id = uuid::Uuid::new_v4();
    let command = ProductCommand::RegisterProduct {
        product_id,
        product_code: req.product_code.clone(),
        product_name: req.product_name.clone(),
        unit: req.unit.clone(),
    };

    // Commandを実行
    cqrs.execute(&product_id.to_string(), command).await
        .map_err(|e| /* エラー変換 */)?;

    Ok(HttpResponse::Created().json(RegisterProductResponse { product_id }))
}
*/
