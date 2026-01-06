# NPC Society Protocol - Java Example

Minimal example demonstrating how to use the NPC Society gRPC protocol from Java.

## Prerequisites

1. Generate the Java code from proto files:
   ```bash
   cd ../..
   buf generate
   ```

2. Java 17 or higher

## Building

```bash
./gradlew build
```

## Running

Make sure the daemon is running, then:

```bash
# Default: connects to localhost:50051
./gradlew run

# Or specify host/port
DAEMON_HOST=192.168.1.100 DAEMON_PORT=50051 ./gradlew run
```

## What This Example Does

1. Connects to the daemon via gRPC
2. Sends a `Hello` handshake
3. Sends `WorldTick` updates at 10Hz for 5 seconds
4. Sends a sample `ChatObservation`
5. Handles incoming `ActionDirective`, `SpeakDirective`, and `AudioChunk` messages

## Integration Notes

In a real Paper plugin:
- Run the gRPC client on an async thread (never block the main thread)
- Map `ActionDirective` to actual Minecraft actions
- Stream audio chunks to Simple Voice Chat
- Capture real world state for `WorldTick` snapshots

