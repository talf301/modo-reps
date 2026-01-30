# MTGO Replay Capture

Capture MTGO replays with automatic sideboard extraction, enabling personal analysis and easy file-based sharing.

## Project Status

**Current Phase:** 1 - Capture Infrastructure & Proof of Concept

Building a Windows-only desktop application that intercepts Magic: The Gathering Online's network protocol to capture game replays and extract sideboard data.

## Tech Stack

- Rust 1.80+ (core language)
- Tauri 2.0 (desktop framework)
- Tokio 1.35+ (async runtime)
- WinDivert (Windows packet capture)
- Vanilla JS (frontend)

## Building

### From Windows (Native)

```bash
# Install prerequisites
cargo install tauri-cli

# Build
cargo tauri build

# Executable at: src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/MTGO_Replay_Capture_*.exe
```

### From Linux (Cross-compile)

**Prerequisites:**
```bash
# Add Windows target
rustup target add x86_64-pc-windows-msvc

# Install cross-compilation tool
cargo install cargo-xwin
```

**Build:**
```bash
# Run build script
./build-windows.sh

# Or manually:
cargo xwin build --target x86_64-pc-windows-msvc --release

# Executable at: src-tauri/target/x86_64-pc-windows-msvc/release/mtgo-replay.exe
```

### Development Build

```bash
# Run development server (requires Tauri CLI)
cargo tauri dev
```

## Running on Windows

1. **Install WinDivert Driver:**
   - Download from: https://reqrypt.org/windivert.html
   - Place `WinDivert.dll` and `WinDivert64.sys` in application directory

2. **Run as Administrator:**
   - Right-click the executable
   - Select "Run as Administrator"
   - Required for WinDivert packet capture

3. **Verify Requirements:**
   - Admin status: ✓ Yes
   - WinDivert Driver: ✓ Found

4. **Test Capture:**
   - Click "Start Capture"
   - Verify packet count increases
   - Click "Stop Capture"

## Development

### Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

### Project Structure

```
mtgo-replay/
├── src/                    # Frontend (vanilla JS)
│   ├── index.html
│   ├── capture-status.js
│   └── assets/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── capture/         # Packet capture modules
│   │   ├── common/         # Error types
│   │   └── ui/            # Tauri commands
│   └── Cargo.toml
└── build-windows.sh         # Cross-compilation script
```

## Architecture

- **Layered Architecture:** Clear boundaries between capture, protocol, and application logic
- **Event-Driven:** Components communicate via events for loose coupling
- **Producer-Consumer:** Packet capture produces, protocol decoder consumes with bounded buffer

## Known Limitations

- Windows-only application (MTGO is Windows-only)
- Requires Administrator privileges for packet capture
- WinDivert driver installation required

## Future Phases

1. ✅ Phase 1: Capture Infrastructure (in progress)
2. ⏳ Phase 2: Protocol Reverse Engineering & Replay Format
3. ⏳ Phase 3: Game State Reconstruction
4. ⏳ Phase 4: Sideboard Detection
5. ⏳ Phase 5: Replay Playback & Analytics
6. ⏳ Phase 6: Performance Optimization
7. ⏳ Phase 7: Advanced Features
