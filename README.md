# Rivulet

A complete Rust reimplementation of OBS Studio.

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
