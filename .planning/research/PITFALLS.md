# Domain Pitfalls

**Domain:** MTGO (Magic: The Gathering Online) Replay Capture
**Researched:** 2026-01-28
**Confidence:** MEDIUM

## Critical Pitfalls

Mistakes that cause rewrites or major issues.

### Pitfall 1: Loopback Traffic Capture Blindness

**What goes wrong:**
Developers assume they can capture localhost/loopback traffic the same way as network traffic, but discover that the game client communicates with servers via localhost (127.0.0.1) and their packet capture library returns nothing. Months of work building a capture system that captures zero packets.

**Why it happens:**
- Standard Windows packet capture libraries (WinPcap/Npcap) by default only capture on physical network interfaces
- Loopback traffic never touches the physical network stack
- MTGO client may route through localhost for some connections (e.g., local authentication or cache servers)

**Consequences:**
- Capture system works in testing (with real network traffic) but fails in production
- User reports: "I start MTGO and run your tool, but it never detects any packets"
- Development time wasted debugging non-existent network issues
- Complete architecture redesign may be needed if initial assumption was wrong

**Prevention:**
1. **Early phase research:** Before writing any capture code, verify where MTGO traffic actually flows:
    ```bash
    # On Windows with Wireshark
    # Install Npcap with loopback support enabled or use WinDivert
    # Use Wireshark to check for MTGO traffic patterns
    # Check if traffic appears on physical interfaces or loopback
    ```

2. **Use WinDivert for capture:**
    - WinDivert captures both physical and loopback traffic on Windows
    - No need to switch between capture modes
    - Auto-detect which interfaces show MTGO traffic

3. **Document MTGO's actual communication pattern:**
    - Do packet capture during actual gameplay
    - Document which servers MTGO connects to
    - Document whether any traffic is loopback
    - Note differences across MTGO client versions

**Detection:**
- Test capture tool with actual MTGO client running (not just synthetic traffic)
- Verify packet capture shows game-related traffic (look for MTGO server IPs or ports)
- If you see NO packets when MTGO is running, check if traffic is on loopback interface
- Use netstat/connections to see if MTGO is using 127.0.0.1

**Phase to address:**
Phase 1 (Research & Proof of Concept) - Must verify traffic paths before architecture decisions

---

### Pitfall 2: Protocol Version Fragility

**What goes wrong:**
Developers spend months reverse-engineering the MTGO protocol, build a parser that works perfectly, then Wizards of the Coast releases an MTGO client update and the entire parser breaks. Every game packet fails to parse, users complain, and the project needs complete re-engineering.

**Why it happens:**
- MTGO protocol is undocumented and proprietary
- Wizards can change packet formats, field ordering, or message IDs without notice
- No version handshake or backward compatibility guaranteed by the server
- Parser built for version X may interpret version Y's packets as corrupted data
- Reverse-engineering from captured traffic only reveals current state

**Consequences:**
- Tool becomes useless after each MTGO update
- High maintenance burden - constant re-reverse-engineering
- Users abandon tool due to unreliability
- Developers burn out chasing protocol changes
- Inability to support older replays after update

**Prevention:**

1. **Design for multiple protocol versions:**
   ```python
   # Pseudocode for version-aware parsing
   class PacketParser:
       def __init__(self, version):
           self.version = version
           self.message_handlers = self.get_handlers_for_version(version)

       def get_handlers_for_version(self, version):
           # Return different parsers based on version
           if version < 4.5:
               return LEGACY_HANDLERS
           else:
               return CURRENT_HANDLERS
   ```

2. **Version detection at capture time:**
   - Store MTGO client version in replay file header
   - Capture version from initial handshake or client executable
   - Log version for each replay session

3. **Protocol fuzzing / extensibility:**
   - Design parsers to tolerate unknown fields
   - Skip bytes rather than fail on unexpected data
   - Log unknown packet types for later analysis
   - Separate strict parsing (for display) from permissive parsing (for capture)

