# Stack Research

**Domain:** MTGO Replay Capture - Network traffic interception and protocol reverse engineering
**Researched:** 2026-01-29
**Confidence:** MEDIUM

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **Rust** | 1.80+ | Core language | Memory safety for network parsing, Tauri-native, excellent binary data handling ecosystem. Cross-platform compilation makes Windows/Mac/Linux support trivial. |
| **Tauri** | 2.0 | Desktop framework | 10x smaller bundles than Electron, native performance, Rust backend with web UI. Perfect for system-level tools that need user interface. |
| **Tokio** | 1.35+ | Async runtime | Rust's de facto async runtime, essential for non-blocking packet capture and handling multiple concurrent connections. |
| **WinDivert** | 1.4+ | Packet capture (Windows) | Captures localhost traffic. Required because MTGO is Windows-only and uses loopback connections. |
| **Serde** | 1.0+ | Serialization framework | Rust's serialization ecosystem. Type-safe, zero-cost abstractions, supports multiple formats through derive macros. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **windivert-sys** crate | 0.4+ | WinDivert bindings | Windows packet capture. Captures MTGO's localhost traffic. Requires WinDivert driver installation. |
| **rmp-serde** | 1.1+ | MessagePack + Serde | When storing replays in MessagePack format. Combines MessagePack's compactness with Serde's type safety. |
| **bincode** | 1.3+ | Binary serialization | Alternative to MessagePack. Smaller binary size but less human-readable debugging. |
| **thiserror** | 1.0+ | Error handling | Custom error types with Display/Error traits. Essential for user-friendly error messages in RE tools. |
| **tracing** | 0.1+ | Logging/observability | Structured logging for debugging packet capture issues and protocol reconstruction. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| **Wireshark** | Packet analysis | Current version 4.6.3 (verified from official site). Use for initial protocol discovery and validating captured packets. Export to PCAP for testing. |
| **Radare2** | Binary reverse engineering | Open-source, cross-platform. Good for analyzing MTGO client binaries if static RE is needed. |
| **Ghidra** | Binary reverse engineering | NSA's open-source tool. Excellent for protocol reverse engineering when packet capture isn't enough. |
| **cargo-watch** | Hot reloading | Auto-recompile during development. Useful for rapid iteration on protocol parsing code. |

## Installation

```bash
# Core
cargo add tauri --features all
cargo add tokio --features full
cargo add serde --features derive

# Network capture (Windows only)
cargo add windivert-sys

# Serialization
cargo add rmp-serde  # For MessagePack
# OR
cargo add bincode       # For pure binary

# Error handling & logging
cargo add thiserror
cargo add tracing
cargo add tracing-subscriber --features fmt,env-filter
```

**Windows Setup Note:**
WinDivert requires driver installation. Users will need admin privileges. Bundle WinDivert64.sys with installer.

## Alternatives Considered

| Recommended | Alternative | Why Not |
|--------------|---------------|-----------|
| Tauri 2.0 | Electron 40.0.0 | Electron bundles Chromium (~120MB) vs Tauri (~3MB). Electron can't access WinDivert driver easily. Electron's JS runtime is slower for binary parsing than Rust. |
| Rust | Python + Scapy | Python is too slow for real-time packet capture of high-volume game traffic. Scapy is great for analysis, not production capture. GIL limits concurrency. |
| Rust | C++ | Rust's memory safety prevents buffer overflows in packet parsing. Cargo's dependency management beats CMake. Modern async/await beats callback hell. |
| MessagePack | Protocol Buffers | Protobuf requires schema definition before parsing. We don't know MTGO's protocol yet. MessagePack is schema-flexible, easier for unknown protocol exploration. |
| MessagePack | JSON | JSON is text-based, 3-5x larger than binary. Parsing is slower. Not suitable for compact replay storage. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| **Pure Node.js** | Single-threaded event loop can't handle high-volume packet capture without dropping packets. No access to raw sockets without native modules. | Rust backend with Tokio async runtime |
| **Go** | Great language, but Tauri requires Rust. Go's goroutines would be nice, but interop complexity not worth it. | Rust with Tokio |
| **Raw Sockets API** | WinDivert is better for loopback traffic capture, which MTGO uses. | WinDivert on Windows |
| **Screen recording** (OBS, FRAPS) | Huge file sizes (GB/hr vs MB/hr). Can't extract structured data like card IDs, player info. Requires manual transcription. | Network packet capture + protocol parsing |
| **Man-in-the-middle proxy** | TLS interception requires certificate pinning bypass. MTGO client validates certs. More complex than packet capture. | Direct packet capture (no decryption needed if protocol isn't TLS) |
| **Old versions of Tauri (1.x)** | Missing 2.0 features: better mobile support, improved security model, better plugin system. | Tauri 2.0 |
| **Scapy for production capture** | Python GIL prevents parallel packet processing. Too slow for real-time analysis. | Rust windivert-sys with tokio channels |
| **Plain JSON for replays** | File size 3-5x larger than binary formats. Slower to parse. Poor for long games (hundreds of turns). | MessagePack or Bincode |

## Stack Patterns by Variant

**Capture (Windows only):**
- Use `windivert-sys` for capture
- Because: MTGO is Windows-only and WinDivert captures localhost traffic better than alternatives
- Note: WinDivert requires admin privileges and driver installation

**If protocol is encrypted (TLS):**
- Use Frida or SSLKEYLOGFILE
- Because: Packet capture alone only shows encrypted bytes
- Note: Certificate pinning may block MITM approaches

**If protocol is plaintext (custom binary):**
- Use Wireshark for initial discovery
- Because: Visual packet inspection, dissector can be written as plugin
- Note: Once protocol is understood, move to Rust parser for performance

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| Tauri 2.0 | Rust 1.70+ | Minimum supported Rust version. Recommend 1.75+ for latest features. |
| Tokio 1.35+ | Rust 1.70+ | Tokio's MSRV matches Tauri. |
| windivert-sys | Windows 7+ | WinDivert driver requires Windows 7 or later. 64-bit only. |
| rmp-serde 1.1+ | Serde 1.0+ | Compatible with latest serde derive macros. |

## Sources

- Tauri 2.0 official site (v2.tauri.app) — Verified current version and features
- Electron 40.0.0 official site (electronjs.org) — Verified for comparison
- Wireshark 4.6.3 official site (wireshark.org) — Verified stable release for packet analysis
- MessagePack official site (msgpack.org) — Verified binary format specification and compactness
- Protocol Buffers official site (protobuf.dev) — Verified as alternative binary format
- Cap'n Proto official site (capnproto.org) — Verified as alternative binary format
- Radare2 GitHub (radareorg/radare2) — Verified for reverse engineering
- Ghidra official site (ghidra-sre.org) — Verified for protocol RE
- libpcap official site (tcpdump.org) — Verified as standard packet capture library
- rust-pcap GitHub (rust-pcap/pcap) — Verified Rust bindings for libpcap

---
*Stack research for: MTGO Replay Capture*
*Researched: 2026-01-29*
*Updated: 2026-01-30 - Removed Unix/Mac support (MTGO is Windows-only)*
*Confidence: MEDIUM - Core stack verified from official sources, but protocol-specific tools may need adjustment based on MTGO's actual protocol characteristics*
