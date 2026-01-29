# Architecture Research

**Domain:** MTGO Replay Capture System
**Researched:** 2026-01-29
**Confidence:** MEDIUM

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Presentation Layer                     │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │ Replay Viewer│  │   UI Layer   │  │   Config    │ │
│  │   (Future)  │  │   (Electron/ │  │   Manager   │ │
│  │             │  │   Tauri)    │  │             │ │
│  └──────┬───────┘  └──────────────┘  └────────────┘ │
└─────────┼──────────────────────────────────────────────────┘
          │
┌─────────┼──────────────────────────────────────────────────┐
│         ▼              Application Layer                   │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │   Game      │  │   Sideboard  │  │    Replay   │ │
│  │   State     │  │   Tracker    │  │   Manager   │ │
│  │ Reconstructor│  │             │  │             │ │
│  └──────┬───────┘  └──────────────┘  └──────┬─────┘ │
└─────────┼──────────────────────────────────────────┼───────┘
          │                                         │
┌─────────┼──────────────────────────────────────────┼───────┐
│         ▼              Protocol Layer                    │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │   MTGO      │  │  Packet      │  │   Message   │ │
│  │  Protocol   │  │  Parser      │  │   Queue     │ │
│  │  Decoder    │  │             │  │             │ │
│  └──────┬───────┘  └──────────────┘  └────────────┘ │
└─────────┼──────────────────────────────────────────────────┘
          │
