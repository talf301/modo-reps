---
phase: 01-capture-infrastructure-proof-of-concept
plan: 03
type: execute
completed: true
---

# Phase 1 Plan 3: WinDivert Handle and Filter Management Summary

Create WinDivert handle and filter management module to initialize packet capture with proper filtering for MTGO traffic.

## Overview

Successfully implemented WinDivert handle management with typestate builder pattern and automatic cleanup via RAII. The module includes broad TCP outbound filter for discovery phase and filter analysis function for future BPF refinement based on captured traffic patterns.

## Tasks Completed

| Task | Name | Commit | Status |
| ---- | ----- | ------- | ------ |
| 1 | Define MTGO traffic filter and create WinDivert handle | 7945e7f | Complete |
| 2 | Implement handle cleanup on shutdown | 7945e7f | Complete |
| 3 | Add filter analysis task for BPF refinement | 7945e7f | Complete |

**Commits:**
- `7945e7f`: feat(01-03): create WinDivert handle and filter management module

## Components Implemented

### 1. MTGO Traffic Filter (`src-tauri/src/capture/filter.rs`)

**Filter Constant:**
```rust
pub const MTGO_FILTER: &str = "outbound and tcp";
```

**Design Rationale:**
- **Broad initial filter**: Captures all outbound TCP traffic for MTGO server discovery
- **Avoids hardcoded ports**: MTGO server characteristics unknown (RESEARCH.md Open Question 1)
- **Future refinement**: Will narrow to specific IPs after analyzing captured traffic

**Filter Analysis Function:**
- `analyze_and_suggest_refined_filter()`: Processes captured (IP, port) tuples
- Returns refined filter string with top 5 most common IP addresses
- Requires minimum 100 captured packets before suggesting refinement
- Suggestion only - manual review required before applying

### 2. WinDivert Handle Management (`src-tauri/src/capture/handle.rs`)

**CaptureHandle Structure:**
```rust
pub struct CaptureHandle {
    inner: Arc<WinDivert<Network>>,
}
```

**Key Features:**
- **Arc sharing**: Allows safe sharing between capture loop and control commands
- **Typestate builder pattern**: Ensures correct WinDivert API usage at compile time
- **Filter validation**: Basic syntax check (balanced parentheses) before creation
- **SNIFF mode**: Copies packets without dropping them (non-intrusive capture)

**Methods:**
- `new()`: Creates WinDivert handle with MTGO filter and SNIFF flag
- `inner()`: Returns reference to inner WinDivert handle
- `clone_handle()`: Clones Arc for task sharing

### 3. RAII Cleanup (Drop Trait)

**Automatic Handle Closure:**
```rust
impl Drop for CaptureHandle {
    fn drop(&mut self) {
        // Arc<WinDivert<Network>> automatically closes when refcount reaches zero
    }
}
```

**Benefits:**
- No manual cleanup required
- Prevents resource leaks on panic or early return
- Ensures clean shutdown even on unexpected errors

### 4. Cross-Platform Support

**Windows Implementation:**
- Full WinDivert integration with typestate pattern
- Filter syntax validation
- Arc-based handle sharing
- RAII cleanup via Drop

**Non-Windows Stubs:**
- `CaptureHandle` struct with empty implementation
- `new()` returns Ok without actual capture
- Allows development on non-Windows platforms
- Production builds target Windows only

## Key Files Created

### Created
- `src-tauri/src/capture/filter.rs`: MTGO filter constant and analysis function
- `src-tauri/src/capture/handle.rs`: WinDivert handle wrapper with Arc and RAII

### Modified
- `src-tauri/src/capture/mod.rs`: Added `handle` and `filter` module declarations

## Architecture Decisions

### 1. Broad Initial Filter Strategy

**Decision:** Start with "outbound and tcp" filter, narrow after discovery

**Rationale:**
- MTGO server IPs/ports unknown (RESEARCH.md Open Question 1)
- Broad capture allows traffic pattern discovery
- Filter analysis function suggests refinement after 100+ packets

**Trade-offs:**
- **Pro**: Captures all potential MTGO traffic, enables discovery
- **Con**: Higher packet rate initially, may need more filtering later

**Future refinement:**
```
"outbound and tcp and (ip.DstAddr == SERVER_IP_1 or ip.DstAddr == SERVER_IP_2)"
```

### 2. Arc-based Handle Sharing

**Decision:** Use `Arc<WinDivert<Network>>` for handle sharing

