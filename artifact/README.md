# artifact — GPU Artifact Simulator

A portable, fun GPU artifacting system for Wayland compositors. Simulates the visual artifacts of a dying GPU without actually breaking your hardware.

## Features

- **Progressive corruption**: 5 stages over 25 minutes (5 min per 20%)
  - Stage 0: Nothing
  - Stage 1: Green dots (VRAM artifacts)
  - Stage 2: + Horizontal tearing
  - Stage 3: + More tearing
  - Stage 4: + Random corruption noise
  - Stage 5: + Screen error message ("Display not found" or "GPU not found")

- **Background daemon**: Runs independently, controllable via IPC
- **Wayland-native**: Works on any Wayland compositor
- **Safe**: Pure visual overlay, no actual hardware impact
- **Portable**: No dependency on specific desktops (Niri, GNOME, KDE, etc.)

## Building

```bash
cd ~/artifact
cargo build --release
```

## Usage

### Start the daemon

```bash
./artifact daemon
```

Or run in background:

```bash
./artifact daemon &
```

### Control the simulation

```bash
./artifact start    # Start the corruption progression
./artifact stop     # Stop/pause the simulation
./artifact status   # Check current status
```

### Direct binary usage

```bash
./target/release/artifact                  # Run as daemon
./target/release/artifact start            # IPC command
./target/release/artifact status           # Check status
```

## How it works

- The binary runs as both daemon and client
- Daemon listens on `/tmp/artifact.sock` for IPC commands
- Client sends commands via Unix socket
- Wayland surface renders corruption at ~12 FPS
- Clicks/input pass through (transparent to user interaction)
- Corruption meter tracks elapsed time: `(elapsed / 25min) * 100%`

## Architecture

```
main.rs          → Entry point (daemon or client mode)
wayland.rs       → Wayland surface creation & rendering
corruption.rs    → Artifact pattern generators
progress.rs      → Time-based progression tracking
ipc.rs           → Unix socket server/client
```

## IPC Commands

- `start` — Begin the 25-minute corruption progression
- `stop` — Pause the simulation
- `status` — Returns JSON: `{running, corruption_level, stage}`

## Notes

- Renders at ~12 FPS for authentic "struggling GPU" feel
- 50% frame drop chance for stuttering effect
- Works on any Wayland compositor (tested: Niri, GNOME Wayland, KDE Plasma)
- No xdg-shell or layer-shell required (minimal protocol deps)
