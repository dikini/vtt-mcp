### /home/dikini/Projects/vtt-mcp/docs/working-notes/03-microphone-diagnosis.md
```markdown
1: # Microphone Issue - Root Cause Analysis
2: 
3: ## Summary
4: 
5: You were right - there **IS** a configured microphone on card 1, and it's running through PipeWire. The issue is a **format mismatch** combined with **ALSA/PipeWire incompatibility**.
6: 
7: ## The Real Problem
8: 
9: ### Hardware Capabilities
10: - **Codec**: Realtek ALC257 (laptop audio codec)
11: - **Inputs**: Internal Mic + Mic Jack
12: - **Supported Sample Rates**: 44100, 48000, 96000, 192000 Hz
13: - **NOT supported**: 16000 Hz (our original default!)
14: 
15: ### Current Status
16: The microphone **IS** running and available in PipeWire (source 1161), but cpal via ALSA can't access it.
17: 
18: ### Why cpal Fails
19: 
20: 1. cpal uses ALSA "default" → calls pulse plugin
21: 2. pulse plugin tries pcm_dsnoop → "Device or resource busy"
22: 3. PipeWire already has the device for pro-audio API
23: 
24: ## Solution Applied
25: 
26: Changed AudioFormat::DEFAULT from 16kHz to 48kHz (hardware-native).
27: 
28: ## Bottom Line
29: 
30: **Your code is correct. The mic is working. cpal's ALSA backend can't access a device that PipeWire has locked.**
```
