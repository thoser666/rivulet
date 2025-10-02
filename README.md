# Rivulet 🌊

A complete Rust reimplementation of OBS Studio - modern, cross-platform streaming and recording software built from the ground up in Rust.

## 🎯 Project Goals

Rivulet aims to be a complete reimplementation of OBS Studio in Rust, providing:

- **Feature Parity**: All core OBS Studio functionality rebuilt in Rust
- **Memory Safety**: Leveraging Rust's ownership system for crash-free streaming
- **Performance**: Zero-cost abstractions and efficient memory management
- **Cross-Platform**: Native support for Windows, macOS, and Linux
- **Modern Architecture**: Clean, modular codebase built for maintainability
- **Plugin Ecosystem**: Full compatibility with existing OBS Studio plugins

## 🚀 OBS Studio Features (Implementation Status)

- **Multi-Scene Management**: Create and switch between different streaming scenes
- **Multiple Source Types**: 
  - Window Capture
  - Display/Screen Capture
  - Camera/Webcam Input
  - Text Overlays
  - Image/Video Files
  - Browser Sources
- **Recording & Streaming**: Save to file or stream to platforms like Twitch, YouTube
- **Modern UI**: Built with egui for a responsive, native experience
- **Plugin System**: Extensible architecture for custom sources and effects
- **Cross-Platform**: Windows, macOS, and Linux support

## 🏗️ Architecture

Rivulet is built with a modular architecture using Rust workspaces:

- **`rivulet-core`**: Core engine, scene management, and data structures
- **`rivulet-gui`**: GUI application using egui framework
- **`rivulet-capture`**: Screen/window capture implementations
- **`rivulet-audio`**: Audio capture and processing
- **`rivulet-streaming`**: RTMP streaming and recording outputs
- **`rivulet-plugins`**: Native Rivulet plugin system and built-in plugins
- **`rivulet-obs-compat`**: OBS Studio plugin compatibility layer

## 🛠️ Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git

### Building
```bash
# Clone the repository
git clone https://github.com/thoser666/rivulet.git
cd rivulet

# Build all crates
cargo build

# Run the GUI application
cargo run --bin rivulet
