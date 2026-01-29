# Project Research Summary

**Project:** MTGO Replay Capture
**Domain:** MTGO (Magic: The Gathering Online) Replay Capture System - Network traffic interception and protocol reverse engineering
**Researched:** 2026-01-29
**Confidence:** MEDIUM

## Executive Summary

MTGO Replay Capture is a desktop application that intercepts Magic: The Gathering Online's proprietary network protocol to capture game replays and extract match data. This is a network protocol reverse-engineering domain that requires packet capture, binary protocol parsing, and state reconstruction. Experts build such systems using layered architectures with capture, protocol decoding, and application layers separated for maintainability. The recommended approach is Rust with Tauri 2.0 for memory-safe binary parsing and native performance, using WinDivert on Windows (MTGO's primary platform) or libpcap on Unix/Mac for packet capture.

Key risks center on MTGO's undocumented protocol and Windows platform constraints. The protocol is undocumented and changes frequently with client updates, so parsers must be designed for version flexibility and tolerance of unknown fields. Loopback traffic capture on Windows requires WinDivert (libpcap misses localhost traffic), and packet capture requires administrator privileges with kernel-level drivers. Game state reconstruction from network packets can be ambiguous if packets are lost or out-of-order. Mitigation strategies include version-aware parsers, bounded message queues for backpressure, streaming packets to disk immediately, and building robust replay file formats with versioning from day one.

## Key Findings

### Recommended Stack

MTGO replay capture requires memory-safe binary parsing with platform-specific packet capture. Rust 1.80+ is recommended for core language due to its memory safety for network parsing, Tauri-native framework, excellent binary data handling ecosystem, and trivial cross-platform compilation. Tauri 2.0 provides 10x smaller bundles than Electron with native performance. Platform-specific capture is critical: WinDivert 1.4+ for Windows (required because libpcap/WinPcap misses localhost traffic), and libpcap 1.10+ for Unix/Mac. Tokio 1.35+ handles non-blocking packet capture and concurrent connections. Serde 1.0+ with MessagePack (rmp-serde) or Bincode provides compact binary serialization for replay storage.

**Core technologies:**
- **Rust 1.80+**: Core language — Memory safety for binary parsing, cross-platform, Tauri-native
- **Tauri 2.0**: Desktop framework — 10x smaller than Electron, native performance, system-level tool capability
- **Tokio 1.35+**: Async runtime — Non-blocking packet capture, concurrent connection handling
- **WinDivert 1.4+** (Windows): Packet capture — Captures localhost traffic libpcap misses (MTGO uses loopback)
- **libpcap 1.10+** (Unix/Mac): Packet capture — Industry standard for network interception
- **Serde + MessagePack**: Serialization — Type-safe, compact binary format for replay storage

**Version requirements:**
- Rust 1.75+ recommended for latest features (Tauri 2.0 MSRV: 1.70+)
- WinDivert requires Windows 7+, 64-bit only, admin privileges, driver installation
- Npcap on Windows must be installed with loopback support enabled

### Expected Features

MTGO replay tools have clear table stakes, competitive differentiators, and anti-features. The ecosystem has two main approaches: log file parsing (simpler, offline, MTGO-Tracker) and SDK-based real-time tracking (complex, live tracking, Videre Tracker). There's a gap in the market—no tool combines automatic replay capture with automatic sideboard extraction. MTGO-Tracker parses existing logs but doesn't capture, Magic Online Replay Tool analyzes replays but requires manual capture, and Videre uses SDK for real-time but doesn't emphasize file-based capture.

**Must have (table stakes):**
- **Replay capture** — Core value proposition, users need to save replays
- **Game state reconstruction** — Essential for viewing replay content
- **Deck list extraction** — Users expect to see what they played (maindeck, sideboard)
- **Play-by-play breakdown** — Necessary for game analysis, what cards were played when
- **Basic playback controls** — Play, pause, step forward/backward
- **Match metadata** — Format, date, opponent identification, duration
- **File export/sharing** — Export to standard formats (JSON, CSV) for sharing
- **Search and filtering** — Finding specific replays in collection by date, format, opponent

