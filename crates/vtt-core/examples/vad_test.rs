//! VAD testing example
//!
//! Run with: cargo run --example vad_test

use vtt_core::vad::{VadConfig, VadDetector, VadResult};

fn main() -> anyhow::Result<()> {
    println!("=== Voice Activity Detection Test ===\n");

    // Test 1: Default configuration
    println!("Test 1: Default Configuration");
    let mut detector = VadDetector::new();
    println!("  Threshold: {:.4}", detector.threshold());
    println!("  Initialized: {}", detector.is_initialized());

    // Test 2: Process silence
    println!("\nTest 2: Processing Silence");
    detector.reset();
    let silence_samples = vec![0.0; 512];
    for i in 0..10 {
        let result = detector.process_frame(&silence_samples)?;
        println!("  Frame {}: {:?}", i, result);
    }

    // Test 3: Process speech-like audio
    println!("\nTest 3: Processing Speech-like Audio");
    detector.reset();
    for i in 0..15 {
        // Generate a sine wave pattern
        let speech_samples: Vec<f32> = (0..512)
            .map(|j| 0.2 * ((i * 512 + j) as f32 * 0.05).sin())
            .collect();
        let result = detector.process_frame(&speech_samples)?;
        println!("  Frame {}: {:?}", i, result);
    }

    // Test 4: Test back to silence
    println!("\nTest 4: Return to Silence");
    for i in 0..20 {
        let result = detector.process_frame(&silence_samples)?;
        println!("  Frame {}: {:?}", i, result);
        if result == VadResult::Silence {
            println!("  -> Silence detected at frame {}", i);
            break;
        }
    }

    // Test 5: Sensitive configuration
    println!("\nTest 5: Sensitive Configuration");
    let config = VadConfig::sensitive();
    println!("  Threshold: {:.4}", config.energy_threshold);
    let mut sensitive_detector = VadDetector::with_config(config);

    // Test with quieter audio
    let quiet_samples: Vec<f32> = (0..512).map(|j| 0.05 * (j as f32 * 0.1).sin()).collect();

    for i in 0..15 {
        let result = sensitive_detector.process_frame(&quiet_samples)?;
        println!("  Frame {}: {:?}", i, result);
        if result == VadResult::Speech {
            println!("  -> Speech detected with sensitive config!");
            break;
        }
    }

    // Test 6: Strict configuration
    println!("\nTest 6: Strict Configuration");
    let config = VadConfig::strict();
    println!("  Threshold: {:.4}", config.energy_threshold);
    let mut strict_detector = VadDetector::with_config(config);

    // Test with moderate audio - strict might not detect it
    let moderate_samples: Vec<f32> = (0..512).map(|j| 0.15 * (j as f32 * 0.1).sin()).collect();

    for i in 0..15 {
        let result = strict_detector.process_frame(&moderate_samples)?;
        println!("  Frame {}: {:?}", i, result);
    }

    println!("\n=== All Tests Complete ===");
    Ok(())
}
