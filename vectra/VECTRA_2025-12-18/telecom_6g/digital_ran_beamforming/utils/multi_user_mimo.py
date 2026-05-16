"""
Multi-User MIMO Extensions

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Multi-User MIMO Extensions
Extends beamforming to support multiple simultaneous users
"""

import torch
import torch.nn as nn
import torch.nn.functional as F
import numpy as np
from typing import List, Tuple, Optional

class MultiUserBeamformer(nn.Module):
    """
    Multi-user MIMO beamformer with interference cancellation
    Supports simultaneous transmission to multiple users
    """
    
    def __init__(self, num_antennas: int, num_users: int, method: str = 'zero_forcing'):
        """
        Args:
            num_antennas: Number of transmit antennas
            num_users: Number of simultaneous users
            method: 'zero_forcing', 'mmse', 'dirty_paper'
        """
        super().__init__()
        self.Nt = num_antennas
        self.K = num_users
        self.method = method
        
        # Learnable regularization for MMSE
        if method == 'mmse':
            self.regularization = nn.Parameter(torch.tensor(0.1))
        else:
            self.register_buffer('regularization', torch.tensor(0.0))
    
    def zero_forcing(self, H: torch.Tensor) -> torch.Tensor:
        """
        Zero-forcing beamforming
        H: (batch, K, Nt) channel matrix
        Returns: (batch, Nt, K) beamforming matrix
        """
        # H^H (H H^H)^(-1)
        H_H = H.conj().transpose(1, 2)  # (batch, Nt, K)
        HHH = torch.bmm(H, H_H)  # (batch, K, K)
        
        # Add small regularization for numerical stability
        HHH_reg = HHH + 1e-6 * torch.eye(self.K, device=H.device).unsqueeze(0)
        HHH_inv = torch.linalg.inv(HHH_reg)
        
        W = torch.bmm(H_H, HHH_inv)  # (batch, Nt, K)
        
        # Normalize columns
        norms = torch.norm(W, dim=1, keepdim=True)
        W = W / (norms + 1e-8)
        
        return W
    
    def mmse(self, H: torch.Tensor, noise_power: float = 0.01) -> torch.Tensor:
        """
        MMSE beamforming
        H: (batch, K, Nt) channel matrix
        Returns: (batch, Nt, K) beamforming matrix
        """
        # W = H^H (H H^H + sigma^2 I)^(-1)
        H_H = H.conj().transpose(1, 2)  # (batch, Nt, K)
        HHH = torch.bmm(H, H_H)  # (batch, K, K)
        
        # Regularization
        reg = self.regularization.abs() + noise_power
        HHH_reg = HHH + reg * torch.eye(self.K, device=H.device).unsqueeze(0)
        HHH_inv = torch.linalg.inv(HHH_reg)
        
        W = torch.bmm(H_H, HHH_inv)
        
        # Normalize
        norms = torch.norm(W, dim=1, keepdim=True)
        W = W / (norms + 1e-8)
        
        return W
    
    def dirty_paper_coding(self, H: torch.Tensor, power_allocation: torch.Tensor) -> torch.Tensor:
        """
        Dirty Paper Coding (DPC) for interference cancellation
        H: (batch, K, Nt) channel matrix
        power_allocation: (batch, K) power allocation per user
        Returns: (batch, Nt, K) beamforming matrix
        """
        # Simplified DPC implementation
        # Full DPC requires QR decomposition and sequential encoding
        
        # Sort users by channel quality
        channel_norms = torch.norm(H, dim=-1)  # (batch, K)
        _, user_order = torch.sort(channel_norms, dim=-1, descending=True)
        
        W = torch.zeros(H.shape[0], self.Nt, self.K, dtype=torch.cfloat, device=H.device)
        
        for b in range(H.shape[0]):
            # Process users in order
            for k_idx, k in enumerate(user_order[b]):
                h_k = H[b, k:k+1]  # (1, Nt)
                
                # Project out interference from previous users
                if k_idx > 0:
                    # Orthogonalize against previous beamformers
                    prev_W = W[b, :, user_order[b, :k_idx]]
                    proj = torch.mm(prev_W, prev_W.conj().T)
                    h_k_proj = h_k - torch.mm(h_k, proj)
                else:
                    h_k_proj = h_k
                
                # Normalize
                w_k = h_k_proj.conj().T / (torch.norm(h_k_proj) + 1e-8)
                w_k = w_k * torch.sqrt(power_allocation[b, k])
                
                W[b, :, k] = w_k.squeeze()
        
        return W
    
    def forward(self, H: torch.Tensor, method: Optional[str] = None, 
                power_allocation: Optional[torch.Tensor] = None) -> torch.Tensor:
        """
        Compute multi-user beamforming matrix
        H: (batch, K, Nt) channel matrix
        Returns: (batch, Nt, K) beamforming matrix
        """
        method = method or self.method
        
        if method == 'zero_forcing':
            return self.zero_forcing(H)
        elif method == 'mmse':
            noise_power = 0.01
            return self.mmse(H, noise_power)
        elif method == 'dirty_paper':
            if power_allocation is None:
                power_allocation = torch.ones(H.shape[0], self.K, device=H.device) / self.K
            return self.dirty_paper_coding(H, power_allocation)
        else:
            raise ValueError(f"Unknown method: {method}")


