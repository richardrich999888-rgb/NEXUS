"""
Adaptive Quantization System

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Adaptive Quantization System
Dynamically adjusts quantization precision based on channel conditions
"""

import torch
import torch.nn as nn
import numpy as np
from typing import Dict, Tuple, Optional
from enum import Enum

class QuantizationMode(Enum):
    """Quantization precision modes"""
    INT4 = 4
    INT8 = 8
    INT16 = 16
    FP16 = 16  # Half precision float
    FP32 = 32  # Full precision float

class AdaptiveQuantizer(nn.Module):
    """
    Adaptive quantization that adjusts precision based on:
    - Channel quality (SNR)
    - Model performance requirements
    - Available compute resources
    """
    
    def __init__(self, 
                 initial_bits: int = 8,
                 min_bits: int = 4,
                 max_bits: int = 16,
                 adaptation_rate: float = 0.1):
        super().__init__()
        self.initial_bits = initial_bits
        self.min_bits = min_bits
        self.max_bits = max_bits
        self.adaptation_rate = adaptation_rate
        
        # Current quantization level (learnable)
        self.register_buffer('current_bits', torch.tensor(float(initial_bits)))
        
        # Performance history for adaptation
        self.register_buffer('performance_history', torch.zeros(100))
        self.register_buffer('history_idx', torch.tensor(0))
    
    def quantize_tensor(self, x: torch.Tensor, num_bits: int) -> Tuple[torch.Tensor, torch.Tensor]:
        """
        Quantize tensor to specified bit width
        """
        if num_bits >= 16:
            # Use float16 or float32
            if num_bits == 16:
                return x.half(), torch.tensor(1.0)
            else:
                return x, torch.tensor(1.0)
        
        # Integer quantization
        qmin = -2 ** (num_bits - 1)
        qmax = 2 ** (num_bits - 1) - 1
        
        scale = x.abs().max() / qmax if x.abs().max() > 0 else torch.tensor(1.0)
        x_scaled = x / (scale + 1e-8)
        x_quantized = torch.clamp(x_scaled.round(), qmin, qmax)
        
        return x_quantized.to(torch.int8 if num_bits == 8 else torch.int32), scale
    
    def dequantize_tensor(self, x_quantized: torch.Tensor, scale: torch.Tensor) -> torch.Tensor:
        """Dequantize tensor"""
        if x_quantized.dtype in [torch.float16, torch.float32]:
            return x_quantized
        return x_quantized.float() * scale
    
    def forward(self, x: torch.Tensor, channel_snr: Optional[torch.Tensor] = None) -> Tuple[torch.Tensor, Dict]:
        """
        Adaptive quantization forward pass
        x: Input tensor
        channel_snr: Channel SNR for adaptation (optional)
        Returns: Quantized tensor and metadata
        """
        # Determine quantization bits based on channel quality
        if channel_snr is not None:
            # Higher SNR -> can use lower precision
            # Lower SNR -> need higher precision
            snr_normalized = torch.clamp((channel_snr - 10) / 20, 0, 1)  # Normalize to [0, 1]
            target_bits = self.min_bits + (self.max_bits - self.min_bits) * (1 - snr_normalized)
            target_bits = torch.clamp(target_bits, self.min_bits, self.max_bits)
        else:
            target_bits = self.current_bits
        
        # Smooth adaptation
        self.current_bits = (1 - self.adaptation_rate) * self.current_bits + \
                            self.adaptation_rate * target_bits
        
        # Round to nearest valid bit width
        bits_int = int(round(self.current_bits.item()))
        bits_int = max(self.min_bits, min(self.max_bits, bits_int))
        
        # Quantize
        x_quantized, scale = self.quantize_tensor(x, bits_int)
        x_dequantized = self.dequantize_tensor(x_quantized, scale)
        
        # Compute quantization error
        quantization_error = torch.mean(torch.abs(x - x_dequantized))
        
        metadata = {
            'bits': bits_int,
            'scale': scale.item(),
            'quantization_error': quantization_error.item(),
            'compression_ratio': 32 / bits_int
        }
        
        return x_dequantized, metadata
    
    def update_performance(self, performance_metric: float):
        """
        Update performance history for adaptation
        performance_metric: Current performance (e.g., accuracy, SNR loss)
        """
        idx = int(self.history_idx.item()) % 100
        self.performance_history[idx] = performance_metric
        self.history_idx += 1
    
    def adapt_bits(self, target_performance: float, current_performance: float):
        """
        Adapt quantization bits based on performance
        """
        performance_diff = current_performance - target_performance
        
        if performance_diff < -0.01:  # Performance too low
            # Increase precision
            self.current_bits = torch.clamp(
                self.current_bits + 1,
                self.min_bits,
                self.max_bits
            )
        elif performance_diff > 0.01:  # Performance higher than needed
            # Decrease precision for efficiency
            self.current_bits = torch.clamp(
                self.current_bits - 0.5,
                self.min_bits,
                self.max_bits
            )


