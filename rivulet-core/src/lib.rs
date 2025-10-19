// In rivulet-core/src/lib.rs

use rusty_ffmpeg::avcodec::{AsCodec, Codec, VideoEncoder};
use rusty_ffmpeg::avformat::format::ll::AV_PIX_FMT_YUV420P;
use rusty_ffmpeg::avutil::{
    frame::{Frame, Video},
    pix_fmt::AVPixelFormat,
    rational::AVRational,
};
use rusty_ffmpeg::error::RsmpegError;
use rusty_ffmpeg::swscale::{self, SwsContext};

pub struct RivuletEngine {
    scaler: Option<SwsContext>,
    encoder: Option<VideoEncoder>,
    width: u32,
    height: u32,
    frame_count: i64,
    is_streaming: bool,
}

impl Default for RivuletEngine {
    fn default() -> Self {
        Self {
            scaler: None,
            encoder: None,
            width: 0,
            height: 0,
            frame_count: 0,
            is_streaming: false,
        }
    }
}

impl RivuletEngine {
    pub fn new() -> Self {
        // Bei rusty_ffmpeg ist keine explizite init()-Funktion nötig.
        println!("[Engine] Bereit.");
        Self::default()
    }

    pub fn start_streaming(&mut self) {
        if self.is_streaming {
            return;
        }
        println!("[Engine] Streaming wird gestartet...");
        self.frame_count = 0;
        self.is_streaming = true;
    }

    pub fn stop_streaming(&mut self) {
        if !self.is_streaming {
            return;
        }
        println!("[Engine] Streaming wird gestoppt...");

        if let Some(mut encoder) = self.encoder.take() {
            // Sende leeren Frame, um den Encoder zu flushen
            match encoder.encode(None) {
                Ok(Some(packet)) => {
                    println!("[Engine] Flushing-Paket empfangen.");
                    // TODO: Sende das letzte Paket an die RTMP-Crate
                }
                _ => {}
            }
        }

        self.scaler = None;
        self.is_streaming = false;
        println!("[Engine] Streaming gestoppt.");
    }

    fn initialize_ffmpeg(&mut self, width: u32, height: u32) -> Result<(), RsmpegError> {
        println!(
            "[Engine] Initialisiere FFmpeg für Auflösung {}x{}",
            width, height
        );
        self.width = width;
        self.height = height;

        // --- Encoder-Setup ---
        let codec = Codec::find_encoder_by_name("libx264").expect("H.264 Encoder nicht gefunden.");
        let mut encoder = VideoEncoder::new(
            codec,
            width as i32,
            height as i32,
            AVPixelFormat::AV_PIX_FMT_YUV420P,
        )?;

        encoder.set_time_base(AVRational { num: 1, den: 60 });
        encoder.set_gop_size(10);
        encoder.set_max_b_frames(1);

        // Optionen setzen
        encoder.set_option("preset", "ultrafast")?;
        encoder.set_option("tune", "zerolatency")?;

        self.encoder = Some(encoder);

        // --- Scaler-Setup ---
        let scaler = SwsContext::get(
            width as i32,
            height as i32,
            AVPixelFormat::AV_PIX_FMT_RGBA,
            width as i32,
            height as i32,
            AVPixelFormat::AV_PIX_FMT_YUV420P,
            swscale::SWS_BILINEAR,
        )?;

        self.scaler = Some(scaler);
        Ok(())
    }

    pub fn process_raw_frame(&mut self, frame_data: &[u8], width: u32, height: u32) {
        if !self.is_streaming {
            return;
        }

        if self.encoder.is_none() {
            if self.initialize_ffmpeg(width, height).is_err() {
                eprintln!("[Engine] FFmpeg-Initialisierung fehlgeschlagen.");
                self.is_streaming = false;
                return;
            }
        }

        let encoder = self.encoder.as_mut().unwrap();
        let scaler = self.scaler.as_mut().unwrap();

        // 1. Erstelle einen Quell-Frame aus den RGBA-Daten.
        let mut source_frame =
            Video::new(AVPixelFormat::AV_PIX_FMT_RGBA, width as i32, height as i32).unwrap();
        source_frame.get_data_mut(0).copy_from_slice(frame_data);
        // `linesize` muss korrekt gesetzt werden!
        source_frame.set_linesize(0, width as i32 * 4);
        source_frame.set_pts(self.frame_count);
        self.frame_count += 1;

        // 2. Erstelle einen leeren Ziel-Frame.
        let mut yuv_frame = Video::new(
            AVPixelFormat::AV_PIX_FMT_YUV420P,
            width as i32,
            height as i32,
        )
        .unwrap();
        yuv_frame.set_pts(source_frame.get_pts());

        // 3. Führe die Farbkonvertierung durch.
        if scaler.scale(&source_frame, &mut yuv_frame).is_err() {
            eprintln!("[Engine] Fehler beim Skalieren des Frames.");
            return;
        }

        // 4. Sende den konvertierten Frame an den Encoder und empfange Pakete.
        match encoder.encode(Some(&yuv_frame)) {
            Ok(Some(packet)) => {
                println!(
                    "[Engine] Komprimiertes Paket empfangen, Größe: {} bytes.",
                    packet.get_size()
                );
                // TODO: Sende das `packet` an die RTMP-Crate.
            }
            Ok(None) => {
                // Encoder hat den Frame gepuffert, gibt noch kein Paket aus.
            }
            Err(e) => {
                eprintln!("[Engine] Fehler beim Enkodieren: {:?}", e);
            }
        }
    }
}
