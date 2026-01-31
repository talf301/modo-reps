#[cfg(target_os = "windows")]
use windivert::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info, warn};

/// Channel capacity for packet capture (PERF-004)
///
/// Per RESEARCH.md Open Question 2, optimal capacity is unknown.
/// Starting with 1000 packets based on typical burst patterns.
/// Will adjust based on proof-of-concept metrics.
const CHANNEL_CAPACITY: usize = 1000;

/// Packet data with metadata
#[derive(Debug, Clone)]
#[cfg(target_os = "windows")]
pub struct CapturedPacket {
    pub data: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub length: usize,
}

/// Capture loop statistics
#[derive(Debug, Clone, Default)]
pub struct CaptureStats {
    pub packet_count: u64,
    pub bytes_captured: u64,
    pub last_packet_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Run the packet capture loop
///
/// This function creates a bounded MPSC channel and spawns an async task that:
/// 1. Receives packets from WinDivert handle
/// 2. Sends packets to the channel (with backpressure)
/// 3. Updates capture statistics
/// 4. Responds to shutdown signals
///
/// The channel provides backpressure (PERF-004): if the consumer (protocol decoder,
/// implemented in future phases) is too slow, the channel fills up and the producer
/// blocks on send(), preventing unbounded memory growth.
///
/// # Arguments
/// * `handle` - WinDivert handle for packet capture
/// * `shutdown_tx` - Optional shutdown signal sender (None = manual shutdown)
///
/// # Returns
/// Tuple of (packet receiver, shutdown handle) for control integration
#[cfg(target_os = "windows")]
pub fn capture_loop(
    handle: Arc<WinDivert<NetworkLayer>>,
    shutdown_tx: broadcast::Sender<()>,
) -> (mpsc::Receiver<CapturedPacket>, tokio::task::AbortHandle) {
    // Create bounded channel for packet flow (PERF-004)
    let (packet_tx, packet_rx) = mpsc::channel::<CapturedPacket>(CHANNEL_CAPACITY);

    // Spawn capture task
    let task = tokio::spawn(async move {
        info!("Packet capture loop started (channel capacity: {})", CHANNEL_CAPACITY);

        let mut stats = CaptureStats::default();
        let mut shutdown_rx = shutdown_tx.subscribe();
        let mut last_throughput_check = Instant::now();
        let mut bytes_at_last_check = 0u64;

        loop {
            // Check for shutdown signal
            if shutdown_rx.try_recv().is_ok() {
                info!("Shutdown signal received, stopping capture");
                break;
            }

            // Receive packet from WinDivert (with timeout for shutdown check)
            // Need to use spawn_blocking with the buffer and handle
            let handle_clone = Arc::clone(&handle);
            let recv_result = tokio::task::spawn_blocking(move || {
                let mut buffer = Box::new([0u8; 1500]);
                handle_clone.recv_wait(&mut *buffer, 100).map(|opt| opt.map(|p| {
                    // Convert packet data to owned Vec
                    WinDivertPacket {
                        address: p.address,
                        data: std::borrow::Cow::from(p.data.to_vec()),
                    }
                }))
            }).await;
            
            match recv_result {
                Ok(Ok(Some(packet))) => {
                    // Capture successful
                    let timestamp = chrono::Utc::now();
                    let length = packet.data.len();

                    // Update statistics
                    stats.packet_count += 1;
                    stats.bytes_captured += length as u64;
                    stats.last_packet_time = Some(timestamp);

                    // Create captured packet structure
                    let captured = CapturedPacket {
                        data: packet.data.to_vec(),
                        timestamp,
                        length,
                    };

                    // Send to channel with backpressure
                    match packet_tx.send(captured).await {
                        Ok(_) => {
                            debug!("Packet {} captured: {} bytes", stats.packet_count, length);
                        }
                        Err(_) => {
                            // Channel closed, stop capture
                            warn!("Packet channel closed, stopping capture");
                            break;
                        }
                    }

                    // Calculate and log throughput every second
                    let elapsed = last_throughput_check.elapsed();
                    if elapsed >= Duration::from_secs(1) {
                        let bytes_delta = stats.bytes_captured - bytes_at_last_check;
                        let throughput = bytes_delta as f64 / elapsed.as_secs_f64();

                        if stats.packet_count % 100 == 0 {
                            info!(
                                "Captured {} packets, {:.2} bytes/s",
                                stats.packet_count, throughput
                            );
                        }

                        // Log traffic volume against 10MB/hour threshold (Success Criterion #5)
                        // 10MB/hour = 10 * 1024 * 1024 bytes / 3600 seconds ≈ 2913 bytes/s
                        const MAX_BYTES_PER_SECOND: f64 = 10.0 * 1024.0 * 1024.0 / 3600.0;

                        if stats.packet_count % 600 == 0 {  // Log every 600 packets (≈ every 6 seconds at typical rates)
                            if throughput > MAX_BYTES_PER_SECOND {
                                warn!(
                                    "Traffic volume {:.2} bytes/s exceeds 10MB/hour threshold ({:.2} bytes/s). Filter: '{}'. Consider refining filter to specific MTGO servers.",
                                    throughput, MAX_BYTES_PER_SECOND, crate::capture::filter::MTGO_FILTER
                                );
                            } else {
                                info!(
                                    "Traffic volume {:.2} bytes/s within 10MB/hour threshold ({:.2} bytes/s). Filter: '{}'",
                                    throughput, MAX_BYTES_PER_SECOND, crate::capture::filter::MTGO_FILTER
                                );
                            }
                        }

                        bytes_at_last_check = stats.bytes_captured;
                        last_throughput_check = Instant::now();
                    }
                }
                Ok(Ok(None)) => {
                    // Timeout - continue loop (allows shutdown check)
                    continue;
                }
                Ok(Err(e)) => {
                    error!("Error receiving packet: {}", e);
                    break;
                }
                Err(e) => {
                    error!("Task join error: {}", e);
                    break;
                }
            }
        }

        info!("Packet capture loop stopped. Total: {} packets, {} bytes",
              stats.packet_count, stats.bytes_captured);
    });

    // Return receiver and abort handle for control
    let abort_handle = task.abort_handle();

    // Note: For now, we're NOT returning the packet receiver to the protocol decoder
    // because Phase 1 is just capture infrastructure. In Phase 2, we'll connect
    // the receiver to the protocol decoder.

    (packet_rx, abort_handle)
}

/// Stub implementation for non-Windows targets (development only)
#[cfg(not(target_os = "windows"))]
pub struct CapturedPacket;

#[cfg(not(target_os = "windows"))]
pub struct CaptureStats;

#[cfg(not(target_os = "windows"))]
pub fn capture_loop(
    _handle: (),
    _shutdown_tx: (),
) -> (tokio::sync::mpsc::Receiver<CapturedPacket>, tokio::task::AbortHandle) {
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let handle = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        drop(tx);
    });
    (rx, handle.abort_handle())
}
