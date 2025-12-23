// Benchmark tool for measuring Whisper transcription latency
// Usage: cargo run --example benchmark --package vtt-core

use std::path::Path;
use std::thread;
use std::time::Instant;
use vtt_core::audio::AudioCapture;
use vtt_core::whisper::{WhisperConfig, WhisperContext};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== VTT-MCP Latency Benchmark ===\n");

    // Check model
    let model_path = "models/ggml-base.bin";
    if !Path::new(model_path).exists() {
        eprintln!("Error: Model not found at {}", model_path);
        eprintln!("Download with: wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin -O models/ggml-base.bin");
        return Err("Model not found".into());
    }

    println!("1. Cold Start Latency (Model Loading)");
    println!("------------------------------------");
    let cold_start = Instant::now();
    let config = WhisperConfig::default().with_model_path(model_path);
    let _ctx = WhisperContext::new(config)?;
    let cold_time = cold_start.elapsed();
    println!("Cold start time: {:.2}ms\n", cold_time.as_millis());

    println!("2. Warm Start Latency (Transcription Only)");
    println!("------------------------------------------");

    let durations = vec![3u64, 5, 10];
    for duration in durations {
        println!("\nTesting with {}s audio:", duration);

        let mut total_time = 0u128;
        let runs = 3;

        for run in 1..=runs {
            // Capture audio
            let capture_start = Instant::now();
            let mut capture = AudioCapture::new()?;
            capture.start()?;
            thread::sleep(std::time::Duration::from_secs(duration));
            capture.stop()?;
            let audio_data = capture.take_buffer();
            let capture_time = capture_start.elapsed();

            // Transcribe
            let transcribe_start = Instant::now();
            let ctx = WhisperContext::new(WhisperConfig::default().with_model_path(model_path))?;
            let sample_rate = capture.format().sample_rate;
            let result = ctx.transcribe(&audio_data, sample_rate)?;
            let transcribe_time = transcribe_start.elapsed();

            let e2e_time = capture_time + transcribe_time;
            total_time += e2e_time.as_millis();

            println!(
                "  Run {}: capture={:.2}ms, transcribe={:.2}ms, total={:.2}ms",
                run,
                capture_time.as_millis(),
                transcribe_time.as_millis(),
                e2e_time.as_millis()
            );

            if run == 1 && !result.text.is_empty() {
                println!("  Text: \"{}\"", result.text.trim());
            }
        }

        let avg = total_time / runs;
        println!("  Average: {}ms", avg);
    }

    println!("\n3. Accuracy Test");
    println!("---------------");
    println!("To test accuracy:");
    println!("  1. Speak clearly: \"The quick brown fox jumps over the lazy dog\"");
    println!("  2. Run: vtt-cli --duration 5");
    println!("  3. Compare output with expected text");
    println!("\nExpected WER (Word Error Rate) for Whisper base:");
    println!("  - Clear speech: <5%");
    println!("  - Noisy environment: 10-20%");

    println!("\n=== Benchmark Complete ===");
    Ok(())
}
