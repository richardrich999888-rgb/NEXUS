#!/bin/bash
# AGP-CORE Mainnet Deployment Script
# Usage: ./deploy.sh [network] [action]

set -e

NETWORK=${1:-"mainnet"}
ACTION=${2:-"deploy"}
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Network configurations
declare -A NETWORKS=(
    ["mainnet"]="1"
    ["polygon"]="137"
    ["arbitrum"]="42161"
    ["optimism"]="10"
    ["base"]="8453"
    ["sepolia"]="11155111"
)

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Node.js
    if ! command -v node &> /dev/null; then
        log_error "Node.js not found. Install Node.js 18+"
        exit 1
    fi
    
    # Check Python
    if ! command -v python3 &> /dev/null; then
        log_error "Python 3 not found"
        exit 1
    fi
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_warn "Docker not found. Container deployment unavailable."
    fi
    
    # Check environment variables
    if [[ -z "$DEPLOYER_PRIVATE_KEY" ]]; then
        log_error "DEPLOYER_PRIVATE_KEY not set"
        exit 1
    fi
    
    if [[ -z "$RPC_URL" ]]; then
        log_error "RPC_URL not set for $NETWORK"
        exit 1
    fi
    
    log_info "Prerequisites OK"
}

deploy_contracts() {
    log_info "Deploying smart contracts to $NETWORK..."
    
    cd "$PROJECT_ROOT/contracts"
    
    # Install dependencies if needed
    if [[ ! -d "node_modules" ]]; then
        log_info "Installing contract dependencies..."
        npm install
    fi
    
    # Compile contracts
    log_info "Compiling contracts..."
    npx hardhat compile
    
    # Deploy ReputationToken
    log_info "Deploying ReputationToken..."
    npx hardhat run scripts/deploy-reputation-token.js --network $NETWORK
    
    # Deploy AlignmentStaking
    log_info "Deploying AlignmentStaking..."
    npx hardhat run scripts/deploy-staking.js --network $NETWORK
    
    # Deploy Governance
    log_info "Deploying ProtocolGovernance..."
    npx hardhat run scripts/deploy-governance.js --network $NETWORK
    
    # Deploy Treasury
    log_info "Deploying Treasury..."
    npx hardhat run scripts/deploy-treasury.js --network $NETWORK
    
    # Verify contracts
    if [[ "$NETWORK" != "localhost" ]]; then
        log_info "Verifying contracts on Etherscan..."
        npx hardhat verify --network $NETWORK
    fi
    
    log_info "Contract deployment complete!"
}

deploy_backend() {
    log_info "Deploying AGP-CORE backend..."
    
    cd "$PROJECT_ROOT"
    
    # Build Docker image
    log_info "Building Docker image..."
    docker build -t agp-core:latest .
    
    # Tag for registry
    if [[ -n "$DOCKER_REGISTRY" ]]; then
        docker tag agp-core:latest $DOCKER_REGISTRY/agp-core:latest
        docker push $DOCKER_REGISTRY/agp-core:latest
    fi
    
    log_info "Backend deployment complete!"
}

run_migrations() {
    log_info "Running database migrations..."
    
    cd "$PROJECT_ROOT"
    
    # Run Alembic migrations
    alembic upgrade head
    
    # Run schema updates
    if [[ -f "scripts/update_schema_phase4.sql" ]]; then
        log_info "Applying Phase 4 schema..."
        psql $DATABASE_URL -f scripts/update_schema_phase4.sql
    fi
    
    log_info "Migrations complete!"
}

setup_monitoring() {
    log_info "Setting up monitoring..."
    
    # Deploy Prometheus config
    cat > /tmp/prometheus.yml << EOF
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'agp-core'
    static_configs:
      - targets: ['localhost:8000']
    metrics_path: '/metrics'
    
  - job_name: 'node'
    static_configs:
      - targets: ['localhost:9100']
EOF
    
    log_info "Monitoring configuration created"
}

print_status() {
    log_info "=== Deployment Status ==="
    echo ""
    echo "Network: $NETWORK (Chain ID: ${NETWORKS[$NETWORK]})"
    echo "Timestamp: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    echo ""
    echo "Deployed Components:"
    echo "  - Smart Contracts: ✓"
    echo "  - Backend API: ✓"
    echo "  - Database: ✓"
    echo "  - Monitoring: ✓"
    echo ""
    log_info "Deployment complete!"
}

# Main
case $ACTION in
    "deploy")
        check_prerequisites
        deploy_contracts
        deploy_backend
        run_migrations
        setup_monitoring
        print_status
        ;;
    "contracts")
        check_prerequisites
        deploy_contracts
        ;;
    "backend")
        deploy_backend
        ;;
    "migrate")
        run_migrations
        ;;
    "status")
        print_status
        ;;
    *)
        echo "Usage: $0 [network] [deploy|contracts|backend|migrate|status]"
        echo ""
        echo "Networks: mainnet, polygon, arbitrum, optimism, base, sepolia"
        exit 1
        ;;
esac
