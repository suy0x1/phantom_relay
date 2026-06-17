# PhantomRelay Architecture

## Overview

**PhantomRelay** is a sophisticated proxy relay system written in Rust that manages transparent proxy routing, DNS resolution, and dynamic proxy rotation. It serves as a daemon (`phantomrelayd`) controlled by a CLI tool (`prctl`), providing real-time service management, event monitoring, and network interception capabilities.

---

## High-Level System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        PhantomRelay System                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐              ┌─────────────────────────────┐ │
│  │   CLI Tool   │◄──────IPC────►│   PhantomRelay Daemon       │ │
│  │   (prctl)    │    Protocol   │    (phantomrelayd)          │ │
│  └──────────────┘              └─────────────────────────────┘ │
│                                                                 │
│  User-facing command interface      Core relay and proxy logic  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Core Components Architecture

### 1. **Runtime Layer** (`runtime/*`)

The runtime manages the lifecycle of all services and provides the central control plane for the daemon.

**Key Responsibilities:**
- Service lifecycle management (start, stop, restart)
- Service factory creation and configuration
- Runtime signal handling (graceful shutdown)
- IPC message routing and command execution

**Key Types:**
- **RuntimeController**: Central orchestrator that manages all running services, their states, and modes
- **RuntimeContext**: Shared application state passed to all services (configurations, caches, connection maps)
- **ServiceHandle**: Wrapper for spawned tokio tasks with cancellation tokens

**Service Types Managed:**
- Logger, DNS, TProxy, Proxy, Proxy Rotator
- Collector, Metrics, Cache PreLoader, Cache Cleaner, Cache Refresher

---

### 2. **Monitoring & Event System** (`monitor/*`)

A publish-subscribe event system that enables loose coupling between components and provides observability.

**Architecture Pattern: Event Bus**

```
┌────────────────────────────────────────────────┐
│          Event Bus (Broadcast Channel)         │
├────────────────────────────────────────────────┤
│                                                │
│  Publishers:        Bus:       Subscribers:   │
│  - DNS Resolver  ──►         ◄── Logger       │
│  - Proxy Handler ──►  Events ◄── Collector    │
│  - Router        ──►         ◄── Metrics      │
│                                                │
└────────────────────────────────────────────────┘
```

**Event Types:**
- Service lifecycle events (startup, shutdown)
- Connection events (opened, closed)
- DNS events (request, cache hit/miss, cleanup)
- Proxy events (connected, failed)
- Network capability changes
- Error events with context

**Key Components:**
- **Bus**: Tokio broadcast channel wrapper providing pub-sub semantics
- **Event Enum**: Comprehensive event type definitions with metadata
- **Subscribers**: Logger, Metrics, Collector consume events independently

---

### 3. **IPC Communication** (`ipc/*`)

Inter-process communication layer enabling CLI tool to control daemon.

**Protocol Design:**
```
CLI (prctl)                    Daemon (phantomrelayd)
    │                                   │
    ├─ IPCRequest::Runtime ──────────► IPC Server
    │  (Start/Stop/Status)              │
    │                                   ├─ Parse Command
    │                         ┌────────►├─ Execute via RuntimeController
    └─◄─ IPCResponse ────────┘         │
       (Success/Status/Error)           │
```

**Protocol Messages:**
- **IPCRequest**: Serialized runtime commands
- **IPCResponse**: Success messages, service status, or error details

**Transport:** Unix domain sockets with JSON serialization

---

### 4. **Configuration System** (`config/*`)

Structured configuration management for all subsystems with TOML file support.

**Configuration Domains:**
- **DNSConfig**: DNS resolver settings, cache parameters, saturation modes, prewarm domains
- **ProxyConfig**: Proxy server connection parameters
- **TProxyConfig**: Transparent proxy parameters
- **RotationConfig**: Proxy rotation timing
- **CollectorConfig**: Health check parameters, worker count, latency thresholds

**Configuration Loading:**
```
┌────────────────────────────────┐
│  phantomrelay.toml             │
│  (TOML Configuration File)     │
└────────────────┬───────────────┘
                 │
                 ▼
        ┌────────────────────┐
        │  Config Parser     │
        │  (on startup)      │
        └────────────┬───────┘
                     │
    ┌────────────────┼────────────────┐
    ▼                ▼                ▼
  DNS           Proxy            TProxy
  Config        Config           Config
  Arc<Mutex>    Arc<RwLock>      Arc<RwLock>
```

**Design Pattern: Shared Arc-wrapped configs** accessible to all services with interior mutability (Mutex/RwLock) where needed.

---

### 5. **Debug Subsystem** (`debug/*`)

Runtime inspection utilities for troubleshooting and monitoring without stopping services.

