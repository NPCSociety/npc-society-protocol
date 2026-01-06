# NPC Society Protocol - Rust Example

Minimal example demonstrating how to implement the NPC Society daemon server in Rust.

## Prerequisites

1. Generate the Rust code from proto files:
   ```bash
   cd ../..
   buf generate
   ```

2. Rust 1.70 or higher

## Building

```bash
cargo build --release
```

## Running

```bash
# Default: listens on 0.0.0.0:50051
cargo run --release

# Or specify port
PORT=50052 cargo run --release
```

## What This Example Does

1. Starts a gRPC server on port 50051
2. Handles incoming `Connect()` streams from plugins
3. Processes client messages:
   - `Hello` - logs handshake info
   - `WorldTick` - sends example `MoveAction` every 50 ticks
   - `ChatObservation` - responds with `SpeakDirective`
   - `VoicePcmFrame` - echoes dummy audio chunks
   - `ActionResult` - logs completion status

## Integration Notes

In the real daemon:
- Process voice frames through ASR pipeline
- Run autonomy loops on WorldTick updates
- Query LLM for decisions at commit points
- Generate TTS audio and stream back as AudioChunk
- Track action directive completion via ActionResult

