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
 */
public class ExampleClient {
    
    private final ManagedChannel channel;
    private final NpcSocietyServiceGrpc.NpcSocietyServiceStub asyncStub;
    
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
            // Send Hello handshake
            sendHello(requestObserver);
            
            // Simulate sending WorldTick updates at ~10Hz
            for (int i = 0; i < 50; i++) {
                sendWorldTick(requestObserver, i);
                Thread.sleep(100); // 10Hz
            }
            
            // Send a chat observation example
            sendChatObservation(requestObserver);
            
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
    
    private void sendHello(StreamObserver<ClientMessage> requestObserver) {
        Hello hello = Hello.newBuilder()
                .setPluginVersion("1.0.0")
                .setProtocolVersion("1")
                .setServerId("example-server")
                .setMinecraftVersion("1.20.4")
                .build();
        
        ClientMessage message = ClientMessage.newBuilder()
                .setHello(hello)
                .build();
        
        requestObserver.onNext(message);
        System.out.println("Sent Hello handshake");
    }
    
    private void sendWorldTick(StreamObserver<ClientMessage> requestObserver, int tick) {
        // Create example NPC snapshot
        NpcSnapshot npc = NpcSnapshot.newBuilder()
                .setNpcId("guard_01")
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
                .setHeldItem("minecraft:iron_sword")
                .setCurrentActivity("patrolling")
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
    
    private void sendChatObservation(StreamObserver<ClientMessage> requestObserver) {
        ChatObservation chat = ChatObservation.newBuilder()
                .setNpcId("guard_01")
                .setPlayerUuid(UUID.randomUUID().toString())
                .setPlayerName("Steve")
                .setMessage("Hello, guard! How are you today?")
                .setTimestampMs(System.currentTimeMillis())
                .setDistance(5.0f)
                .build();
        
        ClientMessage message = ClientMessage.newBuilder()
                .setChatObservation(chat)
                .build();
        
        requestObserver.onNext(message);
        System.out.println("Sent ChatObservation");
    }
    
    private void handleServerMessage(ServerMessage message) {
        switch (message.getMessageCase()) {
            case ACTION_DIRECTIVE -> {
                ActionDirective directive = message.getActionDirective();
                System.out.println("Received ActionDirective: " + directive.getDirectiveId() 
                        + " for NPC: " + directive.getNpcId());
                // In real plugin: execute the action and send ActionResult
            }
            case SPEAK_DIRECTIVE -> {
                SpeakDirective speak = message.getSpeakDirective();
                System.out.println("Received SpeakDirective for NPC " + speak.getNpcId() 
                        + ": " + speak.getText());
                // In real plugin: display subtitle
            }
            case AUDIO_CHUNK -> {
                AudioChunk audio = message.getAudioChunk();
                System.out.println("Received AudioChunk for NPC " + audio.getNpcId() 
                        + ", stream: " + audio.getStreamId() 
                        + ", seq: " + audio.getSequence()
                        + ", final: " + audio.getIsFinal());
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
        
        System.out.println("Connecting to daemon at " + host + ":" + port);
        
        ExampleClient client = new ExampleClient(host, port);
        try {
            client.connect();
        } finally {
            client.shutdown();
        }
    }
}

