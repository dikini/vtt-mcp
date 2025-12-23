#!/bin/bash
set -e
mkdir -p models
echo Downloading Whisper base model
wget -O models/ggml-base.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
ls -lh models/
