//! NPC Society Protocol Example Server (Rust)
//!
//! Demonstrates the daemon side of the protocol, including:
//! - Hello handshake with v1.1+ fields
//! - Mining perception loop (ScanBlocks -> BreakBlock -> Deposit)
//! - Audio correlation (SpeakDirective with matching AudioChunk stream)
//! - Error case handling (success=false + error_message)

use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{transport::Server, Request, Response, Status, Streaming};
use tracing::{info, warn, error, debug, Level};

// Include the generated proto code
// In production, this would come from the buf-generated crate
pub mod npc_society {
    pub mod v1 {
        tonic::include_proto!("npc_society.v1");
    }
}

use npc_society::v1::{
    npc_society_service_server::{NpcSocietyService, NpcSocietyServiceServer},
    action_directive::Action,
    ActionDirective, AudioChunk, ClientMessage, ServerMessage, SpeakDirective,
    client_message::Message as ClientMsg,
    server_message::Message as ServerMsg,
    action_result::Result as ActionResultType,
    // Action types
    MoveAction, BreakBlockAction, ScanBlocksAction, DepositToChestAction,
    // Common types
    Position, BlockPosition,
};

/// Counter for generating unique directive IDs
static DIRECTIVE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a unique directive ID
fn next_directive_id() -> String {
    format!("dir-{}", DIRECTIVE_COUNTER.fetch_add(1, Ordering::SeqCst))
}

/// Generate a unique stream ID for audio
fn next_stream_id() -> String {
    format!("stream-{}", DIRECTIVE_COUNTER.fetch_add(1, Ordering::SeqCst))
}

/// Example implementation of the NPC Society service.
#[derive(Debug, Default)]
pub struct ExampleNpcSocietyService;

