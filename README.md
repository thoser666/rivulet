<div align="center">

# 🌊 Rivulet

**Modern Screen Recording & Streaming Software**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Status](https://img.shields.io/badge/status-alpha%20v0.1-yellow.svg)](https://github.com/thoser666/rivulet)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/thoser666/rivulet)

*A complete Rust reimplementation of OBS Studio - built for performance, safety, and reliability*

[Features](#-features) • [Installation](#-installation) • [Roadmap](#-roadmap) • [Contributing](#-contributing)

![Rivulet Screenshot](docs/screenshot.png)
<!-- Add screenshot later -->

</div>

---

## 🎯 Vision

Rivulet aims to be a **complete reimplementation of OBS Studio in Rust**, providing all the features streamers and content creators need while leveraging Rust's safety and performance guarantees.

### Why Rust?
- 🔒 **Memory Safety** - No segfaults, no data races
- ⚡ **Performance** - Zero-cost abstractions
- 🛡️ **Reliability** - Catch bugs at compile time
- 🌍 **Cross-Platform** - Write once, run everywhere

### Long-term Goals (v1.0+)
- **Feature Parity** with OBS Studio
- **Plugin Compatibility** with existing OBS plugins
- **Modern Architecture** - Clean, maintainable codebase
- **Active Community** - Open development, regular updates

---

## ✨ Features

### ✅ Currently Available (v0.1)

- **Screen Capture** - Capture your primary monitor in real-time
- **Video Encoding** - H.264 encoding via FFmpeg
- **Live Preview** - See what you're recording as you record
- **Customizable Settings**
  - Adjustable FPS (15-60)
  - Bitrate control (1-50 Mbps)
  - Custom output path with file picker
- **Cross-Platform** - Windows, macOS, and Linux support (via xcap)
- **Modern UI** - Clean interface built with egui

### 🚧 In Development (v0.2 - December 2025)

- **Audio Capture**
  - System audio (desktop sound)
  - Microphone input
  - Audio mixer with volume controls
- **Hardware Encoding**
  - NVIDIA NVENC
  - Intel QuickSync
  - AMD AMF
  - 