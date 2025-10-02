use eframe::egui;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Rivulet")
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_simple_native("Rivulet", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rivulet");
            ui.label("OBS Studio in Rust");
            ui.separator();
            ui.label("Project initialized!");
        });
    })?;

    Ok(())
}
