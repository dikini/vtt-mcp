#!/bin/bash

# AppImage Build Script for vtt-mcp

set -e

# Define variables
APP=vtt-mcp
VERSION=0.1.0
OUTDIR=out
BUILD_DIR=build
APPIMAGE_TOOL=appimagetool

# Prepare build and output directories
mkdir -p $BUILD_DIR $OUTDIR

# Build the project
cargo build --release

# Copy binary and dependencies to build directory
cp target/release/$APP $BUILD_DIR/
cp -r assets/* $BUILD_DIR/ 2>/dev/null || true

# Create AppDir structure
mkdir -p $BUILD_DIR/AppDir/usr/bin
mkdir -p $BUILD_DIR/AppDir/usr/share/applications
mkdir -p $BUILD_DIR/AppDir/usr/share/icons/hicolor/256x256/apps

# Move binaries to AppDir
mv $BUILD_DIR/$APP $BUILD_DIR/AppDir/usr/bin/

# Create .desktop file
cat <<EOF > $BUILD_DIR/AppDir/usr/share/applications/$APP.desktop
[Desktop Entry]
Name=vtt-mcp
Exec=$APP
Icon=$APP
Type=Application
Categories=Utility;
EOF

# Placeholder for application icon
convert -size 256x256 xc:black $BUILD_DIR/AppDir/usr/share/icons/hicolor/256x256/apps/$APP.png

# Build AppImage
$APPIMAGE_TOOL $BUILD_DIR/AppDir $OUTDIR/$APP-$VERSION.AppImage

# Set permissions
chmod +x $OUTDIR/$APP-$VERSION.AppImage

echo "AppImage has been built successfully: $OUTDIR/$APP-$VERSION.AppImage"
