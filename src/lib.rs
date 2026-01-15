// Inventory Management System - MVP Edition
// Event Sourcing/CQRS Architecture with Rust

pub mod domain;
pub mod services;
pub mod web;

// Re-export main types for convenience
pub use domain::{Product, ProductCommand, ProductError, ProductEvent, ProductServices};
pub use services::{ProductView, ProductViewRepository};
pub use web::AppState;
