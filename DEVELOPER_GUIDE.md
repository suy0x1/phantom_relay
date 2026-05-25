# Developer Quick Reference

Essential information for developers working with PhantomRelay.

---

## Project Layout

```
phantom_relay/
├── README.md              # Main project documentation
├── ARCHITECTURE.md            # High-level architecture overview
├── COMPONENTS.md              # Component specifications
├── DATA_FLOWS.md             # Data flow patterns and examples
├── DEPLOYMENT.md             # Deployment and operations guide
│
├── cli/                       # Control tool (prctl)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs          # CLI entry point
│       ├── cli/
│       │   ├── args.rs      # Argument parsing
│       │   ├── client.rs    # IPC client
│       │   ├── mod.rs
│       │   └── parser.rs    # Command parsing
│       ├── ipc/
│       │   ├── mod.rs
│       │   └── protocol.rs  # IPC protocol definitions
│       └── runtime/
│           ├── commands.rs
│           ├── mod.rs
│           └── service.rs
│
└── phantomrelayd/             # Daemon
    ├── Cargo.toml
    └── src/
        ├── main.rs           # Daemon entry point
        ├── lib.rs           # Module declarations
        │
        ├── runtime/         # Service lifecycle
        │   ├── startup.rs   # Daemon initialization
        │   ├── controller.rs # Service management
        │   ├── context.rs   # Shared context
        │   ├── factories.rs # Service factories
        │   ├── commands.rs  # Command types
        │   ├── service.rs   # Service definitions
        │   └── signal.rs    # Signal handling
        │
        ├── config/          # Configuration structures
        │   ├── mod.rs
        │   ├── dns.rs
        │   ├── proxy.rs
        │   ├── tproxy.rs
        │   ├── rotation.rs
        │   ├── collector.rs
        │   └── service.rs
        │
        ├── dns/             # DNS resolution subsystem
        │   ├── mod.rs
        │   ├── listener.rs  # UDP listener
        │   ├── cache.rs     # Cache logic
        │   ├── parse.rs     # DNS parsing
        │   ├── doh.rs       # DNS-over-HTTPS
        │   ├── cleanup.rs   # Cache cleanup
        │   └── prewarmer/   # Cache prewarming
        │       ├── mod.rs
        │       ├── refresh.rs
        │       ├── preload.rs
        │       └── packet.rs
        │
        ├── proxy/           # Direct proxy server
        │   ├── mod.rs
        │   ├── server.rs    # SOCKS5 listener
        │   └── handler.rs   # Connection handling
        │
        ├── tproxy/          # Transparent proxy
        │   ├── mod.rs
        │   ├── listener.rs  # TProxy listener
        │   ├── relay.rs     # Data relay
        │   └── original_dst.rs # Destination extraction
        │
        ├── routing/         # Connection routing
        │   ├── mod.rs
        │   ├── manager.rs   # Connection manager
        │   ├── connection.rs # Connection types
        │   ├── connect.rs   # Connection logic
        │   ├── proxy.rs     # Proxy routing
        │   └── types/
        │       ├── mod.rs
        │       └── socks5.rs # SOCKS5 types
        │
        ├── subsystems/      # Major subsystems
        │   ├── mod.rs
        │   ├── rotation/    # Proxy rotation engine
        │   │   ├── mod.rs
        │   │   ├── manager.rs # Rotation manager
        │   │   ├── route.rs  # Route context
        │   │   └── service.rs # Rotation service
        │   │
        │   └── network/     # Network management
        │       ├── mod.rs
        │       ├── manager.rs
        │       ├── capabilities.rs
        │       └── rules.rs
        │
        ├── collector/       # Health monitoring
        │   ├── mod.rs
        │   ├── collector.rs # Health check logic
        │   ├── health.rs    # Health status
        │   ├── manager.rs   # Health manager
        │   └── service.rs   # Collector service
        │
        ├── monitor/         # Event system & logging
        │   ├── mod.rs
        │   ├── bus.rs       # Event broadcast bus
        │   ├── events.rs    # Event definitions
        │   ├── logger.rs    # Event logger
        │   └── error_ext.rs # Error extensions
        │
        ├── metrics/         # Metrics collection
        │   ├── mod.rs
        │   ├── listener.rs  # Metrics listener
        │   └── metrics.rs   # Metrics definitions
        │
        ├── ipc/             # IPC communication
        │   ├── mod.rs
        │   ├── server.rs    # IPC server
        │   └── protocol.rs  # Protocol definitions
        │
        ├── errors/          # Error types
        │   └── mod.rs
        │
        └── utils/           # Utilities
            └── mod.rs
```

