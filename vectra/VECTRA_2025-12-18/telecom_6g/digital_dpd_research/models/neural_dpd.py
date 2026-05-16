"""
Neural Dpd

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import torch.nn as nn
import torch.nn.functional as F
import numpy as np

class NeuralDPD(nn.Module):
    """
    Neural Network Digital Predistorter
    Based on RVTDNN2L architecture (Real-Valued Time-Delay Neural Network)
    """
    
    def __init__(self, memory_depth=5, hidden_dims=[64, 64], num_antennas=64):
        super().__init__()
        self.memory_depth = memory_depth
        self.num_antennas = num_antennas
        
        # Real-valued time-delay neural network
        self.input_layer = nn.Linear(2 * memory_depth, hidden_dims[0])
        
        # Time-delay layers with skip connections
        self.td_layers = nn.ModuleList([
            nn.Linear(hidden_dims[0] + 2 * memory_depth, hidden_dims[0])
            for _ in range(2)
        ])
        
        self.hidden_layers = nn.ModuleList([
            nn.Linear(hidden_dims[0], hidden_dims[1])
        ])
        
        self.output_layer = nn.Linear(hidden_dims[1], 2)  # I/Q output
        
        # Activation
        self.activation = nn.ReLU()
        
        # Initialize weights
        self._initialize_weights()
    
    def _initialize_weights(self):
        """Xavier initialization for stable training"""
        for m in self.modules():
            if isinstance(m, nn.Linear):
                nn.init.xavier_normal_(m.weight)
                nn.init.zeros_(m.bias)
    
    def forward(self, x):
        """
        Forward pass for neural DPD
        x: complex input signal [batch_size, 2] (I/Q) or [batch_size] complex
        Returns: predistorted signal [batch_size, 2] (I/Q)
        """
        batch_size = x.shape[0]
        
        # Ensure real-valued representation
        if torch.is_complex(x):
            x_real = x.real.unsqueeze(-1)
            x_imag = x.imag.unsqueeze(-1)
            x = torch.cat([x_real, x_imag], dim=-1)
        
        # Create memory taps
        memory_taps = []
        for i in range(self.memory_depth):
            shifted = torch.roll(x, shifts=i, dims=0)
            # Zero pad the first i samples
            if i > 0:
                shifted[:i] = 0
            memory_taps.append(shifted)
        
        # Concatenate memory taps
        x_mem = torch.cat(memory_taps, dim=-1)  # [batch_size, 2 * memory_depth]
        
        # First layer
        h = self.activation(self.input_layer(x_mem))
        
        # Time-delay layers with residual connections
        for td_layer in self.td_layers:
            # Concatenate original memory with hidden state
            h_mem = torch.cat([h, x_mem], dim=-1)
            h = self.activation(td_layer(h_mem)) + h  # Residual connection
        
        # Hidden layers
        for hidden_layer in self.hidden_layers:
            h = self.activation(hidden_layer(h))
        
        # Output layer (no activation for linear output)
        output = self.output_layer(h)
        
        return output

class BeamAwareDPD(nn.Module):
    """
    Beam-aware DPD with conditioning on beamforming weights
    Shares coefficients across antenna clusters
    """
    
    def __init__(self, num_clusters=8, memory_depth=5, hidden_dims=[32, 32], num_antennas=64):
        super().__init__()
        self.num_clusters = num_clusters
        self.memory_depth = memory_depth
        self.num_antennas = num_antennas
        
        # Beam conditioning encoder (adapts to number of antennas)
        self.beam_encoder = nn.Sequential(
            nn.Linear(num_antennas, 32),
            nn.ReLU(),
            nn.Linear(32, 16),
            nn.ReLU(),
            nn.Linear(16, num_clusters)  # Cluster assignments
        )
        
        # Cluster-specific DPD generators
        self.dpd_generators = nn.ModuleList([
            NeuralDPD(memory_depth, hidden_dims, 1)
            for _ in range(num_clusters)
        ])
        
        # Coefficient memory for deployment
        self.register_buffer('cached_coefficients', None)
    
    def forward(self, x, beam_weights=None, use_cached=False):
        """
        Forward pass with beam conditioning
        x: input signal [batch_size, num_antennas, 2]
        beam_weights: beamforming weights [num_antennas] or None
        use_cached: use pre-computed coefficients for inference
        """
        batch_size, num_antennas, _ = x.shape
        
        if use_cached and self.cached_coefficients is not None:
            # Use cached coefficients for fast inference
            return self._apply_cached_dpd(x)
        
        if beam_weights is None:
            # Default to uniform weights
            beam_weights = torch.ones(num_antennas, device=x.device)
        
        # Use static clustering since beam_encoder outputs global state not per-antenna
        # Map antenna i to cluster (i % num_clusters)
        cluster_assignments = torch.arange(num_antennas, device=x.device) % self.num_clusters 
        
        # Apply cluster-specific DPD
        output = torch.zeros_like(x)
        
        for cluster_idx in range(self.num_clusters):
            # Find antennas in this cluster
            mask = (cluster_assignments == cluster_idx)
            if mask.sum() == 0:
                continue
            
            # Get DPD for this cluster
            dpd = self.dpd_generators[cluster_idx]
            
            # Apply to all antennas in cluster
            for ant_idx in torch.where(mask)[0]:
                ant_signal = x[:, ant_idx, :]
                predistorted = dpd(ant_signal)
                output[:, ant_idx, :] = predistorted
        
        # Cache coefficients for next inference
        if not self.training:
            self._cache_coefficients(beam_weights)
        
        return output
    
    def _apply_cached_dpd(self, x):
        """Apply DPD using cached coefficients"""
        # Implementation for production deployment
        # Uses fixed polynomial coefficients instead of neural network
        batch_size, num_antennas, _ = x.shape
        output = torch.zeros_like(x)
        
        # Simple polynomial DPD with cached coefficients
        for ant_idx in range(num_antennas):
            ant_signal = x[:, ant_idx, :]
            # Convert to complex
            if not torch.is_complex(ant_signal):
                ant_complex = torch.complex(ant_signal[:, 0], ant_signal[:, 1])
            else:
                ant_complex = ant_signal
            
            # Apply 3rd order polynomial DPD (example)
            coeffs = self.cached_coefficients[ant_idx]
            output_complex = (
                coeffs[0] * ant_complex +
                coeffs[1] * ant_complex * torch.abs(ant_complex)**2 +
                coeffs[2] * ant_complex * torch.abs(ant_complex)**4
            )
            
            # Convert back to I/Q
            output[:, ant_idx, 0] = output_complex.real
            output[:, ant_idx, 1] = output_complex.imag
        
        return output
    
    def _cache_coefficients(self, beam_weights):
        """Extract and cache polynomial coefficients from neural model"""
        # This is a simplified version - in practice, you'd extract
        # equivalent polynomial coefficients from the neural network
        num_antennas = beam_weights.shape[0]
        
        # Generate random coefficients for demonstration
        # In real implementation, extract from neural network
        cached = torch.randn(num_antennas, 3, dtype=torch.cfloat)
        self.cached_coefficients = cached
    
    def get_model_size(self, quantized=True):
        """Calculate model size in KB"""
        total_params = sum(p.numel() for p in self.parameters())
        
        if quantized:
            # 8-bit quantization
            size_bytes = total_params * 1
        else:
            # 32-bit float
            size_bytes = total_params * 4
        
        size_kb = size_bytes / 1024
        return size_kb

class JointBeamformingDPD(nn.Module):
    """
    Joint beamforming and DPD optimization
    Integrates with existing TT beamformer
    """
    
    def __init__(self, tt_beamformer, neural_dpd, num_antennas=64):
        super().__init__()
        self.beamformer = tt_beamformer
        self.dpd = neural_dpd
        self.num_antennas = num_antennas
        
        # Joint optimization layer
        self.joint_layer = nn.Sequential(
            nn.Linear(128, 64),  # Input: compressed CSI
            nn.ReLU(),
            nn.Linear(64, 32),
            nn.ReLU(),
            nn.Linear(32, 16)   # Output: DPD conditioning vector
        )
    
    def forward(self, channel_state, data_symbols):
        """
        Joint forward pass
        channel_state: compressed CSI [batch_size, csi_dim] (complex)
        data_symbols: input symbols [batch_size, num_streams, time]
        """
        batch_size = channel_state.shape[0]
        
        # Generate beamforming weights
        beam_weights = self.beamformer.compute_beamweights(
            channel_state.unsqueeze(1)  # Add user dimension (B, 1, A)
        )
        
        # Generate DPD conditioning from CSI
        # Handle complex input: flatten real/imag parts
        channel_real = torch.view_as_real(channel_state).reshape(batch_size, -1)
        dpd_condition = self.joint_layer(channel_real)
        
        # Apply beamforming
        # data_symbols: (B, S, T)
        # beam_weights: (B, A)
        # Result: (B, A, T)
        if data_symbols.dim() == 3:
            # Broadcast beam weights over time
            beamformed = torch.einsum('bst,ba->bat', data_symbols, beam_weights.conj())
            time_steps = data_symbols.shape[2]
        else:
            # Assume (B, S) - no time dim
            beamformed = torch.einsum('bs,ba->ba', data_symbols, beam_weights.conj())
            time_steps = 1
            beamformed = beamformed.unsqueeze(-1)
            
        # Apply DPD with beam conditioning
        # Convert to real-valued for DPD: (B, A, T, 2)
        beamformed_real = torch.stack([beamformed.real, beamformed.imag], dim=-1)
        
        # Reshape for DPD: (B*T, A, 2) - DPD expects time/batch in dim 0
        beamformed_reshaped = beamformed_real.permute(0, 2, 1, 3).reshape(batch_size * time_steps, -1, 2)
        
        # Expand beam weights: (B, A) -> (B*T, A)
        beam_weights_expanded = beam_weights.unsqueeze(1).repeat(1, time_steps, 1).reshape(batch_size * time_steps, -1)
        
        # Apply DPD
        predistorted = self.dpd(beamformed_reshaped, 
                               beam_weights=beam_weights_expanded.abs())
        
        # Reshape back: (B*T, A, 2) -> (B, T, A, 2) -> (B, A, T, 2)
        predistorted = predistorted.reshape(batch_size, time_steps, -1, 2).permute(0, 2, 1, 3)
        
        # Convert back to complex
        predistorted_complex = torch.complex(predistorted[..., 0], 
                                            predistorted[..., 1])
        
        return {
            'beam_weights': beam_weights,
            'beamformed': beamformed,
            'predistorted': predistorted_complex,
            'dpd_condition': dpd_condition
        }

