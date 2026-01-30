# Roadmap - MTGO Replay Capture

**Last Updated:** 2026-01-30
**Depth:** Standard (5-8 phases)
**Coverage:** 14/14 requirements mapped ✓

## Overview

This roadmap guides the development of MTGO Replay Capture, a Windows-only desktop application that intercepts Magic: The Gathering Online's proprietary network protocol to capture game replays and extract sideboard data. The project uses Rust 1.80+ with Tauri 2.0 for the desktop framework, WinDivert for Windows packet capture (required for localhost traffic), and MessagePack/Bincode for compact replay storage.

The roadmap follows a layered architecture approach: capture infrastructure first (must verify traffic paths), protocol parsing second (foundation for everything), game state reconstruction third (enables replay features), then sideboard detection and playback/analytics. This ordering addresses critical pitfalls early (loopback capture, protocol fragility) before investing heavily in features.

---

## Phase 1: Capture Infrastructure & Proof of Concept

**Goal:** Establish working packet capture on Windows with MTGO traffic visible and platform-specific issues verified.

**Dependencies:** None

**Requirements:**
- CAPT-001: Capture MTGO network traffic from local Windows machine
- PERF-004: Handle packet bursts with bounded message queues

**Success Criteria:**
1. User can start/stop packet capture from the desktop application
2. Capture shows MTGO game-related traffic (verified with actual MTGO client running)
3. Application detects and handles missing administrator privileges with clear error messages
4. WinDivert driver installation is verified (or bundled with installer)
5. BPF filter successfully filters MTGO server traffic, reducing captured packets to < 10MB/hour

**Risk Flags:**
- 🔴 **HIGH:** Loopback traffic capture blindness — MTGO may use localhost (127.0.0.1) for some connections. WinDivert required, libpcap misses loopback. Test early with actual MTGO traffic.
- 🟡 **MEDIUM:** Platform-specific capture failures — Packet capture requires admin privileges, kernel-level drivers, and may be blocked by antivirus. Test on Windows 10/11 with various AV configurations.

**Deliverables:**
- Tauri 2.0 desktop application skeleton
- WinDivert integration with windivert-sys crate
- BPF filter engine for MTGO traffic
- Administrator privilege detection and elevation
- Basic capture status UI (packet count, data rate, last packet time)

**Avoids:** Loopback traffic capture blindness, platform-specific capture failures

**Plans:** 5 plans in 3 waves

**Plan List:**
- [ ] 01-01-PLAN.md — Initialize Tauri 2.0 project with required dependencies (windivert, tokio, windows, thiserror, tracing, serde)
- [ ] 01-02-PLAN.md — Implement administrator privilege detection with structured error types
- [ ] 01-03-PLAN.md — Create WinDivert handle and filter management module
- [ ] 01-04-PLAN.md — Build basic capture status UI with Tauri command integration
- [ ] 01-05-PLAN.md — Implement async packet capture loop with bounded MPSC channels and real-time status updates (checkpoint:human-verify)

---

## Phase 2: Protocol Reverse Engineering & Replay Format Design

**Goal:** Reverse-engineer MTGO's proprietary protocol and design a versioned, extensible replay file format.

**Dependencies:** Phase 1 (Must capture MTGO traffic to analyze protocol)

**Requirements:**
- PROT-001: Parse MTGO network protocol and reconstruct game state
- REPL-001: Store replays in compact, shareable file format

**Success Criteria:**
1. Protocol decoder identifies and parses at least 10 distinct MTGO message types from captured traffic
2. Parser tolerates unknown fields without crashing (logs unknown message types)
3. Replay file format includes version field, MTGO client version, and timestamp
4. Replay serializer/deserializer successfully saves and loads basic replay data
5. Bounded message queue prevents memory exhaustion during packet bursts

