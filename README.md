# PhantomRelay

**Advanced Transparent Proxy & DNS Relay System**

A high-performance Rust-based proxy relay daemon that combines transparent proxy interception (TProxy), intelligent DNS resolution with caching, and dynamic proxy rotation. Designed for scenarios requiring seamless traffic interception, DNS control, and proxy failover.

---

## Features

### 🌐 **Transparent Proxy (TProxy)**
- Linux netfilter integration for transparent traffic interception
- Original destination extraction at kernel level
- Zero-modification connections from client perspective
- Bidirectional relay with minimal latency

### 🔄 **Intelligent Proxy Rotation**
- Round-robin proxy selection with configurable intervals
- Health-aware routing (automatically removes unhealthy proxies)
- Atomic cursor-based rotation for concurrent safety
- Real-time health monitoring and status tracking

### 📡 **Advanced DNS Resolution**
- Full DNS query caching with TTL support
- Background cache prewarming for frequently accessed domains
- Periodic cache cleanup to manage memory
- DNS-over-HTTPS (DoH) resolver support
- Cache saturation ("turbo") mode for aggressive preloading

### 🛡️ **Connection Management**
- Connection lifecycle tracking with DashMap
- SOCKS5 protocol support
- Graceful connection pooling
- Real-time connection state monitoring

### 📊 **Observability & Monitoring**
- Broadcast event bus for pub-sub monitoring
- Prometheus-compatible metrics
- Comprehensive logging system
- Real-time service health status
- Event-driven architecture for loose coupling

### 🎮 **Runtime Control**
- CLI tool (`prctl`) for service management
- Individual service start/stop/restart
- Mode control (e.g., DNS turbo mode)
- Real-time status inspection
- IPC communication between CLI and daemon

### ⚡ **Performance**
- Async/await throughout (Tokio runtime)
- Lock-free concurrent data structures (DashMap)
- Minimal allocations in hot paths
- Connection pooling and reuse

---

## System Architecture

```
┌──────────────────────────────────────────────────────┐
│                    PhantomRelay                      │
├──────────────────────────────────────────────────────┤
│                                                      │
│  CLI Tool (prctl)                                   │
│  └─ IPC Client                                      │
│     └─ Unix Socket                                  │
│                                                      │
│  Daemon (phantomrelayd)                             │
│  ├─ Runtime Controller (service lifecycle)          │
│  ├─ Event Bus (pub-sub)                             │
│  │                                                  │
│  ├─ DNS Subsystem                                   │
│  │  ├─ Listener (UDP/DoH)                           │
│  │  ├─ Cache (DashMap)                              │
│  │  ├─ Prewarmer (background refresh)               │
│  │  ├─ Cleaner (TTL expiry)                         │
│  │  └─ Refresh service                              │
│  │                                                  │
│  ├─ TProxy Subsystem                                │
│  │  ├─ Listener (intercept port)                    │
│  │  ├─ Original destination extraction              │
│  │  └─ Relay handler                                │
│  │                                                  │
│  ├─ Proxy Subsystem                                 │
│  │  ├─ Server (SOCKS5 listener)                     │
│  │  └─ Handler (connection relay)                   │
│  │                                                  │
│  ├─ Routing Layer                                   │
│  │  ├─ Connection Manager                           │
│  │  ├─ Proxy Router                                 │
│  │  └─ SOCKS5 protocol handler                      │
│  │                                                  │
│  ├─ Proxy Rotation Engine                           │
│  │  ├─ Cursor-based round-robin                     │
│  │  ├─ Healthy proxies registry                     │
│  │  └─ Route context (current proxy)                │
│  │                                                  │
│  ├─ Collector Service                               │
│  │  ├─ Health checks                                │
│  │  └─ Availability tracking                        │
│  │                                                  │
│  └─ Monitoring                                      │
│     ├─ Logger                                       │
│     └─ Metrics listener                             │
│                                                      │
└──────────────────────────────────────────────────────┘
```

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed component documentation.

---

## Building

### Prerequisites
- Rust 1.70+ (edition 2024)
- Linux kernel with netfilter support (for TProxy)
- libssl-dev (for HTTPS support)
- nftables v1.0.0+ 

### Build Commands

