# Roland

A compositor-agnostic touch gesture recognizer for Linux, built on top of the `input` crate.

## Features

- Multi-finger touch support
- Recognizes swipe (8 directions), pinch, and hold gestures
- Edge-aware gestures (trigger only from screen edges)
- Duration and distance thresholds
- Configurable actions via shell commands

## Usage

### Standalone

1. Build the project:
```sh
   cargo build --release
```
2. Run the binary:
```sh
   ./target/release/roland --config config.example.toml
```

See [config.example.toml](config.example.toml) for configuration reference.

### Nix Flake

Roland ships both an overlay (to add `pkgs.roland`) and a Home Manager module.
**Both must be applied.** The module references `pkgs.roland` from the overlay.

#### 1. Add the input

```nix
inputs.roland.url = "github:hftsai256/roland";
inputs.roland.inputs.nixpkgs.follows = "nixpkgs";
```

#### 2. Apply the overlay

The overlay bundles `rust-overlay` internally, so no additional overlays are needed.

```nix
nixpkgs.overlays = [ inputs.roland.overlays.default ];
```

#### 3. Import the Home Manager module

```nix
imports = [ inputs.roland.homeModules.default ];
```

#### 4. Configure

```nix
services.roland = {
  enable = true;
  settings.gestures = [
    {
      num_fingers = 3;
      kind = "SwipeUp";
      min_duration = 50;
      min_distance = 100.0;
      action = ''hyprctl dispatch "hl.dsp.focus({ workspace = '+1' })"'';
    }
    {
      num_fingers = 3;
      kind = "SwipeDown";
      min_duration = 50;
      min_distance = 100.0;
      action = ''hyprctl dispatch "hl.dsp.focus({ workspace = '-1' })"'';
    }
  ];
};
```

See [config.example.toml](config.example.toml) for all available gesture options.

## Configuration Reference

| Field | Type | Required | Description |
|---|---|---|---|
| `num_fingers` | int | yes | Number of fingers for the gesture |
| `kind` | string | yes | `SwipeUp`, `SwipeDown`, `SwipeLeft`, `SwipeRight`, `SwipeUpLeft`, `SwipeUpRight`, `SwipeDownLeft`, `SwipeDownRight`, `PinchIn`, `PinchOut`, `Hold` |
| `action` | string | yes | Shell command to execute |
| `min_distance` | float | no | Minimum movement in pixels |
| `max_distance` | float | no | Maximum movement in pixels |
| `min_duration` | int (ms) | no | Minimum hold duration before gesture fires |
| `max_duration` | int (ms) | no | Maximum hold duration |
| `on_edge` | table | no | Restrict to screen edge: `{ Top = 300 }`, `{ Bottom = 300 }`, `{ Left = 300 }`, `{ Right = 300 }` |