**Rationale:**
- Capture loop and control commands both need handle access
- Arc provides thread-safe reference counting
- Avoids complex lifetime management
- RAII ensures cleanup when last reference drops

**Benefits:**
- No manual close() calls needed
- Safe sharing across async tasks
- Leak-free even on panic

### 3. Typestate Builder Pattern

**Decision:** Use WinDivert's typestate builder for handle creation

**Rationale:**
- Compile-time API correctness (RESEARCH.md Pattern 1)
- Clear initialization sequence
- Filter syntax validation at build time
- Avoids Pitfall 4 (incorrect filter syntax)

**Implementation:**
```rust
let handle = WinDivert::builder()
    .layer(Network::Network)
    .filter(MTGO_FILTER)
    .flags(WinDivertFlags::SNIFF)
    .build()?;
```

### 4. SNIFF Mode for Non-Intrusive Capture

**Decision:** Use WinDivertFlags::SNIFF for packet capture

**Rationale:**
- Copies packets without dropping them
- MTGO traffic continues normally
- No impact on gameplay performance
- Safe for production use

**Alternatives considered:**
- `DROP`: Drops captured packets (would break MTGO)
- No flag: Default behavior varies by layer

## Deviations from Plan

None. Plan executed exactly as written.

All three tasks were completed in a single commit because:
- Task 1's action included creating all three files with complete implementations
- Task 2 (Drop implementation) was included in Task 1's handle.rs
- Task 3 (filter analysis) was included in Task 1's filter.rs

## Technical Notes

### Filter Syntax Validation

**Implementation:**
```rust
let mut paren_count = 0;
for ch in MTGO_FILTER.chars() {
    match ch {
        '(' => paren_count += 1,
        ')' => paren_count -= 1,
        _ => {}
    }
}
if paren_count != 0 {
    return Err(CaptureError::CaptureLoopError(...));
}
```

**Rationale:**
- Catches unbalanced parentheses early
- Basic validation before expensive WinDivert::builder().build()
- Full syntax validation performed by WinDivert builder itself

**Limitation:**
- Only checks parentheses, not full WinDivert filter language
- Complex filters may still fail at build() stage
- WinDivert::builder().build() provides full validation

### Filter Analysis Logic

**Algorithm:**
1. Collect (IP, port) tuples from captured packets
2. Count occurrences of each unique pair
3. Sort by frequency (descending)
4. Take top 5 most common pairs
5. Build filter: `"outbound and tcp and (ip.DstAddr == IP1 or ip.DstAddr == IP2 ...)"`

**Safeguards:**
- Minimum 100 packets before suggesting refinement
- Returns current filter if insufficient data
- Suggestion only - requires manual review
- No automatic filter updates (prevents filtering out legitimate traffic)

**Future enhancement:**
- Add port-based filtering refinement
- Consider IP ranges (CIDR notation)
- Exclude known non-MTGO servers

### Error Handling

**Errors propagated:**
- `CaptureError::CaptureLoopError`: Filter syntax validation failures
- `CaptureError::WinDivertInitFailed`: WinDivert::builder().build() errors (from thiserror)

**No errors generated by this module directly:**
- Filter analysis function returns String (no allocation failures expected)
- Drop implementation cannot fail (by design)

## Metrics

- **Duration:** 4 minutes 23 seconds (263 seconds)
- **Completed:** 2026-01-30
- **Tasks:** 3/3 completed
- **Commits:** 1
- **Lines added:** 190 lines (3 files)
- **Functions implemented:** 5 (including stubs)

## Success Criteria Met

