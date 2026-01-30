# Phase 1: Capture Infrastructure & Proof of Concept - Research

**Researched:** 2026-01-30
**Domain:** Windows packet capture with WinDivert, Tauri desktop app
**Confidence:** MEDIUM

## Summary

Phase 1 requires establishing a working packet capture system for MTGO network traffic on Windows. The research confirms that **WinDivert 2.2.2** is the correct choice for Windows packet capture, as it uniquely supports loopback (localhost) traffic capture—a critical requirement given MTGO may use 127.0.0.1 for some connections. The **windivert-sys** and **windivert** Rust crates provide well-documented bindings, though they are pre-1.0.0 so breaking changes are possible.

The stack should use **Tauri 2.0** for the desktop application framework, leveraging its Rust backend with web UI. For message handling between packet capture and protocol decoding, **Tokio's bounded MPSC channels** provide async, backpressure-aware queues that directly address PERF-004 (packet bursts with bounded queues).

A key finding is that **administrator privilege detection** in Rust requires using the `windows` crate's security APIs to check token elevation status before attempting WinDivert operations. Without this check, applications will fail with ERROR_ACCESS_DENIED without clear user guidance.

**Primary recommendation:** Use windivert 0.6.0 + Tauri 2.0 + Tokio 1.35+, implementing admin privilege detection before WinDivert initialization and using bounded channels for packet flow control.

## Standard Stack

The established libraries/tools for Windows packet capture with Tauri:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| windivert-sys | 0.10.0+ | Raw FFI bindings to WinDivert | Provides direct access to WinDivert 2.2.2 API, actively maintained |
| windivert | 0.6.0 | Higher-level Rust wrapper | Friendlier API with abstractions, 100% documented on docs.rs |
| tauri | 2.0 | Desktop application framework | Modern, secure, Rust-native desktop framework with web UI |
| tokio | 1.35+ | Async runtime | De facto standard for async Rust, provides bounded MPSC channels |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| windows | 0.48+ | Windows-specific APIs | Administrator privilege detection via security APIs |
| thiserror | 1.0+ | Error handling | Structured error types with context |
| tracing | 0.1+ | Logging | Instrumentation and debugging for capture operations |
| serde | 1.0+ | Serialization | For replay file format (future phases) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| WinDivert | libpcap/npcap | libpcap cannot capture loopback traffic on Windows (critical gap) |
| windivert | windivert-sys only | windivert-sys is raw FFI only; windivert provides safer abstractions |
| Tokio channels | crossbeam channels | Tokio is async-native, better integration with async packet capture loop |

**Installation:**
```bash
# Core dependencies
cargo add windivert windivert-sys tauri tokio

# Windows-specific
cargo add windows

# Utilities
cargo add thiserror tracing serde

# Tauri CLI
cargo install tauri-cli --version "^2.0.0" --locked
```

### WinDivert Binary Distribution
- Download from: https://reqrypt.org/windivert.html
- Version: WinDivert 2.2.2-A (signed binaries)
- Required files for x64 Windows:
  - `WinDivert.dll` (64-bit)
  - `WinDivert64.sys` (driver, auto-installed)
- Must be in application directory or path specified via `WINDIVERT_PATH` env var

## Architecture Patterns

### Recommended Project Structure
```
src/
├── capture/              # Packet capture layer
│   ├── mod.rs
│   ├── handle.rs          # WinDivert handle management
│   ├── filter.rs          # BPF/WinDivert filter compilation
│   └── admin.rs           # Administrator privilege detection
├── protocol/             # MTGO protocol handling (future)
├── game/                 # Game state management (future)
├── replay/               # Replay format handling (future)
├── ui/                   # Desktop application
│   ├── mod.rs
│   ├── commands.rs        # Tauri commands for frontend
│   └── events.rs          # Events emitted to frontend
├── config/               # Configuration
├── common/               # Shared utilities
│   └── error.rs           # Application error types
└── main.rs               # Tauri entry point
```

