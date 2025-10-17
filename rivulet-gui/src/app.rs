#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports, dead_code, unused_variables)]

use eframe::egui;
use rivulet_core::RivuletEngine;

// --- Linux-spezifische Imports ---
#[cfg(target_os = "linux")]
use {
    ashpd::desktop::file_chooser::{FileChooser, FileFilter},
    ashpd::desktop::screencast::{CursorMode, Screencast, Session, Source, Stream},
    ashpd::enumflags2::BitFlags,
    ashpd::WindowIdentifier,
    once_cell::sync::Lazy,
    pipewire::{spa::param::video::VideoFormat, types::Fd},
    std::sync::mpsc::{self, Receiver, Sender},
    tokio::runtime::Runtime,
};

// --- ENDGÜLTIG KORREKTE Windows-spezifische Imports ---
#[cfg(target_os = "windows")]
use windows_capture::{self, settings::Settings};

// --- Linux-spezifische Typen ---
#[cfg(target_os = "linux")]
static TOKIO_RT: Lazy<Runtime> = Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

#[cfg(target_os = "linux")]
enum BackendMessage {
    Stream(Stream, Fd),
    Recording(Fd),
    Done,
    Error(anyhow::Error),
}

/// Die Hauptanwendungsstruktur.
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct RivuletApp {
    #[serde(skip)]
    engine: RivuletEngine,

    // --- Linux-spezifische Felder ---
    #[cfg(target_os = "linux")]
    #[serde(skip)]
    is_previewing: bool,
    #[cfg(target_os = "linux")]
    #[serde(skip)]
    is_recording: bool,
    #[cfg(target_os = "linux")]
    #[serde(skip)]
    screencast: Screencast,
    #[cfg(target_os = "linux")]
    #[serde(skip)]
    session: Option<Session<'static>>,
    #[cfg(target_os = "linux")]
    #[serde(skip)]
    stream: Option<Stream>,
    #[cfg(target_os = "linux")]
    #[serde(skip)]
    pipewire_fd: Option<Fd>,
    #[cfg(target_os = "linux")]
    #[serde(skip)]
    sender: Sender<BackendMessage>,
    #[cfg(target_os = "linux")]
    #[serde(skip)]
    receiver: Receiver<BackendMessage>,

    // --- Windows-spezifische Felder ---
    #[cfg(target_os = "windows")]
    #[serde(skip)]
    is_windows_recording: bool,
}

// --- Linux-spezifische Methoden ---
#[cfg(target_os = "linux")]
impl RivuletApp {
    fn start_preview(&mut self) { /* ... */
    }
    fn start_recording(&mut self) { /* ... */
    }
    fn stop_recording(&mut self) { /* ... */
    }
    fn stop_preview(&mut self) { /* ... */
    }
    fn save_recording(&self) { /* ... */
    }
}

// --- Windows-spezifische Methoden (Platzhalter mit korrekter API-Struktur) ---
#[cfg(target_os = "windows")]
impl RivuletApp {
    fn start_windows_recording(&mut self) {
        println!("Starte Aufnahme unter Windows... (TODO)");
        self.is_windows_recording = true;

        // Dieser Code MUSS in einem asynchronen Kontext ausgeführt werden,
        // z.B. mit tokio::spawn.
        // let settings = Settings::default(); // Erstellt Standardeinstellungen
        // let picker = match windows_capture::Picker(&settings) {
        //     Ok(picker) => picker,
        //     Err(e) => {
        //         eprintln!("Fehler beim Erstellen des Pickers: {}", e);
        //         return;
        //     }
        // };
        // let result = picker.pick_async().await;
        // ...
    }

    fn stop_windows_recording(&mut self) {
        println!("Stoppe Aufnahme unter Windows... (TODO)");
        self.is_windows_recording = false;
    }
}

impl RivuletApp {
    pub fn new(cc: &eframe::CreationContext<'_>, engine: RivuletEngine) -> Self {
        if let Some(storage) = cc.storage {
            if let Some(mut app) = eframe::get_value::<Self>(storage, eframe::APP_KEY) {
                app.engine = engine;
                return app;
            }
        }
        Self {
            engine,
            ..Default::default()
        }
    }
}

impl eframe::App for RivuletApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome to Rivulet");
            ui.separator();

            #[cfg(target_os = "linux")]
            {
                ui.label("Linux Recording Controls:");
                if self.is_previewing {
                    // ...
                } else if ui.button("▶ Start Preview").clicked() {
                    self.start_preview();
                }
            }

            #[cfg(target_os = "windows")]
            {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Windows Screen Recording").strong());

                if self.is_windows_recording {
                    if ui.button("⏹ Stop Recording").clicked() {
                        self.stop_windows_recording();
                    }
                } else {
                    if ui.button("⏺ Start Recording").clicked() {
                        self.start_windows_recording();
                    }
                }
            }

            #[cfg(not(any(target_os = "linux", target_os = "windows")))]
            {
                ui.add_space(20.0);
                ui.label(egui::RichText::new("ℹ️ Screen Recording Feature").strong());
                ui.label("This feature is currently only available on Linux and Windows.");
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                });
            });
        });
    }
}
