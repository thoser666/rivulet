use anyhow::{Context, Result};
use ffmpeg_next as ffmpeg;
use std::path::Path;
use tracing;

pub struct VideoEncoder {
    encoder: ffmpeg::encoder::Video,
    octx: ffmpeg::format::context::Output,
    stream_index: usize,
    frame_count: u64,
}

impl VideoEncoder {
    pub fn new(
        output_path: &Path,
        width: u32,
        height: u32,
        fps: u32,
        bitrate: u64,
    ) -> Result<Self> {
        // Initialize FFmpeg
        ffmpeg::init().context("Failed to initialize FFmpeg")?;

        // Create output context
        let mut octx = ffmpeg::format::output(output_path)
            .context("Failed to create output context")?;

        // Find H.264 encoder
        let codec = ffmpeg::encoder::find(ffmpeg::codec::Id::H264)
            .context("H.264 encoder not found")?;

        // Add video stream
        let mut ost = octx.add_stream(codec)
            .context("Failed to add stream")?;
        let stream_index = ost.index();

        // Configure encoder
        let mut encoder = ffmpeg::codec::context::Context::new()
            .encoder()
            .video()
            .context("Failed to create encoder context")?;

        encoder.set_width(width);
        encoder.set_height(height);
        encoder.set_format(ffmpeg::format::Pixel::YUV420P);
        encoder.set_time_base((1, fps as i32));
        encoder.set_frame_rate(Some((fps as i32, 1)));
        encoder.set_bit_rate(bitrate as usize);

        // Open encoder
        let encoder = encoder.open_as(codec)
            .context("Failed to open encoder")?;

        // Set stream parameters
        ost.set_parameters(&encoder);

        // Write header
        octx.write_header()
            .context("Failed to write header")?;

        tracing::info!(
            "Video encoder initialized: {}x{} @ {} fps, {} kbps",
            width, height, fps, bitrate / 1000
        );

        Ok(Self {
            encoder,
            octx,
            stream_index,
            frame_count: 0,
        })
    }

    pub fn encode_frame(&mut self, rgba_data: &[u8], width: u32, height: u32, stride: u32) -> Result<()> {
        // Create frame
        let mut frame = ffmpeg::util::frame::Video::new(
            ffmpeg::format::Pixel::YUV420P,
            width,
            height,
        );

        // Convert BGRA to YUV420P
        self.bgra_to_yuv420p(rgba_data, width, height, stride, &mut frame)?;

        // Set timestamp
        frame.set_pts(Some(self.frame_count as i64));
        self.frame_count += 1;

        // Encode frame
        self.encoder.send_frame(&frame)
            .context("Failed to send frame to encoder")?;

        // Receive packets
        self.receive_packets()?;

        Ok(())
    }

    fn bgra_to_yuv420p(
        &self,
        bgra: &[u8],
        width: u32,
        height: u32,
        stride: u32,
        frame: &mut ffmpeg::util::frame::Video,
    ) -> Result<()> {
        // Get YUV planes
        let y_plane = frame.data_mut(0);
        let u_plane = frame.data_mut(1);
        let v_plane = frame.data_mut(2);

        // Convert BGRA to YUV420P
        for y in 0..height {
            for x in 0..width {
                let bgra_idx = (y * stride + x * 4) as usize;

                if bgra_idx + 3 >= bgra.len() {
                    continue;
                }

                let b = bgra[bgra_idx] as f32;
                let g = bgra[bgra_idx + 1] as f32;
                let r = bgra[bgra_idx + 2] as f32;

                // RGB to YUV conversion
                let y_val = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
                let u_val = ((-0.169 * r - 0.331 * g + 0.5 * b) + 128.0) as u8;
                let v_val = ((0.5 * r - 0.419 * g - 0.081 * b) + 128.0) as u8;

                // Write Y
                y_plane[(y * width + x) as usize] = y_val;

                // Write U and V (subsampled)
                if y % 2 == 0 && x % 2 == 0 {
                    let uv_idx = ((y / 2) * (width / 2) + (x / 2)) as usize;
                    u_plane[uv_idx] = u_val;
                    v_plane[uv_idx] = v_val;
                }
            }
        }

        Ok(())
    }

    fn receive_packets(&mut self) -> Result<()> {
        let mut encoded = ffmpeg::Packet::empty();

        while self.encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(self.stream_index);
            encoded.write_interleaved(&mut self.octx)
                .context("Failed to write packet")?;
        }

        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        // Flush encoder
        self.encoder.send_eof()
            .context("Failed to send EOF")?;

        self.receive_packets()?;

        // Write trailer
        self.octx.write_trailer()
            .context("Failed to write trailer")?;

        tracing::info!("Video encoding finished: {} frames", self.frame_count);
        Ok(())
    }
}