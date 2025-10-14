use eframe::egui;
use rivulet_core::*;

use std::time::Instant;

#[cfg(not(target_arch = "wasm32"))]
use xcap::Monitor;

pub struct RivuletApp {
    #[cfg(not(target_arch = "wasm32"))]
    monitor: Option<Monitor>,
    #[cfg(not(target_arch = "wasm32"))]
    capture_active: bool,
    #[cfg(not(target_arch = "wasm32"))]
    current_frame: Option<CapturedFrameData>,
    #[cfg(not(target_arch = "wasm32"))]
    preview_texture: Option<egui::TextureHandle>,

    #[cfg(not(target_arch = "wasm32"))]
    fps: f32,
    #[cfg(not(target_arch = "wasm32"))]
    last_frame_time: Instant,
    #[cfg(not(target_arch = "wasm32"))]
    frame_count: u32,
}

#[cfg(not(target_arch = "wasm32"))]
struct CapturedFrameData {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl RivuletApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        _engine: RivuletEngine,
        _rt: tokio::runtime::Runtime,
    ) -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            monitor: None,
            #[cfg(not(target_arch = "wasm32"))]
            capture_active: false,
            #[cfg(not(target_arch = "wasm32"))]
            current_frame: None,
            #[cfg(not(target_arch = "wasm32"))]
            preview_texture: None,
            #[cfg(not(target_arch = "wasm32"))]
            fps: 0.0,
            #[cfg(not(target_arch = "wasm32"))]
            last_frame_time: Instant::now(),
            #[cfg(not(target_arch = "wasm32"))]
            frame_count: 0,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start_capture(&mut self) {
        tracing::info!("Starting screen capture from GUI");

        match Monitor::all() {
            Ok(monitors) => {
                if let Some(primary_monitor) = monitors.into_iter().find(|m| m.is_primary()) {
                    tracing::info!(
                        "Selected monitor: {} ({}x{})",
                        primary_monitor.name(),
                        primary_monitor.width(),
                        primary_monitor.height()
                    );

                    self.monitor = Some(primary_monitor);
                    self.capture_active = true;
                    self.last_frame_time = Instant::now();
                    self.frame_count = 0;

                    tracing::info!("Capture started successfully");
                } else {
                    tracing::error!("No primary monitor found");
                }
            }
            Err(e) => {
                tracing::error!("Failed to get monitors: {}", e);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn stop_capture(&mut self) {
        tracing::info!("Stopping screen capture");

        self.monitor = None;
        self.capture_active = false;
        self.current_frame = None;
        self.preview_texture = None;

        tracing::info!("Capture stopped");
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn update_capture(&mut self, ctx: &egui::Context) {
        if !self.capture_active {
            return;
        }

        let monitor = match &self.monitor {
            Some(m) => m,
            None => return,
        };

        match monitor.capture_image() {
            Ok(image) => {
                self.frame_count += 1;
                let now = Instant::now();
                let elapsed = now.duration_since(self.last_frame_time).as_secs_f32();

                if elapsed >= 1.0 {
                    self.fps = self.frame_count as f32 / elapsed;
                    self.frame_count = 0;
                    self.last_frame_time = now;
                }

                // xcap gibt uns RGBA, wir konvertieren zu BGRA für Konsistenz
                let rgba_data = image.as_raw();
                let mut bgra_data = Vec::with_capacity(rgba_data.len());

                for pixel in rgba_data.chunks_exact(4) {
                    bgra_data.push(pixel[2]); // B
                    bgra_data.push(pixel[1]); // G
                    bgra_data.push(pixel[0]); // R
                    bgra_data.push(pixel[3]); // A
                }

                self.current_frame = Some(CapturedFrameData {
                    data: bgra_data,
                    width: image.width(),
                    height: image.height(),
                });

                ctx.request_repaint();
            }
            Err(e) => {
                tracing::error!("Capture error: {}", e);
                self.stop_capture();
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn render_preview(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if let Some(frame_data) = &self.current_frame {
            // Konvertiere BGRA zurück zu RGBA für egui
            let mut rgba_data = Vec::with_capacity(frame_data.data.len());

            for pixel in frame_data.data.chunks_exact(4) {
                rgba_data.push(pixel[2]); // R
                rgba_data.push(pixel[1]); // G
                rgba_data.push(pixel[0]); // B
                rgba_data.push(pixel[3]); // A
            }

            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [frame_data.width as usize, frame_data.height as usize],
                &rgba_data,
            );

            // Get or create texture
            let texture = self.preview_texture.get_or_insert_with(|| {
                ctx.load_texture(
                    "screen_preview",
                    color_image.clone(),
                    egui::TextureOptions::default(),
                )
            });

            // Update texture
            texture.set(color_image, egui::TextureOptions::default());

            let available_size = ui.available_size();
            let aspect_ratio = frame_data.width as f32 / frame_data.height as f32;

            let preview_width = available_size.x.min(800.0);
            let preview_height = preview_width / aspect_ratio;

            let preview_size = egui::vec2(preview_width, preview_height);

            ui.image((texture.id(), preview_size));

            ui.label(format!("Resolution: {}x{}", frame_data.width, frame_data.height));
            ui.label(format!("FPS: {:.1}", self.fps));
        } else {
            ui.label("No frame captured yet...");
        }
    }
}

impl eframe::App for RivuletApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))]
        self.update_capture(ctx);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🌊 Rivulet");

                ui.separator();

                #[cfg(not(target_arch = "wasm32"))]
                {
                    if self.capture_active {
                        if ui.button("⏹ Stop Capture").clicked() {
                            self.stop_capture();
                        }
                        ui.colored_label(egui::Color32::GREEN, "● CAPTURING");
                    } else {
                        if ui.button("⏺ Start Capture").clicked() {
                            self.start_capture();
                        }
                    }
                }

                #[cfg(target_arch = "wasm32")]
                {
                    ui.label("Screen capture not available in web version");
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Screen Capture Preview");
            ui.separator();

            #[cfg(not(target_arch = "wasm32"))]
            {
                if self.capture_active {
                    self.render_preview(ctx, ui);
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.heading("Click 'Start Capture' to begin");
                        ui.add_space(20.0);
                        ui.label("Your screen will be captured and displayed here in real-time");
                    });
                }
            }

            #[cfg(target_arch = "wasm32")]
            {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Platform Not Supported");
                    ui.add_space(20.0);
                    ui.label("Screen capture is not available in the web version");
                });
            }
        });
    }
}