### Pattern 1: WinDivert Handle Lifecycle with Typestate
**What:** Using the `windivert` crate's typestate pattern to ensure correct handle initialization and usage order.

**When to use:** All WinDivert operations to guarantee API correctness at compile time.

**Example:**
```rust
// Source: https://docs.rs/windivert/0.6.0/windivert/
use windivert::{layer::Network, WinDivert, WinDivertFlags};

// Create handle with filter for MTGO traffic
let handle = WinDivert::builder()
    .layer(Network::Network)  // WINDIVERT_LAYER_NETWORK
    .filter("outbound and (tcp.DstPort == 80 or tcp.DstPort == 443)")
    .flags(WinDivertFlags::SNIFF)  // Packet sniffing mode (copy, don't drop)
    .build()?;

// Receive packets in async loop
while let Ok((packet, address)) = handle.recv() {
    // Process packet...
}
```

### Pattern 2: Bounded MPSC Channels for Packet Flow
**What:** Using Tokio's bounded MPSC channels to implement backpressure between packet capture and protocol decoding, addressing PERF-004.

**When to use:** Producer-consumer pattern where packet capture produces and decoder consumes.

**Example:**
```rust
// Source: https://docs.rs/tokio/1.35.1/tokio/sync/mpsc/fn.channel.html
use tokio::sync::mpsc;

const CHANNEL_CAPACITY: usize = 1000;  // Bounded queue size

let (tx, mut rx) = mpsc::channel(CHANNEL_CAPACITY);

// Producer: Packet capture loop
tokio::spawn(async move {
    while let Ok((packet, addr)) = handle.recv() {
        if let Err(_) = tx.send((packet, addr)).await {
            // Receiver dropped, shut down capture
            break;
        }
    }
});

// Consumer: Protocol decoder
tokio::spawn(async move {
    while let Some((packet, addr)) = rx.recv().await {
        // Decode packet...
    }
});
```

### Pattern 3: Administrator Privilege Detection
**What:** Detecting admin privileges before attempting WinDivert operations to provide clear error messages.

**When to use:** Before WinDivert initialization in Tauri commands.

**Example:**
```rust
// Source: Windows security token elevation detection pattern
use windows::Win32::Security::*;

fn is_running_as_admin() -> Result<bool, CaptureError> {
    // Get current process token
    let mut handle = HANDLE::default();
    let success = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle) };

    if !success.as_bool() {
        return Err(CaptureError::PrivilegeDetectionFailed);
    }

    // Check elevation status
    let mut elevation = TOKEN_ELEVATION::default();
    let mut ret_len = 0u32;

    let success = unsafe {
        GetTokenInformation(
            handle,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut c_void),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        )
    };

    if !success.as_bool() {
        return Err(CaptureError::PrivilegeDetectionFailed);
    }

    Ok(elevation.TokenIsElevated.as_bool())
}

// Usage in Tauri command
#[tauri::command]
async fn start_capture() -> Result<CaptureStatus, CaptureError> {
    if !is_running_as_admin()? {
        return Err(CaptureError::RequiresAdminPrivileges);
    }
    // Proceed with WinDivert initialization...
}
```

### Anti-Patterns to Avoid
- **Using libpcap on Windows:** Cannot capture loopback traffic—MTGO may use localhost, causing blind spots. WinDivert required.
- **Blocking WinDivertRecv on main thread:** Blocks Tauri event loop. Use Tokio's async wrapper or spawn dedicated thread.
- **Unbounded message queues:** No backpressure—packet bursts cause unbounded memory growth. Use Tokio bounded channels.
- **Initializing WinDivert without privilege check:** Fails with cryptic ERROR_ACCESS_DENIED. Check admin status first and show user-friendly message.
- **Mixing WINDIVERT_LAYER_NETWORK with WINDIVERT_FLAG_DROP:** Invalid combination, handle won't receive packets.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Windows admin privilege detection | Manual token elevation checks via Win32 API | windows crate security APIs | Complex token elevation logic, edge cases, tested |
| Async packet capture loop | Manual async wrapper around WinDivertRecv | windivert crate's async support or Tokio spawn | Avoids blocking on syscall, integrates with Tokio runtime |
| Bounded message queues | Custom Vec-based ring buffer | tokio::sync::mpsc::channel | Backpressure handling, efficient, well-tested |
| WinDivert filter parsing | Custom string parser for BPF-like filters | WinDivertHelperCompileFilter from windivert-sys | Full filter language support, edge cases handled |
| Error types | Custom error enum without context | thiserror crate with #[error] attributes | Structured errors, display chain for user feedback |

