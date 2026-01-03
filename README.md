# Roland

A simple touch gesture recognizer for Linux, built on top of the `input` crate. This project is a temporary solution to enable touch gestures for my touch device on Niri.

## Features

- Multi finger touch support
- Recognize basic touch gestures (e.g., swipes).
- Configurable gesture callbacks.

## Usage

1. Build the project:
   ```sh
   cargo build --release
   ```
2. Run the binary:
   ```sh
   ./target/release/roland
   ```

## Configuration

See [config.example.toml](config.example.toml) for a detailed example configuration.
