// In rivulet-core/src/lib.rs

use glib;
use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_pbutils as gst_pbutils; // Wir importieren die Crate, die Sie in Cargo.toml deklariert haben
use gstreamer_video as gst_video;
use once_cell::sync::Lazy;
use std::path::PathBuf;
// Wichtig: Der Prelude wird immer noch für Methoden wie .set_state(), .by_name() etc. benötigt.
use gst::prelude::*;

// GStreamer-Initialisierung
static GSTREAMER_INIT: Lazy<()> = Lazy::new(|| {
    gst::init().expect("GStreamer-Initialisierung fehlgeschlagen.");
});

pub struct RivuletEngine {
    pipeline: Option<gst::Pipeline>,
    appsrc: Option<gst_app::AppSrc>,
    is_recording: bool,
    output_path: Option<PathBuf>,
}

impl Default for RivuletEngine {
    fn default() -> Self {
        Lazy::force(&GSTREAMER_INIT);
        Self {
            pipeline: None,
            appsrc: None,
            is_recording: false,
            output_path: None,
        }
    }
}

impl RivuletEngine {
    pub fn new() -> Self {
        println!("[Engine] GStreamer bereit.");
        Self::default()
    }

    fn initialize_and_start_pipeline(&mut self, width: u32, height: u32) {
        let Some(path) = self.output_path.as_ref() else {
            return;
        };
        let location = path.to_str().expect("Dateipfad ist ungültig.");
        println!("[Engine] Initialisiere Aufnahme-Pipeline für: {}", location);

        let pipeline_str = format!(
            "appsrc name=rivulet_src ! videoconvert ! x264enc tune=zerolatency ! mp4mux ! filesink location=\"{}\"",
            location
        );

        // KORREKTUR: Die `parse_launch`-Funktion kommt aus `gstreamer` (Modul `gst::parse`).
        // In v0.24 können wir `gst::parse::launch(&str)` verwenden; bei Bedarf gäbe es auch `launch_full` mit Context/Flags.
        let pipeline = match gst::parse::launch(&pipeline_str) {
            Ok(p) => p.downcast::<gst::Pipeline>().unwrap(),
            Err(e) => {
                eprintln!("[Engine] Fehler beim Erstellen der Pipeline: {}", e);
                return;
            }
        };

        // Der Rest des Codes ist korrekt...
        let appsrc = pipeline
            .by_name("rivulet_src")
            .unwrap()
            .downcast::<gst_app::AppSrc>()
            .unwrap();

        let video_info = gst_video::VideoInfo::builder(gst_video::VideoFormat::Rgba, width, height)
            .fps((30, 1))
            .build()
            .unwrap();
        appsrc.set_caps(Some(&video_info.to_caps().unwrap()));
        appsrc.set_property("format", gst::Format::Time);
        appsrc.set_property("is-live", true);
        appsrc.set_property("do-timestamp", true);

        if pipeline.set_state(gst::State::Playing).is_err() {
            eprintln!("[Engine] Pipeline konnte nicht gestartet werden.");
            return;
        }

        self.pipeline = Some(pipeline);
        self.appsrc = Some(appsrc);
        println!("[Engine] Aufnahme-Pipeline läuft.");
    }

    pub fn start_local_recording(&mut self, path: PathBuf) {
        if self.is_recording {
            return;
        }
        println!("[Engine] Aufnahme vorbereitet für: {:?}", path);
        self.output_path = Some(path);
        self.is_recording = true;
    }

    pub fn stop_recording(&mut self) {
        if !self.is_recording {
            return;
        }
        println!("[Engine] Aufnahme wird gestoppt...");

        if let Some(appsrc) = self.appsrc.as_ref() {
            let _ = appsrc.end_of_stream();
        }
        if let Some(pipeline) = self.pipeline.take() {
            pipeline
                .set_state(gst::State::Null)
                .expect("Pipeline konnte nicht gestoppt werden.");
        }

        self.appsrc = None;
        self.output_path = None;
        self.is_recording = false;
        println!("[Engine] Aufnahme gestoppt und Datei gespeichert.");
    }

    pub fn process_raw_frame(&mut self, frame_data: &[u8], width: u32, height: u32) {
        if !self.is_recording {
            return;
        }

        if self.pipeline.is_none() {
            self.initialize_and_start_pipeline(width, height);
        }

        if let Some(appsrc) = &self.appsrc {
            let mut buffer = gst::Buffer::with_size(frame_data.len()).unwrap();
            {
                let buffer_ref = buffer.get_mut().expect("Buffer not writable");
                let mut map = buffer_ref
                    .map_writable()
                    .expect("Failed to map buffer writable");
                map.as_mut_slice().copy_from_slice(frame_data);
            }

            if let Err(err) = appsrc.push_buffer(buffer) {
                eprintln!(
                    "[Engine] Fehler beim Senden des Frames in die Pipeline: {:?}",
                    err
                );
                self.stop_recording();
            }
        }
    }
}