**Should have (competitive):**
- **Automatic sideboard detection** — Key differentiator, solves major pain point (manual tracking is tedious)
- **Compact replay format** — Enables easy file sharing and storage efficiency (binary, not raw logs)
- **Turn-by-turn replay playback** — Superior to static viewing, allows studying decision points
- **Win/loss statistics** — Basic analytics dashboard for self-improvement
- **Sideboard extraction** — Tournament players highly request this
- **Card usage analytics** — Beyond basic win rate, reveals play patterns and card performance

**Defer (v2+):**
- **Real-time overlay tracking** — Use MTGOSDK for live analysis (high complexity)
- **Real-time streaming to external service** — Privacy concerns, ToS violations, cloud infrastructure
- **Cloud-based replay storage** — Privacy concerns, recurring costs
- **MTGO client modification/injection** — Likely violates EULA, breaks on updates
- **Automated play suggestions** — Creates dependency, liability issues, ToS violation

### Architecture Approach

MTGO replay capture uses a layered architecture with clear separation between capture, protocol, and application logic. This standard pattern isolates protocol complexity, enables testing each layer independently, and allows optimization of different parts independently. The system flows from MTGO client network traffic through capture (WinDivert/libpcap), filtering (BPF), parsing (TCP/IP structures), protocol decoding (state machine), message queuing (bounded for backpressure), game state reconstruction (event-driven), and replay serialization (compressed binary format). Event-driven architecture handles high-frequency packet bursts, while state machine pattern tracks connection state and game phases. Producer-consumer pattern prevents memory exhaustion when capture speed exceeds processing speed.

**Major components:**
1. **Capture Layer** — Intercept network traffic from MTGO client using platform-specific drivers (WinDivert on Windows, libpcap on Unix/Mac), apply BPF filters for MTGO traffic
2. **Protocol Layer** — Reverse-engineer and decode MTGO proprietary protocol using state machine, parse TCP/IP structures, buffer messages in bounded queue
3. **Application Layer** — Maintain game state from protocol messages (event-driven state machine), track sideboard operations between games, serialize states into compact replay format
4. **Presentation Layer** — Desktop application UI (Tauri 2.0) for capture controls, status monitoring, configuration, and future replay viewing

**Key patterns:**
- **Layered Architecture** — Clear boundaries between capture, protocol, and application logic
- **Event-Driven** — Components communicate via events for loose coupling and async processing
- **State Machine** — Protocol decoder tracks connection state and game phases (mulligan, draw, combat, etc.)
- **Producer-Consumer** — Packet capture produces, protocol decoder consumes with bounded buffer and backpressure

### Critical Pitfalls

MTGO replay capture has domain-specific pitfalls that cause rewrites or major issues. Loopback traffic capture blindness is the most common—developers assume standard libpcap works, but MTGO communicates via localhost and WinDivert is required on Windows. Protocol version fragility is equally critical—parsers break after every MTGO update because the protocol changes without warning. Platform-specific capture failures occur when tools work on developer machines but fail on clean Windows installs due to missing drivers, privilege issues, or antivirus blocking. Game state reconstruction ambiguity leads to incomplete or incorrect replays because packets are lost, out-of-order, or client-side state isn't captured. File format brittleness breaks backward compatibility when formats lack versioning and schema evolution.

1. **Loopback traffic capture blindness** — MTGO uses localhost (127.0.0.1) for some connections; libpcap misses this traffic. Must use WinDivert on Windows with loopback support enabled. Test early with actual MTGO traffic, verify packet capture shows game-related traffic. (Phase 1: Proof of Concept)

2. **Protocol version fragility** — MTGO protocol is undocumented and changes frequently without version handshake. Design version-aware parsers with tolerance for unknown fields, store MTGO client version in replay header, implement protocol fuzzing/extensibility. Document reverse-engineered protocol publicly to share burden of re-reverse-engineering. (Phase 2: Protocol & Format Design)

3. **Platform-specific capture failures** — Packet capture requires kernel-level drivers (Npcap/WinPcap), administrator privileges, and may be blocked by antivirus. Detect admin privileges, verify Npcap installation, provide clear error messages with specific fixes, test on Windows 10/11 with various AV. Bundle WinDivert driver with installer for Windows. (Phase 1: Proof of Concept)

