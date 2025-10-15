use eframe::egui;
use rivulet_core::*;
use rivulet_streaming::VideoEncoder;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

#[cfg(not(target_arch = "wasm32"))]
use xcap::Monitor;

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Record,
    Settings,
    About,
}

pub struct RivuletApp {
    // UI State
    current_tab: Tab,

    // Capture State
    #[cfg(not(target_arch = "wasm32"))]
    monitor: Option<Monitor>,
    #[cfg(not(target_arch = "wasm32"))]
    capture_active: bool,
    #[cfg(not(target_arch = "wasm32"))]
    recording_active: bool,

    // Preview
    #[cfg(not(target_arch = "wasm32"))]
    current_frame: Option<CapturedFrameData>,
    #[cfg(not(target_arch = "wasm32"))]
    preview_texture: Option<egui::TextureHandle>,

    // Recording
    #[cfg(not(target_arch = "wasm32"))]
    encoder: Option<Arc<Mutex<VideoEncoder>>>,
    #[cfg(not(target_arch = "wasm32"))]
    recording_start_time: Option<Instant>,

    // Stats
    #[cfg(not(target_arch = "wasm32"))]
    fps: f32,
    #[cfg(not(target_arch = "wasm32"))]
    last_frame_time: Instant,
    #[cfg(not(target_arch = "wasm32"))]
    frame_count: u32,

    // Settings
    #[cfg(not(target_arch = "wasm32"))]
    output_path: String,
    #[cfg(not(target_arch = "wasm32"))]
    recording_fps: u32,
    #[cfg(not(target_arch = "wasm32"))]
    bitrate: u64,
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
            current_tab: Tab::Record,

