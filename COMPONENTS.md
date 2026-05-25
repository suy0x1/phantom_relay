# Component Deep Dive

Detailed specification of each major component in PhantomRelay.

---

## Runtime & Service Management

### Overview
The runtime is the central orchestrator managing all services, their lifecycles, and communication with the CLI tool.

### Key Types

**RuntimeController**
- Maintains `DashMap` of running services
- Stores `RuntimeContext` with shared application state
- Routes CLI commands to appropriate service handlers
- Manages service start/stop/restart lifecycle

**RuntimeContext**
```
- bus: Event broadcast channel
- rotation_config: Proxy rotation parameters
- dns_config: DNS resolver settings
- tproxy_config: Transparent proxy settings
- proxy_config: Direct proxy settings
- collector_config: Health check parameters
- current_route: RwLock protecting current proxy
- conn_map: Connection registry
- dns_cache: DashMap DNS cache
- inflight: In-flight request tracking
- healthy_proxies: Registry of operational proxies
```

**Service Lifecycle**
```
RuntimeController
├─ Factory Function (ServiceFn)
│  └─ Produces ServiceFuture (Tokio task)
├─ CancellationToken
│  └─ Signal graceful shutdown
└─ ServiceHandle
   ├─ tokio::JoinHandle (task reference)
   └─ CancellationToken (termination signal)
```

### Service Types

| Service | Purpose | Dependencies |
|---------|---------|--------------|
| logger | Event bus subscriber, writes to stdout | bus |
| dns | DNS listener and cache queries | dns_config, dns_cache, bus |
| proxy | SOCKS5 server for direct connections | ctx, current_route, bus |
| tproxy | Transparent proxy listener | tproxy_config, conn_map, bus |
| proxy_rotator | Rotates proxy selection | current_route, healthy_proxies |
| proxy_collector | Health checks proxies | proxy_config, healthy_proxies, bus |
| cache_preloader | Background cache refresh | dns_cache, dns_config |
| cache_cleaner | Removes expired cache entries | dns_cache |
| cache_refresher | Refreshes stale DNS entries | dns_cache, dns_config |
| metrics | Prometheus metrics listener | bus |

### Service Factory Pattern

Each service is created by a factory function that:
1. Accepts `RuntimeContext` and `CancellationToken`
2. Returns `ServiceFuture` (Pin<Box<dyn Future>>)
3. Runs until cancellation or error
4. Emits lifecycle events to bus

Example flow:
```rust
dns_service(ctx.clone())  // Factory returns fn
  └─ (cancel_token) -> ServiceFuture
      └─ DNS listener starts
         └─ Accepts queries, checks cache
            └─ Publishes events
               └─ Until cancel_token fires
```

### Startup Sequence

```
1. main() creates Bus
2. startup() initializes:
   - RuntimeContext with configs
   - RotationEngine with AtomicUsize cursor
   - RuntimeController with context
3. spawn_network_manager() starts network subsystem
4. start_rotation_engine() begins proxy rotation
5. IPC server spawned on separate task
6. Daemon waits for SIGINT or IPC commands
```

### Shutdown Sequence

```
1. SIGINT received → runtime.shutdown() called
2. Iterate over all running services
3. For each service:
   - Remove from DashMap
   - Signal cancellation token
   - Await task completion
4. All services gracefully terminate
5. Event bus shut down
6. Process exits
```

---

## DNS Subsystem

### Architecture Layers

**Listener Layer**
- Binds to UDP port (typically 53)
- Accepts DNS queries in wire format
- Routes to Cache or Resolver

**Cache Layer**
- `Arc<DashMap<String, CacheEntry>>`
- Concurrent reads without locks (hot path)
- Stores response with TTL metadata
- Supports cache hit/miss events

**Resolver Layer**
- Upstream DNS server (configurable)
- Falls back to system resolver
- DoH (DNS-over-HTTPS) support
- Handles response parsing

**Prewarmer Layer**
- Background task running continuously
- Identifies cache entries approaching TTL expiry
- Proactively refreshes before expiry
- Reduces cache miss latency

**Cleanup Layer**
- Periodic task removing expired entries
- Prevents unbounded memory growth
- Tracks cleanup metrics

### Cache Structure

```rust
Entry: {
  response: Vec<u8>,           // DNS wire format
  timestamp: Instant,           // Creation time
  ttl: u32,                     // Time to live (seconds)
  hits: AtomicUsize,            // Metrics
  last_accessed: AtomicInstant  // For LRU
}
```

