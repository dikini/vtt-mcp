### /home/dikini/Projects/vtt-mcp/docs/working-notes/06-testing-without-mic.md
```markdown
1: # Testing VTT-MCP Without a Microphone
2: 
3: Since the microphone is unavailable, here are alternative testing approaches:
4: 
5: ## 1. Test with Existing WAV Files
6: 
7: ```bash
8: # Use a test WAV file
9: ffmpeg -f lavfi -i "sine=frequency=1000:duration=3" /tmp/test_1khz.wav
10: 
11: # TODO: Add file input support to vtt-cli
12: # This will be needed for the transcribe_clip tool (task vtt-mcp-csh.3.2)
13: ```
14: 
15: ## 2. Use a Null Device (for testing pipeline)
16: 
17: ```bash
18: # Create a mock capture that generates silence
19: # This allows testing the rest of the pipeline
20: 
21: # In vtt-core, add a MockCapture device for testing
22: ```
23: 
24: ## 3. Use USB Microphone (if available)
25: 
26: ```bash
27: # List all devices
28: arecord -l
29: 
30: # Use specific device
31: arecord -d 3 -f S16_LE -r 16000 -c 1 -D hw:2,0 /tmp/usb_test.wav
32: ```
33: 
34: ## 4. Remote Machine with Working Audio
35: 
36: ```bash
37: # SSH to a machine with working microphone
38: ssh user@machine-with-mic
39: 
40: # Run tests there
41: cd /path/to/vtt-mcp
42: cargo run --bin vtt-cli
43: ```
44: 
45: ## 5. Test Core Functionality
46: 
47: Without audio capture, you can still test:
48: 
49: - ✅ Device enumeration (`list_devices()`)
50: - ✅ WAV writing (`write_wav()`)
51: - ✅ Format conversions
52: - ✅ Error handling
53: - ✅ Configuration system (when implemented)
54: 
55: ## Current Task Status
56: 
57: | Task | Status | Notes |
58: |------|--------|-------|
59: | vtt-mcp-csh.1.3 | ✅ Complete | Implementation done, mic issue is environmental |
60: | vtt-mcp-csh.1.4 | 🔶 Next | Silero VAD - may need file input for testing |
61: | vtt-mcp-csh.2.1 | Ready | Whisper setup - doesn't need mic |
```
