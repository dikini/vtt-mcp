### /home/dikini/Projects/vtt-mcp/docs/working-notes/04-multi-client-audio.md
```markdown
1: # Multiple Audio Capture Users
2: 
3: ## The Problem
4: 
5: By default, ALSA/PipeWire audio devices are **exclusive access** - only one client can capture at a time.
6: 
7: ## Why This Exists
8: 
9: Audio hardware requires precise timing. Multiple clients trying to control the same hardware directly would cause:
10: - **Glitches/pops** from conflicting period sizes
11: - **Sample rate mismatches** 
12: - **Buffer underruns**
13: 
14: ## The Real Answer
15: 
16: **Short answer**: With cpal + ALSA: **NO** - hardware is exclusive access.
17: 
18: **Long answer**: Multiple users IS possible via PipeWire's native API, but cpal doesn't support it yet.
19: 
20: ## Solutions
21: 
22: | Path | Multi-Client? | Notes |
23: |------|---------------|-------|
24: | cpal → ALSA → Hardware | ❌ No | Hardware is exclusive |
25: | PipeWire native API | ✅ Yes | Native multi-client support |
26: | PipeWire monitor ports | ✅ Yes | Read-only copies |
27: | PipeWire loopback/graph | ✅ Yes | Can tee to N outputs |
28: 
29: ## For VTT-MCP
30: 
31: Single ownership is acceptable - it's designed as a background service daemon.
32: 
33: ---
34: 
35: **Bottom Line**: The limitation is because we use cpal's ALSA backend. PipeWire itself supports multi-client natively.
```