**Risk Flags:**
- 🔴 **HIGH:** Protocol version fragility — MTGO protocol is undocumented and changes frequently without warning. Design version-aware parsers with tolerance for unknown fields. Store MTGO client version in replay header.
- 🔴 **HIGH:** File format brittleness — Binary formats without versioning break backward compatibility when fields added/removed. Design versioned format from day one with magic bytes, version field, self-describing sections.

**Deliverables:**
- MTGO protocol decoder with state machine for message types
- Message type definitions from reverse-engineering (documented in resources/protocols/)
- Version detection strategy (extract from MTGO client or handshake)
- Bounded message queue with backpressure handling
- Versioned replay file format specification
- Replay serializer/deserializer (MessagePack/Bincode)
- Protocol fuzzing tests (simulate unknown packets)

**Uses:** Tokio async runtime, Serde serialization, WinDivert from Phase 1

**Implements:** Protocol Layer, Message Queue, Replay Manager (ARCHITECTURE)

---

## Phase 3: Game State Reconstruction & Card Resolution

**Goal:** Transform raw protocol messages into reconstructed game state (board, hands, libraries, life totals) with card resolution strategy.

**Dependencies:** Phase 2 (Must decode protocol messages to reconstruct state)

**Requirements:**
- PROT-001: Parse MTGO network protocol and reconstruct game state
- STAT-001: Track match metadata (format, date, opponent identification, duration)
- STAT-003: Generate play-by-play breakdown (what cards were played when)
- STAT-005: Extract deck lists (maindeck, sideboard)

**Success Criteria:**
1. Reconstructed game state shows correct player life totals, board contents, and hand counts
2. Play-by-play breakdown lists card plays, land drops, and combat steps in chronological order
3. Deck list extraction shows maindeck and sideboard cards (stored as card IDs, not full card data)
4. Match metadata captured includes format, timestamp, player names, and duration
5. State reconstruction validates against invariants (e.g., deck size constant, no negative life)

**Risk Flags:**
- 🟡 **MEDIUM:** Game state reconstruction ambiguity — Network packets only show server-sent data, may be out-of-order or lost. Implement packet reordering and loss handling. Validate reconstructed state against invariants.
- 🟡 **MEDIUM:** Card database coupling — Don't embed full card data in replays. Store card IDs and resolve at load time from external database (Scryfall or bundled).

**Deliverables:**
- Game state machine (maintains board, hands, libraries, life totals)
- Card action extraction (card played, resolved, moved to zones)
- Packet reordering and loss handling (sequence tracking)
- State validation invariants (deck size, life bounds)
- Card ID storage strategy (set code + collector number)
- External card database resolution (Scryfall API or bundled DB)
- Match metadata extraction (format, opponent, date, duration)

**Addresses:** Game state reconstruction, deck list extraction, play-by-play breakdown

**Avoids:** Game state reconstruction ambiguity, card database coupling

---

## Phase 4: Sideboard Detection & Extraction

**Goal:** Automatically detect and extract sideboard changes between games (key competitive differentiator).

**Dependencies:** Phase 3 (Must have deck lists and game boundary detection)

**Requirements:**
- SIDE-001: Extract sideboard data (cards moved in/out between games)

**Success Criteria:**
1. Sideboard detection identifies cards moved between maindeck and sideboard between consecutive games
2. Sideboard diff viewer highlights cards in (green) and cards out (red)
3. Deck list correlation logic links "deck loaded" events with game starts
4. UI allows manual sideboard correction when automatic detection is uncertain
5. Sideboard data is saved in replay file format for later viewing

**Risk Flags:**
- 🔴 **HIGH:** Missing sideboard events — Sideboarding may not generate explicit network messages. May require correlating deck lists between games or reverse-engineering specific message types. Test during actual sideboarding in MTGO.

**Deliverables:**
- Sideboard tracker (compares decks between consecutive games)
- Sideboard diff viewer UI (highlighted cards in/out)
- Game boundary detection (identify when games start/end)
- Deck change correlation logic (deck loaded → game start)
- Manual sideboard correction UI
- Sideboard data storage in replay format

