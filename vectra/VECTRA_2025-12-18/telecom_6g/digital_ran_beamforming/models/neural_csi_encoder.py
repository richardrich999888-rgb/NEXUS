"""
Neural Csi Encoder

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import torch.nn as nn
import torch.nn.functional as F

class PositionalEncoding3D(nn.Module):
    def __init__(self, channels):
        super().__init__()
        self.channels = channels
        assert channels % 6 == 0, "Channels must be divisible by 6"

    def forward(self, x):
        B, C, H, W = x.shape
        c = self.channels // 6

        # coordinate grid
        h = torch.linspace(-1, 1, H, device=x.device)
        w = torch.linspace(-1, 1, W, device=x.device)
        grid_h, grid_w = torch.meshgrid(h, w, indexing="ij")

        pe = [
            torch.sin(grid_h * torch.pi * i).unsqueeze(0) for i in range(1, c + 1)
        ] + [
            torch.cos(grid_h * torch.pi * i).unsqueeze(0) for i in range(1, c + 1)
        ] + [
            torch.sin(grid_w * torch.pi * i).unsqueeze(0) for i in range(1, c + 1)
        ] + [
            torch.cos(grid_w * torch.pi * i).unsqueeze(0) for i in range(1, c + 1)
        ]

        pe = torch.cat(pe, dim=0)  # (C, H, W)
        pe = pe.unsqueeze(0).repeat(B, 1, 1, 1)
        return torch.cat([x, pe], dim=1)

class ResidualBlock(nn.Module):
    def __init__(self, channels):
        super().__init__()
        self.conv1 = nn.Conv2d(channels, channels, 3, padding=1)
        self.conv2 = nn.Conv2d(channels, channels, 3, padding=1)
        self.bn1 = nn.BatchNorm2d(channels)
        self.bn2 = nn.BatchNorm2d(channels)

    def forward(self, x):
        y = F.relu(self.bn1(self.conv1(x)))
        y = self.bn2(self.conv2(y))
        return F.relu(x + y)

class NeuralCSIEncoder(nn.Module):
    """
    Input:  (B, N_ant, N_subc, 2) - Channel matrix with real/imaginary parts
    Output: (B, latent_dim) - Compressed latent representation
    """

    def __init__(self, in_channels=2, base_channels=36, latent_dim=128, num_antennas=64, num_subcarriers=8):
        super().__init__()
        self.num_antennas = num_antennas
        self.num_subcarriers = num_subcarriers
        
        self.pe = PositionalEncoding3D(channels=base_channels)
        
        # PE adds 4 * (base_channels // 6) channels to input
        pe_channels = 4 * (base_channels // 6)
        self.conv1 = nn.Conv2d(in_channels + pe_channels, base_channels, 3, padding=1)
        self.res_blocks = nn.Sequential(
            ResidualBlock(base_channels),
            ResidualBlock(base_channels),
            ResidualBlock(base_channels),
        )

        self.bottleneck = nn.Sequential(
            nn.AdaptiveAvgPool2d((4, 4)),
            nn.Flatten(),
            nn.Linear(base_channels * 4 * 4, latent_dim),
        )

    def forward(self, x):
        # Input: (B, N_ant, N_subc, 2) -> (B, 2, N_ant, N_subc)
        x = x.permute(0, 3, 1, 2)

        x = self.pe(x)
        x = F.relu(self.conv1(x))
        x = self.res_blocks(x)
        z = self.bottleneck(x)
        return z
    
    def compress(self, H, antenna_indices=None):
        """
        Compatibility method for existing training scripts
        H: (batch, Nr, Nt) complex tensor
        Returns: compressed latent
        """
        batch_size, Nr, Nt = H.shape
        
        # Convert complex to real/imaginary representation
        H_real_imag = torch.stack([H.real, H.imag], dim=-1)  # (B, Nr, Nt, 2)
        
        # Use first user's channel or average across users
        if Nr > 1:
            H_processed = H_real_imag[:, 0, :, :]  # Use first user
        else:
            H_processed = H_real_imag.squeeze(1)
            
        # Ensure correct shape: (B, N_ant, N_subc, 2)
        if H_processed.dim() == 3:  # (B, N_ant, 2)
            H_processed = H_processed.unsqueeze(2)  # (B, N_ant, 1, 2)
            H_processed = H_processed.repeat(1, 1, self.num_subcarriers, 1)
        
        return self.forward(H_processed)
