#!/bin/bash
# Build Windows executable from Linux using cross-compilation

# Install cross-compilation toolchain
# Run once:
#   rustup target add x86_64-pc-windows-msvc

# Install cargo-xwin for Windows cross-compilation
# Run once:
#   cargo install cargo-xwin

# Build for Windows
echo "Building Windows x64 from Linux..."
cargo xwin build --target x86_64-pc-windows-msvc --release

# The executable will be at:
# src-tauri/target/x86_64-pc-windows-msvc/release/mtgo-replay.exe

echo "Build complete!"
echo "Executable: src-tauri/target/x86_64-pc-windows-msvc/release/mtgo-replay.exe"
echo ""
echo "Transfer to Windows machine and run as Administrator"
