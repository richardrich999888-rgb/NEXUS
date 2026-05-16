"""
NEXUS Telecom Integration
Copyright (c) 2025 SYNTRIASS Labs Private Limited
Inventor: Katta Naga Sri Ganesh

Physics-first telecom control for NEXUS mesh network.
"""

from nexus_telecom.ran import WakeUpReceiver, WuRConfig, WakeUpSignal
from nexus_telecom.control import LyapunovController, create_identity_controller
from nexus_telecom.models import EnergyModel, EntropyCalculator

__version__ = "0.1.0"
__author__ = "Katta Naga Sri Ganesh"
__organization__ = "SYNTRIASS Labs Private Limited"

__all__ = [
    "WakeUpReceiver",
    "WuRConfig", 
    "WakeUpSignal",
    "LyapunovController",
    "create_identity_controller",
    "EnergyModel",
    "EntropyCalculator",
]