4. **Community protocol documentation:**
   - Document reverse-engineered protocol publicly
   - Create community repository for protocol changes
   - Share packet captures between updates
   - Build tooling to detect protocol changes automatically

5. **Replay file format versioning:**
   - Include replay file version independent of MTGO client version
   - Store both MTGO version and replay format version
   - Allow migration between replay format versions
   - Design format to survive MTGO protocol changes

**Detection:**
- Parser throws exceptions for packets that previously worked
- Replay files from new MTGO client fail to load
- Users report "after MTGO update, your tool stopped working"
- Packet captures show new/unknown message types

**Phase to address:**
Phase 2 (Protocol Research & Design) - Must design for version flexibility from start

---

### Pitfall 3: Platform-Specific Capture Failures

**What goes wrong:**
Tool works perfectly on Windows (where MTGO primarily runs), but users on Windows 10/11 can't capture anything. Or, tool fails on specific Windows versions. Capturing permissions, driver installation, or interface discovery fails silently.

**Why it happens:**
- Packet capture requires kernel-level drivers (WinDivert on Windows)
- Different Windows versions have different security requirements (UAC, driver signing, driver loading)
- Administrator privileges required but not documented
- Antivirus/EDR software blocks packet capture drivers
- Modern Windows (11) has tighter driver signing requirements

**Consequences:**
- Tool only works on developer's machine, nowhere else
- Users confused by "permission denied" or "no capture devices found"
- Negative reviews: "Installed, ran as admin, nothing captured"
- Support burden increases with platform-specific issues
- May need to ship separate installers for different Windows versions

**Prevention:**

1. **Administrator privilege detection and handling:**
    ```python
    import ctypes
    import os

    def is_admin():
        try:
            return ctypes.windll.shell32.IsUserAnAdmin()
        except:
            return False

    def setup_capture():
        if not is_admin():
            print("Packet capture requires Administrator privileges")
            print("Please restart as Administrator")
            # Offer to restart with elevation
            relaunch_as_admin()
        else:
            # Proceed with capture setup
    ```

2. **Driver installation verification:**
    ```python
    def verify_windivert_installed():
        # Check for WinDivert64.sys in system path
        # Check if WinDivert service is running
        # Guide user to install if missing
    ```

3. **Graceful degradation for capture failures:**
    - Detect when WinDivert driver is missing/blocked
    - Provide clear error messages with specific fixes
    - Bundle WinDivert64.sys with installer
    - Test on Windows 10 and Windows 11 during development

4. **Test matrix:**
    - Windows 10 x64, various updates
    - Windows 11 x64, various builds
    - With and without antivirus/EDR enabled

5. **Bundling considerations:**
    - Bundle WinDivert driver with installer
    - Document installation requirements

**Detection:**
- `pcap_open_live()` returns NULL or error
- `pcap_findalldevs()` returns empty list
- Error messages like "generic error" without specifics
- Tool works on dev machine but fails on fresh Windows install

**Phase to address:**
Phase 1 (Proof of Concept) - Must verify capture works on clean Windows installs

---

### Pitfall 4: Game State Reconstruction Ambiguity

**What goes wrong:**
Tool captures packets and can display some game state, but the reconstructed game is missing key details: player life totals, card zones, or phase information. Replays look "partial" or incorrect. Users report "my replay says I lost but I won" because state reconstruction is wrong.

**Why it happens:**
- Network packets only show what the server sent to the client
- Some game state may be client-side only (e.g., card selection highlight, player perspective)
- Packets may be out of order or lost during capture
- Server may send incremental updates, not full state
- Server may compress or omit "derived" state that client calculates
- Race conditions: capture misses a critical state-change packet

**Consequences:**
- Replays don't accurately reflect actual gameplay
- Analysis features are unreliable (e.g., "what did I have on board" shows wrong cards)
- Sideboard extraction fails (sideboarding may not generate distinct network messages)
- Users lose trust in the tool
- Competitors with better reconstruction gain advantage

