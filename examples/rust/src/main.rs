//! Minimal example demonstrating how to implement the NPC Society daemon server.
//!
//! This is a reference implementation showing how to handle the bidirectional
//! Connect() stream from the Paper plugin.

use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{transport::Server, Request, Response, Status, Streaming};
use tracing::{info, warn, error, Level};

// Include the generated proto code
// In production, this would come from the buf-generated crate
pub mod npc_society {
    pub mod v1 {
        tonic::include_proto!("npc_society.v1");
    }
}

use npc_society::v1::{
    npc_society_service_server::{NpcSocietyService, NpcSocietyServiceServer},
    ActionDirective, AudioChunk, ClientMessage, ServerMessage, SpeakDirective,
    client_message::Message as ClientMsg,
    server_message::Message as ServerMsg,
    MoveAction, Position,
};

/// Example implementation of the NPC Society service.
#[derive(Debug, Default)]
pub struct ExampleNpcSocietyService;

impl ExampleNpcSocietyService {
    /// Process an incoming client message and optionally return a response.
    fn handle_client_message(&self, msg: ClientMessage) -> Option<ServerMessage> {
        match msg.message {
            Some(ClientMsg::Hello(hello)) => {
                info!(
                    "Received Hello: plugin={}, protocol={}, server={}, mc={}",
                    hello.plugin_version,
                    hello.protocol_version,
                    hello.server_id,
                    hello.minecraft_version
                );
                // Could send a welcome action directive
                None
            }
            Some(ClientMsg::WorldTick(tick)) => {
                info!(
                    "WorldTick #{}: {} NPCs, {} players nearby",
                    tick.server_tick,
                    tick.npcs.len(),
                    tick.nearby_players.len()
                );
                
                // Example: Send a move directive every 50 ticks
                if tick.server_tick % 50 == 0 && !tick.npcs.is_empty() {
                    let npc = &tick.npcs[0];
                    let directive = ActionDirective {
                        directive_id: format!("move-{}", tick.server_tick),
                        npc_id: npc.npc_id.clone(),
                        priority: 1,
                        action: Some(npc_society::v1::action_directive::Action::Move(MoveAction {
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
                    return Some(ServerMessage {
                        message: Some(ServerMsg::ActionDirective(directive)),
                    });
                }
                None
            }
            Some(ClientMsg::ChatObservation(chat)) => {
                info!(
                    "Chat near NPC {}: {} said '{}'",
                    chat.npc_id, chat.player_name, chat.message
                );
                
                // Example: Respond with a speak directive
                let speak = SpeakDirective {
                    npc_id: chat.npc_id.clone(),
                    text: format!("Hello, {}! Nice to meet you.", chat.player_name),
                    emotion: "friendly".to_string(),
                    duration_ms: 3000,
                };
                Some(ServerMessage {
                    message: Some(ServerMsg::SpeakDirective(speak)),
                })
            }
            Some(ClientMsg::EventObservation(event)) => {
                info!(
                    "Event observed by NPC {}: {:?}",
                    event.npc_id, event.event_type
                );
                None
            }
            Some(ClientMsg::VoicePcmFrame(frame)) => {
                info!(
                    "Voice frame for NPC {} from player {}: {} bytes, seq={}",
                    frame.npc_id,
                    frame.player_uuid,
                    frame.pcm_data.len(),
                    frame.sequence
                );
                
                // In production: buffer audio, run ASR, process with LLM, generate TTS
                // Example: Echo back a dummy audio chunk
                if frame.sequence % 10 == 0 {
                    let audio = AudioChunk {
                        npc_id: frame.npc_id.clone(),
                        stream_id: format!("response-{}", frame.sequence),
                        pcm_data: vec![0u8; 960], // Dummy silence
                        sequence: 0,
                        is_final: true,
                    };
                    return Some(ServerMessage {
                        message: Some(ServerMsg::AudioChunk(audio)),
                    });
                }
                None
            }
            Some(ClientMsg::ActionResult(result)) => {
                info!(
                    "Action result for directive {}: success={}",
                    result.directive_id, result.success
                );
                if !result.success {
                    warn!("Action failed: {}", result.error_message);
                }
                None
            }
            None => {
                warn!("Received empty client message");
                None
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
        
        info!("New connection from: {}", peer_addr);

        let mut in_stream = request.into_inner();
        
        // Channel for sending responses back to client
        let (tx, rx) = mpsc::channel(128);
        
        // Spawn task to process incoming messages
        let service = Arc::new(ExampleNpcSocietyService);
        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(msg) => {
                        if let Some(response) = service.handle_client_message(msg) {
                            if tx.send(Ok(response)).await.is_err() {
                                error!("Failed to send response - client disconnected");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Stream error: {}", e);
                        break;
                    }
                }
            }
            info!("Connection closed: {}", peer_addr);
        });

        let out_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(out_stream) as Self::ConnectStream))
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

    info!("NPC Society daemon starting on {}", addr);

    Server::builder()
        .add_service(NpcSocietyServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