impl ExampleNpcSocietyService {
    /// Process an incoming client message and return responses.
    fn handle_client_message(&self, msg: ClientMessage, tx: &mpsc::Sender<ServerMessage>) {
        match msg.message {
            Some(ClientMsg::Hello(hello)) => {
                // Example A: Log v1.1+ handshake fields
                info!(
                    plugin_version = %hello.plugin_version,
                    protocol_version = %hello.protocol_version,
                    server_id = %hello.server_id,
                    minecraft_version = %hello.minecraft_version,
                    voice_available = hello.voice_available,
                    server_name = %hello.server_name,
                    daemon_mode = %hello.daemon_mode,
                    "Received Hello handshake"
                );
                
                if hello.voice_available {
                    info!("Voice chat is available - TTS audio will be sent");
                }
            }
            
            Some(ClientMsg::WorldTick(tick)) => {
                debug!(
                    server_tick = tick.server_tick,
                    npcs = tick.npcs.len(),
                    players = tick.nearby_players.len(),
                    "WorldTick received"
                );
                
                // Example D: Mining perception loop
                // Every 100 ticks, send a ScanBlocksAction to look for diamond ore
                if tick.server_tick % 100 == 0 && !tick.npcs.is_empty() {
                    let npc = &tick.npcs[0];
                    let directive_id = next_directive_id();
                    
                    let center = npc.position.as_ref().map(|p| BlockPosition {
                        world: p.world.clone(),
                        x: p.x as i32,
                        y: p.y as i32,
                        z: p.z as i32,
                    });
                    
                    if let Some(center) = center {
                        let scan_action = ActionDirective {
                            directive_id: directive_id.clone(),
                            npc_id: npc.npc_id.clone(),
                            priority: 5,
                            action: Some(Action::ScanBlocks(ScanBlocksAction {
                                center: Some(center),
                                radius: 16,
                                block_types: vec![
                                    "minecraft:diamond_ore".to_string(),
                                    "minecraft:deepslate_diamond_ore".to_string(),
                                ],
                                max_results: 10,
                            })),
                        };
                        
                        let _ = tx.blocking_send(ServerMessage {
                            message: Some(ServerMsg::ActionDirective(scan_action)),
                        });
                        
                        info!(directive_id = %directive_id, npc_id = %npc.npc_id, "Sent ScanBlocksAction");
                    }
                }
                
                // Every 50 ticks, send a move directive
                if tick.server_tick % 50 == 0 && !tick.npcs.is_empty() {
                    let npc = &tick.npcs[0];
                    let directive_id = next_directive_id();
                    
                    let directive = ActionDirective {
                        directive_id: directive_id.clone(),
                        npc_id: npc.npc_id.clone(),
                        priority: 1,
                        action: Some(Action::Move(MoveAction {
                            target: Some(Position {
                                world: "world".to_string(),
                                x: npc.position.as_ref().map(|p| p.x + 5.0).unwrap_or(0.0),
                                y: npc.position.as_ref().map(|p| p.y).unwrap_or(64.0),
                                z: npc.position.as_ref().map(|p| p.z).unwrap_or(0.0),
                                yaw: 0.0,
                                pitch: 0.0,
                            }),
                            speed: 0.5,
                            pathfind: true,
                        })),
                    };
                    
                    let _ = tx.blocking_send(ServerMessage {
                        message: Some(ServerMsg::ActionDirective(directive)),
                    });
                    
                    debug!(directive_id = %directive_id, "Sent MoveAction");
                }
            }
            
            Some(ClientMsg::ChatObservation(chat)) => {
                info!(
                    npc_id = %chat.npc_id,
                    player_name = %chat.player_name,
                    message = %chat.message,
                    "Chat observation received"
                );
                
                // Example E: Send SpeakDirective with correlation fields + audio
                let directive_id = next_directive_id();
                let stream_id = next_stream_id();
                
                // Send SpeakDirective with v1.1+ correlation fields
                let speak = SpeakDirective {
                    npc_id: chat.npc_id.clone(),
                    text: format!("Hello, {}! I'll help you find diamonds.", chat.player_name),
                    emotion: "helpful".to_string(),
                    duration_ms: 3000,
                    // v1.1+ fields for correlation
                    directive_id: directive_id.clone(),
                    voice_id: "en-US-Neural2-D".to_string(), // Example TTS voice
                    volume: 0.8,
                    stream_id: stream_id.clone(), // Must match AudioChunk.stream_id
                };
                
                let _ = tx.blocking_send(ServerMessage {
                    message: Some(ServerMsg::SpeakDirective(speak)),
                });
                
                info!(
                    directive_id = %directive_id,
                    stream_id = %stream_id,
                    "Sent SpeakDirective with audio correlation"
                );
                
                // Send correlated AudioChunks (simulated TTS output)
                for seq in 0..3 {
                    let audio = AudioChunk {
                        npc_id: chat.npc_id.clone(),
                        stream_id: stream_id.clone(), // Matches SpeakDirective.stream_id
                        pcm_data: vec![0u8; 960], // Dummy silence (20ms at 48kHz mono)
                        sequence: seq,
                        is_final: seq == 2,
                        // v1.1+ optional correlation
                        directive_id: directive_id.clone(),
                    };
                    
                    let _ = tx.blocking_send(ServerMessage {
                        message: Some(ServerMsg::AudioChunk(audio)),
                    });
                }
                
                debug!(
                    stream_id = %stream_id,
                    chunks = 3,
                    "Sent AudioChunks with correlation"
                );
            }
            
            Some(ClientMsg::ActionResult(result)) => {
                if result.success {
                    info!(
                        directive_id = %result.directive_id,
                        npc_id = %result.npc_id,
                        "Action completed successfully"
                    );
                    
                    // Handle specific result types
                    match result.result {
                        Some(ActionResultType::ScanBlocksResult(scan)) => {
                            // Example D: Process mining scan results
                            info!(
                                matches = scan.matches.len(),
                                "ScanBlocksResult: found ore blocks"
                            );
                            
                            // If we found ore, send a BreakBlockAction for the first one
                            if let Some(first_match) = scan.matches.first() {
                                let directive_id = next_directive_id();
                                
                                let break_action = ActionDirective {
                                    directive_id: directive_id.clone(),
                                    npc_id: result.npc_id.clone(),
                                    priority: 10, // High priority
                                    action: Some(Action::BreakBlock(BreakBlockAction {
                                        position: first_match.position.clone(),
                                    })),
                                };
                                
                                let _ = tx.blocking_send(ServerMessage {
                                    message: Some(ServerMsg::ActionDirective(break_action)),
                                });
                                
                                info!(
                                    directive_id = %directive_id,
                                    block_type = %first_match.block_type,
                                    "Sent BreakBlockAction for found ore"
                                );
                            }
                        }
                        
                        Some(ActionResultType::BreakBlockResult(break_result)) => {
                            // After breaking blocks, deposit to chest
                            if !break_result.items_dropped.is_empty() {
                                info!(
                                    items = break_result.items_dropped.len(),
                                    "BreakBlockResult: picked up items"
                                );
                                
                                // Send DepositToChestAction
                                let directive_id = next_directive_id();
                                
                                let deposit_action = ActionDirective {
                                    directive_id: directive_id.clone(),
                                    npc_id: result.npc_id.clone(),
                                    priority: 5,
                                    action: Some(Action::DepositToChest(DepositToChestAction {
                                        chest_position: Some(BlockPosition {
                                            world: "world".to_string(),
                                            x: 100,
                                            y: 64,
                                            z: -200,
                                        }),
                                        item_types: vec!["minecraft:diamond".to_string()],
                                        max_items: 64,
                                    })),
                                };
                                
                                let _ = tx.blocking_send(ServerMessage {
                                    message: Some(ServerMsg::ActionDirective(deposit_action)),
                                });
                                
                                info!(directive_id = %directive_id, "Sent DepositToChestAction");
                            }
                        }
                        
                        Some(ActionResultType::DepositToChestResult(deposit)) => {
                            info!(
                                deposited = deposit.deposited.len(),
                                "DepositToChestResult: items stored"
                            );
                        }
                        
                        Some(ActionResultType::MoveResult(move_result)) => {
                            debug!(
                                reached = move_result.reached_destination,
                                "MoveResult received"
                            );
                        }
                        
                        _ => {}
                    }
                } else {
                    // Example: Error case handling
                    warn!(
                        directive_id = %result.directive_id,
                        npc_id = %result.npc_id,
                        error = %result.error_message,
                        "Action failed"
                    );
                    
                    // Could retry, fall back, or notify player
                }
            }
            
            Some(ClientMsg::EventObservation(event)) => {
                debug!(
                    npc_id = %event.npc_id,
                    event_type = ?event.event_type,
                    "Event observation received"
                );
            }
            
            Some(ClientMsg::VoicePcmFrame(frame)) => {
                debug!(
                    npc_id = %frame.npc_id,
                    player_uuid = %frame.player_uuid,
                    sequence = frame.sequence,
                    bytes = frame.pcm_data.len(),
                    sample_rate = frame.sample_rate_hz,
                    format = ?frame.format,
                    "Voice frame received"
                );
                // In production: buffer audio, run ASR, process with LLM
            }
            
            None => {
                warn!("Received empty client message");
            }
        }
    }
}

