//! VTT-CLI - Voice-to-Text Command Line Tool
//!
//! This tool provides end-to-end speech-to-text functionality:
//! - Record audio from microphone
//! - Transcribe using Whisper
//! - Save transcription to file

use clap::Parser;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use vtt_core::audio::{write_wav, AudioCapture, AudioFormat};
use vtt_core::whisper::{WhisperConfig, WhisperContext};

/// VTT-CLI: Voice-to-Text Command Line Tool
#[derive(Parser, Debug)]
#[command(name = "vtt-cli")]
#[command(author = "VTT-MCP Contributors")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Record audio and transcribe to text using Whisper", long_about = None)]
struct Args {
    /// Duration of recording in seconds (default: 5)
    #[arg(short, long, default_value = "5")]
    duration: u64,

    /// Path to Whisper model file (default: models/ggml-base.bin)
    #[arg(short, long)]
    model: Option<String>,

    /// Output file for transcription (default: stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Number of threads for transcription (default: auto)
    #[arg(short, long)]
    threads: Option<i32>,

    /// List available audio devices and exit
    #[arg(long)]
    list_devices: bool,

    /// Save captured audio to WAV file
    #[arg(long)]
    save_audio: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // List devices mode
    if args.list_devices {
        list_devices()?;
        return Ok(());
    }

    println!("🎤 VTT-CLI - Voice to Text\n");
    println!("Configuration:");
    println!("  Duration: {}s", args.duration);
    println!(
        "  Model: {}",
        args.model.as_deref().unwrap_or("models/ggml-base.bin")
    );
    if let Some(threads) = args.threads {
        println!("  Threads: {}", threads);
    }
    if let Some(ref audio_path) = args.save_audio {
        println!("  Save audio: {}", audio_path.display());
    }
    println!();

    // Step 1: Record audio
    println!("📻 Recording audio...");
    let audio_data = record_audio(args.duration)?;
    let sample_rate = 48000; // PipeWire default
    println!(
        "✓ Captured {} samples ({}s)",
        audio_data.len(),
        args.duration
    );

    // Save audio if requested
    if let Some(ref audio_path) = args.save_audio {
        let format = AudioFormat::DEFAULT;
        write_wav(audio_path.to_str().unwrap(), &audio_data, &format)?;
        println!("✓ Saved audio to: {}", audio_path.display());
    }

    // Step 2: Load Whisper model
    println!("\n🧠 Loading Whisper model...");
    let model_path = args
        .model
        .unwrap_or_else(|| "models/ggml-base.bin".to_string());

    let mut config = WhisperConfig::default().with_model_path(&model_path);
    if let Some(threads) = args.threads {
        config = config.with_threads(threads);
    }

    let ctx = WhisperContext::new(config)?;
    println!("✓ Model loaded successfully");

    // Step 3: Transcribe
    println!("\n🔊 Transcribing...");
    let result = ctx.transcribe(&audio_data, sample_rate)?;

    println!("✓ Transcription complete!");
    println!("\n─────────────────────────────────────");
    println!("TEXT:");
    println!("─────────────────────────────────────");
    println!("{}", result.text);
    println!("─────────────────────────────────────");
    println!("Duration: {}ms", result.duration_ms());
    println!("─────────────────────────────────────");

    // Save to file if requested
    if let Some(output_path) = args.output {
        std::fs::write(&output_path, &result.text)?;
        println!("✓ Saved transcription to: {}", output_path);
    }

    Ok(())
}

/// List available audio input devices
fn list_devices() -> Result<(), Box<dyn std::error::Error>> {
    let devices = vtt_core::audio::list_devices()?;

    if devices.is_empty() {
        println!("No audio input devices found");
        return Ok(());
    }

    println!("Available audio input devices:");
    for dev in &devices {
        let marker = if dev.is_default { " (default)" } else { "" };
        println!("  {}{}", dev.name, marker);
    }

    Ok(())
}

/// Record audio for the specified duration
fn record_audio(duration_secs: u64) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let mut capture = AudioCapture::with_format(AudioFormat::DEFAULT)?;

    capture.start()?;
    println!("  Recording... (speak now!)");

    thread::sleep(Duration::from_secs(duration_secs));

    capture.stop()?;
    let samples = capture.take_buffer();

    Ok(samples)
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