**Addresses:** Sideboard extraction (FEATURES - key differentiator)

**Avoids:** Missing sideboard events

---

## Phase 5: Replay Playback & Basic Analytics

**Goal:** Provide users with the ability to view replays and analyze match data locally.

**Dependencies:** Phase 3 (Game state reconstruction required for playback) and Phase 4 (Sideboard data for analytics)

**Requirements:**
- VIEW-001: Display replays locally with built-in viewer
- XPORT-001: Export replays as standalone files for sharing
- STAT-002: Calculate win/loss statistics
- STAT-004: Provide search and filtering (find replays by date, format, opponent)

**Success Criteria:**
1. Replay viewer UI displays turn-by-turn game state with playback controls (play, pause, step forward/backward)
2. Basic statistics dashboard shows win rates, game length, and mulligan counts
3. Replay browser allows searching and filtering replays by date, format, and opponent
4. Export functionality saves replays to custom binary format and optionally to JSON/CSV
5. Replay loading completes within 2 seconds for typical replay files

**Risk Flags:**
- 🟢 **LOW:** Standard UI patterns and data visualization. Skip `/gsd-research-phase`.

**Deliverables:**
- Replay viewer UI with turn-by-turn playback controls
- Basic statistics dashboard (win rates, game length, mulligans)
- Replay browser with search and filtering
- Export functionality (binary format, JSON, CSV)
- File size optimization for easy sharing
- Async loading for large replay libraries

**Uses:** Tauri UI framework, game state reconstruction from Phase 3, sideboard data from Phase 4

**Addresses:** Basic playback controls, win/loss tracking, statistics, search/filtering, file export

---

## Phase 6: Performance Optimization & Stability

**Goal:** Ensure the application handles long capture sessions (tournaments) and large replay libraries without crashes or memory leaks.

**Dependencies:** Phase 1-5 (Must have complete feature set to optimize)

**Requirements:**
- PERF-001: Stream packets to disk immediately (no in-memory storage leaks)
- PERF-002: Support long capture sessions (8+ hours) without crashes
- PERF-003: Compress replay files for storage efficiency

**Success Criteria:**
1. Application can capture continuously for 8+ hours without memory leaks or crashes
2. Packet streaming writes to disk immediately with compression (target < 5MB/hour)
3. Memory usage remains stable (< 500MB) during long capture sessions
4. Replay file compression reduces file size by at least 50% without sacrificing load speed
5. Application handles replay libraries of 100+ files without performance degradation

**Risk Flags:**
- 🟢 **LOW:** Standard performance optimization patterns. Skip `/gsd-research-phase`.

**Deliverables:**
- Stream-to-disk packet handling (immediate compression and write)
- Periodic state checkpoints (recovery from crashes)
- Replay file compression (zstd or similar)
- Memory profiling fixes (8+ hour capture testing)
- Async loading for large replay libraries
- Performance benchmarks and optimizations

**Avoids:** Memory leaks in long sessions, insufficient packet filtering

---

## Phase 7: Advanced Features (Future Consideration)

**Goal:** After core capture and playback works, add advanced differentiators and analytics.

**Dependencies:** Phase 1-6 (Complete, stable core)

**Requirements:**
- None (v2+ features)

**Success Criteria:**
1. Real-time overlay tracking via MTGOSDK displays live match information
2. Card usage analytics tracks how often cards resolve and their effectiveness
3. Mulligan analysis shows keep vs mulligan decisions and outcomes
4. Mana curve visualization displays mana usage across games
5. Trend analysis charts show performance improvements over time
6. Replay anonymization option removes usernames before sharing
7. Multi-replay comparison allows side-by-side replay viewing

**Risk Flags:**
- 🟡 **MEDIUM:** MTGOSDK integration requires `/gsd-research-phase`. Standard analytics skip research.