### Query Flow

```
1. Incoming DNS query
   └─ Parse domain name from wire format
      └─ Lookup in DashMap
         ├─ Cache Hit:
         │  ├─ Check TTL validity
         │  ├─ Emit CacheHit event
         │  └─ Return cached response
         │
         └─ Cache Miss:
            ├─ Emit CacheMiss event
            ├─ Query upstream resolver
            ├─ Update DashMap with new entry
            ├─ Schedule prewarmer refresh
            └─ Return response
```

### Mode: DNS Turbo (Cache Saturation)

When enabled:
- Prewarmer interval reduced (more aggressive refresh)
- Cache preload on startup for common domains
- Higher TTL cache entries
- Designed for latency-sensitive applications

Enable/disable via:
```bash
prctl enable dns-turbo
prctl disable dns-turbo
```

### Performance Characteristics

- **Cache Hit**: DashMap lookup + response formatting (~microseconds)
- **Cache Miss**: Upstream query + network latency (~milliseconds)
- **Concurrency**: Lock-free reads, lock on write (rare)
- **Memory**: O(entries × response_size)

---

## Proxy Rotation Engine

### Design Pattern: Cursor-Based Round Robin

```
Active Proxies: [P1, P2, P3, P4, P5]
Cursor: AtomicUsize = 2

Request 1: cursor -> P3, cursor++ (cursor=3)
Request 2: cursor -> P4, cursor++ (cursor=4)
Request 3: cursor -> P5, cursor++ (cursor=5)
Request 4: cursor -> P1, cursor++ % 5 (cursor=0)
```

### Route Selection Logic

```
1. Atomic load current cursor
2. Index into healthy proxies list
3. Check if proxy is healthy
   ├─ Healthy: Use it
   └─ Unhealthy: Find next healthy
4. Increment cursor atomically
5. Return selected proxy and port
```

### RouteContext

```rust
Arc<RwLock<RouteContext>> {
  current_proxy: SocketAddr,
  backup_proxies: Vec<SocketAddr>,
  health_status: HashMap<SocketAddr, Health>,
  rotate_interval: Duration,
  last_rotation: Instant
}
```

### Healthy Proxies Registry

```
Arc<DashMap<SocketAddr, HealthStatus>> {
  status: enum { Healthy, Unhealthy, Unknown },
  last_check: Instant,
  consecutive_failures: usize,
  latency_ms: f64
}
```

### Health Check Integration

Collector service periodically:
1. Attempts connection to each proxy
2. Measures latency
3. Updates status in registry
4. Emits HealthStatusChanged events
5. Rotation engine avoids unhealthy proxies

### Rotation Timing

```
Configuration:
  rotate_sec: 60

Rotation Task:
├─ Every 60 seconds:
│  ├─ Load cursor
│  ├─ Increment atomically
│  ├─ Emit RotateProxy event
│  └─ Next connection uses new cursor position
└─ Backpressure: No artificial delays
```

### Handling Proxy Failures

When proxy fails during connection:
1. Handler catches error
2. Increments cursor
3. Attempts next proxy
4. After MAX_RETRIES: emit ProxyFailed event
5. Collector marks proxy unhealthy
6. Rotation engine skips this proxy going forward

---

## Transparent Proxy (TProxy) Subsystem

### Kernel Integration

TProxy requires Linux netfilter rules to redirect traffic:

```bash
# Example iptables rules (setup required before daemon start)
iptables -t mangle -A PREROUTING -p tcp --dport 80 \
  -j TPROXY --on-port 8888 --on-ip 127.0.0.1
iptables -t mangle -A PREROUTING -p tcp --dport 443 \
  -j TPROXY --on-port 8888 --on-ip 127.0.0.1
```

### Original Destination Extraction

Platform-specific APIs retrieve original destination before DNAT:

**Linux (`SO_ORIGINAL_DST`):**
```c
struct sockaddr_in orig_addr;
socklen_t addr_len = sizeof(orig_addr);
getsockopt(fd, SOL_IP, SO_ORIGINAL_DST, &orig_addr, &addr_len);
```

**Fallback for older kernels (`IP_RECVORIGDSTADDR`):**
- Ancient but more compatible
- Receives destination as ancillary message

### Listener Flow

