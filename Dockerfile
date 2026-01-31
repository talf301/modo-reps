# Use Wine with Rust cross-compilation for Windows

FROM ghcr.io/cross/rust-mingw:latest

WORKDIR /app

# Install Wine (for Windows binary testing if needed)
RUN apt-get update && apt-get install -y wine64

# Copy project files
COPY . /app/

# Build for Windows
RUN cargo build --target x86_64-pc-windows-gnu --release

# Output executable location
RUN echo "Executable built at: target/x86_64-pc-windows-gnu/release/mtgo-replay.exe"
