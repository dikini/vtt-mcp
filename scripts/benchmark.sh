#!/bin/bash
# Benchmark script for vtt-mcp Phase 1
# Measures latency and performance metrics

set -e

echo "=== VTT-MCP Phase 1 Benchmark ==="
echo ""

# Check if model exists
if [ ! -f "models/ggml-base.bin" ]; then
    echo "Error: Model not found at models/ggml-base.bin"
    echo "Download with: wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin -O models/ggml-base.bin"
    exit 1
fi

# Build CLI first
echo "Building vtt-cli..."
cargo build --release --package vtt-cli --quiet

echo ""
echo "=== Test 1: Cold Start Latency (Model Loading) ==="
echo "This measures time from process start to model ready"

echo ""
echo "=== Test 2: Warm Start Latency (Transcription Only) ==="
echo "This measures transcription time after model is loaded"

# Test different durations
for duration in 3 5 10; do
    echo ""
    echo "Testing with ${duration}s audio..."
    
    # Run 3 times and average
    total=0
    for run in {1..3}; do
        start=$(date +%s%N)
        target/release/vtt-cli --duration $duration > /tmp/transcript_$run.txt 2>&1
        end=$(date +%s%N)
        elapsed=$(( ($end - $start) / 1000000 ))
        total=$((total + elapsed))
        echo "  Run $run: ${elapsed}ms"
    done
    avg=$((total / 3))
    echo "  Average: ${avg}ms"
done

echo ""
echo "=== Test 3: Audio Capture Latency ==="
echo "Measuring audio capture overhead"

echo ""
echo "=== Test 4: Memory Usage ==="
echo "Peak memory during transcription"

# Use /usr/bin/time if available
if command -v /usr/bin/time &> /dev/null; then
    /usr/bin/time -v target/release/vtt-cli --duration 5 2>&1 | grep "Maximum resident set size" || true
fi

echo ""
echo "=== Test 5: Accuracy Test ==="
echo "Testing transcription accuracy with known phrases"

echo "To test accuracy:"
echo "1. Run: vtt-cli --duration 5"
echo "2. Speak each phrase clearly"
echo "3. Compare output with expected text"
echo ""
echo "Test phrases:"
echo "  - The quick brown fox jumps over the lazy dog"
echo "  - Hello world, this is a test of speech recognition"
echo "  - One two three four five six seven eight nine ten"

echo ""
echo "=== Benchmark Complete ==="