```
1. Socket binds to intercept port (e.g., 8888)
2. Set SO_REUSEADDR and SO_REUSEPORT
3. Accept incoming connections from iptables DNAT
4. For each connection:
   ├─ Extract SO_ORIGINAL_DST
   ├─ Query DNS if needed
   ├─ Register in Connection Manager
   ├─ Select proxy via Routing Engine
   ├─ Connect to proxy
   ├─ Relay bidirectional data
   ├─ On close: Unregister connection
   └─ Emit event to bus
```

### Connection States

```
Created:
  └─ Original dest extracted
     └─ Proxy selected
        └─ Connecting to proxy
           ├─ Connected (relay starts)
           │  └─ Data flowing
           │     └─ Close
           │
           └─ Connection failed
              └─ Retry next proxy or fail
```

### Relay Implementation

Two tasks per connection:
- **Client→Proxy**: Read from client socket, write to proxy socket
- **Proxy→Client**: Read from proxy socket, write to client socket

Both tasks run concurrently:
```
tokio::select! {
  result = client_to_proxy() => { handle_close }
  result = proxy_to_client() => { handle_close }
}
```

### Performance

- Zero-copy relay (kernel buffer pinning)
- Bidirectional throughput not limited by relay
- Latency: Original destination extraction overhead + proxy selection
- Concurrent connections limited by file descriptor count

---

## Direct Proxy Subsystem

### SOCKS5 Protocol

Handshake sequence:
```
1. Client connects to SOCKS5 listener
2. Negotiation:
   - Client: [version(1), methods(1)]
   - Server: [version(1), selected_method(1)]
3. Authentication (if required)
4. Request:
   - Client: [version(1), command(1), reserved(1), atyp(1), dst_addr, dst_port]
   - Server: [version(1), status(1), reserved(1), atyp(1), bind_addr, bind_port]
5. Data relay begins
```

### Connection Handler

```
Listener → Accept connection
         → Parse SOCKS5 handshake
         → Select proxy via Rotation Engine
         → Connect to proxy
         → Send SOCKS5 CONNECT request
         → Relay data client ↔ proxy
         → Close on error
```

### Proxy Selection

Same as TProxy:
1. Load cursor from rotation engine
2. Check health status of selected proxy
3. Connect with timeout
4. On failure: retry next proxy
5. After retries exhausted: close connection

---

## Routing & Connection Management

### Connection Manager

`Arc<DashMap<ConnectionId, ConnectionState>>`

```rust
ConnectionState {
  id: Uuid,
  origin: SocketAddr,
  destination: SocketAddr,
  proxy: SocketAddr,
  state: ConnectionPhase,
  created_at: Instant,
  closed_at: Option<Instant>,
  bytes_tx: u64,
  bytes_rx: u64
}
```

### Connection Phases

```
Created → Opening → Open → Closing → Closed
  │         │        │       │        │
  │         │        │       │        └─ Emit ConnectionClosed
  │         │        │       └─ Proxy connection closed
  │         │        └─ Data relay active
  │         └─ Attempting proxy connection
  └─ Connection allocated, waiting to open
```

### Concurrent Access Patterns

**Reads** (lock-free via DashMap):
- Status queries
- Metrics collection
- Connection enumeration

**Writes** (rare):
- Connection creation
- Connection closure
- State transitions

---

## Collector Service (Health Monitoring)

### Health Check Algorithm

```
For each proxy in config:
  1. Create TCP connection to proxy
  2. Send SOCKS5 CONNECT to test destination
  3. Measure latency
  4. Record result
     ├─ Success: Mark healthy, store latency
     └─ Failure: Mark unhealthy
  5. Emit HealthStatusChanged event
  6. Update DashMap
```

### Failure Counting

```
Consecutive Failures:
0 → Success → Reset to 0
1 → Failure
2 → Failure
3 → Failure
3+ → Mark Unhealthy, pause checks
   → On next success: Reset to 0
```

### Check Frequency

- Healthy proxies: Check every 30 seconds
- Unhealthy proxies: Backoff exponentially
- After recovery: Resume normal frequency

---

## Monitoring & Events

### Event Bus Design

```
Publisher          Bus               Subscriber
(Service)        (Broadcast)        (Logger, Metrics)
  │
  ├─ Emit event ─────►
                       ├─ All subscribers receive copy
                       │
                       ├─────► Logger writes to file
                       │
                       ├─────► Metrics aggregates
                       │
                       └─────► Collector updates status
```

