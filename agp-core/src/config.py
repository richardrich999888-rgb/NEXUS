"""
AGP-CORE Configuration
Endocrine-based reputation system configuration
"""

from pydantic_settings import BaseSettings
from pydantic import Field
from typing import Optional
import os


class Settings(BaseSettings):
    """Application settings with endocrine system parameters"""
    
    # Application
    app_name: str = "AGP-CORE"
    app_version: str = "1.0.0"
    debug: bool = False
    environment: str = "development"
    
    # Server
    host: str = "0.0.0.0"
    port: int = 8000
    workers: int = 4
    
    # Database
    database_url: str = Field(
        default="postgresql+asyncpg://agp:agp@localhost:5432/agp_core",
        description="PostgreSQL connection URL"
    )
    db_pool_size: int = 20
    db_max_overflow: int = 10
    
    # KAIRON Native Cache (replaces Redis)
    kairon_node_id: str = "agp-core-1"
    kairon_sync_interval_ms: int = 5000
    kairon_snapshot_interval: int = 1000
    
    # Security
    secret_key: str = Field(
        default="agp-core-secret-key-change-in-production",
        description="Secret key for JWT tokens"
    )
    api_key_header: str = "X-API-Key"
    access_token_expire_minutes: int = 60 * 24  # 24 hours
    
    # Sentry (optional)
    sentry_dsn: Optional[str] = None
    
    # ==========================================================
    # ENDOCRINE SYSTEM PARAMETERS
    # ==========================================================
    
    # Hormone half-lives (seconds)
    cortisol_half_life: float = 5400.0      # 90 min
    oxytocin_half_life: float = 180.0       # 3 min
    serotonin_half_life: float = 86400.0    # 24 hours
    dopamine_half_life: float = 300.0       # 5 min
    adrenaline_half_life: float = 120.0     # 2 min
    endorphin_half_life: float = 1200.0     # 20 min
    norepinephrine_half_life: float = 90.0  # 1.5 min
    growth_hormone_half_life: float = 900.0 # 15 min
    
    # Receptor binding affinity (Km)
    cortisol_km: float = 0.3
    oxytocin_km: float = 0.1
    serotonin_km: float = 0.5
    dopamine_km: float = 0.2
    adrenaline_km: float = 0.05
    endorphin_km: float = 0.4
    norepinephrine_km: float = 0.15
    growth_hormone_km: float = 0.6
    
    # Homeostasis
    homeostasis_baseline: float = 0.5
    homeostasis_tolerance: float = 0.1
    allostasis_adaptation_rate: float = 0.01
    circadian_amplitude: float = 0.15
    
    # Decay interval (seconds)
    decay_interval: int = 60
    
    # Privilege thresholds
    privilege_low_threshold: float = 0.3
    privilege_high_threshold: float = 0.7
    
    # Verification tiers
    zkml_risk_threshold: float = 0.8
    tee_risk_threshold: float = 0.4
    
    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"
        case_sensitive = False


# Global settings instance
settings = Settings()


# Hormone configuration (derived from settings)
HORMONE_CONFIG = {
    "cortisol": {
        "half_life": settings.cortisol_half_life,
        "km": settings.cortisol_km,
        "dimension": "accuracy",
    },
    "oxytocin": {
        "half_life": settings.oxytocin_half_life,
        "km": settings.oxytocin_km,
        "dimension": "cooperation",
    },
    "serotonin": {
        "half_life": settings.serotonin_half_life,
        "km": settings.serotonin_km,
        "dimension": "stability",
    },
    "dopamine": {
        "half_life": settings.dopamine_half_life,
        "km": settings.dopamine_km,
        "dimension": "uniqueness",
    },
    "adrenaline": {
        "half_life": settings.adrenaline_half_life,
        "km": settings.adrenaline_km,
        "dimension": "latency",
    },
    "endorphins": {
        "half_life": settings.endorphin_half_life,
        "km": settings.endorphin_km,
        "dimension": "ethics",
    },
    "norepinephrine": {
        "half_life": settings.norepinephrine_half_life,
        "km": settings.norepinephrine_km,
        "dimension": "novelty",
    },
    "growth_hormone": {
        "half_life": settings.growth_hormone_half_life,
        "km": settings.growth_hormone_km,
        "dimension": "longevity",
    },
}
