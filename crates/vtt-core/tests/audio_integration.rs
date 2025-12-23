//! Integration tests for audio capture

use std::thread;
use std::time::Duration;
use vtt_core::audio::{list_devices, write_wav, AudioCapture};

#[test]
fn test_device_enumeration() {
    let devices = list_devices();
    assert!(devices.is_ok());

    if let Ok(devs) = devices {
        println!("Found {} audio devices", devs.len());
        for dev in devs {
            println!("  - {} (default: {})", dev.name, dev.is_default);
        }
    }
}

#[test]
#[ignore] // Requires audio hardware
fn test_capture_and_save() {
    let mut capture = AudioCapture::new().expect("Failed to create audio capture");

    capture.start().expect("Failed to start capture");
    thread::sleep(Duration::from_millis(500));
    capture.stop().expect("Failed to stop capture");

    let samples = capture.take_buffer();
    assert!(!samples.is_empty(), "No audio captured");

    write_wav("/tmp/test_capture.wav", &samples, capture.format()).expect("Failed to write WAV");

    println!("Captured {} samples", samples.len());
}