**Prevention:**

1. **Understand state vs. event model:**
   - Identify which game aspects are state (board contents, life totals)
   - Identify which are events (card played, attack declared)
   - Capture both full state snapshots and event deltas
   - Reconstruct state by applying events to base state

2. **Packet reordering and loss handling:**
   ```python
   # Pseudocode for reliable reconstruction
   class GameStateReconstructor:
       def __init__(self):
           self.sequences = {}  # Track packet sequences
           self.state = InitialGameState()

       def on_packet(self, packet):
           seq_num = packet.sequence_number
           self.sequences[seq_num] = packet

           # Apply in order
           while (self.next_expected in self.sequences):
               packet = self.sequences.pop(self.next_expected)
               self.apply_packet_to_state(packet)
               self.next_expected += 1

       def apply_packet_to_state(self, packet):
           # Apply delta or full state update
           if packet.is_full_state:
               self.state = packet.state
           elif packet.is_delta:
               packet.delta.apply_to(self.state)
   ```

3. **Capture redundancy:**
   - Capture packets from multiple perspectives if possible (not applicable to MTGO, but principle)
   - Capture multiple passes if capture is unreliable
   - Validate reconstructed state against invariants (e.g., deck size constant, no negative mana)

4. **Sideboard detection strategy:**
   - Sideboarding may manifest as specific message types (e.g., "SideboardSetChanged")
   - May appear as card zone changes during match setup phase
   - May need to correlate deck list messages with in-game card movements
   - Must identify sideboard as separate zone, not just "out of game"

5. **Ambiguity logging:**
   - Log when reconstruction has gaps (missing sequences)
   - Flag reconstructed state as "incomplete" if packets lost
   - Allow users to annotate replays with corrections
   - Include checksums or hashes to detect state corruption

**Detection:**
- Reconstructed game state has impossible values (negative life, wrong deck size)
- Replays show "unknown" or default values for key attributes
- Comparing two replays of same game shows divergence
- User reports incorrect game outcome in replay

**Phase to address:**
Phase 3 (Game State Reconstruction) - Core complexity that needs thorough validation

---

### Pitfall 5: File Format Brittleness

**What goes wrong:**
Replay files are saved in a binary format that works, but the format is undocumented and tightly coupled to current implementation. Adding a new feature (e.g., "player notes" or "deck tags") breaks backward compatibility. Old replay files can't be loaded after a minor update.

**Why it happens:**
- Binary formats require precise byte offsets for field locations
- Adding/removing fields shifts all subsequent offsets
- No version field or format identifier at file start
- Assumption that "replay files are internal, version lock is okay"
- No schema evolution plan from the start

**Consequences:**
- Tool update breaks all existing user replays
- Users can't access their replay history
- Forced migration path (convert all old replays to new format)
- Data loss risk during migration
- Reputation damage: "update deleted my replays"

**Prevention:**

1. **Versioned file format from day one:**
   ```
   [File Header - Fixed Size]
   - Magic bytes: "MTGOREPLAY" (10 bytes)
   - File version: 2 bytes (little endian)
   - MTGO client version: 4 bytes (major.minor.build.rev)
   - Timestamp: 8 bytes (unix epoch)
   - Replay metadata size: 4 bytes
   [Metadata Section - Variable]
   [Game Data Section - Variable]
   ```

2. **Self-describing sections:**
   ```
   Section Header:
   - Section ID: 2 bytes (e.g., 0x0001 = game states, 0x0002 = deck info)
   - Section length: 4 bytes
   - Section version: 2 bytes (allows per-section evolution)
   ```

3. **Backward compatibility strategy:**
   - File format v2 reader can read v1 files (skips unknown sections)
   - File format v2 writer writes v2 format with v1-compatible header
   - Maintain legacy reader code for v1 files
   - Never remove support for old formats until users have migrated

