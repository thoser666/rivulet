use anyhow::Result;

pub mod screen;

#[cfg(windows)]
pub use screen::windows::DxgiScreenCapture;

pub trait CaptureSource {
    /// Start capturing
    fn start(&mut self) -> Result<()>;

    /// Stop capturing
    fn stop(&mut self) -> Result<()>;

    /// Get next frame as RGBA bytes
    fn capture_frame(&mut self) -> Result<Option<CapturedFrame>>;

    /// Get capture dimensions
    fn dimensions(&self) -> (u32, u32);

    /// Check if currently capturing
    fn is_capturing(&self) -> bool;
}

#[derive(Debug)]
pub struct CapturedFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub timestamp: std::time::Instant,
}

impl CapturedFrame {
    pub fn new(data: Vec<u8>, width: u32, height: u32, stride: u32) -> Self {
        Self {
            data,
            width,
            height,
            stride,
            timestamp: std::time::Instant::now(),
        }
    }
}