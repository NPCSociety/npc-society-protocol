//! Minimal integration test that:
//! - sends Hello
//! - receives SpeakDirective
//! - sends ActionResult

// Include the generated proto code
pub mod npc_society {
    pub mod v1 {
        tonic::include_proto!("npc_society.v1");
    }
}

use npc_society::v1::{
    client_message::Message as ClientMsg,
    ActionResult, ClientMessage, Hello,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hello_with_v11_fields() {
        // Test Hello with v1.1+ fields
        let hello = Hello {
            plugin_version: "1.1.0".to_string(),
            protocol_version: "1".to_string(),
            server_id: "test".to_string(),
            minecraft_version: "1.20.4".to_string(),
            voice_available: true,
            server_name: "Test Server".to_string(),
            daemon_mode: "external".to_string(),
        };

        let msg = ClientMessage {
            message: Some(ClientMsg::Hello(hello)),
        };

        // Verify serialization works
        use prost::Message;
        let bytes = msg.encode_to_vec();
        let decoded = ClientMessage::decode(&bytes[..]).unwrap();
        
        match decoded.message {
            Some(ClientMsg::Hello(h)) => {
                assert!(h.voice_available);
                assert_eq!(h.server_name, "Test Server");
                assert_eq!(h.daemon_mode, "external");
            }
            _ => panic!("Decoding failed"),
        }
        
        println!("✓ Hello with v1.1+ fields serializes correctly");
    }

    #[tokio::test]
    async fn test_speak_directive_with_correlation() {
        use npc_society::v1::{server_message::Message as ServerMsg, ServerMessage, SpeakDirective};
        
        let speak = SpeakDirective {
            npc_id: "test_npc".to_string(),
            text: "Hello!".to_string(),
            emotion: "friendly".to_string(),
            duration_ms: 2000,
            directive_id: "speak-1".to_string(),
            voice_id: "en-US-Neural2-D".to_string(),
            volume: 0.8,
            stream_id: "stream-1".to_string(),
        };
        
        let msg = ServerMessage {
            message: Some(ServerMsg::SpeakDirective(speak)),
        };
        
        use prost::Message;
        let bytes = msg.encode_to_vec();
        let decoded = ServerMessage::decode(&bytes[..]).unwrap();
        
        match decoded.message {
            Some(ServerMsg::SpeakDirective(s)) => {
                assert_eq!(s.directive_id, "speak-1");
                assert_eq!(s.stream_id, "stream-1");
                assert_eq!(s.voice_id, "en-US-Neural2-D");
                assert!((s.volume - 0.8).abs() < 0.01);
            }
            _ => panic!("Decoding failed"),
        }
        
        println!("✓ SpeakDirective with correlation fields serializes correctly");
    }

    #[tokio::test]
    async fn test_action_result_with_scan_blocks() {
        use npc_society::v1::{ScanBlocksResult, BlockMatch, BlockPosition};

        let result = ActionResult {
            directive_id: "scan-1".to_string(),
            npc_id: "miner".to_string(),
            success: true,
            error_message: String::new(),
            result: Some(npc_society::v1::action_result::Result::ScanBlocksResult(
                ScanBlocksResult {
                    matches: vec![
                        BlockMatch {
                            position: Some(BlockPosition {
                                world: "world".to_string(),
                                x: 10,
                                y: 20,
                                z: 30,
                            }),
                            block_type: "minecraft:diamond_ore".to_string(),
                        },
                    ],
                },
            )),
        };

        // Verify serialization
        use prost::Message;
        let bytes = result.encode_to_vec();
        let decoded = ActionResult::decode(&bytes[..]).unwrap();
        
        assert!(decoded.success);
        match decoded.result {
            Some(npc_society::v1::action_result::Result::ScanBlocksResult(scan)) => {
                assert_eq!(scan.matches.len(), 1);
                assert_eq!(scan.matches[0].block_type, "minecraft:diamond_ore");
            }
            _ => panic!("Expected ScanBlocksResult"),
        }
        
        println!("✓ ActionResult with ScanBlocksResult serializes correctly");
    }
    
    #[tokio::test]
    async fn test_audio_chunk_with_directive_id() {
        use npc_society::v1::{server_message::Message as ServerMsg, ServerMessage, AudioChunk};
        
        let audio = AudioChunk {
            npc_id: "test_npc".to_string(),
            stream_id: "stream-1".to_string(),
            pcm_data: vec![0u8; 960],
            sequence: 0,
            is_final: true,
            directive_id: "speak-1".to_string(),
        };
        
        let msg = ServerMessage {
            message: Some(ServerMsg::AudioChunk(audio)),
        };
        
        use prost::Message;
        let bytes = msg.encode_to_vec();
        let decoded = ServerMessage::decode(&bytes[..]).unwrap();
        
        match decoded.message {
            Some(ServerMsg::AudioChunk(a)) => {
                assert_eq!(a.stream_id, "stream-1");
                assert_eq!(a.directive_id, "speak-1");
                assert!(a.is_final);
            }
            _ => panic!("Decoding failed"),
        }
        
        println!("✓ AudioChunk with directive_id serializes correctly");
    }
    
    #[tokio::test]
    async fn test_voice_pcm_frame_with_format() {
        use npc_society::v1::{VoicePcmFrame, PcmFormat};
        
        let frame = VoicePcmFrame {
            npc_id: "test_npc".to_string(),
            player_uuid: "player-1".to_string(),
            pcm_data: vec![0u8; 1920],
            sequence: 0,
            timestamp_ms: 1234567890,
            sample_rate_hz: 48000,
            format: PcmFormat::S16le as i32,
        };
        
        let msg = ClientMessage {
            message: Some(ClientMsg::VoicePcmFrame(frame)),
        };
        
        use prost::Message;
        let bytes = msg.encode_to_vec();
        let decoded = ClientMessage::decode(&bytes[..]).unwrap();
        
        match decoded.message {
            Some(ClientMsg::VoicePcmFrame(f)) => {
                assert_eq!(f.sample_rate_hz, 48000);
                assert_eq!(f.format, PcmFormat::S16le as i32);
            }
            _ => panic!("Decoding failed"),
        }
        
        println!("✓ VoicePcmFrame with format serializes correctly");
    }
}
