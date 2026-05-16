"""
Pa Behavioral

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import torch.nn as nn
import numpy as np

class PAModel(nn.Module):
    """
    Base class for Power Amplifier behavioral models
    """
    
    def __init__(self, saturation_amplitude=1.0):
        super().__init__()
        self.saturation_amplitude = saturation_amplitude
    
    def forward(self, x):
        """
        Apply PA nonlinearity
        x: complex input signal
        Returns: distorted output
        """
        raise NotImplementedError

class RappModel(PAModel):
    """
    Rapp model for solid-state PAs
    Models AM-AM distortion only
    """
    
    def __init__(self, smoothness_factor=3.0, saturation_amplitude=1.0):
        super().__init__(saturation_amplitude)
        self.smoothness_factor = smoothness_factor
    
    def forward(self, x):
        # Convert to magnitude/phase
        if torch.is_complex(x):
            mag = torch.abs(x)
            phase = torch.angle(x)
        else:
            # Assume I/Q format
            mag = torch.norm(x, dim=-1)
            phase = torch.atan2(x[..., 1], x[..., 0])
        
        # Rapp model AM-AM conversion
        denominator = (1 + (mag / self.saturation_amplitude) ** 
                     (2 * self.smoothness_factor)) ** (1 / (2 * self.smoothness_factor))
        
        mag_out = mag / denominator
        
        # Convert back
        if torch.is_complex(x):
            output = mag_out * torch.exp(1j * phase)
        else:
            output = torch.stack([
                mag_out * torch.cos(phase),
                mag_out * torch.sin(phase)
            ], dim=-1)
        
        return output

class SalehModel(PAModel):
    """
    Saleh model for traveling-wave tube amplifiers
    Models both AM-AM and AM-PM distortion
    """
    
    def __init__(self, alpha_a=2.0, beta_a=1.0, alpha_phi=2.0, beta_phi=1.0):
        super().__init__()
        self.alpha_a = alpha_a  # AM-AM parameter
        self.beta_a = beta_a    # AM-AM parameter
        self.alpha_phi = alpha_phi  # AM-PM parameter
        self.beta_phi = beta_phi    # AM-PM parameter
    
    def forward(self, x):
        # Convert to magnitude/phase
        if torch.is_complex(x):
            mag = torch.abs(x)
            phase = torch.angle(x)
        else:
            mag = torch.norm(x, dim=-1)
            phase = torch.atan2(x[..., 1], x[..., 0])
        
        # Saleh model equations
        mag_out = (self.alpha_a * mag) / (1 + self.beta_a * mag ** 2)
        phase_out = phase + (self.alpha_phi * mag ** 2) / (1 + self.beta_phi * mag ** 2)
        
        # Convert back
        if torch.is_complex(x):
            output = mag_out * torch.exp(1j * phase_out)
        else:
            output = torch.stack([
                mag_out * torch.cos(phase_out),
                mag_out * torch.sin(phase_out)
            ], dim=-1)
        
        return output

class GhorbaniModel(PAModel):
    """
    Ghorbani model - more accurate for modern PAs
    """
    
    def __init__(self, a=[8.1081, 1.5413, 6.5202, -0.0718], 
                 b=[4.6645, -2.0965, 10.88, -0.0030]):
        super().__init__()
        self.a = torch.tensor(a)
        self.b = torch.tensor(b)
    
    def forward(self, x):
        mag = torch.abs(x) if torch.is_complex(x) else torch.norm(x, dim=-1)
        phase = torch.angle(x) if torch.is_complex(x) else torch.atan2(x[..., 1], x[..., 0])
        
        # Ghorbani equations
        mag_out = (self.a[0] * mag + self.a[1] * mag**2 + self.a[2] * mag**3) / \
                  (1 + self.a[3] * mag**2)
        phase_out = phase + (self.b[0] * mag + self.b[1] * mag**2 + self.b[2] * mag**3) / \
                    (1 + self.b[3] * mag**2)
        
        if torch.is_complex(x):
            return mag_out * torch.exp(1j * phase_out)
        else:
            return torch.stack([
                mag_out * torch.cos(phase_out),
                mag_out * torch.sin(phase_out)
            ], dim=-1)

class PAArrayModel(nn.Module):
    """
    Array of PAs with individual variations
    Models non-identical PAs in massive MIMO
    """
    
    def __init__(self, num_antennas=64, model_type='rapp', **kwargs):
        super().__init__()
        self.num_antennas = num_antennas
        
        # Create individual PA models with variations
        self.pa_models = nn.ModuleList()
        
        for i in range(num_antennas):
            if model_type == 'rapp':
                # Add random variations to parameters
                sf = kwargs.get('smoothness_factor', 3.0) * \
                     (1 + 0.1 * torch.randn(1).item())  # ±10% variation
                sa = kwargs.get('saturation_amplitude', 1.0) * \
                     (1 + 0.05 * torch.randn(1).item())  # ±5% variation
                pa = RappModel(smoothness_factor=sf, saturation_amplitude=sa)
            
            elif model_type == 'saleh':
                aa = kwargs.get('alpha_a', 2.0) * (1 + 0.1 * torch.randn(1).item())
                ba = kwargs.get('beta_a', 1.0) * (1 + 0.1 * torch.randn(1).item())
                ap = kwargs.get('alpha_phi', 2.0) * (1 + 0.2 * torch.randn(1).item())
                bp = kwargs.get('beta_phi', 1.0) * (1 + 0.2 * torch.randn(1).item())
                pa = SalehModel(alpha_a=aa, beta_a=ba, alpha_phi=ap, beta_phi=bp)
            
            else:
                pa = RappModel()
            
            self.pa_models.append(pa)
    
    def forward(self, x):
        """
        Apply PA array distortion
        x: [batch_size, num_antennas, ...] input signals
        """
        batch_size, num_antennas = x.shape[:2]
        outputs = []
        
        for ant_idx in range(num_antennas):
            ant_signal = x[:, ant_idx, ...]
            distorted = self.pa_models[ant_idx](ant_signal)
            outputs.append(distorted)
        
        return torch.stack(outputs, dim=1)

