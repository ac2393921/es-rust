pub mod aggregate;
pub mod commands;
pub mod events;

// Re-export for convenience
pub use aggregate::{Product, ProductServices};
pub use commands::ProductCommand;
pub use events::{ProductError, ProductEvent};
