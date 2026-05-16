"""Training protocols for artificial immune system."""

from .negative_selection import NegativeSelectionTrainer
from .vaccination import VaccinationProtocol
from .live_training import LiveTrainingProtocol

__all__ = [
    'NegativeSelectionTrainer',
    'VaccinationProtocol',
    'LiveTrainingProtocol'
]
