use anyhow::{Context, Result};
use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};

use crate::{EncodableFrame, Encoder, RecordingSettings};

pub struct Recorder {
    settings: RecordingSettings,
    frame_sender: Option<Sender<EncodableFrame>>,
    encoder_thread: Option<JoinHandle<Result<()>>>,
    is_recording: Arc<AtomicBool>,
}

impl Recorder {
    pub fn new(settings: RecordingSettings) -> Result<Self> {
        Ok(Self {
            settings,
            frame_sender: None,
            encoder_thread: None,
            is_recording: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn start(&mut self) -> Result<()> {
        if self.is_recording.load(Ordering::SeqCst) {
            anyhow::bail!("Recording already in progress");
        }

        let (tx, rx) = bounded::<EncodableFrame>(100);
        self.frame_sender = Some(tx);

        let settings = self.settings.clone();
        let is_recording = Arc::clone(&self.is_recording);

        // Encoder-Thread starten
        let handle = thread::spawn(move || Self::encoder_thread_fn(rx, settings, is_recording));

        self.encoder_thread = Some(handle);
        self.is_recording.store(true, Ordering::SeqCst);

        println!("Recording started: {:?}", self.settings.output_path);
        Ok(())
    }

    pub fn send_frame(&self, frame: EncodableFrame) -> Result<()> {
        if let Some(sender) = &self.frame_sender {
            sender
                .send(frame)
                .context("Failed to send frame to encoder")?;
        }
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if !self.is_recording.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.is_recording.store(false, Ordering::SeqCst);

        // Channel schließen
        drop(self.frame_sender.take());

        // Auf Encoder-Thread warten
        if let Some(handle) = self.encoder_thread.take() {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("Encoder thread panicked"))??;
        }

        println!("Recording stopped");
        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    fn encoder_thread_fn(
        rx: Receiver<EncodableFrame>,
        settings: RecordingSettings,
        is_recording: Arc<AtomicBool>,
    ) -> Result<()> {
        let mut encoder = Encoder::new(settings)?;

        while is_recording.load(Ordering::SeqCst) || !rx.is_empty() {
            match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(frame) => {
                    encoder.encode_frame(&frame)?;
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
            }
        }

        encoder.finalize()?;
        Ok(())
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