#[tonic::async_trait]
impl NpcSocietyService for ExampleNpcSocietyService {
    type ConnectStream = Pin<Box<dyn Stream<Item = Result<ServerMessage, Status>> + Send>>;

    async fn connect(
        &self,
        request: Request<Streaming<ClientMessage>>,
    ) -> Result<Response<Self::ConnectStream>, Status> {
        let peer_addr = request
            .remote_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        info!(peer = %peer_addr, "New plugin connection");

        let mut in_stream = request.into_inner();
        
        // Channel for sending responses back to client
        let (tx, rx) = mpsc::channel(128);
        
        // Spawn task to process incoming messages
        let service = Arc::new(ExampleNpcSocietyService);
        let tx_clone = tx.clone();
        
        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(msg) => {
                        service.handle_client_message(msg, &tx_clone);
                    }
                    Err(e) => {
                        error!(error = %e, "Stream error");
                        break;
                    }
                }
            }
            info!(peer = %peer_addr, "Connection closed");
        });

        let out_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(out_stream.map(Ok)) as Self::ConnectStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50051);
    
    let addr = format!("0.0.0.0:{}", port).parse()?;
    let service = ExampleNpcSocietyService::default();

    info!("=== NPC Society Protocol Example Server ===");
    info!(address = %addr, "gRPC server starting");
    info!("Demonstrating: mining loop, audio correlation, error handling");

    Server::builder()
        .add_service(NpcSocietyServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
