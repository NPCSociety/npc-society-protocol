package com.npcsociety.examples;

import com.npcsociety.protocol.v1.*;
import io.grpc.ManagedChannel;
import io.grpc.ManagedChannelBuilder;
import io.grpc.stub.StreamObserver;

import java.util.UUID;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;

/**
 * Minimal example demonstrating how to connect to the NPC Society daemon
 * and exchange messages over the bidirectional Connect() stream.
 * 
 * This is a reference implementation for the Paper plugin integration.
 * 
 * Demonstrates:
 * - Hello handshake with voice_available and server_name (v1.1+)
 * - WorldTick streaming
 * - ChatObservation sending
 * - SpeakDirective handling with directive_id/stream_id correlation
 * - ActionResult sending for MoveAction
 * - ScanBlocksResult mock response
 */
public class ExampleClient {
    
    private final ManagedChannel channel;
    private final NpcSocietyServiceGrpc.NpcSocietyServiceStub asyncStub;
    
    // Track directive IDs for correlation
    private String lastMoveDirectiveId = null;
    private String lastScanDirectiveId = null;
    
    public ExampleClient(String host, int port) {
        this.channel = ManagedChannelBuilder.forAddress(host, port)
                .usePlaintext() // Use TLS in production
                .build();
        this.asyncStub = NpcSocietyServiceGrpc.newStub(channel);
    }
    
    /**
     * Establishes a bidirectional stream with the daemon.
     */
    public void connect() throws InterruptedException {
        CountDownLatch finishLatch = new CountDownLatch(1);
        
        // Create observer for messages from server
        StreamObserver<ServerMessage> responseObserver = new StreamObserver<>() {
            @Override
            public void onNext(ServerMessage message) {
                handleServerMessage(message);
            }
            
            @Override
            public void onError(Throwable t) {
                System.err.println("Stream error: " + t.getMessage());
                finishLatch.countDown();
            }
            
            @Override
            public void onCompleted() {
                System.out.println("Stream completed");
                finishLatch.countDown();
            }
        };
        
        // Open the bidirectional stream
        StreamObserver<ClientMessage> requestObserver = asyncStub.connect(responseObserver);
        
        try {
            // Example A: Send Hello handshake with v1.1+ fields
            sendHello(requestObserver);
            
            // Simulate sending WorldTick updates at ~10Hz
            for (int i = 0; i < 50; i++) {
                sendWorldTick(requestObserver, i);
                Thread.sleep(100); // 10Hz
            }
            
            // Example B: Send a chat observation (triggers SpeakDirective response)
            sendChatObservation(requestObserver);
            
            // Wait for responses
            Thread.sleep(500);
            
            // Example C: Send action results for any pending directives
            if (lastMoveDirectiveId != null) {
                sendMoveResult(requestObserver, lastMoveDirectiveId, true);
            }
            
            // Example D: Send scan blocks result (simulating response to ScanBlocksAction)
            if (lastScanDirectiveId != null) {
                sendScanBlocksResult(requestObserver, lastScanDirectiveId);
            }
            
        } catch (RuntimeException e) {
            // Cancel stream on error
            requestObserver.onError(e);
            throw e;
        }
        
        // Signal completion
        requestObserver.onCompleted();
        
        // Wait for stream to finish
        if (!finishLatch.await(30, TimeUnit.SECONDS)) {
            System.err.println("Timed out waiting for stream to finish");
        }
    }
    
    /**
     * Example A: Hello handshake with v1.1+ fields.
     */
    private void sendHello(StreamObserver<ClientMessage> requestObserver) {
        Hello hello = Hello.newBuilder()
                .setPluginVersion("1.1.0")
                .setProtocolVersion("1")
                .setServerId("example-server")
                .setMinecraftVersion("1.20.4")
                // v1.1+ fields
                .setVoiceAvailable(true)        // Simple Voice Chat is installed
                .setServerName("Example Server") // Optional display name
                .setDaemonMode("external")       // Diagnostics: daemon runs separately
                .build();
        
        ClientMessage message = ClientMessage.newBuilder()
                .setHello(hello)
                .build();
        
        requestObserver.onNext(message);
        System.out.println("Sent Hello handshake (voice_available=true, server_name='Example Server')");
    }
    
