"""
Antenna Topology

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import numpy as np

class AntennaTopologyBuilder:
    """Simple antenna topology utilities"""
    
    def __init__(self, panel_dims=(8, 8)):
        self.rows, self.cols = panel_dims
        self.num_antennas = self.rows * self.cols
        
    def build_adjacency_matrix(self, connection_radius=1.5):
        """Build simple adjacency matrix"""
        adj = torch.eye(self.num_antennas)
        return adj, []
    
    def build_graph_features(self, H):
        """Simple graph features"""
        batch_size, Nr, Nt = H.shape
        node_features = torch.mean(torch.abs(H), dim=1)  # Average across users
        return node_features.unsqueeze(-1)
