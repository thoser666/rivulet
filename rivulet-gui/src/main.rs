use eframe::egui;
use rivulet_core::*;
use rivulet_obs_compat::PluginManager;
use tracing_subscriber::EnvFilter;

mod app;

use app::RivuletApp;

fn main() {
    // Advanced logging with env_filter
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("rivulet=debug,info"))
        )
        .init();

    tracing::info!("Starting Rivulet GUI");

    // Initialize OBS plugin compatibility
    if let Err(e) = PluginManager::initialize() {
        tracing::warn!("Failed to initialize OBS compatibility: {}", e);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Rivulet - Streaming Software")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Rivulet",
        options,
        Box::new(|cc| {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            let engine = rt.block_on(async {
                RivuletEngine::new().expect("Failed to initialize engine")
            });

            Box::new(RivuletApp::new(cc, engine, rt)) as Box<dyn eframe::App>
        }),
    ) {
        tracing::error!("Application error: {}", e);
        std::process::exit(1);
    }
}