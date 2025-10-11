#[cfg(not(windows))]
fn main() {
    eprintln!("This example only works on Windows");
    std::process::exit(1);
}

#[cfg(windows)]
fn main() -> anyhow::Result<()> {
    use rivulet_capture::{CaptureSource, DxgiScreenCapture};
    use std::time::Duration;

    tracing_subscriber::fmt::init();

    println!("Initializing screen capture...");
    let mut capture = DxgiScreenCapture::new(0)?;
    
    let (width, height) = capture.dimensions();
    println!("Initial dimensions: {}x{}", width, height);

    capture.start()?;
    println!("Capturing started. Capturing 300 frames (~10 seconds at 30fps)...");

    let mut frame_count = 0;
    let start = std::time::Instant::now();

    for i in 0..300 {
        match capture.capture_frame()? {
            Some(frame) => {
                frame_count += 1;
                if frame_count == 1 {
                    println!("First frame captured!");
                    println!("  Resolution: {}x{}", frame.width, frame.height);
                    println!("  Data size: {} bytes", frame.data.len());
                    println!("  Stride: {} bytes", frame.stride);
                }
                if i % 30 == 0 {
                    println!("Frame {}: {}x{}", frame_count, frame.width, frame.height);
                }
            }
            None => {
                // No new frame available
            }
        }
        std::thread::sleep(Duration::from_millis(33)); // ~30fps
    }

    let elapsed = start.elapsed();
    let fps = frame_count as f64 / elapsed.as_secs_f64();
    println!("\n=== Results ===");
    println!("Captured {} frames in {:.2}s", frame_count, elapsed.as_secs_f64());
    println!("Average FPS: {:.2}", fps);

    capture.stop()?;
    println!("Capture stopped successfully!");
    Ok(())
}
