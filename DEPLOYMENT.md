# Deployment & Operations Guide

Instructions for deploying and operating PhantomRelay in production environments.

---

## Pre-Deployment Requirements

### System Requirements

- **OS**: Linux (kernel 4.10+ with netfilter support)
- **CPU**: 2+ cores recommended
- **Memory**: 256 MB minimum, 1 GB+ recommended
- **Disk**: 100 MB for binary and logs
- **Permissions**: Root or CAP_NET_ADMIN capability

### Kernel Configuration

Verify netfilter support for TProxy:

```bash
# Check if netfilter is enabled
grep -i tproxy /boot/config-$(uname -r)
# Should output: CONFIG_NETFILTER_XT_TARGET_TPROXY=m (or =y)

# Load netfilter module if needed
sudo modprobe nf_tproxy_core
sudo modprobe xt_TPROXY
```

### Port Requirements

| Port | Protocol | Purpose | Notes |
|------|----------|---------|-------|
| 53 | UDP | DNS | Or configured DNS listen port |
| 1080 | TCP | SOCKS5 proxy | Or configured proxy port |
| 8888 | TCP | TProxy intercept | Or configured TProxy port |
| 9090 | TCP | Metrics (optional) | Prometheus endpoint |
| Unix socket | IPC | CLI communication | /tmp/phantomrelay.sock |

### Dependencies

PhantomRelay uses rustls for TLS, so OpenSSL development libraries are not required:

```bash
# Ubuntu/Debian
sudo apt-get install -y build-essential

# CentOS/RHEL
sudo yum install -y gcc

# Fedora
sudo dnf install -y gcc
```

The Rust toolchain will provide all other required dependencies via Cargo.

---

## Building from Source

### Clone Repository

```bash
git clone https://github.com/yourusername/phantom_relay.git
cd phantom_relay
```

### Build Both Components

```bash
# Build daemon
cd phantomrelayd
cargo build --release
# Binary: target/release/phantomrelayd

cd ../cli
cargo build --release
# Binary: target/release/prctl
```

### Create Release Package

```bash
mkdir -p release/{bin,config,lib}

# Copy binaries
cp phantomrelayd/target/release/phantomrelayd release/bin/
cp cli/target/release/prctl release/bin/

# Copy configs (if any)
# cp phantomrelayd/config.toml release/config/

# Create tarball
tar czf phantom_relay-$(git describe --tags).tar.gz release/
```

---

## Installation

### Option 1: Manual Installation

```bash
# Create application directory
sudo mkdir -p /opt/phantom_relay
sudo mkdir -p /var/log/phantom_relay

# Copy binaries
sudo cp phantomrelayd/target/release/phantomrelayd /opt/phantom_relay/
sudo cp cli/target/release/prctl /opt/phantom_relay/

# Create symlink for easy access
sudo ln -s /opt/phantom_relay/prctl /usr/local/bin/prctl

# Set permissions
sudo chmod +x /opt/phantom_relay/phantomrelayd
sudo chmod +x /opt/phantom_relay/prctl
sudo chown root:root /opt/phantom_relay/phantomrelayd

# Set capabilities if not running as root
sudo setcap cap_net_admin=ep /opt/phantom_relay/phantomrelayd
```

### Option 2: Systemd Service

Create `/etc/systemd/system/phantomrelayd.service`:

```ini
[Unit]
Description=PhantomRelay Daemon
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStartPre=/usr/local/bin/prctl status  # Verify CLI works
ExecStart=/opt/phantom_relay/phantomrelayd
Restart=on-failure
RestartSec=5s
StandardOutput=journal
StandardError=journal
SyslogIdentifier=phantomrelayd
KillMode=mixed
KillSignal=SIGTERM
TimeoutStopSec=10s

# Capabilities instead of running as root (optional but recommended)
User=phantomrelay
Group=phantomrelay
AmbientCapabilities=CAP_NET_ADMIN
CapabilityBoundingSet=CAP_NET_ADMIN

[Install]
WantedBy=multi-user.target
```