---

## Adding New Functionality

### Adding a New Service

**1. Define service type** in `runtime/service.rs`:
```rust
pub enum Service {
    MyNewService,
    // ...
}
```

**2. Create factory** in `runtime/factories.rs`:
```rust
pub fn my_service(ctx: Arc<RuntimeContext>) -> Arc<dyn Fn(CancellationToken) -> ServiceFuture + Send + Sync> {
    Arc::new(move |cancel: CancellationToken| {
        let ctx = ctx.clone();
        Box::pin(async move {
            // Emit startup event
            ctx.bus.emit(Event::ServiceStartup { ... })?;
            
            // Subscribe to events if needed
            let mut rx = ctx.bus.subscribe();
            
            // Main service loop
            loop {
                tokio::select! {
                    // Listen for cancellation
                    _ = cancel.cancelled() => break,
                    
                    // Service logic
                    _ = my_work(&ctx) => {},
                }
            }
            
            // Emit shutdown event
            ctx.bus.emit(Event::ServiceShutdown { ... })?;
            Ok(())
        })
    })
}
```

**3. Register in controller** in `runtime/controller.rs`:
```rust
Service::MyNewService => {
    let x = self.start_service("my_service", my_service(self.ctx.clone()))?;
    return Ok(x);
}
```

**4. Add CLI command** if user-facing in `cli/cli/parser.rs`

### Adding a New Event Type

**1. Add variant** in `monitor/events.rs`:
```rust
pub enum Event {
    MyNewEvent {
        data: String,
        timestamp: String,
    },
    // ...
}
```

**2. Emit from service**:
```rust
ctx.bus.emit(Event::MyNewEvent {
    data: "some data".to_string(),
    timestamp: chrono::Utc::now().to_rfc3339(),
})?;
```

**3. Subscribe in logger** in `monitor/logger.rs`:
```rust
Event::MyNewEvent { data, timestamp } => {
    println!("[{}] {}", timestamp, data);
}
```

### Adding Configuration

**1. Create config struct** in `config/myconfig.rs`:
```rust
pub struct MyConfig {
    pub param1: String,
    pub param2: u64,
}
```

**2. Add to RuntimeContext** in `runtime/context.rs`:
```rust
pub struct RuntimeContext {
    pub my_config: Arc<Mutex<MyConfig>>,
    // ...
}
```

**3. Initialize in startup** in `runtime/startup.rs`:
```rust
let ctx = RuntimeContext {
    my_config: Arc::new(Mutex::new(MyConfig::default())),
    // ...
};
```

---

## Common Patterns

### Async Loop with Interval

```rust
use tokio::time::{interval, Duration};

async fn my_service(ctx: Arc<RuntimeContext>, cancel: CancellationToken) -> Result<()> {
    let mut ticker = interval(Duration::from_secs(60));
    
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = ticker.tick() => {
                // Do work every 60 seconds
            }
        }
    }
    Ok(())
}
```

### Concurrent HashMap Access

```rust
use dashmap::DashMap;
use std::sync::Arc;

let map = Arc::new(DashMap::new());

// Spawn multiple tasks accessing concurrently
for i in 0..10 {
    let map = map.clone();
    tokio::spawn(async move {
        // Read (lock-free)
        if let Some(entry) = map.get(&i) {
            println!("{}", *entry);
        }
        
        // Write
        map.insert(i, value);
    });
}
```

### Event Bus Subscription

```rust
let bus = Arc::new(Bus::new(128));
let mut rx = bus.subscribe();

tokio::spawn(async move {
    while let Ok(event) = rx.recv().await {
        match event {
            Event::ServiceStartup { .. } => { /* handle */ },
            Event::Error { err, .. } => { /* handle */ },
            _ => {},
        }
    }
});
```

### Graceful Shutdown with CancellationToken

```rust
use tokio_util::sync::CancellationToken;

let cancel = CancellationToken::new();
let cancel_clone = cancel.clone();

// In service
loop {
    tokio::select! {
        _ = cancel_clone.cancelled() => {
            println!("Shutting down gracefully");
            break;
        }
        _ = my_work() => {
            // Do work
        }
    }
}

// To signal shutdown
cancel.cancel();
```

### Error Context

```rust
use anyhow::{Result, Context};

async fn connect_to_proxy(addr: &str) -> Result<TcpStream> {
    TcpStream::connect(addr)
        .await
        .context(format!("Failed to connect to proxy at {}", addr))
}

// Usage
match connect_to_proxy(addr).await {
    Ok(stream) => { /* handle success */ },
    Err(e) => {
        eprintln!("Error: {}", e);
        eprintln!("Context: {:?}", e.source());
    }
}
```

