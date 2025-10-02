use eframe::egui;
use rivulet_core::*;

pub struct RivuletApp {
    engine: RivuletEngine,
    rt: tokio::runtime::Runtime,
}

impl RivuletApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        engine: RivuletEngine,
        rt: tokio::runtime::Runtime,
    ) -> Self {
        Self { engine, rt }
    }
}

impl eframe::App for RivuletApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🌊 Rivulet");
            ui.label("OBS Studio reimplemented in Rust");
            ui.separator();
            ui.label("Project initialized successfully!");
        });
    }
}