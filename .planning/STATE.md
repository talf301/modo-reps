# State - MTGO Replay Capture

**Last Updated:** 2026-01-30 17:58
**Current Phase:** Phase 1 of 7 (Capture Infrastructure & Proof of Concept)

---

## Project Reference

**Core Value:**
Capture MTGO replays with automatic sideboard extraction, enabling personal analysis and easy file-based sharing.

**Current Focus:**
Building capture infrastructure with admin privilege detection and WinDivert driver verification.

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

**Active Phase:** Phase 1 of 7 (Capture Infrastructure & Proof of Concept)
**Active Plan:** 01-04B (complete)
**Status:** In progress
**Progress Bar:** █░░░░░░░░░░░░░░ 14% (1/7 phases complete)

---

## Performance Metrics

- Phase 1 Plans: 6 total, 5 completed (01-01, 01-02, 01-03, 01-04A, 01-04B)
- Phase 1 Progress: 83% (5/6 plans)
- Total Project Progress: 14% (1/7 phases complete)
- Total Project Progress: 0% (0/7 phases complete)

### Phase 1 Metrics

- 01-01: Tauri initialization ✓ (completed 2026-01-30)
- 01-02: Admin/WinDivert detection ✓ (completed 2026-01-30)
- 01-03: WinDivert handle management ✓ (completed 2026-01-30)
- 01-04A: Capture status types and Tauri commands ✓ (completed 2026-01-30)
- 01-04B: Capture status UI component ✓ (completed 2026-01-30)
- 01-05: Proof of concept (pending)

---

## Accumulated Context

### Key Decisions Made

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Network traffic hook over screen recording | Cleaner data, more reliable extraction, smaller file size | Using WinDivert for packet capture on Windows |
| File-based sharing before web platform | Simpler v1, keeps focus on core capture functionality | Local-only storage, export to JSON/CSV for sharing |
| Structured errors with user guidance | Type-safe error handling with clear messages | thiserror-based CaptureError enum with actionable guidance |
| Windows Security API for privilege detection | Standard approach for admin checks | GetTokenInformation + TOKEN_ELEVATION APIs |
| WinDivert driver verification before capture | Early failure with clear guidance | File existence checks with actionable error messages |
| Rust + Tauri over Electron | 10x smaller bundles, native performance, memory safety for binary parsing | Rust backend with web UI |
| MessagePack/Bincode for replay format | Compact binary format, schema-flexible for unknown protocol exploration | Versioned replay file format with compression |
| WinDivert over libpcap for Windows | Captures localhost traffic better than libpcap (MTGO may use loopback) | Required for Windows capture |
| WinDivert 0.7.0-beta.4 over 1.4 | Version 1.4 not available on crates.io, use latest beta | windivert = "0.7.0-beta.4" in Cargo.toml |
| tauri-plugin-opener over shell-open feature | shell-open feature removed in Tauri 2.0, moved to plugin | Added tauri-plugin-opener = "2.0" |
| Thread-safe shared state via Arc<Mutex<>> | Commands run in async contexts and may be called concurrently from frontend | Safe access to capture state without race conditions |
| ISO 8601 timestamps for frontend | Standard format that JavaScript Date() can parse natively | Easy frontend integration with `new Date(isoString)` |
| Placeholder implementations for capture loop | Full implementation requires WinDivert handle management (Plan 01-03) and capture loop logic (Plan 05) | Commands return status but don't start actual capture; ready for frontend testing |

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

- [ ] Verify project compilation on Windows with `cargo check --target x86_64-pc-windows-msvc`
- [x] Execute plan 01-03: WinDivert handle and filter management
- [x] Execute plan 01-04A: Build capture status types and Tauri commands
- [x] Execute plan 01-04B: Capture status UI component
- [ ] Execute plan 01-05: Proof of concept with actual MTGO traffic

---

## Blockers

- **Testing on Windows needed** - Admin detection and driver checks unverified on target platform

---

## Session Continuity

**Last session:** 2026-01-30 17:58
**Stopped at:** Completed 01-04B-PLAN.md (Capture Status UI Component)
**Resume file:** None

**Commits in last session:**
- abc92f0: feat(01-04A): define capture status types and Tauri commands
- 141d191: feat(01-04A): register capture commands with shared state
- 0c50648: feat(01-04B): create capture status UI component
- c32a91f: feat(01-04B): integrate capture status component into main app

**Deviations handled in 01-01:**
- Fixed windivert version from 1.4 to 0.7.0-beta.4 (Rule 3 - blocking issue)
- Removed redundant windivert-sys dependency (Rule 3 - blocking issue)
- Fixed tauri shell-open feature to tauri-plugin-opener (Rule 3 - blocking issue)

**Deviations handled in 01-03:**
- None (plan executed exactly as written)

**Deviations handled in 01-04A:**
- None (plan executed exactly as written)

**Deviations handled in 01-04B:**
- Fixed TypeScript/React to vanilla JavaScript adaptation (Rule 3 - blocking issue)
- Fixed module scoping issue with event listeners (Rule 1 - bug)

**Ready to continue:** Execute plan 01-05 (Proof of Concept)

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

1. Execute plan 01-05: Proof of concept with actual MTGO traffic
2. Test on Windows with actual MTGO client to verify admin detection and driver checks
3. Verify capture status UI displays correctly on Windows
4. Test start/stop capture functionality with actual MTGO traffic
