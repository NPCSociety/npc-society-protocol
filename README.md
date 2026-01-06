# NPC Society Protocol

gRPC/protobuf contracts between the Paper plugin (Java client) and daemon (Rust server).

## Overview

This repository defines the communication protocol for NPC Society, enabling:
- Real-time world state synchronization (5-20Hz)
- Voice audio streaming (ASR input, TTS output)
- Action directives and results
- Chat and event observations

## Architecture

```
┌─────────────────┐                    ┌─────────────────┐
│  Paper Plugin   │◄──── gRPC ────────►│     Daemon      │
│    (Java)       │   bidirectional    │     (Rust)      │
│                 │      stream        │                 │
│ - World capture │                    │ - Autonomy loop │
│ - Action exec   │                    │ - Memory/LLM    │
│ - Voice I/O     │                    │ - ASR/TTS       │
└─────────────────┘                    └─────────────────┘
```

## Quick Start

### Prerequisites

- [buf](https://buf.build/docs/installation) CLI

### Generate Code

```bash
# Generate Java + Rust code
buf generate

# Output structure:
# gen/
# ├── java/          # Java protobuf + gRPC stubs
# └── rust/src/      # Rust prost + tonic code
```

### Lint Proto Files

```bash
buf lint
```

### Check Breaking Changes

```bash
buf breaking --against '.git#branch=main'
```

## Protocol Structure

### Service

```protobuf
service NpcSocietyService {
  rpc Connect(stream ClientMessage) returns (stream ServerMessage);
}
```

### Client Messages (Plugin → Daemon)

| Message | Purpose | Frequency |
|---------|---------|-----------|
| `Hello` | Handshake with version info | Once on connect |
| `WorldTick` | Nearby NPC/player snapshots | 5-20Hz |
| `ChatObservation` | Player chat near NPC | On chat event |
| `EventObservation` | Game events (combat, blocks) | On event |
| `VoicePcmFrame` | Raw PCM from Simple Voice Chat | ~50Hz during speech |
| `ActionResult` | Completed action outcome | After action |

### Server Messages (Daemon → Plugin)

| Message | Purpose |
|---------|---------|
| `ActionDirective` | Command NPC to act (move, break, attack, etc.) |
| `SpeakDirective` | Text for subtitle display |
| `AudioChunk` | TTS audio for Simple Voice Chat playback |

## Examples

- [`examples/java/`](examples/java/) - Minimal Java gRPC client
- [`examples/rust/`](examples/rust/) - Minimal Rust gRPC server
- [`docs/EXAMPLE.md`](docs/EXAMPLE.md) - Pseudo-code snippets with message flow diagram

## Versioning

Protocol versions follow semantic versioning. The `protocol_version` field in `Hello` enables version negotiation.

Breaking changes increment the major version and require updating both plugin and daemon.

## Development

### Adding New Messages

1. Edit `proto/npc_society/v1/npc_society.proto`
2. Run `buf lint` to check style
3. Run `buf generate` to update generated code
4. Update examples if needed

### Testing Changes

```bash
# Start example server
cd examples/rust && cargo run

# In another terminal, run example client
cd examples/java && ./gradlew run
```