4. **Game state reconstruction ambiguity** — Network packets only show server-sent data, may be out-of-order or lost, client-side state missing. Implement packet reordering and loss handling, capture redundancy, validate reconstructed state against invariants (deck size, life bounds), log reconstruction gaps. Compare replay side-by-side with actual MTGO gameplay to verify accuracy. (Phase 3: Game State Reconstruction)

5. **File format brittleness** — Binary formats without versioning break backward compatibility when fields added/removed. Design versioned file format from day one with magic bytes, version field, self-describing sections, backward compatibility strategy. Never remove support for old formats, provide migration tools, document format publicly. (Phase 2: Protocol & Format Design)

## Implications for Roadmap

Based on research, suggested phase structure follows dependency order: capture infrastructure first (must verify traffic paths and platform-specific issues), protocol parsing second (foundation for everything else), game state reconstruction third (enables replay features), then sideboard detection, then playback/analytics. This ordering addresses critical pitfalls early (loopback capture, protocol fragility) before building dependent features.

### Phase 1: Capture Infrastructure & Proof of Concept
**Rationale:** Packet capture is the foundation—without capturing MTGO traffic, nothing else is possible. Must verify traffic paths (loopback vs physical interface) early to avoid loopback capture blindness. Must test on clean Windows installs to avoid platform-specific capture failures.
**Delivers:** Working packet capture on Windows with MTGO traffic visible, BPF filtering for MTGO servers, basic packet parser extracting TCP/IP payload, admin privilege detection, Npcap/WinDivert driver verification.
**Addresses:** Replay capture (FEATURES), Match metadata (partial)
**Avoids:** Loopback traffic capture blindness, Platform-specific capture failures
**Research flag:** HIGH complexity — protocol reverse-engineering requires `/gsd-research-phase` during planning

### Phase 2: Protocol Reverse Engineering & Replay Format Design
**Rationale:** Once capture works, must decode MTGO's proprietary protocol to reconstruct game state. Protocol version fragility is critical—must design version-aware parsers from start. Replay file format must be versioned to avoid brittleness.
**Delivers:** MTGO protocol decoder (state machine handling message types), message type definitions from reverse-engineering, version detection strategy, bounded message queue for backpressure, versioned replay file format specification, basic serializer/deserializer.
**Uses:** Tokio async runtime, Serde serialization, WinDivert/libpcap bindings
**Implements:** Protocol Layer, Message Queue, Replay Manager (ARCHITECTURE)
**Research flag:** HIGH complexity — undocumented protocol, needs `/gsd-research-phase` during planning

### Phase 3: Game State Reconstruction & Card Resolution
**Rationale:** Protocol decoding produces raw messages; game state reconstruction turns them into usable game state. This is the heavy lifting—must handle packet loss, reordering, ambiguous client-side state. Card database resolution strategy must be decided early to avoid coupling.
**Delivers:** Game state machine (maintains board, hands, libraries, life totals), card action extraction (card played, resolved, moved), packet reordering/loss handling, state validation invariants, card ID storage (not embedding card data), external card database resolution (Scryfall or bundled DB).
**Addresses:** Game state reconstruction (FEATURES), Deck list extraction, Play-by-play breakdown
**Avoids:** Game state reconstruction ambiguity, Card database coupling
**Research flag:** MEDIUM complexity — standard event-driven patterns, but MTGO-specific state needs `/gsd-research-phase` during planning

### Phase 4: Sideboard Detection & Extraction
**Rationale:** Sideboarding is the key competitive differentiator but may not generate explicit network messages. May require correlating deck lists between games or reverse-engineering specific message types.
**Delivers:** Sideboard tracker (compares decks between consecutive games), sideboard diff viewer (cards in/out highlighted), correlation logic for "deck loaded" events with game starts, UI for manual sideboard correction.
**Addresses:** Sideboard extraction (FEATURES - key differentiator)
**Avoids:** Missing sideboard events
**Research flag:** HIGH complexity — niche protocol behavior, sparse documentation, definitely needs `/gsd-research-phase` during planning

