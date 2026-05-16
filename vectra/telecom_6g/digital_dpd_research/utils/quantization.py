"""
Quantization

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import numpy as np

class QuantizationUtils:
    """Quantization utilities for DPD models"""
    
    @staticmethod
    def quantize_model(model, num_bits=8, symmetric=True):
        """Apply quantization to model weights"""
        for name, param in model.named_parameters():
            if param.requires_grad:
                quantized, scale = QuantizationUtils.quantize_tensor(
                    param.data, num_bits, symmetric
                )
                param.data = QuantizationUtils.dequantize_tensor(
                    quantized, scale
                )
                # Store quantization parameters
                param.scale = scale
                param.zero_point = 0 if symmetric else 128
    
    @staticmethod
    def quantize_tensor(x, num_bits=8, symmetric=True):
        """Quantize tensor to int"""
        qmin = -2**(num_bits-1) if symmetric else 0
        qmax = 2**(num_bits-1)-1 if symmetric else 2**num_bits-1
        
        scale = x.abs().max() / qmax
        x_int = torch.clamp((x / scale).round(), qmin, qmax)
        
        dtype = torch.int8 if num_bits == 8 else torch.int16
        return x_int.to(dtype), scale
    
    @staticmethod
    def dequantize_tensor(x_int, scale):
        """Dequantize tensor"""
        return x_int.float() * scale
    
    @staticmethod
    def get_model_size(model, quantized=True):
        """Get model size in KB"""
        total_params = sum(p.numel() for p in model.parameters())
        
        if quantized:
            size_bytes = total_params * 1  # 8-bit
        else:
            size_bytes = total_params * 4  # 32-bit
        
        return size_bytes / 1024

class DPDAccelerator:
    """DPD accelerator for deployment"""
    
    def __init__(self, model_path, quantized=True):
        self.model = torch.jit.load(model_path)
        self.quantized = quantized
        
    def apply_dpd(self, signal):
        """Apply DPD in real-time (simulated)"""
        # Convert to tensor if needed
        if not isinstance(signal, torch.Tensor):
            signal = torch.tensor(signal)
        
        # Apply DPD
        with torch.no_grad():
            output = self.model(signal)
        
        return output.numpy()

