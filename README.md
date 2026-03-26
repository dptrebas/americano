# Americano ☕

Cross-platform CLI to keep your machine awake — part of the **NautilOS AI Agentic Operating System**.

Like Amphetamine on macOS, but works on Linux, Windows, and (future) Android.

## Features

- Prevent system sleep and optionally display sleep
- Cross-platform (macOS, Linux, Windows)
- Graceful Ctrl+C handling
- Subcommands: `start` and `stop`

## Installation / Usage

```bash
# Build from source
cargo build --release

# Start for 60 minutes (default) with display awake
./target/release/americano start --display

# Start indefinitely
./target/release/americano start --minutes 0 --display

# Start for 2 hours without keeping display awake
./target/release/americano start --minutes 120

# Stop (graceful)
./target/release/americano stop