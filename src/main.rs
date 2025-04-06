use std::sync::Arc;
use std::thread;

use chat_app::create_chat_framework;
use chat_app::tui::TuiApp;
use chat_app::web::WebApi;

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    let (framework, view_repository) = create_chat_framework();
    let framework = Arc::new(framework);
    
    let web_framework = framework.clone();
    let web_view_repository = view_repository.clone();
    
    thread::spawn(move || {
        let rt = actix_web::rt::System::new();
        rt.block_on(async {
            let web_api = WebApi::new(web_framework, web_view_repository);
            web_api.run("0.0.0.0", 8080).await.expect("Failed to start web API");
        });
    });
    
    let mut tui_app = TuiApp::new(framework, view_repository);
    tui_app.run();
}
