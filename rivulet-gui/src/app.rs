use eframe::egui;
use rivulet_core::*;

#[cfg(windows)]
use rivulet_capture::{CaptureSource, DxgiScreenCapture};

#[cfg(windows)]
use std::sync::{Arc, Mutex};

use std::time::Instant;

pub struct RivuletApp {
    #[cfg(windows)]
    capture: Option<Arc<Mutex<DxgiScreenCapture>>>,
    #[cfg(windows)]
    capture_active: bool,
    #[cfg(windows)]
    current_frame: Option<CapturedFrameData>,
    #[cfg(windows)]
    preview_texture: Option<egui::TextureHandle>,

    #[cfg(windows)]
    fps: f32,
    #[cfg(windows)]
    last_frame_time: Instant,
    #[cfg(windows)]
    frame_count: u32,
}

#[cfg(windows)]
struct CapturedFrameData {
    data: Vec<u8>,
    width: u32,
    height: u32,
    stride: u32,
}

impl RivuletApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        _engine: RivuletEngine,
        _rt: tokio::runtime::Runtime,
    ) -> Self {
        Self {
            #[cfg(windows)]
            capture: None,
            #[cfg(windows)]
            capture_active: false,
            #[cfg(windows)]
            current_frame: None,
            #[cfg(windows)]
            preview_texture: None,
            #[cfg(windows)]
            fps: 0.0,
            #[cfg(windows)]
            last_frame_time: Instant::now(),
            #[cfg(windows)]
            frame_count: 0,
        }
    }

    #[cfg(windows)]
    fn start_capture(&mut self) {
        tracing::info!("Starting screen capture from GUI");

        match DxgiScreenCapture::new(0) {
            Ok(mut capture_device) => {
                if let Err(e) = capture_device.start() {
                    tracing::error!("Failed to start capture: {}", e);
                    return;
                }

                self.capture = Some(Arc::new(Mutex::new(capture_device)));
                self.capture_active = true;
                self.last_frame_time = Instant::now();
                self.frame_count = 0;

                tracing::info!("Capture started successfully");
            }
            Err(e) => {
                tracing::error!("Failed to create capture device: {}", e);
            }
        }
    }

    #[cfg(windows)]
    fn stop_capture(&mut self) {
        tracing::info!("Stopping screen capture");

        if let Some(capture) = &self.capture {
            if let Ok(mut cap) = capture.lock() {
                let _ = cap.stop();
            }
        }

        self.capture = None;
        self.capture_active = false;
        self.current_frame = None;
        self.preview_texture = None;

        tracing::info!("Capture stopped");
    }

    #[cfg(windows)]
    fn update_capture(&mut self, ctx: &egui::Context) {
        if !self.capture_active {
            return;
        }

        let capture = match &self.capture {
            Some(c) => c.clone(),
            None => return,
        };

        let frame_result = {
            let mut cap = match capture.lock() {
                Ok(guard) => guard,
                Err(_) => return,
            };

            cap.capture_frame()
        };

        match frame_result {
            Ok(Some(frame)) => {
                self.frame_count += 1;
                let now = Instant::now();
                let elapsed = now.duration_since(self.last_frame_time).as_secs_f32();

                if elapsed >= 1.0 {
                    self.fps = self.frame_count as f32 / elapsed;
                    self.frame_count = 0;
                    self.last_frame_time = now;
                }

                self.current_frame = Some(CapturedFrameData {
                    data: frame.data,
                    width: frame.width,
                    height: frame.height,
                    stride: frame.stride,
                });

                ctx.request_repaint();
            }
            Ok(None) => {}
            Err(e) => {
                tracing::error!("Capture error: {}", e);
                self.stop_capture();
            }
        }
    }

    #[cfg(windows)]
    fn render_preview(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if let Some(frame_data) = &self.current_frame {
            let mut rgba_data = Vec::with_capacity((frame_data.width * frame_data.height * 4) as usize);

            for y in 0..frame_data.height {
                let row_start = (y * frame_data.stride) as usize;
                for x in 0..frame_data.width {
                    let pixel_start = row_start + (x * 4) as usize;

                    if pixel_start + 3 < frame_data.data.len() {
                        let b = frame_data.data[pixel_start];
                        let g = frame_data.data[pixel_start + 1];
                        let r = frame_data.data[pixel_start + 2];
                        let a = frame_data.data[pixel_start + 3];

                        rgba_data.push(r);
                        rgba_data.push(g);
                        rgba_data.push(b);
                        rgba_data.push(a);
                    }
                }
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
        #[cfg(windows)]
        self.update_capture(ctx);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🌊 Rivulet");

                ui.separator();

                #[cfg(windows)]
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

                #[cfg(not(windows))]
                {
                    ui.label("Screen capture only available on Windows");
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Screen Capture Preview");
            ui.separator();

            #[cfg(windows)]
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

            #[cfg(not(windows))]
            {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Platform Not Supported");
                    ui.add_space(20.0);
                    ui.label("Screen capture is currently only available on Windows");
                    ui.label("Linux and macOS support coming soon!");
                });
            }
        });
    }
}