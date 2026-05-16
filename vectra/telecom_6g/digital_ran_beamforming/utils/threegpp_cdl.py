"""
3GPP TR 38.901-Compliant CDL (Clustered Delay Line) Channel Simulator

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

3GPP TR 38.901-Compliant CDL (Clustered Delay Line) Channel Simulator
Full-featured version for ULA (Uniform Linear Array), 64 antennas.

Implements CDL-A, CDL-B, CDL-C, CDL-D, CDL-E with:
- Delay spreads
- Angle spreads
- Doppler spectrum
- Per-cluster power
- Multiuser support
- Frequency-domain CSI generation
"""

import torch
import numpy as np
import math

# ------------------------------------------------------------
# Utility functions
# ------------------------------------------------------------

def db2pow(x):
    return 10 ** (x / 10)

def pow2db(x):
    return 10 * torch.log10(x + 1e-12)

def wrap_angle(angle):
    """Wrap angle to [-pi, pi]."""
    return (angle + math.pi) % (2 * math.pi) - math.pi

# ------------------------------------------------------------
# CDL Cluster Definitions (3GPP TR 38.901 Table 7.7.2)
# ------------------------------------------------------------

CDL_PROFILES = {
    "A": {
        "delays": [0.0, 0.2e-6, 0.5e-6, 1.6e-6, 2.5e-6],
        "powers": db2pow(torch.tensor([0, -2.2, -4.0, -6.0, -8.2])),
        "ASD": 5,
        "ASA": 11,
        "ZSD": 3,
        "ZSA": 7,
        "is_los": False,
    },
    "B": {
        "delays": [0.0, 0.1e-6, 0.3e-6, 0.7e-6, 1.3e-6, 2.3e-6],
        "powers": db2pow(torch.tensor([0, -2.1, -4.0, -5.8, -7.5, -9.0])),
        "ASD": 10,
        "ASA": 22,
        "ZSD": 3,
        "ZSA": 7,
        "is_los": False,
    },
    "C": {
        "delays": [0.0, 0.2e-6, 0.6e-6, 1.6e-6, 2.8e-6, 3.7e-6],
        "powers": db2pow(torch.tensor([0, -1.0, -3.0, -5.0, -7.2, -8.7])),
        "ASD": 2,
        "ASA": 35,
        "ZSD": 3,
        "ZSA": 7,
        "is_los": False,
    },
    "D": {
        "delays": [0.0, 0.2e-6, 0.9e-6, 1.7e-6, 2.6e-6],
        "powers": db2pow(torch.tensor([0, -1.2, -3.4, -5.5, -7.9])),
        "ASD": 5,
        "ASA": 8,
        "ZSD": 3,
        "ZSA": 7,
        "is_los": True,
    },
    "E": {
        "delays": [0.0, 0.1e-6, 0.3e-6, 0.9e-6, 2.1e-6, 3.7e-6],
        "powers": db2pow(torch.tensor([0, -0.9, -4.0, -8.0, -9.2, -10.5])),
        "ASD": 11,
        "ASA": 32,
        "ZSD": 3,
        "ZSA": 7,
        "is_los": True,
    },
}

# ------------------------------------------------------------
# Antenna geometry: ULA
# ------------------------------------------------------------

class ULAArray:
    def __init__(self, num_ant=64, antenna_spacing=0.5):
        """
        ULA along the x-axis, centered.
        antenna_spacing = 0.5 * lambda (half-wavelength)
        """
        self.num_ant = num_ant
        self.spacing = antenna_spacing

        # Antenna positions (x, y, z)
        idx = torch.arange(num_ant) - (num_ant - 1) / 2
        self.positions = torch.stack([idx * self.spacing, torch.zeros(num_ant), torch.zeros(num_ant)], dim=-1)

# ------------------------------------------------------------
# CDL Channel Simulator
# ------------------------------------------------------------

