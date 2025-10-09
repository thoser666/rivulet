use rivulet_capture::{CaptureSource, DxgiScreenCapture};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    println!("Initializing screen capture...");
    let mut capture = DxgiScreenCapture::new(0)?;

    let (width, height) = capture.dimensions();
    println!("Capture resolution: {}x{}", width, height);

    capture.start()?;
    println!("Capturing started. Press Ctrl+C to stop.");

    let mut frame_count = 0;
    let start = std::time::Instant::now();

    for _ in 0..300 { // Capture for ~10 seconds at 30fps
        if let Some(frame) = capture.capture_frame()? {
            frame_count += 1;
            println!("Frame {}: {}x{}, {} bytes",
                     frame_count, frame.width, frame.height, frame.data.len());
        }
        std::thread::sleep(Duration::from_millis(33)); // ~30fps
    }

    let elapsed = start.elapsed();
    let fps = frame_count as f64 / elapsed.as_secs_f64();
    println!("\nCaptured {} frames in {:.2}s ({:.2} fps)",
             frame_count, elapsed.as_secs_f64(), fps);

    capture.stop()?;
    Ok(())
}