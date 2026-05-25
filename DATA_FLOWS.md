# Data Flow & Patterns

Detailed walkthrough of key operational patterns and data flows in PhantomRelay.

---

## Complete Request Paths

### Path 1: System Traffic Through TProxy

**Scenario**: A system application connects to example.com:443, traffic is intercepted by TProxy.

```
Timeline: T=0ms
System App (curl example.com:443)
  │
  ├─ [T=0] Initiates TCP connection to 93.184.216.34:443
  │
  └─ [Kernel] iptables DNAT intercepts
       │
       └─ Redirects to 127.0.0.1:8888 (TProxy listener)
            │
            ├─ [T=0.1] TProxy Listener accepts connection
            │
            ├─ [T=0.2] SO_ORIGINAL_DST syscall
            │          Returns: 93.184.216.34:443
            │
            ├─ [T=0.3] Connection Manager registers
            │          State: "Created"
            │          Origin: [127.0.0.1:54321]
            │          Destination: [93.184.216.34:443]
            │
            ├─ [T=0.4] Routing Engine called
            │          ├─ Atomic load cursor (value: 42)
            │          ├─ Check proxy[42%4] = Proxy#2
            │          ├─ Check healthy_proxies[Proxy#2] = HEALTHY
            │          ├─ Atomically increment cursor to 43
            │          └─ Return: Proxy#2 (10.0.0.5:1080)
            │
            ├─ [T=0.5] Event Bus emits:
            │          RoutingDecision event
            │
            ├─ [T=1.2] Connect to Proxy#2:1080
            │          ├─ TCP handshake (SYN, SYN-ACK, ACK)
            │          ├─ emit ProxyConnected event
            │          └─ State: "Connected"
            │
            ├─ [T=1.3] TProxy Handler starts relay
            │          ├─ Task 1: System App → Proxy#2 (bidirectional relay)
            │          ├─ Task 2: Proxy#2 → System App
            │          └─ State: "Open"
            │
            └─ [T=1.4 - T=X.X] Data flows:
                   System App → Proxy#2 → Target (example.com)
                   Target ← Proxy#2 ← System App
                   │
                   Event Bus publishes:
                   - ConnectionOpened event (metrics)
                   - Per-second data metrics (collector)
                   │
                   Eventually: Connection closes
                   │
                   ├─ [T=Y.Y] Close detected in relay task
                   │
                   ├─ [T=Y.Y+0.1] Connection Manager removes entry
                   │
                   ├─ [T=Y.Y+0.2] Event Bus emits:
                   │              ConnectionClosed event
                   │
                   └─ State: "Closed"
```

**Event Stream:**
```
T=0.5   RoutingDecision { }
T=1.2   ProxyConnected { proxy: 10.0.0.5, port: 1080 }
T=1.3   ConnectionOpened { 
          origin: 127.0.0.1:54321, 
          destination: 93.184.216.34:443,
          proxy: 10.0.0.5:1080 
        }
T=Y.Y+0.2 ConnectionClosed { 
             origin: 127.0.0.1:54321, 
             bytes_tx: 1248, 
             bytes_rx: 8945
           }
```

---

### Path 2: DNS Query Resolution

**Scenario**: Application sends DNS query for "api.example.com", first query then cached.

#### First Query (Cache Miss)