**Debug Modules:**
- **config.rs**: Current configuration state inspection
- **conn.rs**: Active connection monitoring
- **dns.rs**: DNS cache status and statistics
- **proxy.rs**: Proxy health and status
- **route.rs**: Current proxy route context

**Debug Command Flow:**
```
CLI Command (prctl debug <subcommand>)
    │
    ▼
IPC Request → IPC Server
    │
    ▼
RuntimeController → Debug Handler
    │
    ▼
Fetch State from Services
    ├─ Configuration (from Arc config objects)
    ├─ Connections (from DashMap conn_map)
    ├─ DNS Cache (from DashMap dns_cache)
    ├─ Routes (from RwLock current_route)
    └─ Proxies (from DashMap healthy_proxies)
    │
    ▼
Format & Return via IPC
    │
    ▼
CLI Prints to stdout
```

**Available Debug Commands:**
```bash
prctl debug config    # Show all current configurations
prctl debug conn      # List active connections
prctl debug dns       # Show DNS cache status
prctl debug proxy     # Show proxy health status
prctl debug route     # Show current route selection
```

---

### 6. **DNS Resolution Subsystem** (`dns/*`)

Complete DNS resolution with caching, prewarming, and refresh capabilities.

**Architecture:**
```
┌──────────────────────────────────────┐
│     DNS Subsystem                    │
├──────────────────────────────────────┤
│                                      │
│  ┌──────────────────────────────┐   │
│  │ DNS Listener (UDP/DoH)       │   │
│  │ Accepts queries from system  │   │
│  └──────────────────────────────┘   │
│           │                          │
│           ▼                          │
│  ┌──────────────────────────────┐   │
│  │ Cache Lookup                 │   │
│  │ DashMap for concurrent reads │   │
│  └──────────────────────────────┘   │
│       Hit ▲         │ Miss           │
│           │         ▼                │
│           │  ┌──────────────────┐   │
│           │  │ Query Resolver   │   │
│           │  │ (upstream DNS)   │   │
│           │  └──────────────────┘   │
│           │         │                │
│           └─────────┘                │
│                                      │
│  ┌──────────────────────────────┐   │
│  │ Prewarmer (background cache) │   │
│  │ Refreshes stale entries      │   │
│  └──────────────────────────────┘   │
│                                      │
│  ┌──────────────────────────────┐   │
│  │ Cleanup Task                 │   │
│  │ Removes expired entries      │   │
│  └──────────────────────────────┘   │
│                                      │
└──────────────────────────────────────┘
```

**Key Components:**
- **Listener**: Receives DNS queries from system
- **Cache**: DashMap-based concurrent cache with TTL tracking
- **Parser**: Handles DNS packet parsing
- **Prewarmer**: Background task refreshing cache entries before expiry
- **Cleanup**: Removes expired cache entries periodically
- **DoH Support**: DNS-over-HTTPS resolver integration

---

### 7. **Routing & Connection Management** (`routing/*`)

Manages connection lifecycle and proxy routing decisions.

**Architecture:**
```
┌──────────────────────────────────────┐
│    Routing & Connection System       │
├──────────────────────────────────────┤
│                                      │
│  ┌──────────────────────────────┐   │
│  │ Connection Manager (DashMap) │   │
│  │ Tracks all active conns      │   │
│  └──────────────────────────────┘   │
│           │                          │
│           ▼                          │
│  ┌──────────────────────────────┐   │
│  │ Connection State Machine     │   │
│  │ - New → Open → Closed       │   │
│  └──────────────────────────────┘   │
│           │                          │
│           ▼                          │
│  ┌──────────────────────────────┐   │
│  │ Proxy Selection              │   │
│  │ (via Rotation Engine)        │   │
│  └──────────────────────────────┘   │
│           │                          │
│           ▼                          │
│  ┌──────────────────────────────┐   │
│  │ SOCKS5 Handler               │   │
│  │ Tunnel data through proxy    │   │
│  └──────────────────────────────┘   │
│                                      │
└──────────────────────────────────────┘
```

**Key Components:**
- **ConnectionManager**: DashMap-based registry tracking all active connections
- **Connection State**: Tracks connection lifecycle (created, opened, closed)
- **SOCKS5 Types**: Protocol handlers for SOCKS5 authentication and relay
- **Proxy Router**: Selects proxy from rotation engine
- **Connect Handler**: Establishes proxy connections with timeout/retry logic

---

### 8. **Proxy Rotation Engine** (`subsystems/rotation/*`)

Intelligent proxy rotation with round-robin or dynamic selection.

