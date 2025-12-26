# Roland

A simple touch gesture recognizer for Linux, built on top of the `input` crate. This project is a temporary solution to enable touch gestures for my touch device on Niri.

## Disclaimer

This project is **not intended for long-term maintenance**. It serves as a stopgap until proper touch gesture support is implemented in Niri or other relevant projects. Use it at your own discretion.

## Features

- Recognize basic touch gestures (e.g., swipes).
- Configurable gesture callbacks.
- Lightweight and minimal dependencies.

## Usage

1. Ensure you have the necessary dependencies installed (see `Cargo.toml`).
2. Build the project:
   ```sh
   cargo build --release
   ```
3. Run the binary:
   ```sh
   ./target/release/roland
   ```

## Configuration

Edit the `config.toml` file to customize gesture behavior and callbacks.