Create user (if using non-root):

```bash
sudo useradd -r -s /bin/false -d /var/lib/phantom_relay phantomrelay
sudo mkdir -p /var/lib/phantom_relay
sudo chown phantomrelay:phantomrelay /var/lib/phantom_relay
```

Enable and start service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable phantomrelayd
sudo systemctl start phantomrelayd
```

### Option 3: Docker Deployment

Create `Dockerfile`:

```dockerfile
FROM rust:latest as builder

WORKDIR /app
COPY . .

RUN cd phantomrelayd && cargo build --release
RUN cd cli && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /opt/phantom_relay /var/log/phantom_relay
RUN useradd -r -s /bin/false phantomrelay

COPY --from=builder /app/phantomrelayd/target/release/phantomrelayd /opt/phantom_relay/
COPY --from=builder /app/cli/target/release/prctl /opt/phantom_relay/

RUN chmod +x /opt/phantom_relay/phantomrelayd /opt/phantom_relay/prctl
RUN chown -R phantomrelay:phantomrelay /opt/phantom_relay /var/log/phantom_relay

ENV PATH="/opt/phantom_relay:$PATH"

EXPOSE 53/udp 1080 8888 9090

USER phantomrelay
WORKDIR /opt/phantom_relay

ENTRYPOINT ["/opt/phantom_relay/phantomrelayd"]
```

Build and run:

```bash
docker build -t phantom_relay:latest .

docker run -d \
  --name phantomrelayd \
  --network host \
  --cap-add NET_ADMIN \
  -v /var/log/phantom_relay:/var/log/phantom_relay \
  phantom_relay:latest
```

---

## Configuration

### Configuration File (phantomrelay.toml)

PhantomRelay loads configuration from `phantomrelay.toml` in the daemon's working directory. The configuration file is loaded on startup and defines all subsystem parameters.

#### Complete Configuration Format

```toml
[dns]
# DNS listener address and port
host = "127.0.0.1"
port = 9002

# Maximum parallel DNS lookups
max_parallel_dns_lookups = 100

# Cache cleanup interval (seconds)
cache_cleanup_interval_secs = 30

# Cache refresh interval (seconds)
cache_refresh_secs = 5

# Minimum prewarmer hits threshold
min_prest_hits = 25

# DNS turbo mode (aggressive cache saturation)
cache_saturation = false

# Domains to preload in DNS cache
prewarm_domains = [
    "google.com",
    "github.com",
    "youtube.com",
    "chatgpt.com",
]

[proxy]
# Proxy server listen address and port
host = "127.0.0.1"
port = 9003

[tproxy]
# Transparent proxy listener address and port
host = "127.0.0.1"
port = 9001

[rotation]
# Proxy rotation interval (seconds)
rotate_sec = 60

[collector]
# Number of concurrent health check workers
total_workers = 100

# Health check latency threshold (ms)
latency = 3500

# Whether to fetch and verify public IP
fetch_public = false

# Path to proxy list file
path = "/path/to/proxies.txt"
```

#### Configuration Sections

**DNS Configuration:**
- `host`, `port`: Listen address for DNS queries
- `max_parallel_dns_lookups`: Concurrent lookup limit
- `cache_cleanup_interval_secs`: How often to remove expired entries
- `cache_refresh_secs`: How often to preemptively refresh entries before expiry
- `min_prest_hits`: Minimum hits before prewarming a domain
- `cache_saturation`: Enable aggressive cache preloading (DNS turbo mode)
- `prewarm_domains`: List of domains to keep in cache

**Proxy Configuration:**
- `host`, `port`: SOCKS5 server listen address

**TProxy Configuration:**
- `host`, `port`: Kernel-level interception listener address

**Rotation Configuration:**
- `rotate_sec`: Interval for rotating through available proxies

**Collector Configuration:**
- `total_workers`: Parallel health check threads
- `latency`: Timeout/latency threshold for health checks
- `fetch_public`: Whether to verify public IP during health checks
- `path`: Path to file containing proxy list

#### Loading Configuration

Configuration is loaded during daemon startup. To apply configuration changes:
1. Edit `phantomrelay.toml`
2. Restart the daemon: `prctl shutdown` then restart

Individual service restart is supported for reloading without full daemon restart.

---

## Debug & Inspection

### Debug Commands

Use `prctl debug` commands to inspect daemon state without stopping services:

```bash
# View all current configuration settings
prctl debug config

