//! Example of using the Whisper transcription module
//!
//! This example demonstrates how to load a Whisper model and transcribe audio.

use vtt_core::whisper::{WhisperConfig, WhisperContext};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("VTT-MCP Whisper Transcription Example");
    println!("=====================================");

    // Check if model exists
    let model_path = "models/ggml-base.bin";
    if !std::path::Path::new(model_path).exists() {
        eprintln!("Error: Model file not found at {}", model_path);
        eprintln!("Please download the Whisper base model first.");
        std::process::exit(1);
    }

    // Create configuration
    let config = WhisperConfig::default()
        .with_model_path(model_path)
        .with_threads(4);

    println!("Loading Whisper model from: {}", config.model_path);

    // Create the context
    let ctx = WhisperContext::new(config)?;
    println!("Model loaded successfully!");
    println!(
        "  - Sample rate: {}Hz (will resample input to this)",
        ctx.config().required_sample_rate
    );
    println!("  - GPU enabled: {}", ctx.config().use_gpu);
    println!("  - Threads: {}", ctx.config().n_threads);

    // For testing, we'll create 1 second of silence
    // In real usage, you would capture audio from a microphone or load from a file
    let sample_rate = 16000u32; // 16kHz as required by Whisper
    let audio_data = vec![0.0_f32; sample_rate as usize]; // 1 second of silence

    println!(
        "
Transcribing {} samples ({} seconds)...",
        audio_data.len(),
        audio_data.len() / sample_rate as usize
    );

    // Transcribe the audio
    let result = ctx.transcribe(&audio_data, sample_rate)?;

    println!(
        "
Transcription Result:"
    );
    println!("  - Text: {}", result.text);
    println!("  - Start: {}ms", result.start_timestamp);
    println!("  - End: {}ms", result.end_timestamp);
    println!("  - Duration: {}ms", result.end_timestamp - result.start_timestamp);

    Ok(())
}
