// In rivulet-core/src/lib.rs

// Wir verwenden die Alias-Syntax für eine bessere Lesbarkeit.
use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use ffmpeg_next as ffmpeg;

pub struct RivuletEngine {
    // Der Scaler ist zuständig für die Konvertierung des Farbraums (z.B. von RGBA nach YUV420p).
    scaler: Option<Context>,
    // Der Encoder komprimiert die konvertierten Frames in ein Videoformat (z.B. H.264).
    encoder: Option<ffmpeg::encoder::video::Video>,
    // Wir speichern die Zieldimensionen.
    width: u32,
    height: u32,
    is_streaming: bool,
}

impl Default for RivuletEngine {
    fn default() -> Self {
        Self {
            scaler: None,
            encoder: None,
            width: 1920,  // Standard-Breite, wird beim Start überschrieben
            height: 1080, // Standard-Höhe, wird beim Start überschrieben
            is_streaming: false,
        }
    }
}

impl RivuletEngine {
    /// Erstellt eine neue Instanz der Engine und initialisiert FFmpeg.
    pub fn new() -> Self {
        // Diese Funktion muss nur einmal pro Programmaufruf ausgeführt werden.
        ffmpeg::init().expect("Fehler bei der Initialisierung von FFmpeg");
        println!("[Engine] FFmpeg initialisiert.");
        Self::default()
    }

    /// Startet das Streaming. Initialisiert den Encoder und Scaler.
    pub fn start_streaming(&mut self) {
        if self.is_streaming {
            return;
        }
        println!("[Engine] Streaming wird gestartet...");

        // Wir müssen die Zieldimensionen kennen, bevor wir starten.
        // In einer echten Anwendung würden Sie diese konfigurierbar machen.
        // Für den Moment nehmen wir die Dimensionen des ersten Frames.
        // Das ist ein Henne-Ei-Problem, das wir später lösen. Hier harte Werte.
        self.width = 1920; // Beispiel
        self.height = 1080; // Beispiel

        // --- Encoder-Setup ---
        let codec = ffmpeg::codec::encoder::find_by_name("libx264")
            .expect("H.264 Encoder (libx264) nicht gefunden.");
        let mut encoder_ctx = ffmpeg::codec::context::Context::new();
        encoder_ctx.set_height(self.height);
        encoder_ctx.set_width(self.width);
        encoder_ctx.set_time_base((1, 60)); // Zeitbasis für 60 FPS
        encoder_ctx.set_frame_rate(Some((60, 1))); // 60 FPS
        encoder_ctx.set_format(Pixel::YUV420P); // Das Zielformat für H.264
                                                // Wichtige Einstellung für Streaming mit niedriger Latenz
        encoder_ctx.set_flags(ffmpeg::codec::flag::Flags::LOW_DELAY);

        let mut encoder = encoder_ctx
            .encoder()
            .video()
            .expect("Konnte Video-Encoder nicht öffnen.");
        // Setzt Encoder-spezifische Optionen
        encoder.set_option("preset", "ultrafast").unwrap();
        encoder.set_option("tune", "zerolatency").unwrap();

        self.encoder = Some(encoder);

        // --- Scaler-Setup ---
        let scaler = Context::get(
            Pixel::RGBA,     // Quellformat (von Windows Capture)
            self.width,      // Quellbreite
            self.height,     // Quellhöhe
            Pixel::YUV420P,  // Zielformat (für den Encoder)
            self.width,      // Zielbreite
            self.height,     // Zielhöhe
            Flags::BILINEAR, // Skalierungsalgorithmus
        )
        .expect("Konnte Scaler nicht erstellen.");

        self.scaler = Some(scaler);

        self.is_streaming = true;
        println!("[Engine] Streaming gestartet und Encoder/Scaler bereit.");
    }

    /// Stoppt das Streaming und gibt Ressourcen frei.
    pub fn stop_streaming(&mut self) {
        if !self.is_streaming {
            return;
        }
        println!("[Engine] Streaming wird gestoppt...");

        if let Some(mut encoder) = self.encoder.take() {
            // Signalisiere dem Encoder, dass keine Frames mehr kommen.
            if encoder.send_eof().is_ok() {
                let mut encoded = ffmpeg::Packet::empty();
                // Leere den Encoder von allen verbleibenden Paketen.
                while encoder.receive_packet(&mut encoded).is_ok() {
                    println!("[Engine] Flushing-Paket empfangen.");
                    // TODO: Sende dieses letzte Paket an die RTMP-Crate
                }
            }
        }

        self.scaler = None;
        self.is_streaming = false;
        println!("[Engine] Streaming gestoppt.");
    }

    /// Verarbeitet einen einzelnen rohen Bild-Frame.
    pub fn process_raw_frame(&mut self, frame_data: &[u8], width: u32, height: u32) {
        if !self.is_streaming || self.encoder.is_none() || self.scaler.is_none() {
            return;
        }

        // Wenn sich die Auflösung geändert hat, müssen wir neu initialisieren.
        // (Vereinfachung: Für den Moment ignorieren wir das und nehmen an, sie bleibt gleich)
        if self.width != width || self.height != height {
            println!("[Engine] Auflösungsänderung erkannt! (Nicht implementiert)");
            // In einer echten App: stop_streaming() und start_streaming() mit neuen Dimensionen aufrufen.
            return;
        }

        let encoder = self.encoder.as_mut().unwrap();
        let scaler = self.scaler.as_mut().unwrap();

        // 1. Erstelle einen FFmpeg-Frame aus den rohen RGBA-Daten.
        let mut source_frame = unsafe { Video::from_slice(frame_data, Pixel::RGBA, width, height) };
        source_frame.set_pts(Some(self.next_pts())); // Zeitstempel setzen

        // 2. Erstelle einen leeren Ziel-Frame für die konvertierten Daten.
        let mut yuv_frame = Video::empty();

        // 3. Führe die Farbkonvertierung (Scaling) durch.
        if scaler.run(&source_frame, &mut yuv_frame).is_err() {
            eprintln!("[Engine] Fehler beim Skalieren des Frames.");
            return;
        }
        yuv_frame.set_pts(source_frame.pts());

        // 4. Sende den konvertierten Frame an den Encoder.
        if encoder.send_frame(&yuv_frame).is_ok() {
            let mut encoded = ffmpeg::Packet::empty();
            // 5. Empfange alle komprimierten Pakete, die der Encoder eventuell ausgibt.
            while encoder.receive_packet(&mut encoded).is_ok() {
                println!(
                    "[Engine] Komprimiertes Paket empfangen, Größe: {} bytes.",
                    encoded.size()
                );
                // TODO: Sende das `encoded`-Paket an die RTMP-Crate.
            }
        }
    }

    // Hilfsfunktion, um den Zeitstempel für jeden Frame zu erhöhen.
    // Provisorische Implementierung.
    fn next_pts(&mut self) -> i64 {
        // In einer echten App bräuchten wir einen Frame-Zähler.
        // `ffmpeg_next` bietet dafür leider keine direkte, einfache Lösung.
        // Dies ist eine sehr vereinfachte Annäherung.
        use std::time::{SystemTime, UNIX_EPOCH};
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        // Konvertiere zu Millisekunden und dann in unsere Zeitbasis.
        (since_the_epoch.as_millis() * 60 / 1000) as i64
    }
}