### Phase 5: Replay Playback & Basic Analytics
**Rationale:** Once replays are captured and reconstructed, users need to view and analyze them. Playback requires full game state reconstruction from phase 3. Analytics aggregate match data for insights.
**Delivers:** Replay viewer UI (turn-by-turn playback controls), basic statistics dashboard (win rates, game length, mulligans), replay browser (search/filter by date, format, opponent), export functionality (JSON, CSV).
**Addresses:** Basic playback controls, Win/loss tracking, Basic statistics, Search and filtering, File export (FEATURES)
**Uses:** Tauri UI framework, state reconstruction from Phase 3
**Research flag:** LOW complexity — standard UI patterns and data visualization, skip `/gsd-research-phase`

### Phase 6: Performance Optimization & Stability
**Rationale:** MVP works for single games, but tournaments require multi-hour stability. Memory leaks from in-memory packet storage cause crashes. Performance bottlenecks emerge with large replay libraries.
**Delivers:** Stream-to-disk packet handling (immediate compression and write), periodic state checkpoints, replay file compression (zstd or similar), memory profiling fixes (8+ hour capture testing), async loading for large replay libraries.
**Avoids:** Memory leaks in long sessions, Insufficient packet filtering
**Research flag:** LOW complexity — standard performance optimization patterns, skip `/gsd-research-phase`

### Phase 7: Advanced Features (Future)
**Rationale:** After core capture and playback works, add differentiators like real-time tracking, advanced analytics, and sharing features.
**Delivers:** Real-time overlay tracking via MTGOSDK, card usage analytics, mulligan analysis, mana curve visualization, trend analysis and charts, replay anonymization for sharing, multi-replay comparison.
**Addresses:** Real-time overlay tracking, Card usage analytics, Mulligan analysis, Mana curve analysis, Trend visualization (FEATURES - v2+)
**Research flag:** MEDIUM complexity — MTGOSDK integration requires `/gsd-research-phase`, standard analytics skip

### Phase Ordering Rationale

- **Capture → Protocol → State → Features:** Each phase depends on the previous. Without capture, nothing works. Without protocol decoding, no state. Without state, no features. This follows the architectural layers.
- **Grouping by architectural boundaries:** Phases 1-2 build infrastructure (capture/protocol), Phase 3 builds core logic (game state), Phases 4-7 add features (sideboard, playback, analytics). This allows parallel development within phases.
- **Critical risks early:** Loopback capture and protocol fragility are addressed in Phases 1-2 before investing heavily in features. If capture or protocol fails, minimal time wasted.
- **Differentiator after foundation:** Sideboard detection (Phase 4) is a key differentiator but requires stable game state reconstruction (Phase 3) first.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1 (Capture Infrastructure):** Complex platform-specific capture (WinDivert driver, Npcap loopback, Windows privilege handling), sparse MTGO-specific documentation
- **Phase 2 (Protocol Reverse Engineering):** Undocumented MTGO protocol, requires Wireshark analysis and reverse-engineering tools, may need trial-and-error discovery
- **Phase 4 (Sideboard Detection):** Niche protocol behavior (sideboarding may not generate explicit messages), sparse documentation, correlation logic complexity

Phases with standard patterns (skip research-phase):
- **Phase 5 (Playback & Analytics):** Well-documented UI patterns (Tauri, React), standard data visualization, replay playback is established pattern
- **Phase 6 (Performance & Stability):** Standard performance optimization techniques, memory profiling tools, compression algorithms
- **Phase 7 (Advanced Features):** Standard analytics and visualization patterns, MTGOSDK has documented API

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | MEDIUM | Core stack (Rust, Tauri, Tokio, MessagePack) verified from official sources. MTGO-specific tools (WinDivert) verified but protocol-specific tools may need adjustment based on actual MTGO protocol characteristics. |
| Features | HIGH | Feature landscape well-documented from competitor analysis (MTGO-Tracker, Videre Tracker, Magic Online Replay Tool). Clear distinction between table stakes, differentiators, and anti-features. |
| Architecture | MEDIUM | Standard layered architecture pattern for network capture is well-established. Component boundaries are clear, but MTGO-specific protocol handling may require adjustments during implementation. |
| Pitfalls | MEDIUM | Pitfalls identified from general reverse-engineering and packet capture principles, plus domain-specific risks (loopback capture, protocol fragility). Actual MTGO protocol reverse-engineering in Phase 1 will validate severity. |

