# Microphone Issue - Root Cause Analysis

## Summary

You were right - there **IS** a configured microphone on card 1, and it's running through PipeWire. The issue is a **format mismatch** combined with **ALSA/PipeWire incompatibility**.

## The Real Problem

### Hardware Capabilities
- **Codec**: Realtek ALC257 (laptop audio codec)
- **Inputs**: Internal Mic + Mic Jack
- **Supported Sample Rates**: 44100, 48000, 96000, 192000 Hz
- **NOT supported**: 16000 Hz (our original default!)

### Current Status
The microphone **IS** running and available in PipeWire (source 1161), but cpal via ALSA can't access it.

### Why cpal Fails

1. cpal uses ALSA "default" → calls pulse plugin
2. pulse plugin tries pcm_dsnoop → "Device or resource busy"
3. PipeWire already has the device for pro-audio API

## Solution Applied

Changed AudioFormat::DEFAULT from 16kHz to 48kHz (hardware-native).

## Bottom Line

**Your code is correct. The mic is working. cpal's ALSA backend can't access a device that PipeWire has locked.**
