use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct VideoEncoder {
    output_path: PathBuf,
    width: u32,
    height: u32,
    fps: u32,
    bitrate: u64,
    frame_count: u64,
    // TODO: FFmpeg encoder context hier hinzufügen
}

impl VideoEncoder {
    pub fn new(
        output_path: &Path,
        width: u32,
        height: u32,
        fps: u32,
        bitrate: u64,
    ) -> Result<Self> {
        if width == 0 || height == 0 {
            anyhow::bail!("Invalid dimensions: {}x{}", width, height);
        }

        if fps == 0 {
            anyhow::bail!("Invalid FPS: {}", fps);
        }

        println!("Creating VideoEncoder:");
        println!("  Output: {:?}", output_path);
        println!("  Resolution: {}x{}", width, height);
        println!("  FPS: {}", fps);
        println!("  Bitrate: {} bps", bitrate);

        // TODO: Initialisiere FFmpeg encoder hier
        // Beispiel mit ffmpeg-next oder gstreamer

        Ok(Self {
            output_path: output_path.to_path_buf(),
            width,
            height,
            fps,
            bitrate,
            frame_count: 0,
        })
    }

    pub fn encode_frame(
        &mut self,
        frame_data: &[u8],
        width: u32,
        height: u32,
        stride: u32,
    ) -> Result<()> {
        // Validierung
        if width != self.width || height != self.height {
            anyhow::bail!(
                "Frame dimensions {}x{} don't match encoder {}x{}",
                width, height, self.width, self.height
            );
        }

        let expected_size = (stride * height) as usize;
        if frame_data.len() < expected_size {
            anyhow::bail!(
                "Frame data too small: {} bytes, expected at least {}",
                frame_data.len(),
                expected_size
            );
        }

        // TODO: Encode frame mit FFmpeg
        // Für jetzt nur zählen
        self.frame_count += 1;

        Ok(())
    }

    pub fn finish(self) -> Result<()> {
        println!("Finalizing video...");
        println!("  Total frames: {}", self.frame_count);
        println!("  Duration: {:.2}s", self.frame_count as f64 / self.fps as f64);

        // TODO: Schließe FFmpeg encoder und schreibe Datei

        println!("Video saved to: {:?}", self.output_path);
        Ok(())
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}