**Key insight:** WinDivert's filter language is powerful and complex—hand-rolling even basic filter parsing leads to missed features and subtle bugs. Similarly, Windows security token APIs have complex lifetime and ownership semantics that the `windows` crate abstracts away correctly.

## Common Pitfalls

### Pitfall 1: Loopback Traffic Capture Blindness
**What goes wrong:** Using libpcap or WinDivert without loopback awareness causes MTGO traffic to/from 127.0.0.1 to be invisible.

**Why it happens:** libpcap cannot capture loopback on Windows by design; WinDivert requires explicit filter configuration.

**How to avoid:**
- Use WinDivert (not libpcap)
- Verify `WINDIVERT_LAYER_NETWORK` layer (supports loopback)
- Test filter includes loopback: `outbound and (tcp.DstPort == PORT or (ip.SrcAddr == 127.0.0.1 or ip.DstAddr == 127.0.0.1))`
- Run MTGO client during proof-of-concept testing with actual gameplay to capture real traffic

**Warning signs:** Capture runs but shows zero packets or only packets to/from external IPs, missing local traffic.

### Pitfall 2: Admin Privilege Failure Without Clear Messaging
**What goes wrong:** WinDivertOpen fails with ERROR_ACCESS_DENIED but application shows generic "capture failed" message.

**Why it happens:** No privilege detection before initialization; users don't know they need to "run as administrator."

**How to avoid:**
- Implement `is_running_as_admin()` check before WinDivert initialization
- Return CaptureError::RequiresAdminPrivileges with actionable message: "Please restart the application as Administrator"
- Show Windows UAC shield icon in UI when privileges are missing
- Consider requesting elevation via ShellExecute with "runas" verb (future enhancement)

**Warning signs:** GetLastError returns 5 (ERROR_ACCESS_DENIED) after WinDivertOpen call.

### Pitfall 3: Packet Burst Overflow Without Backpressure
**What goes wrong:** During gameplay, sudden packet bursts cause unbounded memory growth or packet loss in unbounded queues.

**Why it happens:** Unbounded channels (Vec, unbounded MPSC) never apply backpressure—producers keep producing, consumers can't keep up.

**How to avoid:**
- Use Tokio's bounded MPSC channel with capacity tuned to expected burst size (start with 1000 packets)
- Monitor channel fullness metrics; log warnings when near capacity
- Apply backpressure at producer side: block or drop when channel full
- Consider adaptive capacity tuning based on traffic patterns

**Warning signs:** Memory usage grows unbounded during gameplay, or packet loss spikes without explanation.

### Pitfall 4: Incorrect WinDivert Filter Syntax
**What goes wrong:** WinDivertOpen fails with ERROR_INVALID_PARAMETER due to malformed filter string.

**Why it happens:** WinDivert filter language has specific syntax (similar to BPF but with differences); typos or operator errors cause runtime failures.

**How to avoid:**
- Compile filters using WinDivertHelperCompileFilter from windivert-sys during development
- Test filters incrementally: start with `true`, add constraints one by one
- Document filter strings in code with comments explaining each clause
- Consider building filters programmatically using helper functions to avoid string concatenation errors

**Warning signs:** WinDivertOpen returns false, GetLastError returns 87 (ERROR_INVALID_PARAMETER).

### Pitfall 5: Driver Installation Blocked by Antivirus
**What goes wrong:** WinDivertOpen fails with ERROR_DRIVER_BLOCKED even with admin privileges.