class InterferenceAwareEncoder(nn.Module):
    """
    Neural CSI encoder that accounts for multi-user interference
    """
    
    def __init__(self, base_encoder, num_users: int):
        super().__init__()
        self.base_encoder = base_encoder
        self.num_users = num_users
        
        # Interference prediction head
        self.interference_head = nn.Sequential(
            nn.Linear(base_encoder.bottleneck[-1].out_features, 128),
            nn.ReLU(),
            nn.Linear(128, num_users * num_users),  # Interference matrix
            nn.Sigmoid()
        )
    
    def forward(self, H: torch.Tensor):
        """
        H: (batch, K, Nt) multi-user channel
        Returns: compressed CSI and interference matrix
        """
        # Encode each user's channel
        compressed_users = []
        for k in range(self.num_users):
            h_k = H[:, k:k+1, :]  # (batch, 1, Nt)
            z_k = self.base_encoder.compress(h_k)
            compressed_users.append(z_k)
        
        # Stack and predict interference
        compressed = torch.stack(compressed_users, dim=1)  # (batch, K, latent_dim)
        interference = self.interference_head(compressed.mean(dim=1))
        interference = interference.view(-1, self.num_users, self.num_users)
        
        return compressed, interference


class PowerAllocationOptimizer(nn.Module):
    """
    Learnable power allocation for multi-user systems
    Optimizes power distribution across users
    """
    
    def __init__(self, num_users: int, total_power: float = 1.0):
        super().__init__()
        self.num_users = num_users
        self.total_power = total_power
        
        # Power allocation network
        self.power_net = nn.Sequential(
            nn.Linear(num_users, 64),
            nn.ReLU(),
            nn.Linear(64, num_users),
            nn.Softmax(dim=-1)
        )
    
    def forward(self, channel_qualities: torch.Tensor) -> torch.Tensor:
        """
        Compute optimal power allocation
        channel_qualities: (batch, K) channel quality metrics
        Returns: (batch, K) power allocation
        """
        power_weights = self.power_net(channel_qualities)
        power_allocation = power_weights * self.total_power
        return power_allocation
    
    def waterfilling(self, channel_gains: torch.Tensor, noise_power: float = 0.01) -> torch.Tensor:
        """
        Waterfilling power allocation (classical method)
        channel_gains: (batch, K) channel gains
        Returns: (batch, K) power allocation
        """
        batch_size = channel_gains.shape[0]
        power_allocation = torch.zeros_like(channel_gains)
        
        for b in range(batch_size):
            gains = channel_gains[b].cpu().numpy()
            
            # Waterfilling algorithm
            # Find water level lambda such that sum(max(0, lambda - 1/gain)) = total_power
            sorted_indices = np.argsort(gains)[::-1]
            sorted_gains = gains[sorted_indices]
            
            # Iterative waterfilling
            lambda_val = 0.0
            for i in range(len(sorted_gains)):
                lambda_val = (self.total_power + np.sum(1.0 / sorted_gains[:i+1])) / (i + 1)
                if lambda_val > 1.0 / sorted_gains[i]:
                    break
            
            # Allocate power
            for i, idx in enumerate(sorted_indices):
                power = max(0, lambda_val - 1.0 / gains[idx])
                power_allocation[b, idx] = power
        
        return power_allocation


class MultiUserBeamformingPipeline:
    """
    Complete pipeline for multi-user MIMO beamforming
    """
    
    def __init__(self, num_antennas: int, num_users: int, 
                 encoder=None, beamformer=None):
        self.num_antennas = num_antennas
        self.num_users = num_users
        
        if encoder is None:
            from models.neural_csi_encoder import NeuralCSIEncoder
            base_encoder = NeuralCSIEncoder(num_antennas=num_antennas)
            self.encoder = InterferenceAwareEncoder(base_encoder, num_users)
        else:
            self.encoder = encoder
        
        if beamformer is None:
            self.beamformer = MultiUserBeamformer(num_antennas, num_users)
        else:
            self.beamformer = beamformer
        
        self.power_optimizer = PowerAllocationOptimizer(num_users)
    
    def forward(self, H: torch.Tensor, method: str = 'mmse') -> dict:
        """
        Complete forward pass
        H: (batch, K, Nt) multi-user channel
        Returns: beamforming matrix, power allocation, metrics
        """
        # Compress CSI
        compressed, interference = self.encoder(H)
        
        # Compute channel qualities
        channel_norms = torch.norm(H, dim=-1)  # (batch, K)
        
        # Optimize power allocation
        power_allocation = self.power_optimizer(channel_norms)
        
        # Compute beamforming matrix
        W = self.beamformer(H, method=method, power_allocation=power_allocation)
        
        # Compute metrics
        # Signal-to-interference ratio
        desired_signal = torch.abs(torch.einsum('bki,bik->bk', H, W)) ** 2
        interference_signal = torch.sum(
            torch.abs(torch.einsum('bki,bij->bkj', H, W)) ** 2,
            dim=-1
        ) - desired_signal
        
        sir = desired_signal / (interference_signal + 1e-8)
        
        return {
            'beamforming_matrix': W,
            'power_allocation': power_allocation,
            'compressed_csi': compressed,
            'interference_matrix': interference,
            'sir': sir,
            'channel_qualities': channel_norms
        }



