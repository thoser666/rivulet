// In rivulet-core/src/lib.rs

use glib;
use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;
use once_cell::sync::Lazy;

// GStreamer muss nur einmal initialisiert werden.
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
        // Sicherstellen, dass GStreamer initialisiert ist.
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

    pub fn start_streaming(&mut self, width: u32, height: u32) {
        if self.is_streaming {
            return;
        }
        println!("[Engine] Streaming wird gestartet...");

        let rtmp_url = "rtmp://localhost/live/stream"; // Beispiel-URL

        // Die Pipeline als Textbeschreibung erstellen - das ist die Stärke von GStreamer!
        let pipeline_str = format!(
            "appsrc name=rivulet_src ! videoconvert ! x264enc tune=zerolatency ! flvmux ! rtmpsink location={}",
            rtmp_url
        );

        let pipeline = gst::parse_launch(&pipeline_str)
            .expect("Pipeline konnte nicht erstellt werden.")
            .downcast::<gst::Pipeline>()
            .unwrap();

        // Das appsrc-Element aus der Pipeline holen, um Daten hineinzuschieben.
        let appsrc = pipeline
            .by_name("rivulet_src")
            .expect("Konnte appsrc-Element nicht finden.")
            .downcast::<gst_app::AppSrc>()
            .unwrap();

        // Konfigurieren von appsrc: Wir sagen ihm, welches Videoformat wir senden.
        let video_info = gst_video::VideoInfo::builder(gst_video::VideoFormat::Rgba, width, height)
            .fps((60, 1))
            .build()
            .unwrap();
        appsrc.set_caps(Some(&video_info.to_caps().unwrap()));
        appsrc.set_property("format", gst::Format::Time);
        appsrc.set_property("is-live", true);
        appsrc.set_property("do-timestamp", true);

        // Pipeline starten
        pipeline
            .set_state(gst::State::Playing)
            .expect("Pipeline konnte nicht gestartet werden.");

        self.pipeline = Some(pipeline);
        self.appsrc = Some(appsrc);
        self.is_streaming = true;
        println!("[Engine] GStreamer-Pipeline läuft.");
    }

    pub fn stop_streaming(&mut self) {
        if !self.is_streaming {
            return;
        }
        println!("[Engine] Streaming wird gestoppt...");

        if let Some(pipeline) = self.pipeline.take() {
            // Pipeline anhalten und Ressourcen freigeben
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
            // Beim ersten Frame das Streaming initial starten
            self.start_streaming(width, height);
        }

        if let Some(appsrc) = &self.appsrc {
            // Erstelle einen GStreamer-Buffer aus den rohen Daten.
            // Wir müssen die Daten kopieren, da der Buffer sie besitzen muss.
            let mut buffer = gst::Buffer::with_size(frame_data.len()).unwrap();
            {
                let mut map = buffer.map_writable().unwrap();
                map.copy_from_slice(frame_data);
            }

            // Schiebe den Buffer in die Pipeline.
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
