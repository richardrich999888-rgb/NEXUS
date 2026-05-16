"""
Threegpp Channel Simulator

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import numpy as np
from .threegpp_cdl import ThreeGPP_CDL

class ThreeGPPChannelSimulator:
    """
    Wrapper class exposing the older channel simulator API,
    but internally running the new 3GPP CDL implementation.
    Ensures backward compatibility with repo code.
    """

    def __init__(self, num_antennas=64, num_users=8, scenario="CDL-A",
                 carrier_freq=3.5e9, num_subcarriers=12, device="cpu"):

        self.Nt = num_antennas
        self.Nr = num_users
        self.carrier_freq = carrier_freq
        self.scenario = scenario
        self.device = device
        self.num_subcarriers = num_subcarriers

        # Extract CDL profile letter
        if "-" in scenario:
            profile = scenario.split("-")[-1]
        else:
            profile = scenario

        profile = profile.upper().replace("CDL", "")

        self.cdl = ThreeGPP_CDL(
            profile=profile,
            carrier_freq=carrier_freq,
            num_ant=num_antennas,
            num_users=num_users,
            num_subcarriers=num_subcarriers,
            speed=3.0,
            device=device,
        )

    def generate_cdl_channel(self, batch_size=32):
        """
        Generates full CDL channels.
        Output: (B, Nr, Nt) complex
        """
        H = self.cdl.generate_csi_batch(batch_size=batch_size)  # (B, U, A, F)

        # Average over subcarriers → (B, U, A)
        H = torch.mean(H, dim=-1)

        # Transpose into (B, Nr, Nt)
        return H

    def generate_rayleigh_channel(self, batch_size=32):
        """
        IID Rayleigh fallback.
        """
        H = torch.randn(batch_size, self.Nr, self.Nt, dtype=torch.cfloat) * (
            1 / np.sqrt(2)
        )
        return H

    def generate_3gpp_channel(self, batch_size=32):
        """
        Alias to CDL generation for older training scripts.
        """
        return self.generate_cdl_channel(batch_size)

    def generate_training_data(self, num_samples=10000):
        """
        Generates dataset of channel matrices + dummy optimal beams.
        Returns:
            channels: (num_samples, Nr, Nt)
            labels:   (num_samples, Nt)
        """
        channels = []
        labels = []

        batch_size = min(256, num_samples)
        num_batches = (num_samples + batch_size - 1) // batch_size

        for i in range(num_batches):
            n_now = min(batch_size, num_samples - i * batch_size)

            H = self.generate_cdl_channel(n_now)
            channels.append(H)

            # Random normalized beamforming vector as placeholder labels
            w = torch.randn(n_now, self.Nt, dtype=torch.cfloat)
            w = w / torch.norm(w, dim=-1, keepdim=True)
            labels.append(w)

        return torch.cat(channels, dim=0), torch.cat(labels, dim=0)

# Backward-compatible alias for older imports
DigitalChannelSimulator = ThreeGPPChannelSimulator