```
Timeline: T=0ms
Client Application
  │
  ├─ [T=0] Sends DNS query (UDP/53)
  │         QNAME: api.example.com
  │         QTYPE: A (IPv4)
  │
  └─ DNS Listener receives packet
       │
       ├─ [T=0.1] Parse domain from wire format
       │          Extracted: "api.example.com"
       │
       ├─ [T=0.2] DashMap lookup in cache
       │          Key: "api.example.com"
       │          Result: NOT_FOUND (cache miss)
       │
       ├─ [T=0.3] Event Bus emits:
       │          DNSCacheMiss { domain: "api.example.com" }
       │
       ├─ [T=0.4] Query upstream resolver
       │          Target: 8.8.8.8:53 (configurable)
       │          Timeout: 5 seconds
       │
       ├─ [T=150] Upstream response received
       │          Response: api.example.com → 93.184.216.35
       │          TTL: 300 seconds
       │
       ├─ [T=151] Update cache entry
       │          DashMap.insert("api.example.com", {
       │            response: <wire format>,
       │            ttl: 300,
       │            timestamp: T=151,
       │            hits: 0
       │          })
       │
       ├─ [T=152] Event Bus emits:
       │          Info { content: "Cache updated: api.example.com" }
       │
       ├─ [T=153] Prewarmer schedules refresh
       │          Refresh at: T=151 + (300 * 0.8) = T=391ms
       │          (refresh at 80% of TTL)
       │
       └─ [T=154] Send response to client
                 Response: api.example.com → 93.184.216.35
```

**Event Stream (First Query):**
```
T=0.3   DNSCacheMiss { domain: "api.example.com", resolver: 8.8.8.8 }
T=151   Info { content: "Cache updated: api.example.com" }
T=152   DNSRequest { domain: "api.example.com", resolver: 8.8.8.8 }
```

#### Second Query (Cache Hit - within TTL)

```
Timeline: T=160ms (9 ms later)
Client Application
  │
  ├─ [T=160] Sends same DNS query
  │
  └─ DNS Listener receives packet
       │
       ├─ [T=160.1] Parse domain: "api.example.com"
       │
       ├─ [T=160.2] DashMap lookup
       │            Key: "api.example.com"
       │            Result: FOUND
       │            Entry age: 9ms
       │            TTL: 300s (expires in 291ms)
       │
       ├─ [T=160.3] Validate TTL
       │            Current time: 160ms
       │            Expiry: 151 + 300 = 451ms
       │            Status: VALID (291ms remaining)
       │
       ├─ [T=160.4] Increment hit counter
       │            entry.hits.fetch_add(1)
       │
       ├─ [T=160.5] Event Bus emits:
       │            DNSCacheHit { domain: "api.example.com" }
       │
       └─ [T=160.6] Send cached response to client
                    Response: api.example.com → 93.184.216.35
                    Total latency: 0.6ms (vs 154ms first query!)
```

**Event Stream (Second Query):**
```
T=160.5 DNSCacheHit { domain: "api.example.com" }
```

#### Cache Prewarmer Refresh (Background)

```
Timeline: T=391ms
Prewarmer Task running in background
  │
  ├─ [T=391] Wakes up (scheduled for 80% of TTL)
  │
  ├─ [T=391.1] Check if entry still in cache
  │            ├─ Yes: Proceed
  │            └─ No: Skip (entry expired/removed)
  │
  ├─ [T=391.2] Query upstream for fresh copy
  │            Target: 8.8.8.8:53
  │
  ├─ [T=450] Response received
  │          TTL: 300 seconds (reset)
  │
  ├─ [T=451] Update cache entry
  │          timestamp: T=451 (reset)
  │          ttl: 300
  │
  ├─ [T=452] Event Bus emits:
  │          Info { content: "Prewarmed: api.example.com" }
  │
  └─ [T=453] Schedule next refresh
             Refresh at: T=451 + (300 * 0.8) = T=691ms
```

#### Cache Cleanup

```
Timeline: Every 1 hour
Cleanup Task
  │
  ├─ [Periodic] Iterate all cache entries
  │
  ├─ For each entry:
  │   ├─ Check if (current_time - timestamp) > ttl
  │   ├─ If yes: Remove from DashMap
  │   └─ If no: Keep
  │
  ├─ Event Bus emits:
  │  DNSCacheCleanup { entries_cleaned: 42 }
  │
  └─ Continue polling for next cycle
```

---

### Path 3: Proxy Rotation Service

**Scenario**: Proxy rotation service incrementally advances through proxy list.

