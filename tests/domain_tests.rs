// Domain Layer Integration Tests
// Tests for Product Aggregate business logic

use chrono::{TimeZone, Utc};
use cqrs_es::test::TestFramework;
use inventory_system::domain::{Product, ProductCommand, ProductEvent, ProductServices};
use uuid::Uuid;

type ProductTestFramework = TestFramework<Product>;

#[test]
fn test_register_product_success() {
    let product_id = Uuid::new_v4();
    let fixed_time = Utc.with_ymd_and_hms(2026, 1, 16, 12, 0, 0).unwrap();
    let services = ProductServices::with_fixed_time(fixed_time);

    ProductTestFramework::with(services)
        .given_no_previous_events()
        .when(ProductCommand::RegisterProduct {
            product_id,
            product_code: "P001".to_string(),
            product_name: "商品A".to_string(),
            unit: "個".to_string(),
        })
        .then_expect_events(vec![ProductEvent::ProductRegistered {
            product_id,
            product_code: "P001".to_string(),
            product_name: "商品A".to_string(),
            unit: "個".to_string(),
            timestamp: fixed_time,
        }]);
}

#[test]
fn test_register_product_already_exists() {
    let product_id = Uuid::new_v4();

    ProductTestFramework::with(ProductServices::new())
        .given(vec![ProductEvent::ProductRegistered {
            product_id,
            product_code: "P001".to_string(),
            product_name: "商品A".to_string(),
            unit: "個".to_string(),
            timestamp: Utc::now(),
        }])
        .when(ProductCommand::RegisterProduct {
            product_id,
            product_code: "P001".to_string(),
            product_name: "商品A".to_string(),
            unit: "個".to_string(),
        })
        .then_expect_error_message(&format!("Product already exists: {product_id}"));
}

#[test]
fn test_record_inbound_success() {
    let product_id = Uuid::new_v4();
    let inbound_id = Uuid::new_v4();
    let inbound_date = Utc::now();

    ProductTestFramework::with(ProductServices::new())
        .given(vec![ProductEvent::ProductRegistered {
            product_id,
            product_code: "P001".to_string(),
            product_name: "商品A".to_string(),
            unit: "個".to_string(),
            timestamp: Utc::now(),
        }])
        .when(ProductCommand::RecordInbound {
            inbound_id,
            quantity: 100,
            inbound_date,
            supplier: "仕入先A".to_string(),
        })
        .then_expect_events(vec![ProductEvent::InboundRecorded {
            inbound_id,
            quantity: 100,
            inbound_date,
            supplier: "仕入先A".to_string(),
        }]);
}

#[test]
fn test_record_inbound_product_not_found() {
    let inbound_id = Uuid::new_v4();

    ProductTestFramework::with(ProductServices::new())
        .given_no_previous_events()
        .when(ProductCommand::RecordInbound {
            inbound_id,
            quantity: 100,
            inbound_date: Utc::now(),
            supplier: "仕入先A".to_string(),
        })
        .then_expect_error_message("Product not found: Product not registered");
}

#[test]
fn test_record_inbound_invalid_quantity() {
    let product_id = Uuid::new_v4();
    let inbound_id = Uuid::new_v4();

    ProductTestFramework::with(ProductServices::new())
        .given(vec![ProductEvent::ProductRegistered {
            product_id,
            product_code: "P001".to_string(),
            product_name: "商品A".to_string(),
            unit: "個".to_string(),
            timestamp: Utc::now(),
        }])
        .when(ProductCommand::RecordInbound {
            inbound_id,
            quantity: 0, // 不正な数量
            inbound_date: Utc::now(),
            supplier: "仕入先A".to_string(),
        })
        .then_expect_error_message("Invalid quantity: Quantity must be positive, got: 0");
}

