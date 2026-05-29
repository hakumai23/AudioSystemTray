# AudioSystemTray

A native **PipeWire/WirePlumber** audio sink switcher system tray application for **Wayland** desktop environments (with primary support for **Hyprland**).

## Overview

**AudioSystemTray** is a lightweight, fast system tray daemon that allows you to quickly switch between audio output devices (sinks) using a convenient context menu. It implements the **Status Notifier Item (SNI)** protocol, making it compatible with most modern Linux system trays.

### Features

- 🎵 **Dynamic audio sink detection** — automatically discovers available audio devices
- 🎯 **Quick switching** — switch audio outputs from the system tray menu
- 🔄 **Dual backend support** — works with both **PipeWire** (via `wpctl`) and **PulseAudio** (via `pactl`)
- ⚡ **Lightweight & fast** — minimal dependencies, optimized for size and performance
- 🚀 **Wayland-native** — built for modern Linux desktop environments
- 📦 **Multi-distribution support** — pre-built packages for Arch Linux, Debian, Ubuntu, Fedora, and RHEL
- 🔊 **Always up-to-date** — automatically updates the available sinks in real-time

## System Requirements

- **Desktop Environment**: Wayland-based (Hyprland, GNOME, KDE Plasma, etc.)
- **Audio Server**: PipeWire or PulseAudio
- **Architecture**: x86_64 Linux
- **System Tray**: A compatible system tray supporting the Status Notifier Item protocol

### Required Tools

- **For PipeWire systems**: `wpctl` (WirePlumber command-line tool)
- **For PulseAudio systems**: `pactl` (PulseAudio command-line tool)

> **Note**: The application automatically detects which audio backend is available on your system.

## Installation

### Arch Linux

```bash
sudo pacman -S audio-system-tray
```

Or from the official repositories (if available):

```bash
yay -S audio-system-tray
# or
paru -S audio-system-tray
```

### Debian / Ubuntu

```bash
sudo apt-get install audio-system-tray
```

Or download and install the `.deb` package manually:

```bash
wget https://github.com/hakumai23/AudioSystemTray/releases/download/latest/audio-system-tray_*.deb
sudo dpkg -i audio-system-tray_*.deb
```

### Fedora / RHEL / CentOS

```bash
sudo dnf install audio-system-tray
```

Or using `rpm`:

```bash
wget https://github.com/hakumai23/AudioSystemTray/releases/download/latest/audio-system-tray-*.rpm
sudo rpm -i audio-system-tray-*.rpm
```

### Manual Installation from Source

#### Prerequisites

- **Rust toolchain** (1.70+): [Install Rust](https://www.rust-lang.org/tools/install)
- **Make**: `sudo apt-get install make` (Debian/Ubuntu) or `sudo pacman -S make` (Arch)
- **Git**: For cloning the repository

#### Building

1. **Clone the repository**:

```bash
git clone https://github.com/hakumai23/AudioSystemTray.git
cd AudioSystemTray
```

2. **Build the project**:

```bash
# For native (statically linked) binary
make build-native

# Or for a fully static x86_64-musl binary
make setup      # First-time setup for cross-compilation
make build
```

3. **Install**:

```bash
sudo make install

# Or customize the installation prefix
sudo make install PREFIX=/usr
```

The installation includes:
- Binary: `/usr/local/bin/audio-system-tray` (default)
- Desktop entry: `/usr/local/share/applications/audio-system-tray.desktop`
- Autostart configuration: `~/.config/autostart/audio-system-tray.desktop`

4. **Uninstall**:

```bash
sudo rm /usr/local/bin/audio-system-tray
sudo rm /usr/local/share/applications/audio-system-tray.desktop
rm ~/.config/autostart/audio-system-tray.desktop
```

## Usage

### Starting the Application

#### Automatic (Recommended)

The application is automatically added to your autostart directory during installation. It will start automatically when you log in to your desktop environment.

To verify autostart is enabled:

```bash
cat ~/.config/autostart/audio-system-tray.desktop
```

#### Manual Start

```bash
# Start in the background
audio-system-tray &

# Or run directly in foreground
audio-system-tray
```

### Using the Application

1. Look for the **🔊 Audio Output** icon in your system tray
2. Click the icon to open the context menu
3. Select an audio device from the list to switch to it
4. The icon label updates to show the currently active audio device

### Checking the Audio Backend

To verify which audio backend is being used:

```bash
# Check for PipeWire
wpctl status

# Check for PulseAudio
pactl info
```

## Building Packages

The project supports building distributable packages for multiple Linux distributions:

### Build Debian Package

```bash
make deb
# Output: target/*.deb
```

### Build RPM Package (Fedora/RHEL)

```bash
make rpm
# Output: target/rpm/RPMS/**/*.rpm
```

### Build Arch Linux Package

```bash
make arch
# Output: target/arch/*.pkg.tar.zst
```

### Build All Packages

```bash
make packages
# Builds Debian, RPM, and Arch packages
```

## Development

### Project Structure

```
AudioSystemTray/
├── src/
│   ├── main.rs          # Main application and SNI implementation
│   └── backend.rs       # Audio backend interfaces (WpCtl, PaCtl)
├── packaging/           # Distribution-specific package definitions
│   ├── debian/          # Debian package build scripts
│   ├── rpm/             # RPM specification
│   └── arch/            # Arch Linux PKGBUILD
├── Cargo.toml           # Rust project configuration
├── Makefile             # Build and installation commands
└── audio-system-tray.desktop  # Desktop entry file
```

### Running Tests

```bash
make test
```

### Code Quality

The project uses:
- **Rust edition 2021** with strict compiler checks
- **LTO (Link-Time Optimization)** and aggressive optimizations in release builds
- **Stripped debug symbols** for minimal binary size

## Dependencies

### Runtime

- **ksni** (v0.3) — Status Notifier Item protocol implementation
- **tokio** (v1) — Async runtime for system monitoring

### Build-Time

- **Rust compiler** (1.70+)
- **Make** — Build automation
- **dpkg-deb** — Debian package building
- **rpmbuild** — RPM package building
- **makepkg** — Arch Linux package building

## Troubleshooting

### The tray icon doesn't appear

1. Verify your system tray supports the SNI protocol:
   ```bash
   gdbus introspect --system /org/freedesktop/DBus
   ```

2. Check if the application is running:
   ```bash
   ps aux | grep audio-system-tray
   ```

3. Ensure your desktop environment's system tray is properly configured.

### No audio devices are shown

1. Verify your audio backend is working:
   ```bash
   wpctl status    # For PipeWire
   pactl list sinks  # For PulseAudio
   ```

2. Check that `wpctl` or `pactl` is installed:
   ```bash
   which wpctl
   which pactl
   ```

3. Check the application logs (if running in foreground):
   ```bash
   audio-system-tray
   ```

### Application crashes after clicking the tray icon

This could indicate an issue with the audio backend. Try running in foreground to see error messages:

```bash
audio-system-tray
# Look for error messages related to wpctl or pactl
```

## Contributing

Contributions are welcome! Please feel free to open issues, submit pull requests, or suggest improvements.

## License

This project is licensed under the **MIT License** — see the LICENSE file for details.

## Acknowledgments

- Built with [ksni](https://github.com/iovxd/ksni) for Status Notifier Item support
- Powered by [Tokio](https://tokio.rs/) for async runtime
- Supports PipeWire/WirePlumber and PulseAudio audio servers

---

**Made with ❤️ for the Linux desktop community**