**Why it happens:** Antivirus or security software blocks unsigned or unknown drivers; WinDivert driver requires explicit allowlist.

**How to avoid:**
- Use signed WinDivert binaries from official distribution (2.2.2-A variant)
- Document antivirus compatibility for testing (Windows Defender, Norton, McAfee, etc.)
- Include ERROR_DRIVER_BLOCKED handling with user guidance: "Please add WinDivert64.sys to antivirus allowlist"
- Test on clean Windows 10/11 images with different AV configurations during proof-of-concept

**Warning signs:** GetLastError returns 1275 (ERROR_DRIVER_BLOCKED) after admin-verified attempt.

## Code Examples

Verified patterns from official sources:

### Basic WinDivert Capture Loop
```rust
// Source: https://docs.rs/windivert/0.6.0/windivert/struct.WinDivert.html
use windivert::{layer::Network, WinDivert, WinDivertFlags};

fn capture_mongo_traffic() -> windivert::error::Result<()> {
    // Filter: outbound TCP traffic to common MTGO ports (adjust after discovery)
    let filter = "outbound and tcp and (tcp.DstPort == 80 or tcp.DstPort == 443)";

    let handle = WinDivert::builder()
        .layer(Network::Network)
        .filter(filter)
        .flags(WinDivertFlags::SNIFF)  // Packet sniffing: copy, don't drop
        .build()?;

    loop {
        match handle.recv() {
            Ok((packet, address)) => {
                println!("Captured packet: {} bytes", packet.len());
                println!("Address: {:?}", address);
                // Parse packet headers with WinDivertHelperParsePacket...
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    Ok(())
}
```

### Tauri Command with Error Handling
```rust
// Source: https://v2.tauri.app/develop/calling-rust/
#[tauri::command]
async fn start_capture(
    state: tauri::State<'_>,
    window: tauri::Window,
) -> Result<CaptureStatus, String> {
    // Check admin privileges
    match is_running_as_admin() {
        Ok(true) => {}
        Ok(false) => {
            return Err("Application requires Administrator privileges to capture network traffic. Please restart as Administrator.".to_string());
        }
        Err(e) => {
            return Err(format!("Failed to detect admin privileges: {}", e));
        }
    }

    // Initialize WinDivert handle
    let handle = WinDivert::builder()
        .layer(Network::Network)
        .filter(MTGO_FILTER)  // Define in config
        .flags(WinDivertFlags::SNIFF)
        .build()
        .map_err(|e| format!("Failed to initialize WinDivert: {}", e))?;

    // Start capture task
    let tx = state.capture_tx.clone();
    tokio::spawn(async move {
        capture_loop(handle, tx).await;
    });

    Ok(CaptureStatus::Running)
}

#[tauri::command]
async fn stop_capture(state: tauri::State<'_>) -> Result<(), String> {
    let mut tx = state.capture_tx.lock().await;
    *tx = None;  // Signal shutdown
    Ok(())
}
```

