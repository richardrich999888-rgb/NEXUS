"""
Sparse Beam Mask Generator

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import torch.nn as nn
import torch.nn.functional as F

class SparseBeamMaskGenerator(nn.Module):
    """
    Input:  (B, latent_dim) - Compressed CSI representation
    Output: (B, N_ant) - Sparse beam activation mask
    """

    def __init__(self, latent_dim=128, num_antennas=64, hidden=256, topk=16):
        super().__init__()
        self.num_ant = num_antennas
        self.topk = topk

        self.fc1 = nn.Linear(latent_dim, hidden)
        self.fc2 = nn.Linear(hidden, num_antennas)

    def differentiable_topk(self, scores):
        """
        Produce sparse mask with differentiable relaxation.
        scores: (B, N_ant)
        """
        topk_vals, topk_idx = torch.topk(scores, self.topk, dim=-1)

        mask = torch.zeros_like(scores)
        mask.scatter_(1, topk_idx, 1.0)

        # Straight-through estimator
        return mask + (scores - scores.detach())

    def forward(self, z, adjacency_matrix=None, hard=False):
        """
        z: compressed CSI latent (B, latent_dim)
        adjacency_matrix: optional graph structure (for compatibility)
        hard: whether to return hard binary mask
        """
        s = F.relu(self.fc1(z))
        scores = torch.sigmoid(self.fc2(s))

        if hard:
            mask = self.differentiable_topk(scores)
            return mask, scores
        else:
            # For training, return probabilities
            mask = self.differentiable_topk(scores)
            return mask, scores
