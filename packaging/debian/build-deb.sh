#!/usr/bin/env bash
set -e

# Set working directory to project root
cd "$(dirname "$0")/../.."

echo "Building Debian package..."

# Ensure target binary exists
if [ -f "target/x86_64-unknown-linux-musl/release/audio-system-tray" ]; then
    BINARY="target/x86_64-unknown-linux-musl/release/audio-system-tray"
    ARCH="amd64"
elif [ -f "target/release/audio-system-tray" ]; then
    BINARY="target/release/audio-system-tray"
    ARCH=$(dpkg --print-architecture 2>/dev/null || echo "amd64")
else
    echo "Binary not found. Building native release binary first..."
    cargo build --release
    BINARY="target/release/audio-system-tray"
    ARCH=$(dpkg --print-architecture 2>/dev/null || echo "amd64")
fi

STAGING="target/debian-pkg"
rm -rf "$STAGING"
mkdir -p "$STAGING/usr/bin"
mkdir -p "$STAGING/usr/share/applications"
mkdir -p "$STAGING/DEBIAN"

# Copy binary and desktop file
cp "$BINARY" "$STAGING/usr/bin/audio-system-tray"
cp "audio-system-tray.desktop" "$STAGING/usr/share/applications/audio-system-tray.desktop"

# Create DEBIAN/control file
cat <<EOF > "$STAGING/DEBIAN/control"
Package: audio-system-tray
Version: 0.1.0
Section: utils
Priority: optional
Architecture: $ARCH
Maintainer: hakumai23
Description: Native PipeWire/PulseAudio audio sink switcher system tray
 Native Status Notifier Item (SNI) tray icon daemon that dynamically lists
 and switches default audio output sinks. Works with Waybar, Hyprland, GNOME,
 KDE, XFCE, and other desktop environments.
EOF

# Build package
dpkg-deb --build "$STAGING" "target/audio-system-tray_${ARCH}.deb"

echo "Debian package successfully built: target/audio-system-tray_${ARCH}.deb"
rm -rf "$STAGING"