# List active connections with details
prctl debug conn

# Show DNS cache status and statistics
prctl debug dns

# Show proxy health and availability status
prctl debug proxy

# Show current proxy route selection
prctl debug route
```

Debug output is formatted for easy reading and shows real-time state from the running daemon.

---

## Proxy List

Proxy configuration comes from:
1. Configuration file (`path` setting in `[collector]` section)
2. Environment variables (PHANTOM_PROXIES)
3. Runtime API (if implemented)

Example proxy file format:
```
10.0.0.1:1080
10.0.0.2:1080
10.0.0.3:1080
```

One proxy per line in `address:port` format.

---

## Network Configuration

### Setting up TProxy Interception

Traffic must be redirected to TProxy listener via iptables:

#### For HTTP/HTTPS Traffic

```bash
#!/bin/bash
# Setup transparent proxy redirection

# Create iptables rules
sudo iptables -t mangle -A PREROUTING -p tcp --dport 80 \
  -j TPROXY --on-port 8888 --on-ip 127.0.0.1

sudo iptables -t mangle -A PREROUTING -p tcp --dport 443 \
  -j TPROXY --on-port 8888 --on-ip 127.0.0.1

# For all traffic (use with caution):
# sudo iptables -t mangle -A PREROUTING -p tcp \
#   -j TPROXY --on-port 8888 --on-ip 127.0.0.1

# Persist rules
sudo iptables-save > /etc/iptables/rules.v4
```

#### Routing Rules

Linux requires routing table setup for TPROXY:

```bash
#!/bin/bash
# Setup routing for TPROXY

# Create fwmark-based routing table
sudo ip rule add fwmark 1 lookup 100
sudo ip route add local 0.0.0.0/0 dev lo table 100

# Mark TPROXY packets
sudo iptables -t mangle -A PREROUTING -p tcp --dport 443 \
  -j MARK --set-mark 1
```

#### DNS Forwarding (Optional)

Redirect system DNS to PhantomRelay:

```bash
#!/bin/bash
# Forward system DNS queries to PhantomRelay

# Configure /etc/resolv.conf
echo "nameserver 127.0.0.1" | sudo tee /etc/resolv.conf

# Or use netplan (Ubuntu 20.04+)
# /etc/netplan/99-phantomrelay.yaml:
# network:
#   version: 2
#   ethernets:
#     eth0:
#       dhcp4: true
#       dhcp4-overrides:
#         use-dns: false
#       nameservers:
#         addresses: [127.0.0.1]
```

---

## Startup & Verification

### Start Services in Order

```bash
# 1. Start logger (to see events)
prctl start logger

# 2. Start DNS (critical for resolution)
prctl start dns

# 3. Start proxy rotation engine
prctl start proxy_rotator

# 4. Start health collector
prctl start proxy_collector

# 5. Start cache maintenance
prctl start cache_preloader
prctl start cache_cleaner
prctl start cache_refresher

# 6. Start proxy servers
prctl start proxy        # Direct SOCKS5
prctl start tproxy       # Transparent proxy

# 7. Start metrics (optional)
prctl start metrics

# 8. Check status
prctl status
```

### Verification Checklist

```bash
# ✓ All services running
prctl status | grep RUNNING

# ✓ DNS working
nslookup google.com 127.0.0.1

# ✓ SOCKS5 proxy accessible
curl -x socks5://127.0.0.1:1080 http://example.com

# ✓ Metrics endpoint (if enabled)
curl http://127.0.0.1:9090/metrics