```
Timeline: Every 60 seconds (configurable)

T=0     Proxy List: [P1, P2, P3, P4]
        Cursor: 0
        Current: P1
        Health: All HEALTHY

T=0.1   Service starts, spawns rotation task

T=60    Rotation interval fires
        │
        ├─ Atomic load cursor: 0
        ├─ Increment to cursor = 1
        ├─ Emit RotateProxy event
        ├─ Update current_route
        └─ Next connection uses P2

        Current: P2
        Cursor: 1

T=120   Interval fires again
        │
        ├─ Atomic load cursor: 1
        ├─ Increment to cursor = 2
        └─ Next connection uses P3

        Current: P3
        Cursor: 2

T=180   Interval fires
        │
        ├─ Atomic load cursor: 2
        ├─ Increment to cursor = 3
        └─ Next connection uses P4

        Current: P4
        Cursor: 3

T=240   Interval fires
        │
        ├─ Atomic load cursor: 3
        ├─ Increment to cursor = 0 (wrap: 3+1 % 4)
        └─ Next connection uses P1 (cycle repeats)

        Current: P1
        Cursor: 0

T=300   Meanwhile, Collector detects P2 is down
        │
        ├─ Mark P2 unhealthy in registry
        ├─ Emit ProxyFailed event
        └─ Healthy list now: [P1, P3, P4]

T=360   Rotation interval fires
        │
        ├─ Atomic load cursor: 0
        ├─ Index into proxy list
        ├─ Check health of proxy[0] = P1: HEALTHY ✓
        ├─ Use P1
        ├─ Increment cursor to 1
        └─ Next connection will check proxy[1] = P3 (skip P2)

        (Cursor still advances by 1, but routing logic skips dead proxies)

T=420   Rotation fires again
        │
        ├─ Load cursor: 1
        ├─ proxy[1] = P3: Check health: HEALTHY ✓
        ├─ Use P3
        ├─ Increment cursor to 2
        └─ Next: Will check proxy[2] = P4

T=1200  Collector re-checks P2
        │
        ├─ P2 responds: Mark HEALTHY again
        ├─ Emit ProxyConnected event
        └─ Healthy list restored: [P1, P2, P3, P4]

        (Rotation continues with all 4 proxies in cycle)
```

---

### Path 4: Service Lifecycle Management

**Scenario**: User issues `prctl start dns` via CLI.

```
Timeline: T=0ms
User terminal
  │
  ├─ [T=0] Command: prctl start dns
  │
  └─ CLI Tool (prctl)
       │
       ├─ [T=1] Parse command
       │        Command parsed: Start(DNS)
       │
       ├─ [T=2] Connect to IPC socket
       │        Target: /tmp/phantomrelay.sock
       │        Status: CONNECTED
       │
       ├─ [T=3] Serialize IPCRequest
       │        Request::Runtime(Start(DNS))
       │        Serialized to JSON
       │
       ├─ [T=4] Send over socket
       │
       └─ [T=5] Receive and await response
            
            Daemon Side:
            │
            ├─ [T=5.1] IPC Server receives request
            │
            ├─ [T=5.2] Deserialize IPCRequest
            │          Parsed: Start(DNS)
            │
            ├─ [T=5.3] RuntimeController::handle_commands
            │          Match: Start(Service::DNS)
            │
            ├─ [T=5.4] start_service("dns", factory)
            │          ├─ Check if already running: No
            │          ├─ Create CancellationToken
            │          ├─ Call factory: dns_service(ctx)
            │          └─ Spawns Tokio task
            │
            ├─ [T=5.5] Tokio task starts
            │          ├─ Bind UDP socket to port 53
            │          ├─ Accept DNS queries
            │          └─ Subscribe to event bus
            │
            ├─ [T=5.6] Event Bus emits:
            │          ServiceStartup { 
            │            service_name: "dns",
            │            port: 53,
            │            timestamp: "2024-05-25T10:30:00Z"
            │          }
            │
            ├─ [T=5.7] Logger subscriber writes:
            │          "[ok] DNS service started on port 53"
            │
            ├─ [T=5.8] Metrics subscriber increments:
            │          services_started_total{service="dns"} += 1
            │
            ├─ [T=5.9] Insert ServiceHandle into DashMap
            │          Key: "dns"
            │          Value: { task, cancel_token }
            │
            ├─ [T=6.0] Serialize response
            │          Response::Success { 
            │            message: "DNS service started"
            │          }
            │
            └─ [T=6.1] Send response over socket

            CLI Tool (receiving response)
            │
            ├─ [T=6.2] Deserialize IPCResponse
            │
            ├─ [T=6.3] Display result
            │          "[ok] DNS service started"
            │
            └─ [T=6.4] Exit successfully

Timeline Result: User sees "[ok] DNS service started" in ~6ms
                 DNS service now running and listening
```

