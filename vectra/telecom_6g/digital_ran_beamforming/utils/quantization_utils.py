"""
Quantization Utils

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch

class QuantizationUtils:

    @staticmethod
    def quantize_tensor(x, num_bits=8):
        """
        Symmetric quantization to INT8/INT4.
        """
        qmin = -2 ** (num_bits - 1)
        qmax = 2 ** (num_bits - 1) - 1

        scale = x.abs().max() / qmax
        x_int = torch.clamp((x / scale).round(), qmin, qmax)

        return x_int.to(torch.int8 if num_bits == 8 else torch.int32), scale

    @staticmethod
    def dequantize_tensor(x_int, scale):
        return x_int.float() * scale

    @staticmethod
    def quantize_mask(mask):
        """
        Beam masks are binary → pack into bits.
        """
        return mask > 0.5

    @staticmethod
    def quantize_tt_cores(cores, num_bits=8):
        q_cores = []
        scales = []

        for c in cores:
            q, s = QuantizationUtils.quantize_tensor(c, num_bits)
            q_cores.append(q)
            scales.append(s)

        return q_cores, scales

    @staticmethod
    def quantize_model_weights(model, num_bits=8):
        """Apply quantization to all model weights"""
        for name, param in model.named_parameters():
            if param.requires_grad and param.data.numel() > 1:
                quantized, scale = QuantizationUtils.quantize_tensor(param.data, num_bits)
                param.data = QuantizationUtils.dequantize_tensor(quantized, scale)
