# AGP-CORE Operations Runbook

## Overview

This runbook provides procedures for operating AGP-CORE in production.

---

## 1. Deployment

### 1.1 Prerequisites
```bash
# Required environment variables
export DEPLOYER_PRIVATE_KEY="0x..."
export RPC_URL="https://..."
export DATABASE_URL="postgresql://..."
```

### 1.2 Deploy to Mainnet
```bash
cd deploy
./deploy.sh mainnet deploy
```

### 1.3 Deploy to Testnet (Sepolia)
```bash
./deploy.sh sepolia deploy
```

---

## 2. Daily Operations

### 2.1 Health Checks
```bash
# API health
curl https://api.agp-core.io/health

# Ready check
curl https://api.agp-core.io/ready
```

### 2.2 Monitor Metrics
- Prometheus: `https://metrics.agp-core.io`
- Grafana: `https://grafana.agp-core.io`

Key metrics:
- `agp_request_count` - Total requests
- `agp_request_latency_seconds` - Latency histogram
- `agp_active_agents` - Active agent count
- `agp_hormone_levels` - Hormone distribution

### 2.3 Log Analysis
```bash
# View recent logs
docker logs agp-core --tail 100

# Search for errors
docker logs agp-core 2>&1 | grep ERROR
```

---

## 3. Incident Response

### 3.1 High Latency
1. Check database connections: `SELECT count(*) FROM pg_stat_activity;`
2. Check Redis: `redis-cli ping`
3. Scale workers: `docker service scale agp-core=8`

### 3.2 Database Issues
1. Check connection pool: Monitor `agp_db_pool_size`
2. Kill long queries: `SELECT pg_terminate_backend(pid);`
3. Failover to replica if primary down

### 3.3 Contract Issues
1. Pause contract if exploited: Call `pause()` function
2. Check gas prices: `eth_gasPrice`
3. Verify recent transactions on Etherscan

---

## 4. Maintenance

### 4.1 Schema Migrations
```bash
cd /app
alembic upgrade head
```

### 4.2 Decay Scheduler
The decay scheduler runs automatically. To trigger manually:
```bash
curl -X POST https://api.agp-core.io/api/v1/system/decay \
  -H "X-API-Key: $ADMIN_API_KEY"
```

### 4.3 Backup Database
```bash
pg_dump $DATABASE_URL > backup_$(date +%Y%m%d).sql
```

---

## 5. Scaling

### 5.1 Horizontal Scaling
```bash
# Kubernetes
kubectl scale deployment agp-core --replicas=10

# Docker Swarm
docker service scale agp-core=10
```

### 5.2 Database Scaling
1. Add read replicas for read-heavy workloads
2. Increase connection pool size in config
3. Consider partitioning `observations` table by date

---

## 6. Security

### 6.1 Key Rotation
1. Generate new API keys
2. Update in secrets manager
3. Restart services with new config

### 6.2 Audit Log Review
```bash
# Export audit log
curl https://api.agp-core.io/api/v1/compliance/audit/export \
  -H "X-API-Key: $ADMIN_API_KEY" > audit.json
```

### 6.3 Contract Security
- Multi-sig required for treasury operations
- Timelock on governance proposals
- Emergency pause capability

---

## 7. Troubleshooting

| Symptom | Check | Solution |
|---------|-------|----------|
| 503 errors | Database connection | Restart DB or scale pool |
| Slow responses | Redis cache | Clear cache, check memory |
| Bridge stuck | Relayer status | Restart relayer service |
| High gas costs | Network congestion | Wait or use L2 |

---

## 8. Contacts

| Role | Contact |
|------|---------|
| On-call | pagerduty.com/agp-core |
| Security | security@agp-core.io |
| DevOps | devops@agp-core.io |

---

## 9. Recovery Procedures

### 9.1 Full System Restore
```bash
# 1. Restore database
psql $DATABASE_URL < backup.sql

# 2. Redeploy application
./deploy.sh mainnet backend

# 3. Verify contracts
npx hardhat verify --network mainnet
```

### 9.2 Contract Upgrade
1. Deploy new implementation
2. Call upgrade on proxy
3. Verify state preserved
4. Update addresses in config