✅ MTGO filter defined for outbound TCP traffic
✅ WinDivert handle creation with typestate builder pattern and filter validation
✅ SNIFF flag configured (copy, don't drop packets)
✅ RAII cleanup via Drop trait implemented
✅ Filter analysis function for BPF refinement created
✅ Project syntax verified (Linux development limitations noted)

## Verification Results

**Plan Verification Criteria:**
1. ✅ `MTGO_FILTER` constant defined: `"outbound and tcp"`
2. ✅ `WinDivert::builder()` used with typestate pattern
3. ✅ `Network::Network` layer configured (WINDIVERT_LAYER_NETWORK)
4. ✅ `WinDivertFlags::SNIFF` configured for packet copying
5. ✅ `impl Drop` for CaptureHandle with RAII cleanup
6. ✅ `analyze_and_suggest_refined_filter()` function created

**Grep verification passed:**
- MTGO_FILTER found in filter.rs (line 18)
- CaptureHandle struct defined in handle.rs (line 11)
- WinDivert::builder() call in handle.rs (line 49)
- Network::Network layer in handle.rs (line 50)
- WinDivertFlags::SNIFF in handle.rs (line 52)
- impl Drop for CaptureHandle in handle.rs (lines 81, 104)
- analyze_and_suggest_refined_filter in filter.rs (lines 37, 78)

**Compilation Status:**
- Code syntax correct (verified with rustc)
- Full compilation blocked by Linux environment (missing GUI dependencies)
- Windows target compilation not tested (requires Windows toolchain)
- Expected limitation per plan design (Windows-only production)

## Next Phase Readiness

**Completed:**
- ✅ WinDivert handle management module
- ✅ MTGO traffic filter (broad discovery phase)
- ✅ Filter analysis for BPF refinement
- ✅ RAII cleanup implementation

**Ready for Next Steps:**
- 01-04A: Capture status UI (types and commands already created in 01-04A work)
- 01-04B: Start/stop capture commands using CaptureHandle
- 01-05: Proof of concept with actual MTGO traffic capture
- Filter refinement based on captured IPs/ports

**Dependencies on Previous Work:**
- 01-01: Tauri project initialized ✅
- 01-02: Admin privilege detection ✅
- 01-03: WinDivert handle and filter ✅ (this plan)

**Integration Points:**
- CaptureHandle::new() will be called in start_capture command
- Arc<WinDivert<Network>> shared between capture loop and stop command
- analyze_and_suggest_refined_filter() called after capturing 100+ packets
- Drop trait ensures cleanup on stop command

## Blockers

**None.** All code implemented and verified syntactically.

**Known Limitations:**
- Windows target compilation not verified (requires Windows environment)
- WinDivert driver installation not tested (requires Windows + admin)
- Actual MTGO traffic patterns unknown (will be discovered in 01-05)

## Authentication Gates

None encountered during this phase.

## Appendix: Code References

### WinDivert Typestate Pattern (RESEARCH.md Pattern 1)

**Before (without typestate):**
```rust
// Error-prone: Can forget to set layer, filter, or flags
let handle = WinDivert::open("outbound and tcp", ...)?;
```

**After (with typestate):**
```rust
// Compile-time correctness: Builder ensures proper initialization
let handle = WinDivert::builder()
    .layer(Network::Network)
    .filter(MTGO_FILTER)
    .flags(WinDivertFlags::SNIFF)
    .build()?;
```

### Filter Language Examples

**Current (broad):**
```
outbound and tcp
```

**Future (refined):**
```
outbound and tcp and (ip.DstAddr == 199.108.0.101 or ip.DstAddr == 199.108.0.102)
```

**Future (with ports):**
```
outbound and tcp and (ip.DstAddr == 199.108.0.101) and (tcp.DstPort == 4761 or tcp.DstPort == 4762)
```

### RAII Benefits

**Without RAII (manual cleanup):**
```rust
fn start_capture() -> Result<(), CaptureError> {
    let handle = WinDivert::open(...)?;

    // Complex logic with early returns
    if some_condition {
        // Must remember to call handle.close() before returning
        handle.close()?;
        return Err(...);
    }

    if another_condition {
        // Must remember to call handle.close() before returning
        handle.close()?;
        return Err(...);
    }

    // Finally block or manual cleanup needed
    handle.close()?;
    Ok(())
}
```

**With RAII (automatic cleanup):**
```rust
fn start_capture() -> Result<(), CaptureError> {
    let handle = CaptureHandle::new()?;

    // Complex logic with early returns
    if some_condition {
        return Err(...);  // handle.drop() called automatically
    }

    if another_condition {
        return Err(...);  // handle.drop() called automatically
    }

    Ok(())  // handle.drop() called automatically at scope end
}
```

### Cross-Platform Compilation Example

```rust
#[cfg(target_os = "windows")]
pub fn new() -> Result<Self, CaptureError> {
    // Real WinDivert initialization
    let handle = WinDivert::builder()
        .layer(Network::Network)
        .filter(MTGO_FILTER)
        .flags(WinDivertFlags::SNIFF)
        .build()?;
    Ok(CaptureHandle { inner: Arc::new(handle) })
}

#[cfg(not(target_os = "windows"))]
pub fn new() -> Result<Self, CaptureError> {
    // Stub for development on macOS/Linux/WSL
    Ok(CaptureHandle)
}
```
