# State - MTGO Replay Capture

**Last Updated:** 2026-01-30
**Current Phase:** Not Started (Pre-Planning)

---

## Project Reference

**Core Value:**
Capture MTGO replays with automatic sideboard extraction, enabling personal analysis and easy file-based sharing.

**Current Focus:**
Initializing project planning based on research findings. Next step is Phase 1: Capture Infrastructure & Proof of Concept.

**Platform:**
Windows-only desktop application (MTGO is Windows-only).

**Tech Stack:**
- Rust 1.80+ (core language)
- Tauri 2.0 (desktop framework)
- Tokio 1.35+ (async runtime)
- WinDivert 1.4+ (Windows packet capture)
- Serde + MessagePack/Bincode (serialization)

---

## Current Position

**Active Phase:** None (roadmap created, planning pending)
**Active Plan:** None
**Status:** Ready for Planning
**Progress Bar:** 0% (0/7 phases complete)

---

## Performance Metrics

*N/A - Project not started*

---

## Accumulated Context

### Key Decisions Made

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Network traffic hook over screen recording | Cleaner data, more reliable extraction, smaller file size | Using WinDivert for packet capture on Windows |
| File-based sharing before web platform | Simpler v1, keeps focus on core capture functionality | Local-only storage, export to JSON/CSV for sharing |
| Rust + Tauri over Electron | 10x smaller bundles, native performance, memory safety for binary parsing | Rust backend with web UI |
| MessagePack/Bincode for replay format | Compact binary format, schema-flexible for unknown protocol exploration | Versioned replay file format with compression |
| WinDivert over libpcap for Windows | Captures localhost traffic better than libpcap (MTGO may use loopback) | Required for Windows capture |

### Research Summary

- **MTGO Protocol:** Undocumented and proprietary. Requires reverse-engineering with Wireshark. Changes frequently without version handshake.
- **Traffic Capture:** WinDivert required on Windows to capture localhost traffic. Administrator privileges needed. Driver installation may be required.
- **Sideboard Detection:** Key competitive differentiator. May not generate explicit network messages — may require correlating deck lists between games.
- **Replay File Format:** Must be versioned from day one to avoid brittleness. Include MTGO client version in header. Target < 10MB per game session.

### Critical Pitfalls Identified

1. **Loopback Traffic Capture Blindness:** MTGO may use localhost (127.0.0.1). libpcap misses loopback traffic. WinDivert required. Test early with actual MTGO traffic.
2. **Protocol Version Fragility:** MTGO protocol changes frequently without warning. Design version-aware parsers with tolerance for unknown fields. Store MTGO client version in replay header.
3. **Platform-Specific Capture Failures:** Packet capture requires admin privileges, kernel-level drivers, and may be blocked by antivirus. Test on Windows 10/11 with various AV.
4. **Game State Reconstruction Ambiguity:** Packets may be out-of-order or lost. Implement packet reordering and loss handling. Validate against invariants.
5. **File Format Brittleness:** Binary formats without versioning break backward compatibility. Design versioned format from day one with magic bytes, version field, self-describing sections.

### Architecture Approach

- **Layered Architecture:** Clear boundaries between capture, protocol, and application logic.
- **Event-Driven:** Components communicate via events for loose coupling and async processing.
- **State Machine:** Protocol decoder tracks connection state and game phases (mulligan, draw, combat, etc.).
- **Producer-Consumer:** Packet capture produces, protocol decoder consumes with bounded buffer and backpressure.

### Phase Dependencies

```
Phase 1: Capture Infrastructure (WinDivert, BPF filters)
    ↓
Phase 2: Protocol Reverse Engineering (MTGO protocol decoder, replay format)
    ↓
Phase 3: Game State Reconstruction (state machine, card resolution)
    ↓
Phase 4: Sideboard Detection (deck comparison, diff viewer)
    ↓
Phase 5: Replay Playback & Analytics (viewer UI, statistics dashboard)
    ↓
Phase 6: Performance Optimization (streaming to disk, compression)
    ↓
Phase 7: Advanced Features (MTGOSDK integration, advanced analytics)
```

---

## Todos

*No active todos. Next step: `/gsd-plan-phase 1`*

---

## Blockers

*No blockers.*

---

## Session Continuity

*Project initialized. Roadmap created with 7 phases. Ready to begin Phase 1: Capture Infrastructure & Proof of Concept.*

---

## Notes

### Research Sources

- Tauri 2.0 official site (v2.tauri.app)
- WinDivert official site (reqrypt.org/windivert)
- Wireshark 4.6.3 official site (wireshark.org)
- MTGO-Tracker Wiki (github.com/cderickson/MTGO-Tracker/wiki)
- MTGOSDK README (github.com/videre-project/MTGOSDK)
- Videre Tracker README (github.com/videre-project/Tracker)
- MessagePack official site (msgpack.org)

### Confidence Levels

- **Stack:** MEDIUM - Core stack (Rust, Tauri, Tokio, MessagePack) verified. MTGO-specific tools (WinDivert) verified but protocol-specific tools may need adjustment.
- **Features:** HIGH - Feature landscape well-documented from competitor analysis. Clear distinction between table stakes, differentiators, and anti-features.
- **Architecture:** MEDIUM - Standard layered architecture pattern for network capture is well-established. MTGO-specific protocol handling may require adjustments.
- **Pitfalls:** MEDIUM - Pitfalls identified from general reverse-engineering and packet capture principles. Actual MTGO protocol reverse-engineering in Phase 1 will validate severity.

### Gaps to Address

- **MTGO protocol documentation:** Undocumented and proprietary. Reverse-engineering in Phase 1 will reveal actual message formats, field ordering, and state representation.
- **MTGO traffic paths:** Unknown if MTGO uses localhost for some connections. Must verify in Phase 1 proof of concept.
- **Sideboard protocol representation:** Unknown how sideboarding manifests in network packets. Requires dedicated reverse-engineering in Phase 4.
- **Card ID format in MTGO:** Unclear how MTGO references cards. Phase 3 must identify card resolution strategy.
- **Replay file format trade-offs:** MessagePack vs Bincode vs custom binary format need validation with real replay data sizes.

### Next Actions

1. `/gsd-plan-phase 1` - Begin detailed planning for Phase 1: Capture Infrastructure & Proof of Concept
2. During Phase 1 planning, use `/gsd-research-phase` to dive deeper into WinDivert integration and MTGO traffic path verification
3. Set up Rust + Tauri project structure per ARCHITECTURE.md recommendations
4. Verify WinDivert driver availability and installation requirements for Windows
5. Test packet capture with actual MTGO client running to verify traffic paths and loopback requirements
