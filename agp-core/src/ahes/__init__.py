"""
AHES Module - Artificial Human Endocrine System for AGP-OS

Bio-inspired behavioral governance with 8 hormones.
"""

from .endocrine import (
    Hormone,
    HormoneLevel,
    HormoneReceptor,
    EndocrineState,
    EndocrineSystem,
    ahes_system,
)

__all__ = [
    "Hormone",
    "HormoneLevel", 
    "HormoneReceptor",
    "EndocrineState",
    "EndocrineSystem",
    "ahes_system",
]