class ChannelAwareQuantization(nn.Module):
    """
    Quantization that adapts to channel conditions
    Uses different precision for different frequency subcarriers
    """
    
    def __init__(self, num_subcarriers: int, base_bits: int = 8):
        super().__init__()
        self.num_subcarriers = num_subcarriers
        self.base_bits = base_bits
        
        # Learnable bit allocation per subcarrier
        self.bit_allocation = nn.Parameter(
            torch.ones(num_subcarriers) * base_bits
        )
    
    def forward(self, x: torch.Tensor, channel_quality: torch.Tensor) -> torch.Tensor:
        """
        x: (batch, num_subcarriers, ...) input
        channel_quality: (batch, num_subcarriers) channel quality per subcarrier
        Returns: Quantized tensor
        """
        batch_size = x.shape[0]
        outputs = []
        
        for sc in range(self.num_subcarriers):
            x_sc = x[:, sc]
            quality_sc = channel_quality[:, sc].mean()
            
            # Allocate bits based on channel quality
            bits = self.bit_allocation[sc] * (1 + quality_sc)
            bits = torch.clamp(bits, 4, 16)
            bits_int = int(round(bits.item()))
            
            # Quantize this subcarrier
            qmin = -2 ** (bits_int - 1)
            qmax = 2 ** (bits_int - 1) - 1
            
            scale = x_sc.abs().max() / qmax if x_sc.abs().max() > 0 else torch.tensor(1.0)
            x_quantized = torch.clamp((x_sc / scale).round(), qmin, qmax)
            x_dequantized = x_quantized.float() * scale
            
            outputs.append(x_dequantized)
        
        return torch.stack(outputs, dim=1)


class MixedPrecisionQuantizer(nn.Module):
    """
    Mixed precision quantization
    Different layers use different precision
    """
    
    def __init__(self, layer_configs: Dict[str, int]):
        """
        layer_configs: Dict mapping layer names to bit widths
        Example: {'encoder': 8, 'predictor': 4, 'beamformer': 16}
        """
        super().__init__()
        self.layer_configs = layer_configs
        self.quantizers = nn.ModuleDict()
        
        for layer_name, bits in layer_configs.items():
            self.quantizers[layer_name] = AdaptiveQuantizer(initial_bits=bits)
    
    def quantize_layer(self, layer_name: str, x: torch.Tensor, 
                       channel_snr: Optional[torch.Tensor] = None) -> torch.Tensor:
        """Quantize specific layer"""
        if layer_name not in self.quantizers:
            return x  # No quantization
        
        quantized, _ = self.quantizers[layer_name](x, channel_snr)
        return quantized
    
    def get_compression_stats(self) -> Dict:
        """Get compression statistics"""
        stats = {}
        total_params = 0
        total_compressed = 0
        
        for layer_name, quantizer in self.quantizers.items():
            bits = int(quantizer.current_bits.item())
            compression = 32 / bits
            stats[layer_name] = {
                'bits': bits,
                'compression_ratio': compression
            }
            # Note: Actual param count would need model access
            total_compressed += compression
        
        stats['average_compression'] = total_compressed / len(self.quantizers)
        return stats



