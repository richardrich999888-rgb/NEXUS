"""
Tt Beamformer

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import torch.nn as nn
from typing import List

class DigitalPhaseStabilizer(nn.Module):
    """Simple digital phase stabilization"""
    def __init__(self, memory_length=5):
        super().__init__()
        self.memory_length = memory_length
        self.phase_history = []
        self.last_batch_size = None
        
    def forward(self, weights):
        """Apply phase correction to beamforming weights"""
        current_phases = torch.angle(weights)
        batch_size = weights.shape[0]
        
        # Reset history if batch size changes
        if self.last_batch_size is not None and self.last_batch_size != batch_size:
            self.phase_history = []
        self.last_batch_size = batch_size
        
        if len(self.phase_history) > 0:
            # Simple moving average phase correction
            phase_avg = torch.mean(torch.stack(self.phase_history), dim=0)
            correction = 0.1 * (phase_avg - current_phases)
            stabilized_phases = current_phases + correction
        else:
            stabilized_phases = current_phases
            
        # Update history
        self.phase_history.append(current_phases.detach())
        if len(self.phase_history) > self.memory_length:
            self.phase_history.pop(0)
            
        return torch.abs(weights) * torch.exp(1j * stabilized_phases)

class TTBeamformer(nn.Module):
    """
    Tensor-Train beamforming reconstruction.
    Given TT cores, reconstructs beamforming matrix and applies sparse mask.
    """

    def __init__(self, tt_cores: List[torch.Tensor] = None, num_ant=64, num_users=8):
        super().__init__()
        
        if tt_cores is None:
            # Initialize random TT cores if not provided
            tt_cores = self._initialize_random_cores(num_ant, num_users)
            
        self.cores = nn.ParameterList([nn.Parameter(c) for c in tt_cores])
        self.N_ant = num_ant
        self.N_users = num_users
        
        # Phase stabilization
        self.phase_stabilizer = DigitalPhaseStabilizer()

    def _initialize_random_cores(self, num_ant, num_users):
        """Initialize random TT cores for testing"""
        # Simple 2-core decomposition for N_ant x N_users matrix
        core1 = torch.randn(1, num_ant, 8, dtype=torch.cfloat) * 0.1
        core2 = torch.randn(8, num_users, 1, dtype=torch.cfloat) * 0.1
        return [core1, core2]

    def contract_tt(self):
        """
        Contract TT cores → full W matrix shape (N_ant, N_users).
        """
        A = self.cores[0]
        for core in self.cores[1:]:
            A = torch.tensordot(A, core, dims=([-1], [0]))
        return A.reshape(self.N_ant, self.N_users)

    def forward(self, mask, H):
        """
        mask: (B, N_ant) - Sparse beam mask
        H: (B, N_users, N_ant) - Channel matrix
        Returns: (B, N_ant) beamforming weights
        """
        W_full = self.contract_tt()  # (N_ant, N_users)
        W_masked = mask.unsqueeze(-1) * W_full.unsqueeze(0)  # (B, N_ant, N_users)
        
        # Simple beamforming: use first user or combine
        if H.shape[1] > 0:  # If we have users
            # H is (B, Nu, Nt), transpose to (B, Nt, Nu)
            H_T = H.transpose(1, 2)  # (B, N_ant, N_users)
            # Matched filter: sum over users for each antenna
            # W_masked is (B, Nt, Nu), H_T is (B, Nt, Nu)
            weights = torch.einsum('bau,bau->ba', W_masked, H_T.conj())  # (B, N_ant)
        else:
            # Fallback: use first column
            weights = W_masked[:, :, 0]
            
        # Apply phase stabilization
        weights = self.phase_stabilizer(weights)
        return weights
    
    def compute_beamweights(self, H, beam_mask=None):
        """Compatibility method for existing code"""
        if beam_mask is None:
            beam_mask = torch.ones(H.shape[0], self.N_ant, device=H.device)
        return self.forward(beam_mask, H)