**Deliverables:**
- Real-time overlay tracking via MTGOSDK
- Card usage analytics (card performance metrics)
- Mulligan analysis (decision optimization insights)
- Mana curve visualization (deck building optimization)
- Trend analysis and charts (performance over time)
- Replay anonymization for sharing
- Multi-replay comparison

**Addresses:** Real-time overlay tracking, card usage analytics, mulligan analysis, mana curve analysis, trend visualization (FEATURES - v2+)

---

## Progress

| Phase | Status | Completion |
|-------|--------|------------|
| 1 - Capture Infrastructure & Proof of Concept | Not Started | 0% |
| 2 - Protocol Reverse Engineering & Replay Format Design | Not Started | 0% |
| 3 - Game State Reconstruction & Card Resolution | Not Started | 0% |
| 4 - Sideboard Detection & Extraction | Not Started | 0% |
| 5 - Replay Playback & Basic Analytics | Not Started | 0% |
| 6 - Performance Optimization & Stability | Not Started | 0% |
| 7 - Advanced Features | Not Started | 0% |

**Overall Progress:** 0% (0/7 phases complete)

---

## Phase Ordering Rationale

- **Capture → Protocol → State → Features:** Each phase depends on the previous. Without capture, nothing works. Without protocol decoding, no state. Without state, no features. This follows the architectural layers defined in ARCHITECTURE.md.

- **Grouping by architectural boundaries:** Phases 1-2 build infrastructure (capture/protocol), Phase 3 builds core logic (game state), Phases 4-6 add features (sideboard, playback, analytics). This allows parallel development within phases and clear testing boundaries.

- **Critical risks early:** Loopback capture (Phase 1) and protocol fragility (Phase 2) are addressed before investing heavily in features. If capture or protocol fails, minimal time wasted.

- **Differentiator after foundation:** Sideboard detection (Phase 4) is a key competitive differentiator but requires stable game state reconstruction (Phase 3) first.

---

## Research Flags

Phases likely needing deeper research during planning:

- **Phase 1 (Capture Infrastructure):** 🔴 HIGH complexity — Platform-specific capture (WinDivert driver, Windows privilege handling), sparse MTGO-specific documentation. Requires `/gsd-research-phase` during planning.

- **Phase 2 (Protocol Reverse Engineering):** 🔴 HIGH complexity — Undocumented MTGO protocol, requires Wireshark analysis and reverse-engineering tools, may need trial-and-error discovery. Requires `/gsd-research-phase` during planning.

- **Phase 4 (Sideboard Detection):** 🔴 HIGH complexity — Niche protocol behavior (sideboarding may not generate explicit messages), sparse documentation, correlation logic complexity. Requires `/gsd-research-phase` during planning.

Phases with standard patterns (skip research-phase):

- **Phase 5 (Playback & Analytics):** 🟢 LOW complexity — Well-documented UI patterns (Tauri, React), standard data visualization, replay playback is established pattern. Skip `/gsd-research-phase`.

- **Phase 6 (Performance & Stability):** 🟢 LOW complexity — Standard performance optimization techniques, memory profiling tools, compression algorithms. Skip `/gsd-research-phase`.

- **Phase 7 (Advanced Features):** 🟡 MEDIUM complexity — Standard analytics and visualization patterns, MTGOSDK has documented API. MTGOSDK integration may require `/gsd-research-phase`, standard analytics skip.

---

## Platform Constraints

- **Windows-only:** MTGO runs exclusively on Windows. WinDivert driver required for packet capture (captures localhost traffic better than alternatives).

- **Administrator privileges required:** Packet capture requires kernel-level drivers and elevated permissions.

- **MTGO protocol is proprietary and undocumented:** Reverse-engineering required. Protocol changes frequently without version handshake.

- **No cloud dependencies:** Local-only for v1. Replay files shared manually via file transfer.