4. **Human-readable fallback:**
   - Export/import to JSON or other text format
   - Allow manual editing for recovery
   - Include checksum to detect corruption
   - Document format publicly for third-party tools

5. **Migration tools:**
   - Build command-line tool to convert v1 → v2, v1 → v3, etc.
   - Run automatically on first open of old replay
   - Create backup before conversion
   - Log conversion results

**Detection:**
- Old replay files fail to load after update
- Error messages about "unknown format" or "invalid header"
- Users report "my replay files disappeared after update"
- Version mismatch errors when reading replays

**Phase to address:**
Phase 2 (Protocol & Format Design) - Must design extensible format before implementation

---

## Moderate Pitfalls

Mistakes that cause delays or technical debt.

### Pitfall 1: Insufficient Packet Filtering

**What goes wrong:**
Capture system collects all packets from the network interface, including non-MTGO traffic. Performance degrades, replay files are huge, and parsing takes forever. System captures GBs of data per game session, 99% of which is irrelevant.

**Why it happens:**
- Default pcap capture with no filter captures everything
- Developer assumes "filter later" is okay for MVP
- MTGO server IPs or ports not known upfront
- BPF filters syntax misunderstood

**Consequences:**
- Slow replay file loading
- Large disk usage for replay storage
- CPU overhead during capture (parsing all packets)
- Memory exhaustion during long games
- Privacy concerns if user's other traffic is captured

**Prevention:**

1. **BPF filter for MTGO traffic:**
   ```c
   // Filter by MTGO server IPs (example - need actual IPs)
   char filter_exp[] = "host 209.236.126.177 or host 209.236.126.178";

   // Or filter by known ports if MTGO uses specific ports
   char filter_exp[] = "port 47624 or port 47625";
   ```

2. **Dynamic filter updates:**
   - Discover MTGO server IPs during initial handshake
   - Build filter on-the-fly based on discovered connections
   - Combine IP and port filters for precision

3. **Performance testing:**
   - Capture for 1 hour with no filter - measure file size
   - Capture for 1 hour with filter - measure file size
   - Target: < 10MB per game session

**Phase to address:**
Phase 1 (Proof of Concept) - Establish filtering baseline early

---

### Pitfall 2: Card Database Coupling

**What goes wrong:**
Replay file format embeds full card data (name, type, rules text) instead of card IDs. Replay files become huge and fragile. When card sets release, replays referencing older cards fail to load because the embedded card database is outdated.

**Why it happens:**
- Developer assumes "store everything to be self-contained"
- No external card database reference API considered
- Simpler to embed than to resolve dynamically
- Underestimates how often card sets change

**Consequences:**
- Replay file size: 50MB+ (should be < 1MB)
- Cannot load replays with new cards until tool updated
- Version coupling: tool version = card database version
- Difficult to share replays (file too large)

**Prevention:**

1. **Card ID storage only:**
   ```
   In replay file:
   - Card ID: "m21-123" (set code + collector number)
   Not in replay file:
   - Card name, type, mana cost, text
   ```

2. **External card database resolution:**
   ```
   At replay load time:
   - Read card IDs from replay
   - Query Scryfall or other card database API
   - Or query local card database (bundled or user-provided)
   - Cache resolved cards
   ```

3. **Fallback for offline use:**
   - Bundle core card database (Standard/Legal sets)
   - Allow user to download complete database
   - Handle "card not found" gracefully
   - Store minimal metadata that cannot be resolved (custom cards)

**Phase to address:**
Phase 3 (Game State & Card Resolution) - Decide card resolution strategy early

---

### Pitfall 3: Missing Sideboard Events

**What goes wrong:**
Tool captures gameplay perfectly but cannot detect when players sideboard between games. Sideboard extraction feature returns empty or incorrect results. Users can't analyze sideboarding decisions.