---

## State Transitions

### Connection State Machine

```
    ┌──────────────┐
    │   Created    │
    └──────┬───────┘
           │ SO_ORIGINAL_DST extracted
           │ Connection manager registers
           ▼
    ┌──────────────┐
    │   Opening    │
    └──────┬───────┘
           │ Route selection
           │ Proxy connection initiating
           ├─ Success ──────┐
           │                ▼
           │         ┌────────────┐
           │         │  Connected │
           │         └────┬───────┘
           │              │ Relay starts
           │              ▼
           │         ┌────────────┐
           │         │    Open    │
           │         └────┬───────┘
           │              │ Data flowing
           │              │ Close detected
           │              ▼
           │         ┌────────────┐
           │         │  Closing   │
           │         └────┬───────┘
           │              │ Clean shutdown
           │              ▼
           │         ┌────────────┐
           │         │   Closed   │
           │         └────────────┘
           │
           └─ Failure
              │
              ├─ Retry ──┐
              │          └─► Back to Opening
              │
              └─ Exhausted
                 │
                 ▼
            ┌────────────┐
            │   Error    │
            └────────────┘
```

### Service State Machine

```
    ┌──────────────┐
    │  Not Running │
    └──────┬───────┘
           │ CLI: start
           ▼
    ┌──────────────┐
    │  Starting    │
    └──────┬───────┘
           │ Factory creates task
           │ Tokio spawns
           ▼
    ┌──────────────┐
    │   Running    │◄─── CLI: restart ──┐
    └──────┬───────┘                      │
           │ CLI: stop              (stop + start)
           ▼
    ┌──────────────┐
    │  Stopping    │
    └──────┬───────┘
           │ CancellationToken signal
           │ Await task completion
           ▼
    ┌──────────────┐
    │  Not Running │
    └──────────────┘
```

### Proxy Health State Machine

```
    ┌──────────┐
    │ Unknown  │
    └────┬─────┘
         │ Collector runs first check
         ├─ Success ──────────────┐
         │                        ▼
         │                 ┌──────────────┐
         │                 │   Healthy    │◄─┐
         │                 └──┬───────────┘  │
         │                    │              │
         │                    │ Check fails  │
         │                    │ (1-2 times)  │
         │                    │              │
         │                    ├─ Retry ──────┘
         │                    │
         │                    ├─ 3+ failures
         │                    ▼
         │            ┌──────────────┐
         │            │  Unhealthy   │
         │            └──┬───────────┘
         │               │
         │               │ Check succeeds
         │               │ (reset counter)
         │               ▼
         │            Back to Healthy
         │
         └─ Failure
            │
            ▼
         ┌──────────────┐
         │  Unhealthy   │
         └──────────────┘
```

---

## Concurrency Scenarios

### Scenario 1: Simultaneous DNS Queries

```
Query 1: api.example.com
  └─ DashMap lookup ("api.example.com")
     └─ Arc<DashMap> read (no lock needed)
        └─ Returns Some(entry) → reply immediately

Query 2: cdn.example.com (cache miss)
  └─ DashMap lookup ("cdn.example.com")
     └─ Not found → query upstream

Query 3: api.example.com (same as Query 1)
  └─ DashMap lookup ("api.example.com")
     └─ Concurrent read (no lock contention with Query 1)
        └─ Reads same entry simultaneously

Result: All 3 queries execute concurrently
        Queries 1 & 3 hit cache immediately (~microseconds)
        Query 2 waits for upstream (~milliseconds)
        No locks held during I/O
```