    private void sendWorldTick(StreamObserver<ClientMessage> requestObserver, int tick) {
        // Create example NPC snapshot
        NpcSnapshot npc = NpcSnapshot.newBuilder()
                .setNpcId("miner_01")
                .setEntityUuid(UUID.randomUUID().toString())
                .setPosition(Position.newBuilder()
                        .setWorld("world")
                        .setX(100.5)
                        .setY(64.0)
                        .setZ(-200.5)
                        .setYaw(90.0f)
                        .setPitch(0.0f)
                        .build())
                .setHealthNorm(1.0f)
                .setInCombat(false)
                .setHungerNorm(0.8f)
                .setHeldItem("minecraft:diamond_pickaxe")
                .setCurrentActivity("mining")
                .build();
        
        // Create example player snapshot
        PlayerSnapshot player = PlayerSnapshot.newBuilder()
                .setPlayerUuid(UUID.randomUUID().toString())
                .setPlayerName("Steve")
                .setPosition(Position.newBuilder()
                        .setWorld("world")
                        .setX(105.0)
                        .setY(64.0)
                        .setZ(-198.0)
                        .setYaw(270.0f)
                        .setPitch(0.0f)
                        .build())
                .setHealthNorm(0.95f)
                .setHeldItem("minecraft:diamond_pickaxe")
                .setSneaking(false)
                .setSprinting(false)
                .setGameMode("survival")
                .build();
        
        WorldTick worldTick = WorldTick.newBuilder()
                .setServerTick(tick)
                .setTimestampMs(System.currentTimeMillis())
                .addNpcs(npc)
                .addNearbyPlayers(player)
                .build();
        
        ClientMessage message = ClientMessage.newBuilder()
                .setWorldTick(worldTick)
                .build();
        
        requestObserver.onNext(message);
        
        if (tick % 10 == 0) {
            System.out.println("Sent WorldTick #" + tick);
        }
    }
    
    /**
     * Example B: Send ChatObservation (triggers SpeakDirective with directive_id).
     */
    private void sendChatObservation(StreamObserver<ClientMessage> requestObserver) {
        ChatObservation chat = ChatObservation.newBuilder()
                .setNpcId("miner_01")
                .setPlayerUuid(UUID.randomUUID().toString())
                .setPlayerName("Steve")
                .setMessage("Hey miner, can you find some diamonds?")
                .setTimestampMs(System.currentTimeMillis())
                .setDistance(5.0f)
                .build();
        
        ClientMessage message = ClientMessage.newBuilder()
                .setChatObservation(chat)
                .build();
        
        requestObserver.onNext(message);
        System.out.println("Sent ChatObservation: 'Hey miner, can you find some diamonds?'");
    }
    
    /**
     * Example C: Send MoveResult for a completed MoveAction.
     */
    private void sendMoveResult(StreamObserver<ClientMessage> requestObserver, 
                                 String directiveId, boolean success) {
        ActionResult result = ActionResult.newBuilder()
                .setDirectiveId(directiveId)
                .setNpcId("miner_01")
                .setSuccess(success)
                .setErrorMessage(success ? "" : "Path blocked by obstacle")
                .setMoveResult(MoveResult.newBuilder()
                        .setFinalPosition(Position.newBuilder()
                                .setWorld("world")
                                .setX(105.5)
                                .setY(64.0)
                                .setZ(-200.5)
                                .setYaw(90.0f)
                                .setPitch(0.0f)
                                .build())
                        .setReachedDestination(success)
                        .build())
                .build();
        
        ClientMessage message = ClientMessage.newBuilder()
                .setActionResult(result)
                .build();
        
        requestObserver.onNext(message);
        System.out.println("Sent MoveResult: directive=" + directiveId + ", success=" + success);
    }
    
    /**
     * Example D: Send ScanBlocksResult (mining perception loop).
     */
    private void sendScanBlocksResult(StreamObserver<ClientMessage> requestObserver, 
                                       String directiveId) {
        // Simulate finding 3 diamond ore blocks
        ScanBlocksResult scanResult = ScanBlocksResult.newBuilder()
                .addMatches(BlockMatch.newBuilder()
                        .setPosition(BlockPosition.newBuilder()
                                .setWorld("world")
                                .setX(102)
                                .setY(12)
                                .setZ(-195)
                                .build())
                        .setBlockType("minecraft:diamond_ore")
                        .build())
                .addMatches(BlockMatch.newBuilder()
                        .setPosition(BlockPosition.newBuilder()
                                .setWorld("world")
                                .setX(104)
                                .setY(11)
                                .setZ(-198)
                                .build())
                        .setBlockType("minecraft:deepslate_diamond_ore")
                        .build())
                .addMatches(BlockMatch.newBuilder()
                        .setPosition(BlockPosition.newBuilder()
                                .setWorld("world")
                                .setX(99)
                                .setY(10)
                                .setZ(-200)
                                .build())
                        .setBlockType("minecraft:diamond_ore")
                        .build())
                .build();
        
        ActionResult result = ActionResult.newBuilder()
                .setDirectiveId(directiveId)
                .setNpcId("miner_01")
                .setSuccess(true)
                .setScanBlocksResult(scanResult)
                .build();
        
        ClientMessage message = ClientMessage.newBuilder()
                .setActionResult(result)
                .build();
        
        requestObserver.onNext(message);
        System.out.println("Sent ScanBlocksResult: found " + scanResult.getMatchesCount() + " diamond ore blocks");
    }
    
