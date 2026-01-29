# MTGO Replay Capture

## What This Is

A local desktop application that captures Magic: The Gathering Online (MTGO) network traffic, stores game replays in a compact file format, and automatically extracts sideboard data. The tool enables personal match analysis and easy replay sharing via file transfer, with future plans for a web viewing platform.

## Core Value

Capture MTGO replays with automatic sideboard extraction, enabling personal analysis and easy file-based sharing.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Capture MTGO network traffic from local machine
- [ ] Parse MTGO network protocol and reconstruct game state
- [ ] Store replays in compact, shareable file format
- [ ] Display replays locally (built-in viewer)
- [ ] Extract sideboard data (cards moved in/out between games in best-of-3)
- [ ] Export replays as standalone files for sharing

### Out of Scope

- Web platform for viewing replays — defer to v2
- Screen recording / video capture approach — network hook chosen
- MTGO Arena support — different system, different protocol
- Real-time streaming or live viewing — post-match capture only

## Context

MTGO's built-in replay system has been non-functional for over a year, leaving players without a way to review matches or track tournament performance. MTGO uses a proprietary network protocol that requires reverse engineering to capture game state. The tool needs to run locally on the user's machine, intercept network traffic to/from MTGO, parse the protocol to reconstruct game actions, and store the structured data in a format that can be shared and viewed later.

## Constraints

- **Tech**: Must intercept network traffic on local machine — requires appropriate permissions
- **Protocol**: MTGO protocol is undocumented and proprietary — requires reverse engineering
- **Format**: Replay files must be compact and portable for easy sharing
- **Compatibility**: Must work with current MTGO client (protocol may change)
- **Platform**: Desktop application (user's local machine), not cloud-based

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Network traffic hook over screen recording | Cleaner data, more reliable extraction, smaller file size | — Pending |
| File-based sharing before web platform | Simpler v1, keeps focus on core capture functionality | — Pending |

---
*Last updated: 2025-01-28 after initialization*
