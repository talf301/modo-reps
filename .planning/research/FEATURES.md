# Feature Landscape

**Domain:** MTGO Replay Capture and Game Analysis Tools
**Researched:** January 28, 2026
**Updated:** 2026-01-30 - Updated for Windows-only (MTGO is Windows-only)
**Confidence:** HIGH

## Table Stakes (Users Expect These)

Features users expect. Missing these will make the product feel incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Replay capture** | Core value proposition - users need to save replays for analysis | HIGH | Requires network traffic interception or log parsing |
| **Game state reconstruction** | Essential for analyzing replays - must reconstruct board state, hands, libraries | HIGH | Depends on protocol reverse-engineering |
| **Deck list extraction** | Users expect to see what they played (maindeck, sideboard) | MEDIUM | Can be parsed from replay data |
| **Win/loss tracking** | Basic analytics requirement for self-improvement | MEDIUM | Match outcome identification |
| **Play-by-play breakdown** | Necessary for game analysis - what cards were played when | MEDIUM | Requires parsing all game actions |
| **Match metadata** | Context matters - format, opponent, date, duration | LOW | Often available from logs |
| **File export/sharing** | Users want to share replays and export data | LOW | Export to standard formats (JSON, CSV) |
| **Search and filtering** | Finding specific replays in a collection | MEDIUM | Indexing, filtering by date, format, opponent |
| **Basic statistics** | Win rates, mulligans, game length | MEDIUM | Aggregation of match data |
| **Sideboard extraction** | Post-MVP differentiator but highly requested by tournament players | MEDIUM | Track cards in/out between games |

## Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Automatic sideboard detection** | Solves a major pain point - manually tracking sideboarding is tedious | MEDIUM | Analyze deck changes between games automatically |
| **Turn-by-turn replay playback** | Superior to static viewing - allows studying decision points | HIGH | Requires full game state reconstruction |
| **Compact replay format** | Enables easy file sharing and storage efficiency | MEDIUM | Custom binary format, not just raw logs |
| **Real-time overlay tracking** | Live analysis while playing - competitive advantage | HIGH | Requires SDK-based client inspection |
| **Card usage analytics** | Beyond basic win rate - reveals play patterns and card performance | MEDIUM | Track how often cards resolve, effectiveness |
| **Mulligan analysis** | Decision optimization insight - keep vs mulligan decisions | LOW | Track mulligan decisions and outcomes |
| **Mana curve analysis** | Deck building optimization - visual resource usage | MEDIUM | Aggregate mana usage across games |
| **Opponent deck analysis** | Strategic preparation - see what others are playing | LOW | Extract opponent decklists from replays |
| **Format-specific insights** | Different formats have different metagames and priorities | LOW | Standard, Modern, Pauper, Commander analytics |
| **Trend visualization** | Identifies improvement over time | MEDIUM | Charts, graphs of performance metrics |
| **Import from MTGO logs** | Seamless integration with existing MTGO data | LOW | Parse GameLog/DraftLog files from standard locations |
| **Cross-game session tracking** | Track match/league performance as a complete unit | MEDIUM | Aggregate multiple games into session-level stats |

## Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Anti-Feature | Why Requested | Why Problematic | What to Do Instead |
|---------------|----------------|------------------|---------------------|
| **Real-time streaming to external service** | "Share my replays live!" | Requires cloud infrastructure, privacy concerns, ToS violations | Local playback with optional manual upload |
| **Cloud-based replay storage** | "Keep my replays online" | Privacy concerns, recurring costs, dependency on external service | Local file storage with user-controlled export options |
| **Automatic social sharing** | "Post replays to Twitter/Discord automatically" | Privacy concerns, not core value proposition | Export to standard formats, let users decide sharing method |
| **MTGO client modification/injection** | "Integrate into MTGO UI directly" | Likely violates EULA, breaks on MTGO updates, security risk | Separate desktop application that reads logs/memory via SDK |
| **Packet injection/network interference** | "Add cards to my deck during match" | Cheating detection, ToS violation, legal issues | Read-only observation, no modification of game data |
| **Automated play suggestions** | "Tell me what to play during games" | Creates dependency, reduces learning, liability if wrong | Analysis tools only - no in-game assistance |
| **Multiple platform support** | "Support Mac/Linux too!" | MTGO only runs on Windows, cross-platform for analysis only | Windows-only for capture, analysis export works anywhere |
| **Real-money trading integration** | "Buy/sell cards through the tool" | Financial regulations, trust issues, outside scope | Pure analysis tool - no transactional features |
| **AI-powered gameplay assistance** | "Help me win during matches" | Cheating allegations, ToS violation, competitive ethics | Post-game analysis and learning only |
| **Account credential management** | "Auto-login to MTGO" | Security risk, credential storage, ToS violation | User logs in independently, tool doesn't authenticate |

