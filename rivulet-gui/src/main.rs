use eframe::egui;
use rivulet_core::RivuletEngine;

// 1. Deklariere, dass die Datei `src/app.rs` als Modul existiert.
mod app;
// 2. Importiere `RivuletApp` aus deinem eigenen Crate über das `app`-Modul.
use crate::app::RivuletApp;

fn main() -> anyhow::Result<()> {
    // Logging initialisieren
    tracing_subscriber::fmt::init();

    // Tokio Runtime erstellen
    let rt = tokio::runtime::Runtime::new()?;

    // Engine erstellen
    let engine = RivuletEngine::new();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    // Die eframe-App ausführen
    eframe::run_native(
        "Rivulet - Screen Recording & Streaming",
        options,
        Box::new(move |cc| {
            let app = RivuletApp::new(cc, engine, rt);
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {}", e))
}

// Icon-Ladefunktion (Placeholder)
fn load_icon() -> egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        // Erstelle ein einfaches 64x64 blaues Quadrat als Icon
        let image = image::RgbaImage::from_pixel(64, 64, image::Rgba([33, 150, 243, 255]));
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}