### Filter Compilation for Development
```rust
// Source: https://reqrypt.org/windivert-doc.html#divert_helper_compile_filter
use windivert_sys::{
    WinDivertHelperCompileFilter, WinDivertLayer,
};

fn verify_filter(filter: &str) -> Result<(), String> {
    let layer = WinDivertLayer::WINDIVERT_LAYER_NETWORK;

    let mut compiled = std::ptr::null_mut();
    let compiled_len = std::ptr::null_mut();

    let success = unsafe {
        WinDivertHelperCompileFilter(
            filter.as_ptr() as *const i8,
            layer,
            0,  // flags
            &mut compiled,
            compiled_len,
        )
    };

    if success == 0 {
        return Err(format!("Invalid filter: {}", filter));
    }

    // Free compiled filter
    unsafe { LocalFree(compiled as *mut c_void) };

    Ok(())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| libpcap/WinPcap | WinDivert 2.2.2 | WinDivert 2.0 release (2019) | Loopback capture support eliminates blind spots on Windows |
| Manual privilege elevation | Pre-flight admin detection + clear error messaging | Industry standard practice (ongoing) | Better user experience, faster diagnosis |
| Unbounded queues | Tokio bounded MPSC channels | Tokio stable release (1.0, 2019) | Backpressure handling prevents memory leaks under burst traffic |
| Raw windivert-sys FFI | windivert crate with typestate | windivert 0.6.0 (2023) | Compile-time API correctness, safer abstractions |

**Deprecated/outdated:**
- **WinDivert 1.x**: Lacks loopback support, IPv6 support—use 2.2.2+
- **WinPcap**: Abandoned project; use WinDivert on Windows
- **windivert-sys < 0.10.0**: May have breaking changes; use 0.10.0+ for compatibility with windivert 0.6.0

## Open Questions

Things that couldn't be fully resolved:

1. **MTGO Traffic Characteristics**
   - What we know: MTGO uses TCP (likely HTTP/HTTPS), specific ports unknown, may use localhost
   - What's unclear: Exact server IPs, port numbers, protocol version, payload format
   - Recommendation: Proof-of-concept should capture actual MTGO traffic to discover characteristics. Start with broad filter `outbound and tcp`, then narrow based on captured traffic. Log remote IPs/ports to identify MTGO servers.

2. **Optimal Channel Capacity**
   - What we know: Tokio bounded channels provide backpressure, MTGO likely has gameplay bursts
   - What's unclear: Expected packet rate during typical gameplay (cards played, animations, etc.)
   - Recommendation: Start with 1000 packet capacity, monitor fullness events, adjust based on proof-of-concept metrics. Target <1% channel full events during normal gameplay.

3. **MTGO Protocol Documentation**
   - What we know: MTGO protocol is undocumented and proprietary, requires reverse-engineering
   - What's unclear: Any community reverse-engineering efforts exist? Protocol changes frequently?
   - Recommendation: Document captured packet headers during proof-of-concept, share with reverse-engineering community. Check GitHub for MTGO protocol research repos.

## Sources

### Primary (HIGH confidence)
- [WinDivert 2.2 Documentation](https://reqrypt.org/windivert-doc.html) - API reference, filter language, installation
- [WinDivert Official Site](https://reqrypt.org/windivert.html) - Binary downloads, version information
- [windivert crate 0.6.0](https://docs.rs/windivert/0.6.0/windivert/) - Rust API, 100% documented
- [windivert-sys crate 0.10.0](https://docs.rs/windivert-sys/0.10.0/windivert_sys/) - Raw FFI bindings
- [Tokio 1.35 MPSC Channel](https://docs.rs/tokio/1.35.1/tokio/sync/mpsc/fn.channel.html) - Bounded channel API

### Secondary (MEDIUM confidence)
- [Tauri 2.0 Create Project Guide](https://v2.tauri.app/start/create-project/) - Project scaffolding, CLI usage
- [windivert-rust GitHub README](https://github.com/Rubensei/windivert-rust) - Build instructions, usage patterns
- [Windows Security Token API](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Security/index.html) - Privilege detection (Microsoft official docs, but specific token elevation patterns inferred)

### Tertiary (LOW confidence)
- MTGO network traffic characteristics (undocumented; requires actual capture to verify)
- Optimal WinDivert filter strings for MTGO (unknown until traffic captured)
- MTGO protocol reverse-engineering resources (community efforts unknown without WebSearch)

## Metadata

**Confidence breakdown:**
- Standard stack: MEDIUM - WinDivert, windivert, Tauri, Tokio all well-documented from official sources
- Architecture: MEDIUM - Patterns derived from official docs, but MTGO-specific traffic unknown
- Pitfalls: HIGH - WinDivert errors documented, admin privilege patterns from Windows APIs, loopback issue from research summary

**Research date:** 2026-01-30
**Valid until:** 30 days (stable ecosystem; WinDivert 2.2.2 is stable release, Tauri 2.0 recently released)