**Why it happens:**
- Sideboarding may not generate network messages (client-side operation)
- Sideboard changes may be sent as part of next game start (no explicit "sideboarding complete" event)
- Developer assumes sideboarding will be obvious in packet stream
- MTGO may use "deck list" messages instead of "sideboard" messages

**Consequences:**
- Key feature (sideboard extraction) doesn't work
- Cannot analyze sideboarding patterns
- Users report "I sideboarded but replay shows no change"

**Prevention:**

1. **Capture during actual sideboarding:**
   - Start capture before sideboarding
   - Manually perform sideboarding in MTGO
   - Capture all messages during sideboarding phase
   - Reverse-engineer what packets represent sideboard changes

2. **Multi-event correlation:**
   - Correlate "deck loaded" events with game starts
   - Compare deck lists between consecutive games
   - Calculate sideboard diff programmatically
   - Store sideboard as derived data, not direct event

3. **User-annotated sideboarding:**
   - Allow users to mark "game 1", "game 2", etc. in replay
   - Calculate sideboard changes between marked games
   - Provide UI for manual sideboard correction

**Phase to address:**
Phase 4 (Sideboard Detection) - May require dedicated reverse-engineering phase

---

### Pitfall 4: Memory Leaks in Long Capture Sessions

**What goes wrong:**
Tool works fine for single games but crashes after multi-hour tournaments. Memory usage grows linearly during capture, eventually exhausting RAM. Tournament analysis becomes impossible.

**Why it happens:**
- Captured packets stored in memory instead of flushed to disk
- Parser creates object for each packet without cleanup
- Circular references in game state graph
- No garbage collection consideration in language choice (e.g., C++ vs Python)

**Consequences:**
- System crashes during tournament play
- Replays of multi-hour events are incomplete
- User data loss
- Performance degradation over time

**Prevention:**

1. **Stream to disk immediately:**
   ```python
   class CaptureToFile:
       def __init__(self, filename):
           self.file = open(filename, 'wb')

       def on_packet(self, packet):
           # Compress and write immediately
           compressed = compress(packet)
           self.file.write(compressed)
           # Don't keep in memory
   ```

2. **Periodic state checkpoints:**
   - Write full game state to disk every N packets
   - Allow reconstruction from checkpoints if crash occurs
   - Limit in-memory state to delta from last checkpoint

3. **Memory profiling:**
   - Test capture for 8+ continuous hours
   - Monitor memory usage with tools (heaptrack, Valgrind)
   - Set memory limits and handle gracefully

**Phase to address:**
Phase 5 (Performance & Stability) - Test long-duration scenarios

---

## Minor Pitfalls

Mistakes that cause annoyance but are fixable.

### Pitfall 1: UTC vs Local Time Confusion

**What goes wrong:**
Replay timestamps are stored as UTC but displayed as local time (or vice versa). Users report "replay says game was at 3 AM but I played at 8 PM." Makes organizing replays confusing.

**Prevention:**
- Always store timestamps as UTC in file
- Store timezone offset or allow user's system to convert
- Display in user's local time by default
- Include timezone metadata in replay header

**Phase to address:**
Phase 2 (Format Design)

---

### Pitfall 2: Player Identification Ambiguity

**What goes wrong:**
Replay identifies players as "Player 1", "Player 2" but can't tie to usernames. Users can't tell "which one am I in this replay?" or "who was my opponent?"

**Prevention:**
- Capture player usernames from MTGO login/handshake
- Store both username and player ID in replay
- Highlight "current user" if replay loaded on their machine
- Allow manual username annotation for privacy

**Phase to address:**
Phase 3 (Game State)

---

### Pitfall 3: Replay Sharing Privacy Issues

**What goes wrong:**
Replay files contain user's username, opponent's username, deck list, and gameplay decisions. Users share replays publicly not realizing they're revealing private information.

**Prevention:**
- Warn before sharing (show what will be included)
- Provide "anonymize" function before sharing
- Strip usernames or replace with "Player 1", "Player 2"
- Add "private" flag to replay metadata

