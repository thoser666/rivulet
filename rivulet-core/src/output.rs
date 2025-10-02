use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Output settings for recording/streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSettings {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32,
    pub encoder: String,
}

impl Default for OutputSettings {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30,
            bitrate: 5000,
            encoder: "h264".to_string(),
        }
    }
}

/// Trait for all output types (recording, streaming, etc.)
#[async_trait]
pub trait Output: Debug + Send + Sync {
    async fn start(&self) -> anyhow::Result<()>;
    async fn stop(&self) -> anyhow::Result<()>;
    fn is_active(&self) -> bool;
    fn get_settings(&self) -> &OutputSettings;
}

/// File recording output
#[derive(Debug)]
pub struct FileOutput {
    pub file_path: String,
    pub settings: OutputSettings,
    active: std::sync::atomic::AtomicBool,
}

impl FileOutput {
    pub fn new(file_path: impl Into<String>, settings: OutputSettings) -> Self {
        Self {
            file_path: file_path.into(),
            settings,
            active: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl Output for FileOutput {
    async fn start(&self) -> anyhow::Result<()> {
        tracing::info!("Starting file recording to: {}", self.file_path);
        self.active.store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        tracing::info!("Stopping file recording");
        self.active.store(false, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn is_active(&self) -> bool {
        self.active.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn get_settings(&self) -> &OutputSettings {
        &self.settings
    }
}