#[test]
fn test_record_outbound_success() {
    let product_id = Uuid::new_v4();
    let inbound_id = Uuid::new_v4();
    let outbound_id = Uuid::new_v4();
    let outbound_date = Utc::now();

    ProductTestFramework::with(ProductServices::new())
        .given(vec![
            ProductEvent::ProductRegistered {
                product_id,
                product_code: "P001".to_string(),
                product_name: "商品A".to_string(),
                unit: "個".to_string(),
                timestamp: Utc::now(),
            },
            ProductEvent::InboundRecorded {
                inbound_id,
                quantity: 100,
                inbound_date: Utc::now(),
                supplier: "仕入先A".to_string(),
            },
        ])
        .when(ProductCommand::RecordOutbound {
            outbound_id,
            quantity: 50,
            outbound_date,
            destination: "出荷先B".to_string(),
        })
        .then_expect_events(vec![ProductEvent::OutboundRecorded {
            outbound_id,
            quantity: 50,
            outbound_date,
            destination: "出荷先B".to_string(),
        }]);
}

#[test]
fn test_record_outbound_insufficient_stock() {
    let product_id = Uuid::new_v4();
    let inbound_id = Uuid::new_v4();
    let outbound_id = Uuid::new_v4();

    ProductTestFramework::with(ProductServices::new())
        .given(vec![
            ProductEvent::ProductRegistered {
                product_id,
                product_code: "P001".to_string(),
                product_name: "商品A".to_string(),
                unit: "個".to_string(),
                timestamp: Utc::now(),
            },
            ProductEvent::InboundRecorded {
                inbound_id,
                quantity: 30,
                inbound_date: Utc::now(),
                supplier: "仕入先A".to_string(),
            },
        ])
        .when(ProductCommand::RecordOutbound {
            outbound_id,
            quantity: 50, // 在庫30に対して50を出庫
            outbound_date: Utc::now(),
            destination: "出荷先B".to_string(),
        })
        .then_expect_error_message("Insufficient stock - current: 30, requested: 50");
}

#[test]
fn test_record_outbound_product_not_found() {
    let outbound_id = Uuid::new_v4();

    ProductTestFramework::with(ProductServices::new())
        .given_no_previous_events()
        .when(ProductCommand::RecordOutbound {
            outbound_id,
            quantity: 50,
            outbound_date: Utc::now(),
            destination: "出荷先B".to_string(),
        })
        .then_expect_error_message("Product not found: Product not registered");
}

#[test]
fn test_record_outbound_invalid_quantity() {
    let product_id = Uuid::new_v4();
    let inbound_id = Uuid::new_v4();
    let outbound_id = Uuid::new_v4();

    ProductTestFramework::with(ProductServices::new())
        .given(vec![
            ProductEvent::ProductRegistered {
                product_id,
                product_code: "P001".to_string(),
                product_name: "商品A".to_string(),
                unit: "個".to_string(),
                timestamp: Utc::now(),
            },
            ProductEvent::InboundRecorded {
                inbound_id,
                quantity: 100,
                inbound_date: Utc::now(),
                supplier: "仕入先A".to_string(),
            },
        ])
        .when(ProductCommand::RecordOutbound {
            outbound_id,
            quantity: 0, // 不正な数量
            outbound_date: Utc::now(),
            destination: "出荷先B".to_string(),
        })
        .then_expect_error_message("Invalid quantity: Quantity must be positive, got: 0");
}

#[test]
fn test_record_outbound_exact_stock() {
    let product_id = Uuid::new_v4();
    let inbound_id = Uuid::new_v4();
    let outbound_id = Uuid::new_v4();
    let outbound_date = Utc::now();

    ProductTestFramework::with(ProductServices::new())
        .given(vec![
            ProductEvent::ProductRegistered {
                product_id,
                product_code: "P001".to_string(),
                product_name: "商品A".to_string(),
                unit: "個".to_string(),
                timestamp: Utc::now(),
            },
            ProductEvent::InboundRecorded {
                inbound_id,
                quantity: 50,
                inbound_date: Utc::now(),
                supplier: "仕入先A".to_string(),
            },
        ])
        .when(ProductCommand::RecordOutbound {
            outbound_id,
            quantity: 50, // 在庫数ちょうど
            outbound_date,
            destination: "出荷先B".to_string(),
        })
        .then_expect_events(vec![ProductEvent::OutboundRecorded {
            outbound_id,
            quantity: 50,
            outbound_date,
            destination: "出荷先B".to_string(),
        }]);
}
