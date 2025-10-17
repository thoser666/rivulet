// Dieses Attribut muss ganz oben stehen.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Modul-Deklarationen
mod app;

// Importe
use crate::app::RivuletApp;
use eframe::egui::{self, IconData};
use rivulet_core::RivuletEngine;
use tokio::runtime::Runtime;

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();
    run_native("Rivulet", RivuletEngine::new(), Runtime::new().unwrap())
}

fn run_native(app_name: &str, engine: RivuletEngine, _rt: Runtime) -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(800.0, 600.0))
            .with_title(app_name)
            .with_icon(create_icon())
            // KORREKTUR: Die Methode heißt .with_app_id()
            .with_app_id("rivulet_main_window"),

        ..Default::default()
    };

    eframe::run_native(
        app_name,
        native_options,
        Box::new(|cc| {
            let app = RivuletApp::new(cc, engine);
            Box::new(app)
        }),
    )
}

fn create_icon() -> IconData {
    let image = image::RgbaImage::from_fn(64, 64, |_x, _y| image::Rgba([33, 150, 243, 255]));

    IconData {
        rgba: image.into_raw(),
        width: 64,
        height: 64,
    }
}
