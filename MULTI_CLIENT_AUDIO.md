# Multiple Audio Capture Users

## The Problem

By default, ALSA/PipeWire audio devices are **exclusive access** - only one client can capture at a time.

## Why This Exists

Audio hardware requires precise timing. Multiple clients trying to control the same hardware directly would cause:
- **Glitches/pops** from conflicting period sizes
- **Sample rate mismatches** 
- **Buffer underruns**

## The Real Answer

**Short answer**: With cpal + ALSA: **NO** - hardware is exclusive access.

**Long answer**: Multiple users IS possible via PipeWire's native API, but cpal doesn't support it yet.

## Solutions

| Path | Multi-Client? | Notes |
|------|---------------|-------|
| cpal → ALSA → Hardware | ❌ No | Hardware is exclusive |
| PipeWire native API | ✅ Yes | Native multi-client support |
| PipeWire monitor ports | ✅ Yes | Read-only copies |
| PipeWire loopback/graph | ✅ Yes | Can tee to N outputs |

## For VTT-MCP

Single ownership is acceptable - it's designed as a background service daemon.

---

**Bottom Line**: The limitation is because we use cpal's ALSA backend. PipeWire itself supports multi-client natively.