**Phase to address:**
Phase 6 (UX & Sharing)

---

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Packet Capture:** Captures packets on developer's machine but hasn't been tested on:
  - Windows 10 (various updates)
  - Windows 11
  - Fresh OS install (no Npcap)
  - With antivirus software running
  - Without Administrator privileges (should fail gracefully)

- [ ] **Protocol Parsing:** Successfully parses current MTGO packets but:
  - Unknown field tolerance not tested
  - Missing version detection strategy
  - No logging of unknown packet types
  - Cannot handle packet reordering
  - Fails on first packet corruption

- [ ] **Game State Reconstruction:** Reconstructs some game aspects but:
  - Haven't verified against actual game state (watch side-by-side with MTGO)
  - Don't handle out-of-order packets
  - Missing invariants validation (e.g., deck size, life bounds)
  - No detection of reconstruction gaps
  - Sideboard detection untested

- [ ] **Replay File Format:** Can save and load but:
  - No version field in file header
  - No backward compatibility plan
  - Adding/removing fields breaks old files
  - No corruption detection (checksums)
  - Not documented publicly

- [ ] **Sideboard Extraction:** Feature exists but:
  - Tested only on developer's manual sideboarding, not actual MTGO
  - No verification against known sideboarding patterns
  - Doesn't handle "no sideboarding" case gracefully
  - UI doesn't show sideboard changes clearly

- [ ] **Performance:** Works for short games but:
  - Haven't tested for 4+ hour tournaments
  - Memory usage grows unbounded
  - No compression for replay files
  - No progress indication for long operations

---

## Integration Gotchas

Common mistakes when connecting to external services.

| Integration | Common Mistake | Correct Approach |
|--------------|------------------|------------------|
| **WinDivert Installation** | Assume WinDivert is pre-installed | Bundle WinDivert driver with installer, detect presence |
| **Card Database API** | Embed full card data in replay | Store card IDs, resolve at load time from Scryfall or bundled DB |
| **MTGO Server Discovery** | Hardcode server IPs | Discover dynamically from DNS or initial connection, build BPF filter on-the-fly |
| **Administrator Privileges** | Don't mention, fail silently | Detect, prompt with clear instructions, offer relaunch with elevation |
| **Antivirus/EDR** | Assume no conflicts | Test with common AV (Windows Defender, Norton, etc.), whitelist driver |

---

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|-------|-----------|--------------|------------------|
| **In-memory packet storage** | Memory grows, crashes after 1-2 hours | Stream to disk immediately with compression | Tournament play (4+ hours) |
| **No packet filtering** | Replay files > 100MB, slow to load | BPF filter for MTGO server IPs/ports | Daily usage (multiple games) |
| **Synchronous parsing** | UI freezes during replay load | Async parsing with progress indicators | Large replay libraries (100+ files) |
| **Full state per packet** | Huge replay files, wasted space | Store deltas, reconstruct full state on-demand | Long games (>30 min) |

---

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake | Risk | Prevention |
|-----------|--------|------------|
| **Replay injection** | Malicious replay could exploit parser bugs | Validate packet sequences, limit buffer sizes, sandbox replay parsing |
| **Privacy leak in shared replays** | Exposes usernames, decks, play patterns | Warn before sharing, provide anonymization, add private flag |
| **Driver elevation exploits** | Npcap driver runs as kernel, vulnerable | Use official Npcap, don't bundle custom drivers, verify checksums |
| **Arbitrary code execution via replay** | Replay format allows embedded scripts | Strict binary format, no script evaluation from replay data |

