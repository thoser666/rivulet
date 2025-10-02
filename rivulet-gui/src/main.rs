use anyhow::Result;
use eframe::egui;
use rivulet_core::*;
use rivulet_obs_compat::PluginManager;

mod app;

use app::RivuletApp;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("rivulet=debug,info")
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

    eframe::run_native(
        "Rivulet",
        options,
        Box::new(|cc| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let engine = rt.block_on(async {
                RivuletEngine::new()
            }).unwrap();

            Ok(Box::new(RivuletApp::new(cc, engine, rt)))
        }),
    )?;

    Ok(())
}