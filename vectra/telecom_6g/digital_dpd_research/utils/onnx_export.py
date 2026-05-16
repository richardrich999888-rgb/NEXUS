"""
ONNX Export for DPD Models

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

ONNX Export for DPD Models
Specialized export for DPD deployment
"""

import torch
import torch.onnx
import onnx
from pathlib import Path
from typing import Tuple, Optional

class DPDONNXExporter:
    """Export DPD models to ONNX for deployment"""
    
    @staticmethod
    def export_dpd_model(model: torch.nn.Module, 
                        output_path: str,
                        input_shape: Tuple = (1, 64, 2),  # (batch, antennas, I/Q)
                        opset_version: int = 13) -> str:
        """
        Export DPD model to ONNX
        """
        model.eval()
        
        dummy_input = torch.randn(*input_shape)
        
        onnx_path = Path(output_path) / "dpd_model.onnx"
        onnx_path.parent.mkdir(parents=True, exist_ok=True)
        
        torch.onnx.export(
            model,
            dummy_input,
            str(onnx_path),
            export_params=True,
            opset_version=opset_version,
            do_constant_folding=True,
            input_names=['input_signal'],
            output_names=['predistorted_signal'],
            dynamic_axes={
                'input_signal': {0: 'batch_size'},
                'predistorted_signal': {0: 'batch_size'}
            },
            verbose=False
        )
        
        # Validate
        onnx_model = onnx.load(str(onnx_path))
        onnx.checker.check_model(onnx_model)
        
        print(f"✓ DPD model exported to {onnx_path}")
        return str(onnx_path)
    
    @staticmethod
    def export_beam_aware_dpd(model: torch.nn.Module,
                               output_path: str,
                               input_shape: Tuple = (1, 64, 2),
                               beam_shape: Tuple = (64,)) -> str:
        """
        Export beam-aware DPD with beam conditioning input
        """
        model.eval()
        
        dummy_input = torch.randn(*input_shape)
        dummy_beam = torch.randn(*beam_shape)
        
        onnx_path = Path(output_path) / "beam_aware_dpd.onnx"
        onnx_path.parent.mkdir(parents=True, exist_ok=True)
        
        # Create wrapper for export
        class DPDWrapper(torch.nn.Module):
            def __init__(self, base_model):
                super().__init__()
                self.base_model = base_model
            
            def forward(self, signal, beam_weights):
                return self.base_model(signal, beam_weights=beam_weights)
        
        wrapper = DPDWrapper(model)
        
        torch.onnx.export(
            wrapper,
            (dummy_input, dummy_beam),
            str(onnx_path),
            export_params=True,
            opset_version=13,
            input_names=['input_signal', 'beam_weights'],
            output_names=['predistorted_signal'],
            verbose=False
        )
        
        print(f"✓ Beam-aware DPD exported to {onnx_path}")
        return str(onnx_path)