**Overall confidence:** MEDIUM

### Gaps to Address

- **MTGO protocol documentation:** MTGO protocol is undocumented and proprietary. Reverse-engineering in Phase 1 will reveal actual message formats, field ordering, and state representation. Gap must be addressed before Phase 2 protocol design.
- **MTGO traffic paths:** Research assumes MTGO may use localhost for some connections, but actual traffic patterns (which servers, which ports, any loopback) must be verified in Phase 1 proof of concept.
- **Sideboard protocol representation:** Unknown how sideboarding manifests in network packets (explicit messages, deck list deltas, zone changes). Requires dedicated reverse-engineering in Phase 4.
- **Card ID format in MTGO:** Unclear how MTGO references cards (Gatherer IDs, internal IDs, set codes). Phase 3 must identify card resolution strategy.
- **Replay file format trade-offs:** MessagePack vs Bincode vs custom binary format trade-offs need validation with real replay data sizes and parse performance.

## Sources

### Primary (HIGH confidence)

- **Tauri 2.0 official site** (v2.tauri.app) — Verified current version, features, 10x smaller bundles than Electron
- **Rust official site** (rust-lang.org) — Memory safety guarantees, cross-platform compilation, async/await support
- **Tokio 1.35+ official site** (tokio.rs) — Async runtime for Rust, non-blocking I/O
- **libpcap official site** (tcpdump.org) — Industry standard packet capture library
- **WinDivert official site** (reqrypt.org/windivert) — Windows kernel-mode packet filter/divert, captures localhost traffic
- **MessagePack official site** (msgpack.org) — Binary serialization format specification, compactness comparison to JSON
- **Wireshark 4.6.3 official site** (wireshark.org) — Verified for packet analysis, BPF filter syntax, loopback capture setup
- **MTGO-Tracker Wiki** (github.com/cderickson/MTGO-Tracker/wiki) — Table structure, features, log parsing approach
- **MTGOSDK README** (github.com/videre-project/MTGOSDK) — SDK capabilities, memory inspection via ClrMD, real-time tracking
- **Videre Tracker README** (github.com/videre-project/Tracker) — Real-time tracking features, MTGOSDK usage example

### Secondary (MEDIUM confidence)

- **Magic Online Replay Tool** (github.com/PennyDreadfulMTG/MagicOnlineReplayTool) — Replay analysis statistics, C# WPF architecture (last updated Oct 2022)
- **PennyDreadful-Tools README** (github.com/PennyDreadfulMTG/Penny-Dreadful-Tools) — Comprehensive MTGO tooling suite, multiple integration approaches
- **GitHub MTGO topic** (github.com/topics/mtgo) — Ecosystem overview of 19 MTGO-related repositories, competitor landscape
- **Npcap Documentation** (npcap.com/guide/npcap-devguide.html) — Windows packet capture details, loopback support, driver requirements
- **Wireshark Loopback Capture Setup** (wiki.wireshark.org/CaptureSetup/Loopback) — Loopback capture configuration, Npcap vs WinPcap differences
- **libpcap Documentation** (tcpdump.org/manpages/pcap.3pcap.html) — Programming with pcap, packet filtering, BPF usage

### Tertiary (LOW confidence)

- **Radare2 GitHub** (radareorg/radare2) — Binary reverse engineering tool, potential for analyzing MTGO client binaries (not verified for this use case)
- **Ghidra official site** (ghidra-sre.org) — NSA's reverse engineering tool, protocol reverse engineering (not verified for MTGO protocol)
- **GitHub Reverse Engineering Topics** (github.com/topics/reverse-engineering) — General reverse-engineering tools directory listing only
- **MTGGoldfish** (mtggoldfish.com) — Industry standard for MTG data visualization and deck tracking, referenced for analytics inspiration (not specific to capture)

### Context7 library IDs

No Context7 library was used for this research. All findings from official documentation, GitHub repositories, and community sources.

---
*Research completed: 2026-01-29*
*Ready for roadmap: yes*