# ✓ Logs being generated
tail -f /var/log/phantom_relay/access.log
```

---

## Monitoring & Observability

### Log Monitoring

```bash
# Tail logs in real-time
tail -f /var/log/phantom_relay/daemon.log

# Watch service events
journalctl -u phantomrelayd -f

# Count errors
grep ERROR /var/log/phantom_relay/daemon.log | wc -l
```

### Metrics Collection

Prometheus example (`prometheus.yml`):

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'phantom_relay'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
```

Query useful metrics:

```
# Connection rate
rate(proxy_connections_total[1m])

# DNS cache hit ratio
rate(dns_cache_hits_total[1m]) / rate(dns_queries_total[1m])

# Proxy rotation frequency
rate(proxy_rotations_total[1m])

# Active connections
proxy_connections_active
```

### Health Checks

Script to verify daemon health:

```bash
#!/bin/bash
# health_check.sh - Verify PhantomRelay is operational

set -e

echo "Checking PhantomRelay health..."

# 1. Check daemon process
if ! pgrep phantomrelayd > /dev/null; then
    echo "ERROR: Daemon not running"
    exit 1
fi

# 2. Check CLI communication
if ! prctl status > /dev/null 2>&1; then
    echo "ERROR: CLI communication failed"
    exit 1
fi

# 3. Check DNS service
if ! prctl status | grep -q "dns.*RUNNING"; then
    echo "ERROR: DNS service not running"
    exit 1
fi

# 4. Verify DNS works
if ! timeout 5 nslookup google.com 127.0.0.1 > /dev/null 2>&1; then
    echo "ERROR: DNS resolution failed"
    exit 1
fi

# 5. Check proxy service
if ! prctl status | grep -q "proxy.*RUNNING"; then
    echo "ERROR: Proxy service not running"
    exit 1
fi

echo "✓ All health checks passed"
exit 0
```

---

## Performance Tuning

### System Limits

Increase file descriptor limits for high concurrency:

```bash
# /etc/security/limits.conf
phantomrelay soft nofile 65536
phantomrelay hard nofile 65536

# /etc/sysctl.conf (TCP tuning)
net.core.somaxconn = 65536
net.ipv4.tcp_max_syn_backlog = 65536
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_tw_recycle = 0
```

Apply:
```bash
sudo sysctl -p
```

### DNS Cache Tuning

Adjust in config:
```
cache_ttl_max = 3600        # Cache for up to 1 hour
cache_saturation = false    # Set to true for aggressive preload
prewarmer_interval = 10     # Check every 10 seconds
```

Enable turbo mode for latency-sensitive apps:
```bash
prctl enable dns-turbo
```

### Proxy Rotation Tuning

Adjust rotation interval:
```
rotate_sec = 60  # Default: rotate every 60 seconds
                 # Lower: more frequent rotation (higher overhead)
                 # Higher: less frequent rotation (less proxy diversity)
```

### Connection Handling

For high-concurrency scenarios:
```bash
# Allow more concurrent connections
echo 100000 | sudo tee /proc/sys/net/core/netdev_max_backlog

# Adjust TCP keepalive
echo 60 | sudo tee /proc/sys/net/ipv4/tcp_keepalive_time
```

---

## Troubleshooting

### Daemon Won't Start

```bash
# Check logs
sudo journalctl -u phantomrelayd -n 50

# Common issues:
# 1. Port already in use
sudo lsof -i :8888
sudo lsof -i :1080
sudo lsof -i :53

# 2. Missing capabilities
sudo getcap /opt/phantom_relay/phantomrelayd

# 3. IPC socket leftover
rm -f /tmp/phantomrelay.sock
```

### High CPU Usage

```bash
# 1. Check what's consuming CPU
top -p $(pgrep phantomrelayd)

# 2. Reduce prewarmer frequency
# In config: prewarmer_interval = 60  # From 10

# 3. Disable turbo mode
prctl disable dns-turbo

# 4. Reduce rotation frequency
# In config: rotate_sec = 300  # From 60
```

### Connections Not Being Intercepted