┌─────────┼──────────────────────────────────────────────────┐
│         ▼              Capture Layer                       │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐                  │
│  │   Packet     │  │  Filter      │                  │
│  │  Capture    │  │  Engine      │                  │
│  │  (libpcap/  │  │  (BPF)       │                  │
│  │   raw sockets)│  │             │                  │
│  └──────────────┘  └──────────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Communicates With | Typical Implementation |
|-----------|---------------|-------------------|------------------------|
| **Packet Capture** | Intercept network traffic from MTGO client | Filter Engine, Application | libpcap/WinPcap (C/C++) or raw sockets (Node.js/Python) |
| **Filter Engine** | Apply BPF filters to capture only MTGO traffic | Packet Capture, Packet Parser | Kernel-space filtering via libpcap |
| **Packet Parser** | Parse raw packets into TCP/IP structures | Filter Engine, Protocol Decoder | Custom parser using packet structure definitions |
| **MTGO Protocol Decoder** | Reverse-engineer and decode MTGO proprietary protocol | Packet Parser, Game State | State machine handling message types |
| **Message Queue** | Buffer protocol messages for processing | Protocol Decoder, Game State | In-memory queue or persistent buffer |
| **Game State Reconstructor** | Maintain game state from protocol messages | Protocol Decoder, Replay Manager | Event-driven state machine |
| **Sideboard Tracker** | Track sideboard operations between games | Game State, Replay Manager | Event tracker with game boundary detection |
| **Replay Manager** | Serialize game states into compact replay format | Game State, Sideboard, UI Layer | Custom binary format with compression |
| **UI Layer** | Desktop application interface | Replay Manager, Config Manager | Electron (JS/TS), Tauri (Rust), or native (C#/WPF) |
| **Config Manager** | Manage capture settings, file paths | UI Layer, Packet Capture | JSON/YAML configuration |
| **Replay Viewer** | Visualize replay data (future phase) | Replay Manager | Canvas-based or DOM-based renderer |

## Recommended Project Structure

```
mtgo-replay-capture/
├── src/
│   ├── capture/              # Packet capture layer
│   │   ├── interfaces.ts      # Abstract capture interface
│   │   ├── libpcap.ts        # libpcap binding (Node.js)
│   │   ├── pcap-filter.ts    # BPF filter compilation
│   │   └── packet-stream.ts  # Packet data stream
│   │
│   ├── protocol/             # MTGO protocol handling
│   │   ├── decoder.ts         # Protocol message decoder
│   │   ├── messages/          # Message type definitions
│   │   │   ├── game-start.ts
│   │   │   ├── card-played.ts
│   │   │   └── sideboard.ts
│   │   ├── state-machine.ts   # Protocol state machine
│   │   └── message-queue.ts  # Message buffer
│   │
│   ├── game/                 # Game state management
│   │   ├── state.ts           # Game state structure
│   │   ├── reconstructor.ts   # State reconstruction logic
│   │   ├── tracker.ts         # Game/turn tracking
│   │   └── sideboard.ts      # Sideboard tracking logic
│   │
│   ├── replay/               # Replay format handling
│   │   ├── serializer.ts     # State to replay format
│   │   ├── format.ts          # Replay binary format spec
│   │   ├── compressor.ts      # Compression strategy
│   │   └── writer.ts          # File I/O
│   │
│   ├── ui/                   # Desktop application
│   │   ├── main.tsx           # Application entry
│   │   ├── capture-view.tsx   # Capture control UI
│   │   ├── status-view.tsx    # Status monitoring
│   │   └── config-view.tsx    # Configuration
│   │
│   ├── config/               # Configuration
│   │   ├── settings.ts       # Settings types
│   │   └── loader.ts         # Config file I/O
│   │
│   └── common/               # Shared utilities
│       ├── logger.ts          # Logging utilities
│       ├── errors.ts          # Error types
│       └── events.ts          # Event emitter base
│
├── resources/
│   ├── config/
│   │   └── default.json      # Default configuration
│   └── protocols/
│       └── mtgo-specs.md     # MTGO protocol documentation
│
└── test/
    ├── capture/              # Capture layer tests
    ├── protocol/             # Protocol decoder tests
    ├── game/                 # Game state tests
    └── replay/               # Replay format tests
```

### Structure Rationale

- **capture/**: Isolated to allow swapping between different capture methods (libpcap vs raw sockets vs platform-specific)
- **protocol/**: Self-contained to handle evolving MTGO protocol without affecting game logic
- **game/**: Decoupled from protocol details, focuses on game mechanics and state
- **replay/**: Independent of capture and protocol, can be reused for different replay formats
- **ui/**: Presentation layer that depends on application layer but is replaceable
- **config/**: Centralized configuration management

## Architectural Patterns

### Pattern 1: Layered Architecture

**What:** Organizes code into distinct layers with clear separation of concerns and unidirectional dependencies

**When to use:**
- Standard approach for network capture applications
- When you need clear boundaries between capture, protocol, and application logic
- When multiple developers need to work on different parts simultaneously

**Trade-offs:**
- **Pros:** Clear separation, testable, maintainable, easy to reason about
- **Cons:** May have some indirection overhead, requires careful interface design

**Implementation:**
```typescript
// Capture layer - lowest level, knows nothing about MTGO
interface PacketCapture {
  start(): Promise<void>;
  stop(): Promise<void>;
  onPacket(callback: (packet: RawPacket) => void): void;
}

// Protocol layer - depends on capture, knows about MTGO messages
class MTGOProtocolDecoder {
  constructor(capture: PacketCapture) {
    capture.onPacket((packet) => this.handlePacket(packet));
  }

  private handlePacket(packet: RawPacket) {
    const mtgoMessage = this.decode(packet);
    this.emit('message', mtgoMessage);
  }
}

// Application layer - depends on protocol, knows game rules
class GameStateReconstructor {
  constructor(decoder: MTGOProtocolDecoder) {
    decoder.on('message', (msg) => this.updateState(msg));
  }
}
```

### Pattern 2: Event-Driven Architecture

**What:** Components communicate via events rather than direct method calls, enabling loose coupling and async processing

**When to use:**
- For packet capture and real-time processing
- When components operate at different speeds
- When you need to decouple capture speed from processing speed

**Trade-offs:**
- **Pros:** Handles high-frequency events well, scalable, clear data flow
- **Cons:** Debugging can be harder, requires backpressure handling

**Implementation:**
```typescript
import { EventEmitter } from 'events';

// Base event emitter for all components
abstract class MTGOComponent extends EventEmitter {
  protected emitEvent(event: string, data: unknown): void {
    this.emit(event, data);
    // Add logging, metrics, error handling
  }
}

// Protocol decoder emits events
class MTGOProtocolDecoder extends MTGOComponent {
  onMessage(callback: (msg: MTGOMessage) => void): void {
    this.on('message', callback);
  }
}

// Game state listens to messages
class GameStateReconstructor extends MTGOComponent {
  constructor(decoder: MTGOProtocolDecoder) {
    super();
    decoder.on('message', (msg) => this.handleMessage(msg));
  }
}
```

### Pattern 3: State Machine for Protocol Handling

**What:** Protocol decoder uses state machine to track connection state and message context

**When to use:**
- For TCP-based protocols with message ordering requirements
- When protocol has different message types at different connection stages
- When you need to track game phases (mulligan, draw, combat, etc.)

**Trade-offs:**
- **Pros:** Handles complex protocols correctly, predictable behavior, easier to test
- **Cons:** Initial implementation effort, need to model all states

**Implementation:**
```typescript
enum ConnectionState {
  CONNECTING = 'connecting',
  HANDSHAKE = 'handshake',
  AUTHENTICATED = 'authenticated',
  IN_LOBBY = 'in_lobby',
  IN_GAME = 'in_game',
  DISCONNECTED = 'disconnected'
}

enum GamePhase {
  PRE_GAME = 'pre_game',
  MULLIGAN = 'mulligan',
  DRAW = 'draw',
  MAIN = 'main',
  COMBAT = 'combat',
  END_TURN = 'end_turn',
  CLEANUP = 'cleanup',
  END_STEP = 'end_step'
}

class MTGOProtocolDecoder {
  private connectionState: ConnectionState = ConnectionState.DISCONNECTED;
  private gamePhase: GamePhase | null = null;

  private handleMessage(msg: MTGOMessage): void {
    // State transitions
    switch (this.connectionState) {
      case ConnectionState.CONNECTING:
        this.handleConnecting(msg);
        break;
      case ConnectionState.IN_GAME:
        this.handleInGame(msg);
        break;
      // ...
    }
  }

  private handleInGame(msg: MTGOMessage): void {
    switch (this.gamePhase) {
      case GamePhase.MULLIGAN:
        this.handleMulligan(msg);
        break;
      case GamePhase.COMBAT:
        this.handleCombat(msg);
        break;
      // ...
    }
  }
}
```

### Pattern 4: Producer-Consumer for Packet Processing

**What:** Packet capture produces packets at high rate, protocol decoder consumes at its own pace with backpressure

**When to use:**
- Packet capture speed exceeds processing speed
- Need to prevent memory exhaustion from unbounded buffers
- Want to handle network bursts gracefully

**Trade-offs:**
- **Pros:** Handles variable throughput, prevents crashes, tunable performance
- **Cons:** More complex, requires careful buffer sizing

**Implementation:**
```typescript
class BoundedMessageQueue {
  private queue: MTGOMessage[] = [];
  private maxsize: number = 10000;
  private droppedPackets: number = 0;

  push(message: MTGOMessage): boolean {
    if (this.queue.length >= this.maxsize) {
      this.droppedPackets++;
      return false; // Backpressure
    }
    this.queue.push(message);
    return true;
  }

  pop(): MTGOMessage | undefined {
    return this.queue.shift();
  }
}

class ProtocolDecoder {
  private queue: BoundedMessageQueue;

  async processMessages(): Promise<void> {
    while (true) {
      const msg = this.queue.pop();
      if (msg) {
        await this.processMessage(msg);
      } else {
        await this.sleep(10); // Avoid busy-wait
      }
    }
  }
}
```

## Data Flow

### Request Flow

```
[MTGO Client] → [Network Stack]
    ↓ (encrypted TCP packets)
[Packet Capture] → [Packet Parser]
    ↓ (TCP/IP structures)
[Protocol Decoder] → [Message Queue]
    ↓ (MTGO protocol messages)
[Game State Reconstructor] → [Replay Serializer]
    ↓ (game state snapshots)
[Replay Writer] → [File System]
    ↓ (compressed replay file)
[User]
```

### Capture Flow

```
[Capture Start]
    ↓
[Open Capture Device] (libpcap pcap_create)
    ↓
[Set Capture Options] (promiscuous mode, snaplen, timeout)
    ↓
[Compile BPF Filter] (target port, IP ranges)
    ↓
[Activate Capture] (pcap_activate)
    ↓
[Packet Loop] (pcap_loop / pcap_next_ex)
    ↓ (for each packet)
[Callback invoked] with packet data
    ↓
[Parse packet headers] (Ethernet, IP, TCP)
    ↓
[Extract payload] → [Protocol Decoder]
```

### State Management

```
[Packet Stream] → [Protocol Decoder] → [Message Queue]
                                        ↓ (messages)
                              [Game State Reconstructor]
                                        ↓ (events)
                          ┌───────────────┴───────────────┐
                          ▼                               ▼
                    [Game State]                 [Sideboard Tracker]
                          │                                   │
                          └───────────────┬───────────────┘
                                          ↓ (state snapshots)
                                [Replay Serializer]
                                          ↓
                                    [Compressor]
                                          ↓
                                      [File Writer]
```

### Key Data Flows

1. **Packet to Message Flow:**
   - Raw packets → Parser extracts payload → Decoder identifies message type → Queue buffers
   - Speed: High (thousands of packets/second)
   - Buffering: Essential due to rate mismatch

2. **Message to State Flow:**
   - Queue provides messages → Reconstructor updates game state → Events emitted
   - Speed: Medium (dozens of messages/second)
   - State: In-memory representation of game

3. **State to File Flow:**
   - Periodic or event-driven serialization → Compression → Disk write
   - Speed: Low (periodic snapshots)
   - Optimization: Delta encoding, binary format

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 0-1 users | Single-process desktop app, in-memory state, simple file I/O |
| 1-100 users | (Not applicable - local desktop tool only) |
| 100+ concurrent replays | Consider shared replay format spec, cloud storage, viewer web app |

### Scaling Priorities

1. **First bottleneck:** Packet capture and filtering
   - Use BPF filtering in kernel space (libpcap)
   - Set appropriate snaplen to avoid copying unnecessary data
   - Use non-blocking I/O for packet reading

2. **Second bottleneck:** Protocol decoding speed
   - Optimize message parsing (use typed arrays, avoid object allocation)
   - Implement message type lookup with switch/case or maps
   - Use worker threads for CPU-intensive parsing

3. **Third bottleneck:** Replay file size
   - Delta encoding for state changes (only send differences)
   - Binary format instead of JSON
   - Compression (zstd, brotli, or similar)
   - Periodic snapshots + incremental updates

## Anti-Patterns

### Anti-Pattern 1: Monolithic Packet Handler

**What people do:** Handle packet parsing, protocol decoding, and game state all in one giant function

**Why it's wrong:**
- Impossible to test protocol logic in isolation
- Changes to one aspect break everything else
- Cannot optimize different parts independently
- Makes protocol reverse-engineering extremely painful

**Do this instead:**
```
Separate into distinct layers:
- Capture: Only knows about packets, filters to target
- Protocol: Only knows about MTGO message format
- Game: Only knows about Magic rules, not protocol
```

### Anti-Pattern 2: Unbounded Message Queue

**What people do:** Store all protocol messages in an ever-growing array

**Why it's wrong:**
- Network spikes cause memory exhaustion
- Application crashes when queue grows too large
- No backpressure means capture layer can outpace processing

**Do this instead:**
```typescript
// Use bounded queue with drop/overflow handling
class BoundedQueue {
  private maxSize = 10000; // Tunable based on message rate
  private droppedCount = 0;

  push(item: T): boolean {
    if (this.queue.length >= this.maxSize) {
      this.droppedCount++;
      logger.warn(`Dropped message, total: ${this.droppedCount}`);
      return false;
    }
    this.queue.push(item);
    return true;
  }
}
```

### Anti-Pattern 3: Tight Coupling Between Game Logic and Protocol

**What people do:** Game state code directly depends on MTGO protocol message structure

**Why it's wrong:**
- Protocol changes require rewriting game logic
- Cannot test game rules independently
- Hard to support different MTGO versions
- Difficult to implement replay viewer without protocol knowledge

**Do this instead:**
```typescript
// Protocol layer produces domain events
interface CardPlayedEvent {
  cardId: string;
  player: 'local' | 'opponent';
  zone: 'hand' | 'battlefield';
  timestamp: number;
}

// Game layer consumes domain events, doesn't know protocol
class GameState {
  handleCardPlayed(event: CardPlayedEvent): void {
    // Game logic here, no protocol knowledge
  }
}

// Adapter converts protocol messages to domain events
class ProtocolToEventAdapter {
  convertMessage(msg: MTGOCardPlayedMessage): CardPlayedEvent {
    return {
      cardId: msg.card.oracleId,
      player: msg.player.isLocal ? 'local' : 'opponent',
      zone: msg.fromZone,
      timestamp: msg.timestamp
    };
  }
}
```

### Anti-Pattern 4: Synchronous Packet Processing in UI Thread

**What people do:** Process packets directly in UI event handler

**Why it's wrong:**
- UI becomes unresponsive during packet bursts
- Capture misses packets if processing is slow
- No way to implement backpressure

**Do this instead:**
```typescript
// Use separate worker thread/process for packet processing
class CaptureWorker {
  start() {
    // Run capture loop in worker thread
    setInterval(() => {
      const packet = this.pcap.next();
      this.postMessage(packet);
    }, 0);
  }
}

// UI thread only receives processed state
class UIController {
  constructor(worker: CaptureWorker) {
    worker.on('stateUpdate', (state) => this.render(state));
  }
}
```

### Anti-Pattern 5: Assuming Stable MTGO Protocol

**What people do:** Hardcode protocol message structures assuming they won't change

**Why it's wrong:**
- MTGO client updates change protocol regularly
- Message IDs change, formats change, new messages added
- Hardcoded assumptions lead to broken replays

**Do this instead:**
```typescript
// Protocol version detection and mapping
const PROTOCOL_VERSIONS = {
  'v4.0': { cardPlayed: 0x12, sideboard: 0x34 },
  'v4.1': { cardPlayed: 0x13, sideboard: 0x35 },
  'v4.2': { cardPlayed: 0x15, sideboard: 0x38 }
};

class ProtocolDetector {
  detectVersion(handshake: MTGOHandshakeMessage): string {
    // Analyze handshake to determine protocol version
    return this.analyzeHandshake(handshake);
  }
}

class MessageParser {
  private version: string;

  setVersion(version: string) {
    this.version = version;
    this.messageTypes = PROTOCOL_VERSIONS[version];
  }
}
```

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| **MTGO Client** | Network packet capture (passive interception) | No API, must reverse-engineer protocol |
| **Operating System** | libpcap (Linux/macOS) or WinPcap/Npcap (Windows) | Requires elevated privileges |
| **File System** | Local file I/O for replay files | Compact binary format with compression |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| **Capture ↔ Protocol** | Event stream (packets → messages) | Needs backpressure handling |
| **Protocol ↔ Game** | Domain events (MTGO messages → game events) | Decouple protocol from game logic |
| **Game ↔ Replay** | Periodic snapshots (game state → serialized format) | Consider delta encoding |
| **Replay ↔ UI** | Read-only (replay file → viewer) | Viewer can be separate app phase |

## Sources

- **libpcap Documentation** - tcpdump.org (HIGH confidence): https://tcpdump.org/manpages/pcap.3pcap.html
- **Python socket module** - python.org (HIGH confidence): https://docs.python.org/3/library/socket.html
- **Node.js net module** - nodejs.org (HIGH confidence): https://nodejs.org/api/net.html
- **Wireshark Capture Setup** - wiki.wireshark.org (MEDIUM confidence): https://wiki.wireshark.org/CaptureSetup
- **MagicOnlineReplayTool** - PennyDreadfulMTG (MEDIUM confidence): Analyzes existing MTGO replays, C# WPF app

---
*Architecture research for: MTGO Replay Capture System*
*Researched: 2026-01-29*
