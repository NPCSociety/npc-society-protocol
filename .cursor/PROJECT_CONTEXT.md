# npc-society-protocol — Project Context

## Purpose
Defines the stable contract between the Paper plugin and the Rust daemon.
This repo is the “spine” of the system; avoid breaking changes.

## Non-negotiables
- Versioned protocol (v1). Additive changes only.
- Payloads must be compact; frequent messages must be cheap.
- No Minecraft implementation logic here.

## Core flows to support
Plugin -> Daemon:
- Hello / Capabilities (voice available, server_id)
- WorldTick snapshots (players + nearby NPCs)
- ChatObservation (player->npc, npc->npc barks)
- EventObservation (damage, death, block changes near NPC, etc.)
- VoicePcmFrame (48k PCM from voice chat)
- ActionResult (ack execution)

Daemon -> Plugin:
- ActionDirective (primitive action + args_json + deadline)
- SpeakDirective (text + voice_id)
- AudioChunk (PCM chunks for NPC playback)

## Identity
- player_uuid (Minecraft UUID)
- npc_id (stable config id) + entity_uuid (runtime entity UUID)
