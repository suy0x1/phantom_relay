# Data Flow & Patterns

## Overview

This document describes how data moves through PhantomRelay and how the major subsystems interact.

---

## System Traffic Flow

```mermaid
flowchart LR
    APP[Application]
    KERNEL[Kernel Interception]
    TPROXY[TProxy Gateway]
    ROUTER[Route Manager]
    PROXY[Selected Proxy]
    TARGET[Destination]

    APP --> KERNEL
    KERNEL --> TPROXY
    TPROXY --> ROUTER
    ROUTER --> PROXY
    PROXY --> TARGET
```

### Flow

1. Application creates a connection.
2. Traffic is intercepted by the network layer.
3. PhantomRelay receives the connection.
4. Route Manager selects a healthy route.
5. Connection is forwarded through the selected proxy.
6. Data is relayed until termination.
7. Session is removed from the registry.

---

## DNS Resolution Flow

```mermaid
flowchart TD
    QUERY[DNS Query]
    CACHE{Cache Hit?}
    RESOLVER[Upstream Resolver]
    STORE[Cache Store]
    REPLY[DNS Response]

    QUERY --> CACHE
    CACHE -->|Yes| REPLY
    CACHE -->|No| RESOLVER
    RESOLVER --> STORE
    STORE --> REPLY
```

### Cache Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Miss
    Miss --> Cached
    Cached --> Hit
    Hit --> Hit
    Cached --> Refreshing
    Refreshing --> Cached
    Cached --> Expired
    Expired --> Removed
    Removed --> [*]
```

---

## Route Selection Flow

```mermaid
flowchart LR
    REQUEST[Connection Request]
    HEALTH[Health Registry]
    ROUTES[Available Routes]
    SELECT[Route Selection]
    RESULT[Selected Route]

    REQUEST --> SELECT
    HEALTH --> SELECT
    ROUTES --> SELECT
    SELECT --> RESULT
```

### Selection Rules

- Only healthy routes are eligible.
- Failed routes are skipped.
- Recovered routes automatically rejoin rotation.
- Selection strategy remains independent of transport implementation.

---

## Proxy Health Flow

```mermaid
stateDiagram-v2
    [*] --> Unknown
    Unknown --> Healthy
    Unknown --> Unhealthy

    Healthy --> Healthy
    Healthy --> Unhealthy

    Unhealthy --> Unhealthy
    Unhealthy --> Healthy
```

### Health Evaluation

1. Health Monitor performs validation.
2. Results update the Health Registry.
3. Route Manager consumes registry state.
4. Failed routes are excluded.
5. Recovered routes are restored.

---

## Service Management Flow

```mermaid
flowchart TD
    CLI[CLI]
    IPC[IPC Layer]
    RUNTIME[Runtime Controller]
    SERVICE[Service]

    CLI --> IPC
    IPC --> RUNTIME
    RUNTIME --> SERVICE
```

### Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Stopped
    Stopped --> Starting
    Starting --> Running
    Running --> Restarting
    Restarting --> Running
    Running --> Stopping
    Stopping --> Stopped
```

---

## Event Flow

```mermaid
flowchart LR
    PRODUCER[Subsystem]
    BUS[Event Bus]
    LOGGER[Logger]
    METRICS[Metrics]
    MONITOR[Monitoring]

    PRODUCER --> BUS
    BUS --> LOGGER
    BUS --> METRICS
    BUS --> MONITOR
```

### Characteristics

- Fan-out distribution.
- Decoupled publishers and subscribers.
- No direct subsystem dependencies.
- Supports observability and monitoring.

---

## Connection Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Created
    Created --> Opening
    Opening --> Connected
    Connected --> Active
    Active --> Closing
    Closing --> Closed
    Closed --> [*]

    Opening --> Failed
    Failed --> Closed
```

---

## Failure Recovery

### DNS Failure

```mermaid
flowchart TD
    FAIL[Resolver Failure]
    BACKUP[Backup Resolver]
    STALE[Serve Stale Entry]
    ERROR[Failure Response]

    FAIL --> BACKUP
    BACKUP -->|Success| STALE
    BACKUP -->|Failure| ERROR
```

### Proxy Failure

```mermaid
flowchart TD
    CONNECT[Connection Attempt]
    FAIL[Route Failure]
    NEXT[Next Healthy Route]
    CLOSE[Connection Failure]

    CONNECT --> FAIL
    FAIL --> NEXT
    FAIL --> CLOSE
```

---

## Concurrency Model

```mermaid
flowchart LR
    CONNECTIONS[Connection Workers]
    DNS[DNS Services]
    ROUTES[Route Services]
    EVENTS[Event Services]

    CONNECTIONS --> EVENTS
    DNS --> EVENTS
    ROUTES --> EVENTS
```

### Principles

- Independent services.
- Shared state ownership boundaries.
- Read-heavy optimization.
- Graceful shutdown.
- Failure isolation.

---

## Startup Sequence

```mermaid
sequenceDiagram
    participant Runtime
    participant Services
    participant IPC
    participant Monitoring

    Runtime->>Services: Initialize
    Services->>IPC: Register
    Services->>Monitoring: Publish State
```

---

## Shutdown Sequence

```mermaid
sequenceDiagram
    participant Runtime
    participant Services
    participant Registry

    Runtime->>Services: Stop
    Services->>Registry: Deregister
    Services->>Runtime: Complete
```

---

## Architectural Invariants

1. Traffic always passes through route selection.
2. Route selection only uses healthy routes.
3. DNS cache remains optional but transparent.
4. Services remain independently restartable.
5. Monitoring never participates in request processing.
6. Failures remain localized whenever possible.
7. Shutdown is always coordinated by the Runtime Controller.
