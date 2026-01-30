/// MTGO traffic filter string
///
/// This filter captures all outbound TCP traffic. Starting with a broad filter
/// allows us to capture actual MTGO traffic and identify specific server IPs/ports
/// during the proof-of-concept phase.
///
/// Per RESEARCH.md Open Question 1, exact MTGO server characteristics are unknown.
/// We will log captured IPs/ports and narrow the filter in future phases based on
/// actual MTGO traffic patterns.
///
/// Filter components:
/// - "outbound": Only capture outbound traffic from local machine
/// - "tcp": Only capture TCP packets (MTGO uses TCP, likely HTTP/HTTPS)
/// - No port restrictions initially - broad capture for discovery
///
/// Future refinement: After discovering MTGO servers, update filter to:
/// "outbound and tcp and (ip.DstAddr == SERVER_IP_1 or ip.DstAddr == SERVER_IP_2)"
pub const MTGO_FILTER: &str = "outbound and tcp";

/// Analyze captured IP addresses and ports for filter refinement
///
/// This function processes captured packet metadata to identify MTGO server
/// characteristics (IP ranges, specific ports) for refining the BPF filter.
///
/// The filter can then be updated from "outbound and tcp" to:
/// "outbound and tcp and (ip.DstAddr == MTGO_IP_1 or ip.DstAddr == MTGO_IP_2)"
///
/// This addresses Success Criterion #5: "BPF filter successfully filters MTGO server traffic,
/// reducing captured packets to < 10MB/hour"
///
/// # Arguments
/// * `captured_ips` - Vec of (ip_address, port) tuples from captured packets
///
/// # Returns
/// Suggested refined filter string, or current filter if insufficient data
#[cfg(target_os = "windows")]
pub fn analyze_and_suggest_refined_filter(captured_ips: Vec<(String, u16)>) -> String {
    if captured_ips.len() < 100 {
        // Not enough data for meaningful analysis
        return MTGO_FILTER.to_string();
    }

    // Count occurrences of each (ip, port) pair
    use std::collections::HashMap;
    let mut ip_port_counts: HashMap<(String, u16), usize> = HashMap::new();

    for (ip, port) in captured_ips {
        *ip_port_counts.entry((ip, port)).or_insert(0) += 1;
    }

    // Identify top IP/port pairs (likely MTGO servers)
    let mut pairs: Vec<_> = ip_port_counts.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1));

    // Take top 5 most common pairs
    let top_pairs: Vec<_> = pairs.into_iter().take(5).collect();

    if top_pairs.is_empty() {
        return MTGO_FILTER.to_string();
    }

    // Build refined filter: "outbound and tcp and (ip.DstAddr == IP1 or ip.DstAddr == IP2 ...)"
    let ip_conditions: Vec<String> = top_pairs
        .iter()
        .map(|((ip, _port), _count)| format!("ip.DstAddr == {}", ip))
        .collect();

    let refined_filter = format!(
        "outbound and tcp and ({})",
        ip_conditions.join(" or ")
    );

    refined_filter
}

/// Stub for non-Windows targets
#[cfg(not(target_os = "windows"))]
pub fn analyze_and_suggest_refined_filter(_captured_ips: Vec<(String, u16)>) -> String {
    MTGO_FILTER.to_string()
}
