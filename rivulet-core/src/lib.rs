// In rivulet-core/src/lib.rs

use ffmpeg::format::Pixel;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use ffmpeg_next as ffmpeg;

pub struct RivuletEngine {
    scaler: Option<Context>,
    encoder: Option<ffmpeg::codec::encoder::video::Video>,
    encoding_context: Option<ffmpeg::codec::Context>,
    frame_count: i64,
    is_streaming: bool,
}

impl Default for RivuletEngine {
    fn default() -> Self {
        Self {
            scaler: None,
            encoder: None,
            encoding_context: None,
            frame_count: 0,
            is_streaming: false,
        }
    }
}

impl RivuletEngine {
    pub fn new() -> Self {
        ffmpeg::init().expect("Fehler bei der Initialisierung von FFmpeg");
        println!("[Engine] FFmpeg initialisiert.");
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
            if encoder.send_eof().is_ok() {
                let mut encoded = ffmpeg::Packet::empty();
                while encoder.receive_packet(&mut encoded).is_ok() {
                    println!("[Engine] Flushing-Paket empfangen.");
                }
            }
        }
        self.scaler = None;
        self.encoding_context = None;
        self.is_streaming = false;
        println!("[Engine] Streaming gestoppt.");
    }

    fn initialize_ffmpeg(&mut self, width: u32, height: u32) {
        let codec = ffmpeg::codec::encoder::find_by_name("libx264")
            .expect("H.264 Encoder (libx264) nicht gefunden.");
        let mut context = ffmpeg::codec::Context::new();
        context.set_height(height);
        context.set_width(width);
        context.set_time_base((1, 60));
        context.set_frame_rate(Some((60, 1)));
        context.set_format(Pixel::YUV420P);
        let mut opts = context.options();
        opts.set("preset", "ultrafast").unwrap();
        opts.set("tune", "zerolatency").unwrap();
        let encoder = context
            .encoder()
            .video()
            .expect("Konnte Video-Encoder nicht öffnen.");
        self.encoder = Some(encoder);
        self.encoding_context = Some(context);

        let scaler = Context::get(
            Pixel::RGBA,
            width,
            height,
            Pixel::YUV420P,
            width,
            height,
            Flags::BILINEAR,
        )
        .expect("Konnte Scaler nicht erstellen.");
        self.scaler = Some(scaler);
    }

    pub fn process_raw_frame(&mut self, frame_data: &[u8], width: u32, height: u32) {
        if !self.is_streaming {
            return;
        }
        if self.encoder.is_none() {
            self.initialize_ffmpeg(width, height);
        }

        let encoder = self.encoder.as_mut().unwrap();
        let scaler = self.scaler.as_mut().unwrap();

        let mut source_frame = unsafe { Video::from_slice(frame_data, Pixel::RGBA, width, height) };
        source_frame.set_pts(Some(self.frame_count));
        self.frame_count += 1;

        let mut yuv_frame = Video::empty();
        if scaler.run(&source_frame, &mut yuv_frame).is_err() {
            return;
        }
        yuv_frame.set_pts(source_frame.pts());

        if encoder.send_frame(&yuv_frame).is_ok() {
            let mut encoded = ffmpeg::Packet::empty();
            while encoder.receive_packet(&mut encoded).is_ok() {
                println!(
                    "[Engine] Komprimiertes Paket empfangen, Größe: {} bytes.",
                    encoded.size()
                );
            }
        }
    }
}
