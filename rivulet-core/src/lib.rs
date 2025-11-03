// In rivulet-core/src/lib.rs

use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;
use once_cell::sync::Lazy;
// Der entscheidende Import
use gst::prelude::*;

// GStreamer-Initialisierung
static GSTREAMER_INIT: Lazy<()> = Lazy::new(|| {
    gst::init().expect("GStreamer-Initialisierung fehlgeschlagen.");
});

pub struct RivuletEngine {
    pipeline: Option<gst::Pipeline>,
    appsrc: Option<gst_app::AppSrc>,
    is_streaming: bool,
}

impl Default for RivuletEngine {
    fn default() -> Self {
        Lazy::force(&GSTREAMER_INIT);
        Self {
            pipeline: None,
            appsrc: None,
            is_streaming: false,
        }
    }
}

impl RivuletEngine {
    pub fn new() -> Self {
        println!("[Engine] GStreamer bereit.");
        Self::default()
    }

    /// Privater Helfer, um die GStreamer-Pipeline zu erstellen und zu starten.
    fn initialize_and_start_pipeline(&mut self, width: u32, height: u32) {
        println!(
            "[Engine] Initialisiere GStreamer-Pipeline für Auflösung {}x{}",
            width, height
        );

        let rtmp_url = "rtmp://localhost/live/stream";

        let pipeline_str = format!(
            "appsrc name=rivulet_src ! videoconvert ! x264enc tune=zerolatency ! flvmux ! rtmpsink location={}",
            rtmp_url
        );

        // KORREKTUR: Die `parse_launch`-Funktion ist im `gst::parse`-Modul, wenn das Feature aktiv ist.
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
            .expect("Konnte appsrc-Element 'rivulet_src' nicht in der Pipeline finden.")
            .downcast::<gst_app::AppSrc>()
            .unwrap();

        let video_info = gst_video::VideoInfo::builder(gst_video::VideoFormat::Rgba, width, height)
            .fps((60, 1))
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
        println!("[Engine] GStreamer-Pipeline läuft.");
    }

    pub fn start_streaming(&mut self) {
        if self.is_streaming {
            return;
        }
        println!("[Engine] Streaming wird vorbereitet. Warte auf ersten Frame...");
        self.is_streaming = true;
    }

    pub fn stop_streaming(&mut self) {
        if !self.is_streaming {
            return;
        }
        println!("[Engine] Streaming wird gestoppt...");

        if let Some(pipeline) = self.pipeline.take() {
            pipeline
                .set_state(gst::State::Null)
                .expect("Pipeline konnte nicht gestoppt werden.");
        }
        self.appsrc = None;
        self.is_streaming = false;
        println!("[Engine] Streaming gestoppt.");
    }

    pub fn process_raw_frame(&mut self, frame_data: &[u8], width: u32, height: u32) {
        if !self.is_streaming {
            return;
        }

        if self.pipeline.is_none() {
            self.initialize_and_start_pipeline(width, height);
        }

        if let Some(appsrc) = &self.appsrc {
            let mut buffer = gst::Buffer::with_size(frame_data.len()).unwrap();
            {
                // Obtain a mutable reference to the buffer memory and map it writable
                let buffer_ref = buffer.get_mut().expect("Buffer should be uniquely owned");
                let mut map = buffer_ref.map_writable().expect("Failed to map buffer writable");
                map.as_mut_slice().copy_from_slice(frame_data);
            }

            if let Err(err) = appsrc.push_buffer(buffer) {
                eprintln!(
                    "[Engine] Fehler beim Senden des Frames in die Pipeline: {:?}",
                    err
                );
                self.stop_streaming();
            }
        }
    }
}
