# Requirements - MTGO Replay Capture

**Status:** Active
**Last Updated:** 2026-01-30

## Core Value

Capture MTGO replays with automatic sideboard extraction, enabling personal analysis and easy file-based sharing.

## v1 Requirements (Launch)

### Capture Infrastructure (CAPT)

- **CAPT-001**: Capture MTGO network traffic from local Windows machine
  - Use WinDivert for packet capture (captures localhost traffic)
  - Filter MTGO server traffic using BPF filters
  - Stream packets to protocol decoder

### Protocol & Parsing (PROT)

- **PROT-001**: Parse MTGO network protocol and reconstruct game state
  - Reverse-engineer MTGO proprietary protocol (undocumented)
  - Decode protocol messages into structured events
  - Handle packet reordering and loss
  - Support protocol version changes

### Replay Storage (REPL)

- **REPL-001**: Store replays in compact, shareable file format
  - Design versioned binary format (MessagePack/Bincode)
  - Include MTGO client version in header
  - Support backward compatibility
  - Target < 10MB per game session

### Replay Viewing (VIEW)

- **VIEW-001**: Display replays locally with built-in viewer
  - Turn-by-turn replay playback controls
  - Play, pause, step forward/backward
  - Visual board state representation

### Sideboard Extraction (SIDE)

- **SIDE-001**: Extract sideboard data (cards moved in/out between games)
  - Compare deck lists between consecutive games
  - Highlight cards in/out
  - Correlate deck loaded events with game starts
  - Provide UI for manual sideboard correction

### Export & Sharing (XPORT)

- **XPORT-001**: Export replays as standalone files for sharing
  - Save to custom binary format
  - Export to JSON/CSV for compatibility
  - File size optimization for easy sharing

### Analytics & Stats (STAT)

- **STAT-001**: Track match metadata (format, date, opponent identification, duration)
- **STAT-002**: Calculate win/loss statistics
- **STAT-003**: Generate play-by-play breakdown (what cards were played when)
- **STAT-004**: Provide search and filtering (find replays by date, format, opponent)
- **STAT-005**: Extract deck lists (maindeck, sideboard)

### Performance & Stability (PERF)

- **PERF-001**: Stream packets to disk immediately (no in-memory storage leaks)
- **PERF-002**: Support long capture sessions (8+ hours) without crashes
- **PERF-003**: Compress replay files for storage efficiency
- **PERF-004**: Handle packet bursts with bounded message queues

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CAPT-001 | Phase 1 | Pending |
| PROT-001 | Phase 2 | Pending |
| REPL-001 | Phase 2 | Pending |
| SIDE-001 | Phase 4 | Pending |
| VIEW-001 | Phase 5 | Pending |
| XPORT-001 | Phase 5 | Pending |
| STAT-001 | Phase 3 | Pending |
| STAT-002 | Phase 5 | Pending |
| STAT-003 | Phase 3 | Pending |
| STAT-004 | Phase 5 | Pending |
| STAT-005 | Phase 3 | Pending |
| PERF-001 | Phase 6 | Pending |
| PERF-002 | Phase 6 | Pending |
| PERF-003 | Phase 6 | Pending |
| PERF-004 | Phase 1 | Pending |

## Requirements Count

- **Total v1 Requirements:** 14
- **Categories:** 7 (Capture, Protocol, Replay, Viewing, Sideboard, Export, Analytics, Performance)

## Out of Scope

- Web platform for viewing replays — defer to v2
- Screen recording / video capture approach — network hook chosen
- MTGO Arena support — different system, different protocol
- Real-time streaming or live viewing — post-match capture only
- Real-time overlay tracking via MTGOSDK — future consideration (Phase 7)
- Cloud-based replay storage — local-only for v1