## Feature Dependencies

```
[Network Traffic Capture / Log Parsing]
    ├──requires──> [Game State Reconstruction]
    ├──requires──> [Card/Action Extraction]
    ├──enables──> [Replay Playback]
    ├──enables──> [Statistics Generation]
    └──enables──> [Deck Extraction]

[Game State Reconstruction]
    ├──requires──> [Protocol Understanding]
    ├──enables──> [Turn-by-Turn Analysis]
    ├──enables──> [Sideboard Detection]
    └──enables──> [Mana Curve Tracking]

[MTGOSDK / Memory Inspection]
    ├──enables──> [Real-time Tracking]
    ├──enables──> [Live Overlay Display]
    └──requires──> [.NET Runtime]

[Local Database Storage]
    ├──enables──> [Advanced Analytics]
    ├──enables──> [Historical Trends]
    ├──enables──> [Search/Filtering]
    └──enables──> [Export Functionality]
```

### Dependency Notes

- **Network Traffic Capture / Log Parsing** is the foundation - without capturing game data, nothing else is possible
- **Game State Reconstruction** is the heavy lifting - requires understanding MTGO protocol or parsing log format
- **Card/Action Extraction** depends on reconstructed state - parsing individual plays, card resolves
- **Replay Playback** requires both state and action sequence
- **Sideboard Detection** needs comparison between consecutive games - requires deck extraction
- **Real-time Tracking** requires MTGOSDK or similar memory inspection approach - more complex than log parsing
- **Local Database** enables advanced analytics - SQLite or similar embedded database

## MVP Definition

### Launch With (v1)

Minimum viable product - what's needed to validate the concept.

- [x] **Replay capture from MTGO logs** - Core value, users need to save replays
- [x] **Game state reconstruction** - Essential for viewing replay content
- [x] **Basic playback controls** - Play, pause, step forward/backward
- [x] **Deck list extraction** - Show maindeck from replay
- [x] **Match metadata** - Format, date, opponent identification
- [x] **File export** - Save replays as shareable files (custom binary format)
- [x] **Basic search/filtering** - Find replays by date or format

### Add After Validation (v1.x)

Features to add once core is working.

- [ ] **Sideboard detection and extraction** - Core differentiator for tournament players
- [ ] **Play-by-play viewer** - Detailed action breakdown
- [ ] **Win/loss statistics** - Basic analytics dashboard
- [ ] **Import from MTGO log directories** - Seamless integration
- [ ] **Replay bookmarks** - Mark interesting points in replay
- [ ] **Opponent deck extraction** - See what opponents played

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] **Real-time overlay tracking** - Use MTGOSDK for live analysis
- [ ] **Card usage analytics** - Card performance metrics
- [ ] **Mulligan analysis** - Decision optimization insights
- [ ] **Mana curve visualization** - Deck building optimization
- [ ] **Trend analysis and charts** - Performance over time
- [ ] **Multiple replay comparison** - Side-by-side replay viewing
- [ ] **Match/league session aggregation** - Tournament tracking integration
- [ ] **Format-specific dashboards** - Custom views per format
- [ ] **Community sharing features** - Optional upload to shared service (not default)

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|-------------------|----------|
| Replay capture from logs | HIGH | MEDIUM | P1 |
| Game state reconstruction | HIGH | HIGH | P1 |
| Basic playback controls | HIGH | MEDIUM | P1 |
| Deck list extraction | HIGH | MEDIUM | P1 |
| File export | MEDIUM | LOW | P1 |
| Match metadata | MEDIUM | LOW | P1 |
| Basic search/filtering | MEDIUM | MEDIUM | P2 |
| Win/loss statistics | MEDIUM | MEDIUM | P2 |
| Sideboard extraction | HIGH | MEDIUM | P2 |
| Play-by-play viewer | MEDIUM | MEDIUM | P2 |
| Import from MTGO logs | MEDIUM | LOW | P2 |
| Replay bookmarks | LOW | LOW | P3 |
| Opponent deck extraction | MEDIUM | MEDIUM | P3 |
| Real-time overlay tracking | HIGH | HIGH | P3 |
| Card usage analytics | MEDIUM | MEDIUM | P3 |
| Mulligan analysis | LOW | MEDIUM | P3 |
| Mana curve visualization | MEDIUM | MEDIUM | P3 |
| Trend analysis and charts | MEDIUM | MEDIUM | P3 |
| Multiple replay comparison | MEDIUM | MEDIUM | P3 |
| Match/league aggregation | MEDIUM | MEDIUM | P3 |
| Format-specific dashboards | MEDIUM | MEDIUM | P3 |
| Cloud replay storage | LOW | HIGH | P4 |
| Automatic social sharing | LOW | MEDIUM | P4 |
| MTGO client integration | HIGH | HIGH | P4 |