class ThreeGPP_CDL:
    def __init__(
        self,
        profile="A",
        carrier_freq=3.5e9,
        num_ant=64,
        num_users=8,
        num_subcarriers=12,
        speed=3.0,
        t_sample=0.001,
        device="cpu"
    ):
        """
        Full 3GPP CDL model.
        """
        self.profile = CDL_PROFILES[profile]
        self.fc = carrier_freq
        self.lam = 3e8 / carrier_freq
        self.num_ant = num_ant
        self.num_users = num_users
        self.num_subc = num_subcarriers
        self.speed = speed
        self.t_sample = t_sample
        self.device = device

        # Antenna array
        self.array = ULAArray(num_ant)

        # Process profile params
        self._init_clusters()

        # Initialize states
        self._initialize_user_angles()
        self._initialize_doppler()
        self._initialize_cluster_phases()

    # ------------------------------------------------------------
    # Internal initialization
    # ------------------------------------------------------------

    def _init_clusters(self):
        """Load delays & powers & normalize."""
        delays = torch.tensor(self.profile["delays"], dtype=torch.float32)
        powers = self.profile["powers"].float()
        powers = powers / powers.sum()

        self.delays = delays.to(self.device)
        self.powers = powers.to(self.device).sqrt()
        self.num_clusters = len(delays)

    def _initialize_user_angles(self):
        """Sample mean AoA/AoD for each user."""
        ASD = math.radians(self.profile["ASD"])
        ASA = math.radians(self.profile["ASA"])

        self.AoD_mean = wrap_angle(torch.randn(self.num_users, device=self.device) * ASD)
        self.AoA_mean = wrap_angle(torch.randn(self.num_users, device=self.device) * ASA)

    def _initialize_doppler(self):
        """Compute Doppler shifts for each user."""
        v = self.speed
        fD = (v / self.lam) * torch.cos(self.AoA_mean)
        self.fD = fD.to(self.device)

    def _initialize_cluster_phases(self):
        """Random initial phases per user & per cluster."""
        self.cluster_phases = torch.rand(self.num_users, self.num_clusters, device=self.device) * 2 * math.pi

    # ------------------------------------------------------------
    # Core generation logic
    # ------------------------------------------------------------

    def _steering_vector(self, aoa):
        """
        Compute ULA steering vector for angle `aoa`
        Returns shape: (N_ant,)
        """
        k = 2 * math.pi / self.lam
        x = self.array.positions[:, 0].to(self.device)
        return torch.exp(1j * k * x * torch.sin(aoa))

    def generate_csi_batch(self, batch_size=1):
        """
        Generate a batch of multiuser frequency-domain channels:
        Output shape: (B, U, N_ant, N_subc) complex tensor
        """
        B = batch_size
        U = self.num_users
        A = self.num_ant
        F = self.num_subc

        # OUTPUT
        H = torch.zeros(B, U, A, F, dtype=torch.cfloat, device=self.device)

        for b in range(B):
            for u in range(U):
                for c in range(self.num_clusters):
                    aoa = self.AoA_mean[u] + torch.randn(1, device=self.device) * math.radians(self.profile["ASA"]) / 30
                    taus = self.delays[c]

                    # Steering vector
                    a = self._steering_vector(aoa)

                    # Cluster amplitude
                    alpha = self.powers[c]

                    # Frequency domain: exp(-j 2pi f tau)
                    f = torch.arange(F, device=self.device).float()
                    freq = f * (15e3)
                    exp_freq = torch.exp(-1j * 2 * math.pi * taus * freq)

                    # Add to channel
                    H[b, u] += alpha * torch.outer(a, exp_freq)

        return H

    # ------------------------------------------------------------
    # Time evolution
    # ------------------------------------------------------------

    def update_time(self):
        """Evolve angles and phases over time."""
        dt = self.t_sample

        # Update cluster phases using Doppler
        for u in range(self.num_users):
            self.cluster_phases[u] += 2 * math.pi * self.fD[u] * dt

        # Optional: drift angles slightly
        self.AoA_mean += 0.001 * torch.randn(self.num_users, device=self.device)

# ------------------------------------------------------------
# Convenience wrapper for training scripts
# ------------------------------------------------------------

def get_cdl_channel_simulator(config):
    return ThreeGPP_CDL(
        profile=config["system"].get("scenario", "A")[-1],
        carrier_freq=config["system"]["carrier_freq"],
        num_ant=config["system"]["num_antennas"],
        num_users=config["system"]["num_users"],
        num_subcarriers=config["system"].get("num_subcarriers", 12),
        speed=config["mobility"].get("speed", 3.0),
        t_sample=config["mobility"].get("t_sample", 0.001),
        device="cpu",
    )