### Scenario 2: Connection Manager Under Load

```
100 concurrent connections opening
  │
  ├─ Each calls conn_manager.register(connection)
  │
  ├─ DashMap.insert() called 100 times
  │  ├─ Lock-free operation (shard-level locking)
  │  ├─ No global contention
  │  └─ All 100 complete in parallel
  │
  └─ Event Bus emits 100 ConnectionOpened events
     ├─ Broadcast to all subscribers
     ├─ Each subscriber processes independently
     └─ No blocking on bus publish

Result: Near-linear scaling up to CPU count
        Connection creation isn't bottleneck
```

### Scenario 3: Proxy Rotation with Health Checks

```
T=60s: Rotation increment
  │
  ├─ Atomic load cursor: 42
  ├─ Atomic increment to 43
  │
  └─ Meanwhile: Collector checks proxy health
     │
     ├─ Read from healthy_proxies DashMap
     ├─ Update status
     └─ No conflict with rotation increment (different operations)

Result: Rotation and health checks don't contend
        Atomic operations minimize lock time
        Both proceed concurrently
```

### Scenario 4: Route Context Access Patterns

```
Read-heavy: Route selection during connection open
Write-rare: Rotation service updates every 60 seconds

Using Arc<RwLock<RouteContext>>:

Writers (Rotation):
  └─ .write() lock acquired
     └─ Update current_proxy
     └─ Very infrequent (every 60s)
     └─ Readers temporarily block

Readers (Connection handlers):
  └─ .read() lock acquired
     └─ Load current_proxy
     └─ Hundreds per second
     └─ Multiple readers simultaneously (no block)
     └─ Writer must wait for all readers

Optimization: Write is rare, readers are many
            → RwLock much better than Mutex
```

---

## Error Recovery Paths

### DNS Query Fails

```
Query → Upstream timeout
  │
  ├─ Try backup resolver
  ├─ If backup succeeds
  │  └─ Return response
  │
  └─ If backup fails
     ├─ Check cache (expired entry might exist)
     ├─ If stale cached entry found
     │  └─ Return stale (with TTL=0)
     │
     └─ If nothing available
        ├─ Return SERVFAIL
        ├─ Emit Error event
        └─ Continue running (no panic)
```

### Proxy Connection Fails

```
Connection attempt → proxy unreachable
  │
  ├─ Increment proxy failure counter
  │
  ├─ If failures < MAX_RETRIES (typically 3)
  │  ├─ Rotate to next proxy
  │  ├─ Retry connection
  │  └─ Back to start (or success)
  │
  ├─ If failures >= MAX_RETRIES
  │  ├─ Emit ProxyFailed event
  │  ├─ Collector marks unhealthy
  │  ├─ Rotation engine skips this proxy
  │  └─ Close connection to client with error
  │
  └─ Collector schedules health recheck
     └─ If proxy recovers, mark healthy again
```

### Service Startup Error

```
Factory attempts startup → Error returned
  │
  ├─ Service not added to DashMap
  │
  ├─ CancellationToken dropped
  │ └─ Task cancelled (never spawned)
  │
  ├─ Serialize error response
  │  Response::Error { message: "Failed to bind port" }
  │
  └─ CLI displays error to user
     User can retry after fixing issue
```

### IPC Communication Breaks

```
CLI connects → Socket closes unexpectedly
  │
  ├─ CLI awaiting response
  │ ├─ Socket read returns Err
  │ └─ Display "Connection lost"
  │
  └─ User can retry command

Daemon side:
  │
  ├─ Writing response → pipe broken signal
  │ ├─ Catch and continue
  │ └─ Connection handler dropped
  │
  └─ Daemon continues serving other clients
```

