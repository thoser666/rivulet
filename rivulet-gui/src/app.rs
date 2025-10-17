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

// --- Linux-spezifische statische Variablen und Typen ---
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
    #[serde(skip)] // Die Engine wird nicht serialisiert.
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
}

// --- Linux-spezifische Methoden ---
#[cfg(target_os = "linux")]
impl RivuletApp {
    fn start_preview(&mut self) { /* ... Implementierung ... */
    }
    fn start_recording(&mut self) { /* ... Implementierung ... */
    }
    fn stop_recording(&mut self) { /* ... Implementierung ... */
    }
    fn stop_preview(&mut self) { /* ... Implementierung ... */
    }
    fn save_recording(&self) { /* ... Implementierung ... */
    }
}

impl RivuletApp {
    /// Erstellt eine neue Instanz der RivuletApp.
    /// Diese Funktion wird einmal beim Start aufgerufen.
    pub fn new(cc: &eframe::CreationContext<'_>, engine: RivuletEngine) -> Self {
        // Versucht, den Zustand der App aus dem Speicher zu laden.
        if let Some(storage) = cc.storage {
            if let Some(mut app) = eframe::get_value::<Self>(storage, eframe::APP_KEY) {
                // Setzt die neu erstellte Engine in die geladene App ein,
                // da die Engine selbst nicht gespeichert wurde.
                app.engine = engine;
                return app;
            }
        }

        // Wenn kein gespeicherter Zustand vorhanden ist, wird eine neue Instanz erstellt.
        Self {
            engine,
            ..Default::default()
        }
    }
}

impl eframe::App for RivuletApp {
    /// Speichert den Zustand der App vor dem Beenden.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Wird bei jedem Frame aufgerufen, um die UI zu zeichnen.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- Linux-spezifische Nachrichtenverarbeitung ---
        #[cfg(target_os = "linux")]
        {
            if let Ok(msg) = self.receiver.try_recv() {
                // ... Nachrichtenverarbeitungslogik ...
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        // Sendet den Befehl zum Schließen des Fensters.
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome to Rivulet");
            // Hier können UI-Elemente hinzugefügt werden, die die 'engine' verwenden.
            // z.B. ui.label(format!("Engine status: {:?}", self.engine.get_status()));

            ui.separator();

            // --- Plattformspezifische UI für die Aufnahme ---
            #[cfg(target_os = "linux")]
            {
                // UI für die Aufnahme-Buttons unter Linux
                if self.is_previewing {
                    // ...
                } else if ui.button("▶ Start Preview").clicked() {
                    self.start_preview();
                }
            }

            #[cfg(not(target_os = "linux"))]
            {
                // Hinweis für Benutzer auf Nicht-Linux-Systemen.
                ui.add_space(20.0);
                ui.label(egui::RichText::new("ℹ️ Screen Recording Feature").strong());
                ui.label("This feature is only available on Linux systems using the PipeWire video server.");
            }

            // Footer
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/crates/eframe");
                    ui.label(".");
                });
            });
        });
    }
}
