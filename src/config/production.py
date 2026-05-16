"""
Production Configuration
Loads secrets from environment variables (12-Factor App).
"""
import os

class ProductionConfig:
    ENV = "production"
    DEBUG = False
    
    # Security
    SECRET_KEY = os.getenv("SECRET_KEY", "HARD_FAIL_IF_MISSING")
    ALLOWED_HOSTS = os.getenv("ALLOWED_HOSTS", "*").split(",")
    
    # Database (Redis for Invariants)
    REDIS_URL = os.getenv("REDIS_URL", "redis://redis:6379/0")
    
    # ASIM Keys (Injected via Kubernetes Secrets)
    ASI_MASTER_KEY = os.getenv("ASI_MASTER_KEY")
    
    # Tuning
    TIH_ENTROPY_THRESHOLD = float(os.getenv("TIH_THRESHOLD", "0.6"))
    SFA_COHERENCE_THRESHOLD = float(os.getenv("SFA_THRESHOLD", "0.95"))

config = ProductionConfig()
