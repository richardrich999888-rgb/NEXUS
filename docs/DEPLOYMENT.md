# NEXUS Protocol Deployment Guide

**Version:** 1.0  
**Last Updated:** 2025-01-18

## Overview

This guide covers deploying NEXUS in production environments, including Kubernetes, Docker, and bare metal configurations.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Kubernetes Deployment](#kubernetes-deployment)
3. [Docker Deployment](#docker-deployment)
4. [Configuration](#configuration)
5. [Security](#security)
6. [Monitoring](#monitoring)
7. [Scaling](#scaling)
8. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### System Requirements

- **CPU**: 2+ cores (4+ recommended)
- **Memory**: 4GB+ (8GB+ recommended)
- **Storage**: 50GB+ SSD (100GB+ recommended)
- **Network**: 100Mbps+ (1Gbps+ recommended)

### Software Requirements

- **Rust**: 1.70+ (for building from source)
- **Docker**: 20.10+ (for containerized deployment)
- **Kubernetes**: 1.24+ (for K8s deployment)
- **TLS Certificates**: Valid X.509 certificates (production)

---

## Kubernetes Deployment

### 1. Create Namespace

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: nexus
```

### 2. TLS Secrets

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: nexus-tls
  namespace: nexus
type: kubernetes.io/tls
data:
  tls.crt: <base64-encoded-cert>
  tls.key: <base64-encoded-key>
  ca.crt: <base64-encoded-ca>
```

### 3. Configuration ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: nexus-config
  namespace: nexus
data:
  config.toml: |
    [network]
    bind_addr = "0.0.0.0:8080"
    tls_enabled = true
    rate_limit_connections = 100
    rate_limit_messages_per_sec = 1000

    [storage]
    data_dir = "/data"
    backup_enabled = true
    backup_retention_days = 30

    [executor]
    max_concurrent_executions = 100
    cache_size_mb = 1024
    default_timeout_seconds = 300

    [observability]
    metrics_port = 9090
    log_level = "info"
```

### 4. Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nexus-node
  namespace: nexus
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nexus-node
  template:
    metadata:
      labels:
        app: nexus-node
    spec:
      containers:
      - name: nexus
        image: nexus:1.0.0
        ports:
        - containerPort: 8080
          name: p2p
        - containerPort: 9090
          name: metrics
        env:
        - name: RUST_LOG
          value: "info"
        - name: NEXUS_DATA_DIR
          value: "/data"
        - name: NEXUS_TLS_CERT
          value: "/etc/tls/tls.crt"
        - name: NEXUS_TLS_KEY
          value: "/etc/tls/tls.key"
        volumeMounts:
        - name: tls
          mountPath: /etc/tls
          readOnly: true
        - name: config
          mountPath: /etc/nexus
        - name: data
          mountPath: /data
        resources:
          requests:
            cpu: "2"
            memory: "4Gi"
          limits:
            cpu: "4"
            memory: "8Gi"
        livenessProbe:
          httpGet:
            path: /health
            port: 9090
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 9090
          initialDelaySeconds: 10
          periodSeconds: 5
      volumes:
      - name: tls
        secret:
          secretName: nexus-tls
      - name: config
        configMap:
          name: nexus-config
      - name: data
        persistentVolumeClaim:
          claimName: nexus-data
```

### 5. Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: nexus-node
  namespace: nexus
spec:
  selector:
    app: nexus-node
  ports:
  - name: p2p
    port: 8080
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090
  type: ClusterIP
```

### 6. Persistent Volume

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: nexus-data
  namespace: nexus
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 100Gi
  storageClassName: fast-ssd
```

---

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.70 as builder
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/nexus-node /usr/local/bin/nexus-node
EXPOSE 8080 9090
CMD ["nexus-node"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  nexus1:
    image: nexus:1.0.0
    ports:
      - "8080:8080"
      - "9090:9090"
    volumes:
      - ./data/nexus1:/data
      - ./certs:/etc/tls
    environment:
      - RUST_LOG=info
      - NEXUS_BIND_ADDR=0.0.0.0:8080
    networks:
      - nexus

  nexus2:
    image: nexus:1.0.0
    ports:
      - "8081:8080"
      - "9091:9090"
    volumes:
      - ./data/nexus2:/data
      - ./certs:/etc/tls
    environment:
      - RUST_LOG=info
      - NEXUS_BIND_ADDR=0.0.0.0:8080
      - NEXUS_PEERS=nexus1:8080
    networks:
      - nexus
    depends_on:
      - nexus1

networks:
  nexus:
    driver: bridge
```

---

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `NEXUS_BIND_ADDR` | Bind address | `0.0.0.0:8080` |
| `NEXUS_DATA_DIR` | Data directory | `./data` |
| `NEXUS_TLS_CERT` | TLS certificate path | - |
| `NEXUS_TLS_KEY` | TLS private key path | - |
| `NEXUS_TLS_CA` | TLS CA certificate path | - |
| `NEXUS_LOG_LEVEL` | Log level | `info` |
| `NEXUS_METRICS_PORT` | Metrics port | `9090` |
| `VAULT_ADDR` | Vault address (for secrets) | - |
| `VAULT_TOKEN` | Vault token | - |
| `AWS_REGION` | AWS region (for secrets) | - |

### Configuration File (config.toml)

```toml
[network]
bind_addr = "0.0.0.0:8080"
tls_enabled = true
rate_limit_connections = 100
rate_limit_messages_per_sec = 1000

[storage]
data_dir = "/data"
backup_enabled = true
backup_retention_days = 30
backup_schedule = "0 2 * * *"  # Daily at 2 AM

[executor]
max_concurrent_executions = 100
cache_size_mb = 1024
default_timeout_seconds = 300
max_memory_mb = 1024

[observability]
metrics_port = 9090
log_level = "info"
log_format = "json"

[tenancy]
enabled = true
default_quotas = { max_pcu_executions_per_second = 100, max_memory_bytes = 1073741824 }
```

---

## Security

### TLS Configuration

1. **Generate Certificates:**
   ```bash
   # Self-signed (development only)
   openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

   # Production: Use Let's Encrypt or internal CA
   ```

2. **Configure mTLS:**
   ```toml
   [network.tls]
   verify_client = true
   client_ca_file = "/etc/tls/ca.pem"
   ```

### Secret Management

**Option 1: HashiCorp Vault**
```bash
export VAULT_ADDR=https://vault.example.com
export VAULT_TOKEN=<token>
```

**Option 2: AWS Secrets Manager**
```bash
export AWS_REGION=us-east-1
export AWS_SECRET_ACCESS_KEY=<key>
```

**Option 3: Kubernetes Secrets**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: nexus-secrets
type: Opaque
data:
  tls-key: <base64>
```

### Network Security

- **Firewall Rules**: Only allow necessary ports (8080, 9090)
- **Rate Limiting**: Configure rate limits to prevent DoS
- **mTLS**: Require mutual TLS for all connections
- **Network Policies** (K8s): Restrict pod-to-pod communication

---

## Monitoring

### Prometheus Scraping

```yaml
apiVersion: v1
kind: ServiceMonitor
metadata:
  name: nexus-metrics
  namespace: nexus
spec:
  selector:
    matchLabels:
      app: nexus-node
  endpoints:
  - port: metrics
    interval: 15s
```

### Grafana Dashboard

Key metrics to monitor:
- PCU execution rate and latency
- Cache hit ratio
- Network message rates
- Storage I/O
- Resource usage (CPU, memory)
- Error rates

### Alerting Rules

```yaml
groups:
- name: nexus
  rules:
  - alert: HighExecutionLatency
    expr: histogram_quantile(0.95, nexus_pcu_execution_duration) > 1.0
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "PCU execution latency is high"

  - alert: HighErrorRate
    expr: rate(nexus_pcu_execution_failures[5m]) > 0.1
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High PCU execution failure rate"
```

---

## Scaling

### Horizontal Scaling

**Kubernetes HPA:**
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: nexus-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: nexus-node
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### Vertical Scaling

Adjust resource requests/limits in deployment:
```yaml
resources:
  requests:
    cpu: "4"
    memory: "8Gi"
  limits:
    cpu: "8"
    memory: "16Gi"
```

---

## Troubleshooting

### Common Issues

**1. Connection Failures**
- Check TLS certificates are valid
- Verify network connectivity
- Check rate limiting settings

**2. High Memory Usage**
- Reduce cache size
- Lower concurrent execution limit
- Check for memory leaks in metrics

**3. Slow Execution**
- Check CPU throttling
- Verify storage I/O performance
- Review cache hit ratio

**4. Backup Failures**
- Check disk space
- Verify backup directory permissions
- Review backup logs

### Debug Mode

```bash
export RUST_LOG=debug
export NEXUS_LOG_LEVEL=debug
```

### Health Endpoints

- `/health` - Liveness probe
- `/ready` - Readiness probe
- `/metrics` - Prometheus metrics

---

## Production Checklist

- [ ] TLS certificates configured
- [ ] Secrets management integrated
- [ ] Monitoring and alerting set up
- [ ] Backup schedule configured
- [ ] Rate limiting enabled
- [ ] Resource limits set
- [ ] Health checks configured
- [ ] Log aggregation configured
- [ ] Disaster recovery plan documented
- [ ] Security audit completed

---

*For more information, see [API Reference](./API_REFERENCE.md) and [Threat Model](./THREAT_MODEL.md).*