### Event Ordering Guarantees

Events within single publisher: Strictly ordered

Events across publishers: Eventual consistency (no strict ordering guarantee)

### Subscriber Pattern

```
1. Subscribe to bus: let mut rx = bus.subscribe()
2. Loop in task:
   loop {
     match rx.recv().await {
       Ok(event) => handle_event(event),
       Err(_) => break,  // Bus dropped
     }
   }
```

### Event Publishing

```
Runtime emits ServiceStartup
  └─ Logger writes "Service started at <time>"
     Metrics increments counter
     Collector verifies health

DNS cache publishes CacheHit
  └─ Logger writes for debug
     Metrics increments cache hit counter

Connection opens
  └─ Metrics increments active connections
     Logger writes connection info (if DEBUG)
     Collector may update proxy health
```

---

## IPC Communication

### Protocol

```
Unix Domain Socket (/tmp/phantomrelay.sock)

Client (prctl):
  1. Connect to socket
  2. Serialize IPCRequest to JSON
  3. Send over socket
  4. Wait for IPCResponse
  5. Parse and display

Server (phantomrelayd):
  1. Listen on socket
  2. Accept connection
  3. Deserialize IPCRequest
  4. Pass to RuntimeController
  5. Serialize response
  6. Send back
```

### Request/Response Types

**IPCRequest**
```
enum IPCRequest {
  Runtime(RuntimeCommand),
}
```

**IPCResponse**
```
enum IPCResponse {
  Success { message: String },
  Status { services: Vec<ServiceStatus> },
  Error { message: String },
}
```

### Service Status

```
ServiceStatus {
  name: String,           // "dns", "proxy", etc.
  active: bool,           // Running vs stopped
  is_mode: bool,          // Service vs mode
}
```

---

## Error Handling Strategy

### Result-Based Propagation

All fallible operations return `Result<T, Error>`:
- Startup errors prevent service creation
- Connection errors emit events but don't panic
- Health check failures update status
- Handler errors trigger retry logic

### Error Extension Trait

```
Result<T> → add_context("operation X")
          → Error { message, context, source }
```

### Recovery Strategies

| Error Type | Strategy |
|------------|----------|
| DNS upstream unreachable | Fall back to cache, retry |
| Proxy connection failed | Try next proxy in rotation |
| Service startup error | Return error to CLI, prevent start |
| Event bus full | Drop event (backpressure) |
| Cache write collision | Retry with exponential backoff |
| IPC socket error | Close connection, listeners retry |

---

## Configuration Management

### DNS Configuration
```
DNSConfig {
  resolver_addr: IpAddr,           // Upstream DNS
  cache_ttl_max: u32,              // Maximum cache TTL
  cache_saturation: bool,          // Turbo mode
  prewarmer_interval_secs: u64,    // Refresh timing
}
```

### Proxy Configuration
```
ProxyConfig {
  proxies: Vec<ProxyInfo>,         // Proxy list
  connection_timeout_secs: u64,    // TCP timeout
  retry_attempts: usize,           // Retry count
}
```

### Rotation Configuration
```
RotationConfig {
  rotate_sec: u64,                 // Interval (default 60)
}
```

All configs are `Arc` wrapped for shared immutable access or `Arc<Mutex>` for controlled mutation.

---

## Concurrency Model

### Key Invariants

1. **Single-threaded per service**: Each service runs in own Tokio task
2. **Shared state protection**: All `Arc` or interior mutability
3. **No blocking calls**: All I/O is async
4. **Graceful shutdown**: CancellationTokens signal tasks

### Synchronization Primitives Used

| Type | Use Case | Characteristics |
|------|----------|-----------------|
| `Arc<DashMap>` | DNS cache, connections | Lock-free reads, optimal contention |
| `Arc<RwLock>` | Route context | Multiple readers, exclusive writer |
| `Arc<Mutex>` | Config mutations | Simple mutual exclusion |
| `broadcast::Channel` | Event bus | Multi-producer, multi-consumer |
| `CancellationToken` | Graceful shutdown | Non-allocating, one-shot signal |
| `AtomicUsize` | Cursor tracking | Lockless atomic operations |

### Deadlock Prevention

- No nested locks
- Lock ordering: RwLock < Mutex < DashMap access
- Events don't require locks (immutable copies)

