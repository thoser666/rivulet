use anyhow::{Context, Result};
use crate::{EncodableFrame, RecordingSettings};

mod video;
pub use video::VideoEncoder;

pub struct Encoder {
    settings: RecordingSettings,
    start_time: Option<std::time::Instant>,
    frame_count: u64,
}

impl Encoder {
    pub fn new(settings: RecordingSettings) -> Result<Self> {
        if settings.width == 0 || settings.height == 0 {
            anyhow::bail!("Invalid dimensions: {}x{}", settings.width, settings.height);
        }

        Ok(Self {
            settings,
            start_time: None,
            frame_count: 0,
        })
    }

    pub fn encode_frame(&mut self, frame: &EncodableFrame) -> Result<()> {
        if self.start_time.is_none() {
            self.start_time = Some(frame.timestamp);
        }

        self.frame_count += 1;
        Ok(())
    }

    pub fn finalize(&mut self) -> Result<()> {
        println!("Encoded {} frames to {:?}", self.frame_count, self.settings.output_path);
        Ok(())
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}