    /**
     * Send a failed ActionResult to demonstrate error handling.
     */
    private void sendFailedBreakBlockResult(StreamObserver<ClientMessage> requestObserver, 
                                             String directiveId) {
        ActionResult result = ActionResult.newBuilder()
                .setDirectiveId(directiveId)
                .setNpcId("miner_01")
                .setSuccess(false)
                .setErrorMessage("Block is out of reach")
                .setBreakBlockResult(BreakBlockResult.newBuilder().build())
                .build();
        
        ClientMessage message = ClientMessage.newBuilder()
                .setActionResult(result)
                .build();
        
        requestObserver.onNext(message);
        System.out.println("Sent failed BreakBlockResult: 'Block is out of reach'");
    }
    
    private void handleServerMessage(ServerMessage message) {
        switch (message.getMessageCase()) {
            case ACTION_DIRECTIVE -> {
                ActionDirective directive = message.getActionDirective();
                System.out.println("Received ActionDirective: id=" + directive.getDirectiveId() 
                        + ", npc=" + directive.getNpcId()
                        + ", action=" + directive.getActionCase());
                
                // Track directive IDs for sending results
                switch (directive.getActionCase()) {
                    case MOVE -> lastMoveDirectiveId = directive.getDirectiveId();
                    case SCAN_BLOCKS -> lastScanDirectiveId = directive.getDirectiveId();
                    default -> {}
                }
                
                // In real plugin: execute the action and send ActionResult
            }
            case SPEAK_DIRECTIVE -> {
                SpeakDirective speak = message.getSpeakDirective();
                
                // Example E: Verify correlation fields (v1.1+)
                System.out.println("Received SpeakDirective:");
                System.out.println("  npc_id: " + speak.getNpcId());
                System.out.println("  text: " + speak.getText());
                System.out.println("  emotion: " + speak.getEmotion());
                System.out.println("  duration_ms: " + speak.getDurationMs());
                
                // v1.1+ correlation fields
                if (!speak.getDirectiveId().isEmpty()) {
                    System.out.println("  directive_id: " + speak.getDirectiveId());
                }
                if (!speak.getVoiceId().isEmpty()) {
                    System.out.println("  voice_id: " + speak.getVoiceId());
                }
                if (speak.getVolume() > 0) {
                    System.out.println("  volume: " + speak.getVolume());
                }
                if (!speak.getStreamId().isEmpty()) {
                    System.out.println("  stream_id: " + speak.getStreamId() + " (expect matching AudioChunks)");
                }
                
                // In real plugin: display subtitle
            }
            case AUDIO_CHUNK -> {
                AudioChunk audio = message.getAudioChunk();
                
                // Example E: Verify audio correlation with SpeakDirective
                System.out.println("Received AudioChunk:");
                System.out.println("  npc_id: " + audio.getNpcId());
                System.out.println("  stream_id: " + audio.getStreamId());
                System.out.println("  sequence: " + audio.getSequence());
                System.out.println("  bytes: " + audio.getPcmData().size());
                System.out.println("  is_final: " + audio.getIsFinal());
                
                // v1.1+ optional correlation
                if (!audio.getDirectiveId().isEmpty()) {
                    System.out.println("  directive_id: " + audio.getDirectiveId() + " (correlates with SpeakDirective)");
                }
                
                // In real plugin: queue audio for Simple Voice Chat playback
            }
            default -> System.out.println("Unknown message type: " + message.getMessageCase());
        }
    }
    
    public void shutdown() throws InterruptedException {
        channel.shutdown().awaitTermination(5, TimeUnit.SECONDS);
    }
    
    public static void main(String[] args) throws Exception {
        String host = System.getenv().getOrDefault("DAEMON_HOST", "localhost");
        int port = Integer.parseInt(System.getenv().getOrDefault("DAEMON_PORT", "50051"));
        
        System.out.println("=== NPC Society Protocol Example Client ===");
        System.out.println("Connecting to daemon at " + host + ":" + port);
        System.out.println();
        
        ExampleClient client = new ExampleClient(host, port);
        try {
            client.connect();
        } finally {
            client.shutdown();
        }
    }
}
