"""
T Cell Implementation - Coordinators of immune response.

Types:
- Helper T: Activate other immune components
- Killer T: Destroy compromised components
- Regulatory T: Prevent overreaction (autoimmunity)

PATENT CLAIM 7.1: Multi-layered artificial immune system
"""

import torch
import torch.nn as nn
import torch.nn.functional as F
from typing import Dict, List
from enum import Enum


class TCellType(Enum):
    """T cell functional types."""
    HELPER = "helper"
    KILLER = "killer"
    REGULATORY = "regulatory"


class TCell(nn.Module):
    """
    T Cell - Coordinator of immune response.
    
    Functions:
    - Recognize specific threat patterns
    - Coordinate response (helper)
    - Destroy compromised components (killer)
    - Suppress overreaction (regulatory)
    """
    
    def __init__(
        self,
        cell_type: TCellType,
        behavior_dim: int = 512,
        hidden_dim: int = 256,
        cell_id: int = 0
    ):
        super().__init__()
        
        self.cell_type = cell_type
        self.behavior_dim = behavior_dim
        self.cell_id = cell_id
        
        # Recognition receptor
        self.receptor = nn.Sequential(
            nn.Linear(behavior_dim, hidden_dim),
            nn.LayerNorm(hidden_dim),
            nn.ReLU(),
            nn.Dropout(0.1),
            nn.Linear(hidden_dim, hidden_dim // 2),
            nn.ReLU(),
            nn.Linear(hidden_dim // 2, 1),
            nn.Sigmoid()
        )
        
        # Activation state
        self.register_buffer('activation_level', torch.tensor(0.0))
        self.activation_threshold = 0.7
        
        # Response network
        self.response_net = nn.Sequential(
            nn.Linear(behavior_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 4),  # [antibody, killer, inflammatory, tolerance]
            nn.Softplus()
        )
        
    def recognize(self, behavior: torch.Tensor) -> torch.Tensor:
        """Check if this T cell recognizes the threat pattern."""
        if behavior.dim() == 1:
            behavior = behavior.unsqueeze(0)
        return self.receptor(behavior).squeeze(-1)
    
    def activate(self, stimulation: float):
        """Increase activation level."""
        self.activation_level = torch.clamp(
            self.activation_level + stimulation, 0.0, 1.0
        )
    
    def deactivate(self, suppression: float):
        """Decrease activation level."""
        self.activation_level = torch.clamp(
            self.activation_level - suppression, 0.0, 1.0
        )
    
    def coordinate_response(self, behavior: torch.Tensor) -> Dict[str, float]:
        """Emit coordination signals based on activation and cell type."""
        if self.activation_level < self.activation_threshold:
            return {
                'produce_antibodies': 0.0,
                'recruit_killers': 0.0,
                'inflammatory_signal': 0.0,
                'tolerance_signal': 0.0
            }
        
        if behavior.dim() == 1:
            behavior = behavior.unsqueeze(0)
        
        signals = self.response_net(behavior).mean(dim=0)
        scaling = self.activation_level.item()
        
        if self.cell_type == TCellType.HELPER:
            return {
                'produce_antibodies': signals[0].item() * scaling * 10.0,
                'recruit_killers': signals[1].item() * scaling * 5.0,
                'inflammatory_signal': signals[2].item() * scaling * 2.0,
                'tolerance_signal': 0.0
            }
        elif self.cell_type == TCellType.KILLER:
            return {
                'produce_antibodies': 0.0,
                'recruit_killers': signals[1].item() * scaling * 3.0,
                'inflammatory_signal': signals[2].item() * scaling,
                'tolerance_signal': 0.0
            }
        else:  # REGULATORY
            return {
                'produce_antibodies': 0.0,
                'recruit_killers': 0.0,
                'inflammatory_signal': -signals[2].item() * scaling,
                'tolerance_signal': signals[3].item() * scaling * 5.0
            }
    
    def check_self_reactivity(
        self,
        aligned_behavior: torch.Tensor,
        threshold: float = 0.7
    ) -> bool:
        """Check if this T cell would attack aligned behavior."""
        with torch.no_grad():
            recognition = self.recognize(aligned_behavior)
            if recognition.dim() == 0:
                recognition = recognition.unsqueeze(0)
            return (recognition > threshold).any().item()


class TCellPopulation:
    """Manager for populations of T cells."""
    
    def __init__(
        self,
        behavior_dim: int,
        num_helpers: int = 20,
        num_killers: int = 20,
        num_regulatory: int = 20
    ):
        self.behavior_dim = behavior_dim
        
        self.helpers = [
            TCell(TCellType.HELPER, behavior_dim, cell_id=i)
            for i in range(num_helpers)
        ]
        self.killers = [
            TCell(TCellType.KILLER, behavior_dim, cell_id=i)
            for i in range(num_killers)
        ]
        self.regulatory = [
            TCell(TCellType.REGULATORY, behavior_dim, cell_id=i)
            for i in range(num_regulatory)
        ]
    
    def negative_selection(
        self,
        aligned_behaviors: List[torch.Tensor],
        threshold: float = 0.7
    ):
        """
        Remove T cells that would attack aligned behavior.
        
        PATENT CLAIM 7.2: Negative selection for self-tolerance
        """
        print("\n🔬 Performing negative selection (thymic education)...")
        
        def filter_cells(cells, cell_name):
            surviving = []
            for tcell in cells:
                is_self_reactive = False
                for aligned in aligned_behaviors:
                    if tcell.check_self_reactivity(aligned, threshold):
                        is_self_reactive = True
                        break
                if not is_self_reactive:
                    surviving.append(tcell)
            initial = len(cells)
            print(f"  {cell_name}: {initial} → {len(surviving)} "
                  f"({len(surviving)/initial*100:.1f}% survived)")
            return surviving
        
        self.helpers = filter_cells(self.helpers, "Helper T cells")
        self.killers = filter_cells(self.killers, "Killer T cells")
        self.regulatory = filter_cells(self.regulatory, "Regulatory T cells")
    
    def activate_relevant_cells(
        self,
        behavior: torch.Tensor,
        threshold: float = 0.7
    ) -> int:
        """Activate T cells that recognize this threat."""
        activated_count = 0
        
        for tcell in self.helpers + self.killers + self.regulatory:
            recognition = tcell.recognize(behavior)
            if recognition.mean() > threshold:
                tcell.activate(recognition.mean().item())
                activated_count += 1
        
        return activated_count
    
    def get_coordination_signals(self, behavior: torch.Tensor) -> Dict[str, float]:
        """Aggregate coordination signals from all activated T cells."""
        combined = {
            'produce_antibodies': 0.0,
            'recruit_killers': 0.0,
            'inflammatory_signal': 0.0,
            'tolerance_signal': 0.0
        }
        
        for tcell in self.helpers + self.killers + self.regulatory:
            signals = tcell.coordinate_response(behavior)
            for key in combined:
                combined[key] += signals[key]
        
        return combined
