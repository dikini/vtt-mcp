//! CLI for voice-to-text processing
use std::thread;
use std::time::Duration;
use vtt_core::audio::{list_devices, write_wav, AudioCapture, AudioFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("VTT-CLI - Audio Capture Test\n");

    println!("Available audio input devices:");
    let devices = list_devices()?;
    if devices.is_empty() {
        println!("  No devices found");
        return Ok(());
    }

    for dev in &devices {
        let marker = if dev.is_default { "*" } else { " " };
        println!("  {} {}", marker, dev.name);
    }
    println!();

    println!("Starting 3-second audio capture at 48kHz stereo...");
    let mut capture = AudioCapture::with_format(AudioFormat::DEFAULT)?;

    capture.start()?;
    println!("Recording...");

    thread::sleep(Duration::from_secs(3));

    capture.stop()?;
    let samples = capture.take_buffer();

    println!("Captured {} samples", samples.len());

    let output_path = "/tmp/vtt_test_capture.wav";
    write_wav(output_path, &samples, capture.format())?;
    println!("Saved to: {}", output_path);

    Ok(())
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