**Architecture:**
```
┌──────────────────────────────────────┐
│    Proxy Rotation Engine             │
├──────────────────────────────────────┤
│                                      │
│  RouteContext (current_route)        │
│  Arc<RwLock<>>                       │
│    ├─ Current proxy IP               │
│    ├─ Port information               │
│    └─ Health status                  │
│           ▲                          │
│           │                          │
│  ┌───────┴──────────────────────┐   │
│  │ Rotation Service             │   │
│  │ - Fixed interval rotation     │   │
│  │ - Health-aware selection      │   │
│  └───────┬──────────────────────┘   │
│           │                          │
│           ▼                          │
│  ┌──────────────────────────────┐   │
│  │ Healthy Proxies Registry     │   │
│  │ (DashMap)                    │   │
│  └──────────────────────────────┘   │
│                                      │
└──────────────────────────────────────┘
```

**Key Mechanisms:**
- **Cursor-based round-robin**: Atomic counter tracks current proxy index
- **Route Context**: RwLock-protected current active route
- **Health Tracking**: Maintains map of healthy vs unhealthy proxies
- **Rotation Interval**: Configurable timing (default: 60 seconds)

---

### 9. **Transparent Proxy (TProxy) Subsystem** (`tproxy/*`)

Kernel-level transparent proxy interception.

**Architecture:**
```
┌────────────────────────────────────────┐
│    TProxy Subsystem                    │
├────────────────────────────────────────┤
│                                        │
│  ┌──────────────────────────────────┐ │
│  │ Listener                         │ │
│  │ - Binds to intercept port        │ │
│  │ - IP_RECVORIGDSTADDR socket opt  │ │
│  └──────────────────────────────────┘ │
│           │                            │
│           ▼                            │
│  ┌──────────────────────────────────┐ │
│  │ Original Destination Extraction  │ │
│  │ (SO_ORIGINAL_DST / IP_RECVORIGDSTADDR)
│  │ - Retrieves original target      │ │
│  │ - Before DNAT happened           │ │
│  └──────────────────────────────────┘ │
│           │                            │
│           ▼                            │
│  ┌──────────────────────────────────┐ │
│  │ Relay Handler                    │ │
│  │ - Connects to proxy              │ │
│  │ - Bidirectional data relay       │ │
│  └──────────────────────────────────┘ │
│                                        │
└────────────────────────────────────────┘
```

**Key Capabilities:**
- Linux netfilter integration (iptables rules required)
- Original destination extraction via platform-specific APIs
- Connection relay with bidirectional data flow

---

### 10. **Network Subsystem** (`subsystems/network/*`)

Manages network rules, capabilities, and system-level network configuration.

**Components:**
- **Manager**: Orchestrates network configuration changes
- **Capabilities**: Enum of network capabilities (e.g., IPv4 support, IPv6)
- **Rules**: iptables/netfilter rules for traffic interception

---

### 11. **Collector Service** (`collector/*`)

Health checking and proxy availability monitoring.

**Architecture:**
```
┌──────────────────────────────────────┐
│    Collector Service                 │
├──────────────────────────────────────┤
│                                      │
│  ┌──────────────────────────────┐   │
│  │ Health Manager               │   │
│  │ - Monitors proxy health      │   │
│  │ - Manages alive/dead list    │   │
│  └──────────────────────────────┘   │
│           │                          │
│           ▼                          │
│  ┌──────────────────────────────┐   │
│  │ Periodic Health Checks       │   │
│  │ - Connect to each proxy      │   │
│  │ - Measure latency/timeout    │   │
│  └──────────────────────────────┘   │
│           │                          │
│           ▼                          │
│  ┌──────────────────────────────┐   │
│  │ Update Healthy Proxies Map   │   │
│  │ - Affects routing decisions  │   │
│  │ - Emits events on change     │   │
│  └──────────────────────────────┘   │
│                                      │
└──────────────────────────────────────┘
```

**Functions:**
- Periodic connectivity verification of proxy list
- Dynamic updates to healthy proxy registry
- Event emission on health status changes

---

### 12. **Direct Proxy Subsystem** (`proxy/*`)

SOCKS5-compatible proxy server for applications connecting directly.

**Architecture:**
```
┌──────────────────────────────────────┐
│    Proxy Server Subsystem            │
├──────────────────────────────────────┤
│                                      │
│  ┌──────────────────────────────┐   │
│  │ Proxy Server Listener        │   │
│  │ - Accepts SOCKS5 connections │   │
│  └──────────────────────────────┘   │
│           │                          │
│           ▼                          │
│  ┌──────────────────────────────┐   │
│  │ Connection Handler           │   │
│  │ - SOCKS5 negotiation         │   │
│  │ - Auth handling              │   │
│  └──────────────────────────────┘   │
│           │                          │
│           ▼                          │
│  ┌──────────────────────────────┐   │
│  │ Routing & Relay              │   │
│  │ - Route through rotated proxy│   │
│  │ - Tunnel data                │   │
│  └──────────────────────────────┘   │
│                                      │
└──────────────────────────────────────┘
```

---

### 13. **Metrics & Observability** (`metrics/*`)

Prometheus-compatible metrics collection.