```bash
# Build the daemon
cd phantomrelayd
cargo build --release

# Build the CLI tool
cd ../cli
cargo build --release

# Run tests
cargo test
```

### Output Binaries
- `target/release/phantomrelayd` - Daemon executable
- `target/release/prctl` - CLI control tool

---

## Running

### Start the Daemon

```bash
# As root (required for TProxy operations)
sudo ./target/release/phantomrelayd
```

The daemon will:
1. Initialize event bus and runtime context
2. Spawn IPC server for CLI communication
3. Wait for commands via `prctl`
4. Handle graceful shutdown on SIGINT

### Controlling via CLI

```bash
# Check status of all services
prctl status

# Start specific services
prctl start dns
prctl start proxy
prctl start tproxy

# Stop services
prctl stop dns

# Restart a service
prctl restart proxy

# Enable DNS turbo mode (aggressive caching)
prctl enable dns-turbo

# Disable DNS turbo mode
prctl disable dns-turbo

# View all available services
prctl status  # Shows full service list
```

### Available Services
- **logger** - Event logging system
- **dns** - DNS resolution and caching
- **proxy** - SOCKS5 proxy server
- **tproxy** - Transparent proxy interceptor
- **proxy_rotator** - Proxy rotation engine
- **proxy_collector** - Health check service
- **cache_preloader** - Background cache prewarmer
- **cache_cleaner** - Cache expiry cleanup
- **cache_refresher** - Cache entry refresh
- **metrics** - Prometheus metrics endpoint

### Available Modes
- **dns-turbo** - Aggressive DNS cache saturation mode

---

## Configuration

Configuration is managed through the daemon's runtime context. Key configuration areas:

### DNS Configuration
```
- DNS resolver address
- Cache TTL settings
- Prewarmer interval
- Cache saturation (turbo mode)
```

### Proxy Configuration
```
- Proxy list
- Connection timeout
- Retry policy
```

### TProxy Configuration
```
- Intercept port
- Firewall rules
- Original destination extraction method
```

### Rotation Configuration
```
- Rotation interval (default: 60 seconds)
- Health check frequency
- Unhealthy proxy timeout
```

See configuration files in `phantomrelayd/src/config/` for detailed structure.

---

## Event System

PhantomRelay uses a broadcast event bus for loose coupling and observability:

```rust
// Events published to the bus:
Event::ServiceStartup { service_name, port, timestamp }
Event::ServiceShutdown { service_name, port, timestamp }
Event::DNSRequest { domain, resolver, timestamp }
Event::DNSCacheHit { domain, timestamp }
Event::DNSCacheMiss { domain, timestamp }
Event::ConnectionOpened { host, port, proxy, proxy_port, timestamp }
Event::ConnectionClosed { host, port, proxy, proxy_port, timestamp }
Event::ProxyConnected { host, port, timestamp }
Event::ProxyFailed { host, port, timestamp }
Event::Error { err, timestamp }
Event::RotateProxy { timestamp }
// ... and more
```

Subscribers (logger, metrics, collector) independently consume events without tight coupling to publishers.

---

## Connection Flow

### Transparent Proxy (TProxy) Flow
```
1. System traffic → iptables DNAT
2. TProxy Listener receives at intercept port
3. SO_ORIGINAL_DST extracts destination
4. Connection Manager tracks connection
5. Routing Engine selects proxy (round-robin + health)
6. SOCKS5 handler connects to selected proxy
7. Data relayed between client and upstream proxy
8. On close: Connection Manager removes entry
```

### Direct Proxy Flow
```
1. Client connects to SOCKS5 listener
2. SOCKS5 negotiation and authentication
3. Routing Engine selects proxy
4. Handler connects to selected proxy
5. CONNECT relay established
6. Data tunneled through proxy
```

### DNS Resolution Flow
```
1. DNS query arrives at UDP listener
2. Check cache (DashMap concurrent lookup)
3. Cache hit: Return immediately
4. Cache miss: Query upstream resolver
5. Update cache with TTL
6. Prewarmer schedules background refresh before TTL
7. Periodic cleanup removes expired entries
```

---

## Data Structures

### Core Concurrent Collections
- **Arc<DashMap>**: Lock-free concurrent hash map for DNS cache, connections, healthy proxies
- **Arc<RwLock>**: Route context for current proxy (high-read concurrency)
- **Arc<Mutex>**: Configuration (low-contention mutable state)
- **broadcast::Channel**: Event bus for pub-sub

