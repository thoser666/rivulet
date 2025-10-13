use anyhow::{Context, Result};
use crossbeam_channel::{bounded, Sender, Receiver};
use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;

pub mod encoder;
pub mod recorder;

pub use encoder::{Encoder, VideoEncoder};
pub use recorder::*;

/// Frame data for encoding
#[derive(Debug, Clone)]
pub struct EncodableFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub timestamp: std::time::Instant,
}

/// Recording settings
#[derive(Debug, Clone)]
pub struct RecordingSettings {
    pub output_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u64,
    pub codec: String,
}

impl Default for RecordingSettings {
    fn default() -> Self {
        Self {
            output_path: PathBuf::from("recording.mp4"),
            width: 1920,
            height: 1080,
            fps: 30,
            bitrate: 5_000_000,
            codec: "h264".to_string(),
        }
    }
}