```bash
# 1. Verify iptables rules are in place
sudo iptables -L -t mangle -n

# 2. Check TProxy service is running
prctl status | grep tproxy

# 3. Verify netfilter module loaded
lsmod | grep tproxy

# 4. Check routing table
ip rule list
ip route list table 100
```

### DNS Not Resolving

```bash
# 1. Verify DNS service is running
prctl status | grep dns

# 2. Test DNS directly
dig @127.0.0.1 google.com

# 3. Check upstream resolver is reachable
telnet 8.8.8.8 53

# 4. View cache status
# Would need debug endpoint
```

### Proxy Connections Failing

```bash
# 1. Check proxy health
prctl status | grep collector

# 2. Verify proxy endpoints are reachable
for proxy in 10.0.0.1 10.0.0.2 10.0.0.3; do
    nc -zv $proxy 1080
done

# 3. Check rotation is working
# Would need metrics endpoint

# 4. View error logs
grep -i "proxy.*failed" /var/log/phantom_relay/daemon.log
```

---

## Backup & Recovery

### Configuration Backup

```bash
# Backup configs
sudo tar czf /backup/phantom_relay_config.tar.gz \
    /opt/phantom_relay \
    /etc/systemd/system/phantomrelayd.service

# Backup logs
sudo tar czf /backup/phantom_relay_logs_$(date +%s).tar.gz \
    /var/log/phantom_relay
```

### Clean Shutdown

```bash
# Graceful stop (allows pending connections to complete)
sudo systemctl stop phantomrelayd

# Force stop if hung (after 10 second timeout)
sudo systemctl kill phantomrelayd
```

### State Recovery

After restart, PhantomRelay automatically:
- Rebuilds DNS cache (prewarmer will repopulate)
- Reinitializes proxy rotation
- Re-evaluates proxy health
- Reconnects to upstream services

No manual state recovery needed in normal operation.

---

## Security Considerations

### Principle of Least Privilege

```bash
# Don't run as root if possible
sudo useradd -r phantomrelay
sudo chown phantomrelay:phantomrelay /opt/phantom_relay

# Use capabilities instead
sudo setcap cap_net_admin=ep /opt/phantom_relay/phantomrelayd

# Run as non-root user
# SystemD: User=phantomrelay
```

### Network Isolation

```bash
# Only listen on loopback for sensitive services
# Default configuration listens on 127.0.0.1:1080 and 127.0.0.1:53

# If exposing over network, use firewall rules
sudo ufw default deny incoming
sudo ufw allow from 10.0.0.0/8 to any port 1080
sudo ufw allow from 10.0.0.0/8 to any port 53/udp
```

### Logging & Auditing

```bash
# Enable verbose logging for audit trail
prctl start logger

# Ship logs to centralized system
tail -f /var/log/phantom_relay/daemon.log | logshipper

# Monitor for suspicious activity
grep "ProxyFailed\|Error" /var/log/phantom_relay/daemon.log
```

### Proxy List Security

```bash
# Proxy list should come from trusted source
# Never expose proxy passwords in logs
# Use environment variables for sensitive config

export PROXY_USER="username"
export PROXY_PASS="password"
```

---

## Scaling Considerations

### Single Instance Limits

- Connection concurrency: 10,000+ (system fd limits)
- DNS queries/sec: 100,000+
- Memory: ~500MB typical
- CPU: <100% for normal loads

### Multi-Instance Deployment

```
┌─────────────────────────────────────┐
│     Load Balancer / Router          │
├─────────────────────────────────────┤
│  Distributes traffic to instances:  │
├─────────────────────────────────────┤
│  - Instance 1: phantomrelayd        │
│  - Instance 2: phantomrelayd        │
│  - Instance N: phantomrelayd        │
└─────────────────────────────────────┘
```

Configuration for HA:
1. Deploy multiple instances
2. Use load balancer for SOCKS5 (port 1080)
3. For DNS: Use anycast or round-robin on port 53
4. Share proxy list configuration across instances
5. Monitor each instance independently