**Priority key:**
- **P1:** Must have for launch (MVP + immediate post-MVP)
- **P2:** Should have, add when possible (enhanced experience)
- **P3:** Nice to have, future consideration (advanced features)
- **P4:** Deliberate anti-features (avoid these)

## Competitor Feature Analysis

| Feature | MTGO-Tracker | Magic Online Replay Tool | Videre Tracker | Our Approach |
|---------|--------------|------------------------|----------------|--------------|
| **Data source** | Log file import | Replay file analysis | Real-time via MTGOSDK | Network capture + optional SDK integration |
| **Replay capture** | Manual import only | Analyzes existing replays | Real-time capture + file import | Automatic file-based capture |
| **Sideboard tracking** | Not explicit | Not mentioned | Yes | **Focus feature** - automatic detection |
| **Deck extraction** | Manual entry | Implicit in replay analysis | Automatic extraction from game state |
| **Playback** | No playback | Limited playback mentioned | Full playback with controls |
| **Win rate** | Basic statistics | Not explicitly mentioned | Yes | Comprehensive analytics dashboard |
| **Real-time** | No | Yes (via MTGOSDK) | Yes (via MTGOSDK) | Future consideration |
| **Platform** | Windows (Python) | Windows (C#) | Windows | Windows-only (MTGO is Windows-only) |
| **Storage** | Local database | Local files | Local binary format |
| **Active maintenance** | Last update Oct 2023 | Last update Oct 2022 | Fresh development |

## Sources

### HIGH Confidence (Official Documentation)

- **MTGO-Tracker Wiki** - https://github.com/cderickson/MTGO-Tracker/wiki - Table structure, features, data cleaning methods
- **MTGOSDK README** - https://github.com/videre-project/MTGOSDK - SDK capabilities, architecture guides
- **Videre Tracker README** - https://github.com/videre-project/Tracker - Real-time tracking features, MTGOSDK usage
- **PennyDreadful-Tools README** - https://github.com/PennyDreadfulMTG/Penny-Dreadful-Tools - Comprehensive MTGO tooling suite
- **MTGGoldfish** - https://www.mtggoldfish.com - Industry standard for MTG data visualization, deck tracking

### MEDIUM Confidence (Multiple Verified Sources)

- **Magic Online Replay Tool** - https://github.com/PennyDreadfulMTG/MagicOnlineReplayTool - Replay analysis statistics (last updated Oct 2022)
- **Cockatrice** - https://github.com/Cockatrice/Cockatrice - Virtual tabletop architecture (for comparison of replay features)
- **GitHub MTGO topic** - https://github.com/topics/mtgo - Ecosystem overview of 19 MTGO-related repositories

### Key Insights from Research

1. **Two main approaches exist:**
   - **Log file parsing:** Simpler, works offline, analyzed after games complete (MTGO-Tracker)
   - **Real-time SDK-based:** Complex, provides live tracking, requires memory inspection (Videre Tracker/MTGOSDK)

2. **Gap in current ecosystem:**
   - No tool combines **automatic replay capture** with **automatic sideboard extraction**
   - MTGO-Tracker doesn't capture replays, just parses existing logs
   - Magic Online Replay Tool analyzes replays but requires manual capture
   - Videre Tracker uses SDK for real-time but doesn't emphasize file-based replay capture

3. **User needs identified:**
   - Sideboard tracking is manual and painful (tournament players)
   - Compact replay files for easy sharing
   - Both real-time insights and post-game analysis
   - Personal analysis with file-based sharing (not cloud)

4. **Technical considerations:**
    - MTGO protocol is proprietary (no official documentation)
    - SDK approach (MTGOSDK) uses memory inspection via ClrMD
    - Network capture provides automatic replay recording
    - Protocol reverse-engineering required for full state reconstruction

5. **Market positioning opportunity:**
    - Focus on **automatic network capture** (not just log parsing)
    - **Sideboard extraction** as differentiator
    - **Offline-first** with optional export/sharing
    - **Windows-only** application (matches MTGO platform)
