"""
Production Configuration for AURA Protocol
"""
import os
from dataclasses import dataclass
from typing import Dict, Any
import json
import logging

@dataclass
class ProductionConfig:
    """Production configuration with environment variables"""
    
    # Core Settings
    NODE_ID: str = os.getenv("AURA_NODE_ID", "production-01")
    ENVIRONMENT: str = os.getenv("AURA_ENV", "production")
    LOG_LEVEL: str = os.getenv("LOG_LEVEL", "INFO")
    VERSION: str = "1.0.0"
    BASE_URL: str = os.getenv("BASE_URL", "https://api.aura-protocol.com")
    
    # Cryptography
    CURVE_TYPE: str = os.getenv("AURA_CURVE", "supersingular_521")
    PRIME_BITS: int = int(os.getenv("AURA_PRIME_BITS", "521"))
    SEED: bytes = os.getenv("AURA_SEED", "production-seed").encode()
    
    # Database
    DATABASE_URL: str = os.getenv("DATABASE_URL", "postgresql://user:pass@localhost/aura")
    REDIS_URL: str = os.getenv("REDIS_URL", "redis://localhost:6379/0")
    CACHE_TTL: int = int(os.getenv("CACHE_TTL", "3600"))
    
    # Network
    HTTP_PORT: int = int(os.getenv("HTTP_PORT", "8080"))
    P2P_PORT: int = int(os.getenv("P2P_PORT", "54321"))
    MAX_PEERS: int = int(os.getenv("MAX_PEERS", "100"))
    SYNC_INTERVAL: int = int(os.getenv("SYNC_INTERVAL", "30"))
    
    # Monetization
    FREE_TIER_LIMIT: int = int(os.getenv("FREE_TIER_LIMIT", "10000000"))
    RATE_PER_VERIFICATION: float = float(os.getenv("RATE_PER_VERIFICATION", "0.001"))
    ENTERPRISE_RATE: float = float(os.getenv("ENTERPRISE_RATE", "10000.0"))
    RUNTIME_FEE_BPS: float = float(os.getenv("RUNTIME_FEE_BPS", "0.1"))
    
    # Security
    JWT_SECRET: str = os.getenv("JWT_SECRET", "change-me-in-production")
    API_KEY_SALT: str = os.getenv("API_KEY_SALT", "change-me-in-production")
    ENCRYPTION_KEY: str = os.getenv("ENCRYPTION_KEY", "change-me-in-production")
    
    # Performance
    WORKER_COUNT: int = int(os.getenv("WORKER_COUNT", "4"))
    MAX_VERIFICATIONS_PER_SECOND: int = int(os.getenv("MAX_VERIFICATIONS_PER_SECOND", "10000"))
    BATCH_SIZE: int = int(os.getenv("BATCH_SIZE", "100"))
    
    # Monitoring
    SENTRY_DSN: str = os.getenv("SENTRY_DSN", "")
    PROMETHEUS_PORT: int = int(os.getenv("PROMETHEUS_PORT", "9090"))
    METRICS_INTERVAL: int = int(os.getenv("METRICS_INTERVAL", "30"))
    
    # External Services
    STRIPE_API_KEY: str = os.getenv("STRIPE_API_KEY", "")
    COINBASE_API_KEY: str = os.getenv("COINBASE_API_KEY", "")
    AWS_ACCESS_KEY: str = os.getenv("AWS_ACCESS_KEY", "")
    AWS_SECRET_KEY: str = os.getenv("AWS_SECRET_KEY", "")
    
    @classmethod
    def from_env(cls) -> 'ProductionConfig':
        """Create config from environment variables"""
        return cls()
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary (without sensitive data)"""
        data = {}
        for key in self.__dataclass_fields__:
            value = getattr(self, key)
            # Remove sensitive data
            sensitive_keys = ['JWT_SECRET', 'API_KEY_SALT', 'ENCRYPTION_KEY', 
                             'STRIPE_API_KEY', 'COINBASE_API_KEY', 
                             'AWS_ACCESS_KEY', 'AWS_SECRET_KEY']
            if key in sensitive_keys:
                data[key] = '[REDACTED]'
            elif isinstance(value, bytes):
                data[key] = value.decode() if value else ''
            else:
                data[key] = value
        return data
    
    def validate(self) -> bool:
        """Validate configuration"""
        required = [
            'JWT_SECRET',
            'API_KEY_SALT',
            'ENCRYPTION_KEY'
        ]
        
        for field in required:
            value = getattr(self, field, None)
            if not value or value.startswith('change-me'):
                logging.warning(f"Configuration warning: {field} using default value")
                # Don't fail in dev mode
                if self.ENVIRONMENT == 'production':
                    logging.error(f"Invalid configuration: {field} not set for production")
                    return False
        
        return True
    
    def setup_logging(self):
        """Setup production logging"""
        log_format = '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        
        if self.LOG_LEVEL == "DEBUG":
            logging.basicConfig(level=logging.DEBUG, format=log_format)
        else:
            logging.basicConfig(level=logging.INFO, format=log_format)
        
        # Add file handler for production
        if self.ENVIRONMENT == 'production':
            file_handler = logging.FileHandler('aura_production.log')
            file_handler.setLevel(getattr(logging, self.LOG_LEVEL))
            file_handler.setFormatter(logging.Formatter(log_format))
            logging.getLogger().addHandler(file_handler)
    
    @classmethod
    def load_from_file(cls, filepath: str) -> 'ProductionConfig':
        """Load configuration from file"""
        with open(filepath, 'r') as f:
            data = json.load(f)
        
        # Update environment variables
        for key, value in data.items():
            if value is not None:
                os.environ[f"AURA_{key}"] = str(value)
        
        return cls.from_env()

# Global configuration instance
config = ProductionConfig.from_env()

def get_config() -> ProductionConfig:
    """Get configuration instance"""
    return config
