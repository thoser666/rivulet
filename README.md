# Rivulet

## About
Rivulet is an experimental, from-scratch reimplementation of OBS Studio written in Rust. The goal is to build a modern, memory-safe streaming and recording application with a modular architecture and first-class plugin support, while remaining familiar to OBS users.

### Why Rivulet?
- Memory safety and performance via Rust
- Modular crates per domain (capture, audio, streaming, GUI, plugins)
- OBS plugin compatibility layer to reuse the existing ecosystem
- Cross-platform focus (Windows first; Linux and macOS planned)

### Current status
Rivulet is in early development. What exists today:
- A basic GUI scaffold using egui/eframe
- A bootstrapped core engine (rivulet-core)
- An experimental OBS compatibility initialization layer

Expect frequent changes; this is not yet ready for production.

### Planned features
- Scene and source graph
- Screen/window/game capture
- Audio capture, filters, and mixing
- Streaming (RTMP/SRT) and local recording
- Themeable egui-based UI
- Plugin system with Rust-first APIs and OBS compatibility

## Quick Start

Build all crates:
    cargo build

Run the GUI application:
    cargo run --bin rivulet

## Architecture

- rivulet-core: Core engine and data structures
- rivulet-gui: GUI application using egui
- rivulet-capture: Screen/window capture
- rivulet-audio: Audio capture and processing
- rivulet-streaming: RTMP streaming and recording
- rivulet-plugins: Native plugin system
- rivulet-obs-compat: OBS Studio plugin compatibility

## License

MIT License
