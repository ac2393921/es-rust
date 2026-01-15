use actix_web::{middleware::Logger, web, App, HttpServer};
use inventory_system::web::{
    get_all_products, get_product, record_inbound, record_outbound, register_product, AppState,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let host = "127.0.0.1";
    let port = 8080;

    log::info!("========================================");
    log::info!("Inventory Management System - MVP Edition");
    log::info!("Event Sourcing/CQRS Architecture with Rust");
    log::info!("========================================");
    log::info!("Starting server at http://{host}:{port}");
    log::info!("Using in-memory event store");
    log::info!("========================================");

    let app_state = AppState::new();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/api/products")
                    .route("", web::post().to(register_product))
                    .route("", web::get().to(get_all_products))
                    .route("/{id}", web::get().to(get_product))
                    .route("/{id}/inbound", web::post().to(record_inbound))
                    .route("/{id}/outbound", web::post().to(record_outbound)),
            )
    })
    .bind((host, port))?
    .run()
    .await
}