---

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Single-threaded
cargo test -- --test-threads=1
```

### Test Structure

Create tests in `src/module.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_my_function() {
        let result = my_function().await;
        assert!(result.is_ok());
    }
}
```

---

## Performance Debugging

### Profile with Flamegraph

```bash
cargo install flamegraph
cargo flamegraph --bin phantomrelayd -- [args]
# Opens flamegraph.svg
```

### Tokio Console

```toml
[dependencies]
tokio = { version = "1", features = ["full", "tracing"] }
```

```bash
cargo install tokio-console
TOKIO_CONSOLE=1 cargo run
tokio-console # in another terminal
```

### Metrics Inspection

Start daemon and query metrics:
```bash
curl http://localhost:9090/metrics
```

---

## Debugging Tips

### Verbose Logging

Enable event logging to see all system activities:
```bash
prctl start logger
prctl status  # Confirm logger is running
```

### Connection Inspection

Check active connections:
```rust
// In debug mode, could add CLI command to dump connections
for entry in runtime.ctx.conn_map.iter() {
    println!("{:?}", entry.value());
}
```

### Cache Inspection

Query current DNS cache:
```rust
for entry in runtime.ctx.dns_cache.iter() {
    println!("Domain: {}, TTL: {}, Hits: {}", 
        entry.key(), entry.value().ttl, entry.value().hits);
}
```

### Health Status

Check proxy health:
```rust
for entry in runtime.ctx.healthy_proxies.iter() {
    println!("Proxy: {}, Status: {:?}", entry.key(), entry.value());
}
```

---

## Common Issues

### "Service already running"
- Stop the service first: `prctl stop service_name`
- Or restart: `prctl restart service_name`

### "Failed to bind port"
- Port already in use: Check with `lsof -i :port`
- Insufficient permissions: May need elevated privileges

### DNS not working
- Check DNS service is running: `prctl status`
- Verify upstream resolver is reachable
- Check cache with appropriate debug logging

### High memory usage
- DNS cache may be growing unbounded
- Ensure cache cleanup service is running
- Verify prewarmer interval is reasonable

### Slow connections
- Check proxy health: `prctl status`
- Monitor cache hit rate
- Check if rotation is happening frequently
- Verify network connectivity to proxies

---

## Code Style

### Naming Conventions
- Types: `PascalCase` (e.g., `RuntimeController`)
- Functions: `snake_case` (e.g., `start_service`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RETRIES`)
- Module names: `lowercase` (e.g., `runtime`, `monitor`)

### Error Handling
Always use `Result<T>` instead of panicking:
```rust
// Good
fn do_work() -> Result<()> {
    // ...
}

// Avoid
fn do_work() -> () {
    // ...something.unwrap() // ❌
}
```

### Comments
Prefer code clarity over comments:
```rust
// Good: Self-documenting code
let should_skip = is_unhealthy && consecutive_failures > MAX_RETRIES;

// Bad: Unclear code needing comment
let s = h && f > 3;  // skip if unhealthy and failed
```

### Imports
Group and organize:
```rust
// Standard library
use std::net::SocketAddr;
use std::sync::Arc;

// Third-party
use tokio::time::interval;
use dashmap::DashMap;

// Internal
use crate::runtime::context::RuntimeContext;
use crate::monitor::events::Event;
```

---

## Documentation Guidelines

### Function Documentation
```rust
/// Selects next proxy from rotation list
/// 
/// Returns the proxy address if healthy, otherwise
/// attempts next proxy in list until one is found.
/// 
/// # Errors
/// Returns error if no healthy proxies available
/// 
/// # Example
/// ```
/// let proxy = select_proxy(&healthy_proxies)?;
/// ```
pub async fn select_proxy(healthy_proxies: &DashMap<SocketAddr, Health>) -> Result<SocketAddr> {
```

### Module Documentation
```rust
//! DNS resolution subsystem
//!
//! Handles DNS query resolution with caching,
//! background prewarming, and cleanup.

pub mod cache;
pub mod listener;
```

---

## Release Checklist

- [ ] All tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Code formatted: `cargo fmt`
- [ ] Version updated in `Cargo.toml`
- [ ] CHANGELOG updated
- [ ] Documentation reviewed
- [ ] Benchmark performance vs. previous version
- [ ] Test on target platform
- [ ] Create git tag
- [ ] Build release binary: `cargo build --release`

