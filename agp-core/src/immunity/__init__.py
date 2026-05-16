"""
AIS-ASI: Artificial Immune System for ASI Safety

PATENT-PENDING: Bio-inspired safety architecture for AI systems.

Multi-layered defense:
1. Innate immunity (fast, pattern-based)
2. Adaptive immunity (learned, specific)
3. Memory cells (rapid recall)
4. Clonal selection (evolutionary optimization)
5. Negative selection (self-tolerance)

Copyright (c) 2026 SYNTRIASS Labs Private Limited
Inventor: Katta Naga Sri Ganesh
"""

from .antibody import Antibody, AntibodyPool, AntibodyMetadata
from .tcell import TCell, TCellType, TCellPopulation
from .memory import MemoryCell, MemoryBank, MemoryMetadata
from .innate import InnateImmuneSystem, PatternDetector
from .adaptive import AdaptiveImmuneSystem, Threat
from .immune_system import ArtificialImmuneSystem, ImmuneConfig
from .integration import EndocrineImmuneIntegration, IntegratedBioSafetySystem

__all__ = [
    # Core components
    'Antibody',
    'AntibodyPool',
    'AntibodyMetadata',
    'TCell',
    'TCellType',
    'TCellPopulation',
    'MemoryCell',
    'MemoryBank',
    'MemoryMetadata',
    
    # Systems
    'InnateImmuneSystem',
    'PatternDetector',
    'AdaptiveImmuneSystem',
    'ArtificialImmuneSystem',
    
    # Config and types
    'ImmuneConfig',
    'Threat',
    
    # Integration
    'EndocrineImmuneIntegration',
    'IntegratedBioSafetySystem',
]

__version__ = '1.0.0'
__author__ = 'SYNTRIASS Labs'