**Capabilities:**
- Real-time metrics collection
- Connection statistics
- Proxy performance metrics
- Event listener for metric aggregation

---

## Data Flow Patterns

### Pattern 1: Inbound Connection (TProxy Path)

```
System Traffic
    │
    ▼
[iptables DNAT]
    │
    ▼
TProxy Listener (intercept port)
    │
    ▼
Original Destination Extraction
    │
    ▼
Connection Manager (track)
    │
    ▼
Routing Engine (select proxy)
    │
    ▼
Proxy Handler (SOCKS5 relay)
    │
    ▼
Upstream Proxy
    │
    ▼
Target Host
```

### Pattern 2: DNS Resolution

```
DNS Query (UDP port 53)
    │
    ▼
DNS Listener
    │
    ▼
Cache Lookup
    │
    ├─ Cache Hit ──► Return cached response
    │
    └─ Cache Miss
           │
           ▼
        Upstream Resolver
           │
           ▼
        Cache Update & Return
           │
           ▼
        Prewarmer schedules refresh
```

### Pattern 3: Service Lifecycle

```
CLI Command (prctl start dns)
    │
    ▼
IPC Client (TCP connection)
    │
    ▼
IPC Server (Unix socket)
    │
    ▼
RuntimeController (parse command)
    │
    ▼
Service Factory (create task)
    │
    ▼
Tokio spawn with CancellationToken
    │
    ▼
Service runs with event bus subscriptions
    │
    ▼
Event Bus publishes lifecycle events
```

---

## Concurrency Model

**Tokio-based async runtime:**
- All I/O operations are non-blocking
- Services run as independent spawned tasks
- Shared state protected via Arc, RwLock, Mutex, DashMap

**Synchronization Primitives:**
- **Arc<RwLock<T>>**: Configuration, route context
- **Arc<DashMap<K,V>>**: Cache, connections, healthy proxies (lock-free reads)
- **Arc<Mutex<T>>**: Small mutable state requiring exclusive access
- **broadcast::Channel**: Event bus for pub-sub
- **CancellationToken**: Graceful service shutdown signaling

---

## Error Handling Strategy

**Design Pattern: Result-based error propagation**
- All operations return `Result<T, Error>`
- Error extension traits provide context enrichment
- Bus broadcasts Error events for observability
- Graceful degradation without panics

---

## Key Design Principles

1. **Loose Coupling**: Event bus decouples services from direct dependencies
2. **Composability**: Services are independent, manageable units
3. **Observability**: Rich event stream enables monitoring/logging
4. **Concurrency**: Lock-free data structures (DashMap) minimize contention
5. **Graceful Shutdown**: CancellationTokens enable clean service termination
6. **Fail-Safety**: Health checks prevent dead proxies from being routed to

---

## Dependency Graph

```
┌─────────────────────────────────────┐
│      Runtime & IPC Layer            │
│  (Controller, Factories, Commands)  │
└──────────────┬──────────────────────┘
               │
       ┌───────┴───────────────────────┐
       │                               │
       ▼                               ▼
┌──────────────────┐          ┌─────────────────┐
│  DNS Subsystem   │          │  Proxy Relay    │
├──────────────────┤          ├─────────────────┤
│ - Listener       │          │ - Direct SOCKS5 │
│ - Cache          │          │ - Handler       │
│ - Prewarmer      │          │ - Relay         │
│ - Cleanup        │          └─────────────────┘
│ - DoH            │                 ▲
└──────────────────┘                 │
       │                             │
       └────────────┬────────────────┘
                    │
                    ▼
         ┌──────────────────────┐
         │  Routing Layer       │
         ├──────────────────────┤
         │ - Connection Manager │
         │ - SOCKS5 Types       │
         │ - Proxy Router       │
         └──────────────────────┘
                    ▲
                    │
         ┌──────────┴──────────┐
         │                     │
         ▼                     ▼
  ┌─────────────┐    ┌──────────────────┐
  │ Rotation    │    │ Collector        │
  │ Engine      │    ├──────────────────┤
  └─────────────┘    │ - Health checks  │
         ▲           │ - Proxy status   │
         │           └──────────────────┘
         └───────────────┬──────────────┘
                         │
                         ▼
         ┌──────────────────────────┐
         │  Network Subsystem       │
         ├──────────────────────────┤
         │ - TProxy                 │
         │ - Rules & Capabilities   │
         └──────────────────────────┘
                    ▲
                    │
         ┌──────────┴──────────────┐
         │   All Services        │
         │   (subscribe to)       │
         │                        │
         ▼                        ▼
    ┌─────────────┐      ┌──────────────┐
    │ Event Bus   │      │ Monitoring   │
    │ (Broadcast) │      ├──────────────┤
    └─────────────┘      │ - Logger     │
                         │ - Metrics    │
                         └──────────────┘
```