---

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|----------|---------------|-----------------|
| **Silent capture failures** | "I ran your tool but it didn't do anything" | Clear status: "Capturing: 527 packets, 12.3 MB, last: 2 seconds ago" |
| **No error recovery** | Tool crashes, user loses entire replay | Auto-save every N packets, recover from crash, show "unsaved data" warning |
| **Complex sideboard UI** | "I can't figure out how to see sideboard changes" | Simple diff view: "Game 1 deck" → "Game 2 deck" with highlighted changes |
| **No replay organization** | All replays in one folder, hard to find | Auto-organize by date/event/opponent, allow tagging and search |
| **Locked proprietary format** | Users can't use replays in other tools | Export to JSON, support community extensions, document format |

---

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|----------|---------------|----------------|
| **Loopback capture failure** | MEDIUM | 1. Identify if MTGO uses localhost<br>2. Verify Npcap loopback support installed<br>3. Update capture code to use "\Device\NPF_Loopback"<br>4. Release patch with loopback detection |
| **Protocol change breaks parser** | HIGH | 1. Capture new MTGO traffic<br>2. Reverse-engineer changed packets (use Wireshark diff)<br>3. Update parser with version detection<br>4. Add new version handler<br>5. Test with old and new replays<br>6. Release within 24 hours |
| **File format incompatibility** | MEDIUM | 1. Keep old format reader code<br>2. Write migration tool (v1→v2)<br>3. Auto-migrate on first load<br>4. Backup old files<br>5. Document migration |
| **Game state corruption** | LOW-MEDIUM | 1. Detect corruption (invalid values, missing sequences)<br>2. Flag replay as "corrupted" but still openable<br>3. Allow user to edit/correct state<br>4. Recover from last checkpoint if checkpoints saved |

---

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|----------|------------------|--------------|
| Loopback capture blindness | Phase 1: Research & Proof of Concept | Capture traffic on clean Windows install with loopback enabled |
| Protocol version fragility | Phase 2: Protocol & Format Design | Simulate protocol change, test parser with artificial unknown packets |
| Platform-specific capture failures | Phase 1: Proof of Concept | Test on Windows 10/11, various AV, without Npcap |
| Game state reconstruction ambiguity | Phase 3: Game State Reconstruction | Compare replay side-by-side with actual MTGO gameplay |
| File format brittleness | Phase 2: Protocol & Format Design | Try adding/removing fields, test old file compatibility |
| Insufficient packet filtering | Phase 1: Proof of Concept | Measure file size with/without filter, target < 10MB/game |
| Card database coupling | Phase 3: Game State & Card Resolution | Test with unknown card ID, load without DB fallback |
| Missing sideboard events | Phase 4: Sideboard Detection | Capture during actual sideboarding, verify extraction accuracy |
| Memory leaks in long sessions | Phase 5: Performance & Stability | Capture for 8+ hours, monitor memory, check for leaks |
| UTC vs local time confusion | Phase 2: Format Design | Test across timezone changes, verify timestamp consistency |
| Player identification ambiguity | Phase 3: Game State | Test with multiple users, verify username capture |
| Replay sharing privacy issues | Phase 6: UX & Sharing | Review replay contents, test anonymization feature |

---

## Sources

- **Wireshark Documentation:** Loopback capture setup, Npcap vs WinPcap differences (https://wiki.wireshark.org/CaptureSetup/Loopback) - HIGH confidence
- **libpcap Documentation:** Programming with pcap, packet filtering, BPF usage (https://www.tcpdump.org/pcap.html) - HIGH confidence
- **Npcap Documentation:** Windows packet capture, loopback support, driver requirements (https://npcap.com/guide/npcap-devguide.html) - HIGH confidence
- **GitHub Reverse Engineering Topics:** General reverse-engineering tools and challenges (https://github.com/topics/reverse-engineering) - LOW confidence (directory listing only)

*Note: MTGO-specific protocol documentation is not publicly available, so some pitfalls are inferred from general network interception and reverse-engineering principles. Actual MTGO protocol reverse-engineering will require Phase 1 research.*

---

*Pitfalls research for: MTGO Replay Capture*
*Researched: 2026-01-28*