### Thread Safety
- All shared state is `Send + Sync`
- No unsafe code in business logic
- Atomic operations for cursor-based rotation
- CancellationTokens for graceful shutdown

---

## Monitoring & Metrics

### Event Bus Subscribers
- **Logger**: Writes events to stdout/log files
- **Metrics Listener**: Aggregates events for Prometheus metrics
- **Collector**: Monitors proxy health and publishes status

### Key Metrics
- Connections opened/closed per second
- DNS cache hit rate
- Proxy health status
- Service uptime
- Event processing latency

### Health Status
Collected via periodic health checks:
```
Proxy Status → Healthy Proxies Registry
                    ↓
             Rotation Engine uses for routing
                    ↓
             Events published on changes
```

---

## Error Handling

All operations return `Result<T>` with context:
- Service startup errors propagate to CLI
- Connection errors emitted as events
- Health check failures tracked in proxy status
- Graceful degradation when proxies fail

---

## Performance Considerations

### Concurrency
- DashMap provides lock-free reads in hot path (DNS cache)
- Each service runs independently in Tokio task
- No global locks in connection handling

### Memory
- LRU/TTL cache prevents unbounded DNS cache growth
- Connection cleanup on close
- Prewarmer limits active refresh operations

### Latency
- Async I/O throughout (no blocking operations)
- Connection pooling reduces handshake overhead
- Cache hit path: single DashMap lookup (~nanoseconds)

---

## Development

### Project Structure
```
phantom_relay/
├── cli/                          # prctl command-line tool
│   └── src/
│       ├── cli/                 # Argument parsing and client
│       ├── ipc/                 # IPC communication
│       └── runtime/             # Command handling
│
├── phantomrelayd/               # Main daemon
│   └── src/
│       ├── config/              # Configuration structures
│       ├── dns/                 # DNS subsystem
│       ├── proxy/               # Proxy server
│       ├── routing/             # Connection routing
│       ├── subsystems/
│       │   ├── rotation/        # Proxy rotation engine
│       │   └── network/         # Network management
│       ├── tproxy/              # Transparent proxy
│       ├── collector/           # Health collector
│       ├── metrics/             # Metrics
│       ├── monitor/             # Event bus and logging
│       ├── runtime/             # Runtime control
│       └── ipc/                 # IPC server
│
└── ARCHITECTURE.md              # Detailed architecture
```

### Key Dependencies
- **tokio**: Async runtime
- **dashmap**: Lock-free concurrent collections
- **serde/serde_json**: Serialization
- **anyhow**: Error handling
- **socket2**: Low-level socket operations
- **fast-socks5**: SOCKS5 protocol
- **reqwest**: HTTP client (DoH support)
- **chrono**: Timestamps

### Adding a New Service
1. Create factory in `runtime/factories.rs`
2. Define service type in `runtime/service.rs`
3. Implement service logic
4. Register in `RuntimeController::handle_commands`
5. Emit lifecycle events to bus
6. Add CLI command if user-facing

---

## Troubleshooting

### Daemon Won't Start
- Check kernel supports netfilter/TProxy: `grep -i tproxy /boot/config-*`
- Verify running as root for TProxy operations
- Check IPC socket not in use: `lsof /tmp/phantomrelay.sock`

### High CPU Usage
- Verify DNS prewarmer interval is reasonable
- Check cache cleanup is running
- Monitor event bus subscribers aren't bottlenecks

### DNS Not Resolving
- Ensure DNS service is running: `prctl status`
- Check upstream resolver is reachable
- Monitor cache hit rate via metrics

### Proxy Failures
- Check proxy health: `prctl status` (collector service status)
- Verify proxy list configuration
- Check network connectivity to proxy hosts

---

## License

See LICENSE file in repository root.

---

## Contributing

Contributions welcome! Please:
1. Create feature branch
2. Add tests for new functionality
3. Ensure all tests pass: `cargo test`
4. Document design decisions in commit messages
5. Follow existing code style

---

## Architecture Deep Dive

For detailed component information, threading model, concurrency patterns, and data flow diagrams, see [ARCHITECTURE.md](./ARCHITECTURE.md).