            #[cfg(not(target_arch = "wasm32"))]
            monitor: None,
            #[cfg(not(target_arch = "wasm32"))]
            capture_active: false,
            #[cfg(not(target_arch = "wasm32"))]
            recording_active: false,
            #[cfg(not(target_arch = "wasm32"))]
            current_frame: None,
            #[cfg(not(target_arch = "wasm32"))]
            preview_texture: None,
            #[cfg(not(target_arch = "wasm32"))]
            encoder: None,
            #[cfg(not(target_arch = "wasm32"))]
            recording_start_time: None,
            #[cfg(not(target_arch = "wasm32"))]
            fps: 0.0,
            #[cfg(not(target_arch = "wasm32"))]
            last_frame_time: Instant::now(),
            #[cfg(not(target_arch = "wasm32"))]
            frame_count: 0,
            #[cfg(not(target_arch = "wasm32"))]
            output_path: Self::get_default_output_path(),
            #[cfg(not(target_arch = "wasm32"))]
            recording_fps: 30,
            #[cfg(not(target_arch = "wasm32"))]
            bitrate: 8_000_000,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_default_output_path() -> String {
        let videos_dir = if cfg!(windows) {
            dirs::video_dir()
        } else if cfg!(target_os = "macos") {
            dirs::home_dir().map(|p| p.join("Movies"))
        } else {
            dirs::video_dir()
        };

        if let Some(dir) = videos_dir {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            dir.join(format!("rivulet_{}.mp4", timestamp))
                .to_string_lossy()
                .to_string()
        } else {
            format!("rivulet_recording.mp4")
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start_preview(&mut self) {
        tracing::info!("Starting preview");

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
    fn stop_preview(&mut self) {
        tracing::info!("Stopping preview");
        self.monitor = None;
        self.capture_active = false;
        self.current_frame = None;
        self.preview_texture = None;
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start_recording(&mut self) {
        tracing::info!("Starting recording");

        let monitor = match &self.monitor {
            Some(m) => m,
            None => {
                tracing::error!("No monitor selected");
                return;
            }
        };

        let width = monitor.width();
        let height = monitor.height();

        match VideoEncoder::new(
            &PathBuf::from(&self.output_path),
            width,
            height,
            self.recording_fps,
            self.bitrate,
        ) {
            Ok(encoder) => {
                self.encoder = Some(Arc::new(Mutex::new(encoder)));
                self.recording_active = true;
                self.recording_start_time = Some(Instant::now());
                tracing::info!("Recording started: {}", self.output_path);
            }
            Err(e) => {
                tracing::error!("Failed to create encoder: {}", e);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn stop_recording(&mut self) {
        tracing::info!("Stopping recording");
        self.recording_active = false;
        self.recording_start_time = None;

        if let Some(encoder) = self.encoder.take() {
            thread::spawn(move || {
                if let Ok(enc) = Arc::try_unwrap(encoder) {
                    if let Ok(encoder) = enc.into_inner() {
                        if let Err(e) = encoder.finish() {
                            tracing::error!("Failed to finish encoding: {}", e);
                        }
                    }
                }
            });
        }
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

                let rgba_data = image.as_raw();
                let mut bgra_data = Vec::with_capacity(rgba_data.len());

                for pixel in rgba_data.chunks_exact(4) {
                    bgra_data.push(pixel[2]); // B
                    bgra_data.push(pixel[1]); // G
                    bgra_data.push(pixel[0]); // R
                    bgra_data.push(pixel[3]); // A
                }

                if self.recording_active {
                    if let Some(encoder) = &self.encoder {
                        let encoder = encoder.clone();
                        let frame_data = bgra_data.clone();
                        let width = image.width();
                        let height = image.height();

                        thread::spawn(move || {
                            if let Ok(mut enc) = encoder.lock() {
                                let _ = enc.encode_frame(&frame_data, width, height, width * 4);
                            }
                        });
                    }
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
                self.stop_preview();
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn render_preview(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if let Some(frame_data) = &self.current_frame {
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

            let texture = self.preview_texture.get_or_insert_with(|| {
                ctx.load_texture(
                    "screen_preview",
                    color_image.clone(),
                    egui::TextureOptions::default(),
                )
            });

            texture.set(color_image, egui::TextureOptions::default());

            let available_size = ui.available_size();
            let aspect_ratio = frame_data.width as f32 / frame_data.height as f32;
            let preview_width = available_size.x.min(1200.0);
            let preview_height = preview_width / aspect_ratio;
            let preview_size = egui::vec2(preview_width, preview_height);

            ui.image((texture.id(), preview_size));

            // Stats overlay
            ui.horizontal(|ui| {
                ui.label(format!("{}x{}", frame_data.width, frame_data.height));
                ui.separator();
                ui.label(format!("{:.1} FPS", self.fps));

                if self.recording_active {
                    ui.separator();
                    ui.colored_label(egui::Color32::RED, "● REC");

                    if let Some(start) = self.recording_start_time {
                        let duration = start.elapsed().as_secs();
                        let mins = duration / 60;
                        let secs = duration % 60;
                        ui.label(format!("{:02}:{:02}", mins, secs));
                    }
                }
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No preview available");
            });
        }
    }

    fn render_tab_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🌊 Rivulet");

                ui.separator();

                // Tabs
                ui.selectable_value(&mut self.current_tab, Tab::Record, "📹 Record");
                ui.selectable_value(&mut self.current_tab, Tab::Settings, "⚙️ Settings");
                ui.selectable_value(&mut self.current_tab, Tab::About, "ℹ️ About");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if self.recording_active {
                            ui.colored_label(egui::Color32::RED, "● RECORDING");
                        }
                    }
                });
            });
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn render_record_tab(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("Screen Recording");
        ui.separator();

        // Control buttons
        ui.horizontal(|ui| {
            if self.capture_active {
                if ui.button("⏹ Stop Preview").clicked() {
                    if self.recording_active {
                        self.stop_recording();
                    }
                    self.stop_preview();
                }
            } else {
                if ui.button("▶ Start Preview").clicked() {
                    self.start_preview();
                }
            }

            ui.separator();

            if self.capture_active {
                if self.recording_active {
                    if ui.button("⏹ Stop Recording").clicked() {
                        self.stop_recording();
                    }
                } else {
                    if ui.button("⏺ Start Recording").clicked() {
                        self.start_recording();
                    }
                }
            }
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Preview
        if self.capture_active {
            self.render_preview(ctx, ui);
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.heading("Click 'Start Preview' to begin");
                ui.add_space(20.0);
                ui.label("Your screen will be captured and displayed here");
            });
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.separator();
        ui.add_space(10.0);

        // Output Settings
        ui.group(|ui| {
            ui.heading("Output");
            ui.add_space(5.0);

            ui.label("Output File:");
            let display_path = if self.output_path.len() > 50 {
                let path = std::path::Path::new(&self.output_path);
                format!("...{}",
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                )
            } else {
                self.output_path.clone()
            };

            ui.label(&display_path)
                .on_hover_text(&self.output_path);

            ui.horizontal(|ui| {
                if ui.button("📁 Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name(&format!(
                            "rivulet_{}.mp4",
                            chrono::Local::now().format("%Y%m%d_%H%M%S")
                        ))
                        .add_filter("MP4 Video", &["mp4"])
                        .save_file()
                    {
                        self.output_path = path.to_string_lossy().to_string();
                    }
                }

                if ui.button("🔄 Reset").clicked() {
                    self.output_path = Self::get_default_output_path();
                }
            });
        });

        ui.add_space(10.0);

        // Video Settings
        ui.group(|ui| {
            ui.heading("Video");
            ui.add_space(5.0);

            ui.label("Frame Rate:");
            ui.add(egui::Slider::new(&mut self.recording_fps, 15..=60).suffix(" FPS"));

            ui.add_space(5.0);

            ui.label("Bitrate:");
            let mut bitrate_mbps = (self.bitrate / 1_000_000) as u32;
            if ui.add(egui::Slider::new(&mut bitrate_mbps, 1..=50).suffix(" Mbps")).changed() {
                self.bitrate = bitrate_mbps as u64 * 1_000_000;
            }
        });

        ui.add_space(10.0);

        // Info
        ui.group(|ui| {
            ui.heading("Estimated File Size");
            ui.add_space(5.0);

            let bitrate_mbps = self.bitrate / 1_000_000;
            let mb_per_second = bitrate_mbps / 8;
            let mb_per_minute = mb_per_second * 60;

            ui.label(format!("~{} MB/minute", mb_per_minute));
            ui.label(format!("~{} MB/hour", mb_per_minute * 60));
        });
    }

    fn render_about_tab(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);

            ui.heading("🌊 Rivulet");
            ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));

            ui.add_space(20.0);

            ui.label("Modern screen recording & streaming software");
            ui.label("Built with Rust 🦀");

            ui.add_space(30.0);

            ui.hyperlink_to(
                "🌐 GitHub Repository",
                "https://github.com/thoser666/rivulet"
            );

            ui.add_space(10.0);

            ui.label("© 2025 Rivulet Team");
            ui.label("Licensed under MIT");

            ui.add_space(30.0);

            ui.group(|ui| {
                ui.heading("Technologies");
                ui.add_space(5.0);
                ui.label("• egui - GUI framework");
                ui.label("• xcap - Screen capture");
                ui.label("• FFmpeg - Video encoding");
            });
        });
    }
}

impl eframe::App for RivuletApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))]
        self.update_capture(ctx);

        // Tab bar
        self.render_tab_bar(ctx);

        // Content area
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Record => {
                    #[cfg(not(target_arch = "wasm32"))]
                    self.render_record_tab(ctx, ui);

                    #[cfg(target_arch = "wasm32")]
                    ui.centered_and_justified(|ui| {
                        ui.label("Platform not supported");
                    });
                }
                Tab::Settings => {
                    #[cfg(not(target_arch = "wasm32"))]
                    self.render_settings_tab(ui);

                    #[cfg(target_arch = "wasm32")]
                    ui.centered_and_justified(|ui| {
                        ui.label("Platform not supported");
                    });
                }
                Tab::About => {
                    self.render_about_tab(ui);
                }
            }
        });
    }
}