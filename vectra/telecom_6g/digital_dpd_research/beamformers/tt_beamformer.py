"""
Tt Beamformer

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import torch.nn as nn
from typing import List

class TTBeamformer(nn.Module):
    """
    Tensor-Train Beamformer for integration with DPD
    Compatible with joint optimization
    """
    
    def __init__(self, tt_cores: List[torch.Tensor] = None, 
                 num_ant=64, num_users=8):
        super().__init__()
        
        if tt_cores is None:
            tt_cores = self._initialize_cores(num_ant, num_users)
            
        self.cores = nn.ParameterList([nn.Parameter(c) for c in tt_cores])
        self.N_ant = num_ant
        self.N_users = num_users
    
    def _initialize_cores(self, num_ant, num_users):
        """Initialize TT cores"""
        core1 = torch.randn(1, num_ant, 8, dtype=torch.cfloat) * 0.1
        core2 = torch.randn(8, num_users, 1, dtype=torch.cfloat) * 0.1
        return [core1, core2]
    
    def contract_tt(self):
        """Contract TT cores to full matrix"""
        A = self.cores[0]
        for core in self.cores[1:]:
            A = torch.tensordot(A, core, dims=([-1], [0]))
        return A.reshape(self.N_ant, self.N_users)
    
    def compute_beamweights(self, H):
        """
        Compute beamforming weights from channel
        H: [batch_size, 1, N_ant] channel (compatible with DPD interface)
        """
        batch_size = H.shape[0]
        W_full = self.contract_tt()  # [N_ant, N_users]
        
        # Use dominant eigenmode
        weights = torch.zeros(batch_size, self.N_ant, dtype=torch.cfloat)
        
        for b in range(batch_size):
            # Simple matched filter
            h = H[b, 0]  # [N_ant]
            weights[b] = h.conj() / torch.norm(h)
        
        return weights
    
    def get_compression_ratio(self):
        """Compute compression ratio"""
        full_params = self.N_ant * self.N_users
        tt_params = sum(core.numel() for core in self.cores)
        return tt_params / full_params

