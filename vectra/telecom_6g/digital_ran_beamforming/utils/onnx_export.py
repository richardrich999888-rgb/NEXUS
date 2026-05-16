"""
ONNX Export Utilities for Hardware Deployment

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

ONNX Export Utilities for Hardware Deployment
Exports neural models to ONNX format for edge deployment
"""

import torch
import torch.onnx
import onnx
import onnxruntime as ort
import numpy as np
from pathlib import Path
from typing import Optional, Dict, Tuple

class ONNXExporter:
    """Export PyTorch models to ONNX for hardware deployment"""
    
    def __init__(self, model, model_name: str, input_shape: Tuple, 
                 opset_version: int = 13, dynamic_axes: Optional[Dict] = None):
        """
        Args:
            model: PyTorch model to export
            model_name: Name for exported model
            input_shape: Input tensor shape (batch, ...)
            opset_version: ONNX opset version
            dynamic_axes: Dynamic axes for variable batch sizes
        """
        self.model = model
        self.model_name = model_name
        self.input_shape = input_shape
        self.opset_version = opset_version
        self.dynamic_axes = dynamic_axes or {}
        self.exported_path = None
    
    def export(self, output_path: str, quantize: bool = True) -> str:
        """
        Export model to ONNX format
        Returns: Path to exported ONNX model
        """
        self.model.eval()
        
        # Create dummy input
        dummy_input = torch.randn(*self.input_shape)
        
        # Export to ONNX
        onnx_path = Path(output_path) / f"{self.model_name}.onnx"
        onnx_path.parent.mkdir(parents=True, exist_ok=True)
        
        torch.onnx.export(
            self.model,
            dummy_input,
            str(onnx_path),
            export_params=True,
            opset_version=self.opset_version,
            do_constant_folding=True,
            input_names=['input'],
            output_names=['output'],
            dynamic_axes=self.dynamic_axes,
            verbose=False
        )
        
        self.exported_path = str(onnx_path)
        
        # Validate ONNX model
        onnx_model = onnx.load(str(onnx_path))
        onnx.checker.check_model(onnx_model)
        
        print(f"✓ Exported {self.model_name} to {onnx_path}")
        
        # Quantize if requested
        if quantize:
            quantized_path = self.quantize_onnx(str(onnx_path))
            return quantized_path
        
        return str(onnx_path)
    
    def quantize_onnx(self, onnx_path: str) -> str:
        """
        Quantize ONNX model to INT8
        Returns: Path to quantized model
        """
        from onnxruntime.quantization import quantize_dynamic, QuantType
        
        quantized_path = onnx_path.replace('.onnx', '_quantized.onnx')
        
        quantize_dynamic(
            model_input=onnx_path,
            model_output=quantized_path,
            weight_type=QuantType.QUInt8
        )
        
        print(f"✓ Quantized model saved to {quantized_path}")
        return quantized_path
    
    def verify_onnx(self, test_input: torch.Tensor, rtol: float = 1e-3) -> bool:
        """
        Verify ONNX model matches PyTorch output
        """
        if self.exported_path is None:
            raise ValueError("Model not exported yet. Call export() first.")
        
        # PyTorch inference
        self.model.eval()
        with torch.no_grad():
            pytorch_output = self.model(test_input).numpy()
        
        # ONNX Runtime inference
        session = ort.InferenceSession(self.exported_path)
        onnx_input = {session.get_inputs()[0].name: test_input.numpy()}
        onnx_output = session.run(None, onnx_input)[0]
        
        # Compare outputs
        max_diff = np.max(np.abs(pytorch_output - onnx_output))
        mean_diff = np.mean(np.abs(pytorch_output - onnx_output))
        
        print(f"ONNX Verification:")
        print(f"  Max difference: {max_diff:.6f}")
        print(f"  Mean difference: {mean_diff:.6f}")
        print(f"  Match: {'✓' if max_diff < rtol else '✗'}")
        
        return max_diff < rtol
    
    def get_model_info(self) -> Dict:
        """Get model size and metadata"""
        if self.exported_path is None:
            raise ValueError("Model not exported yet.")
        
        model_size = Path(self.exported_path).stat().st_size / 1024  # KB
        
        session = ort.InferenceSession(self.exported_path)
        input_shape = session.get_inputs()[0].shape
        output_shape = session.get_outputs()[0].shape
        
        return {
            'model_path': self.exported_path,
            'model_size_kb': model_size,
            'input_shape': input_shape,
            'output_shape': output_shape,
            'providers': session.get_providers()
        }


class HardwareOptimizer:
    """Optimize ONNX models for specific hardware targets"""
    
    @staticmethod
    def optimize_for_fpga(onnx_path: str, output_path: str):
        """
        Optimize ONNX model for FPGA deployment
        - Fuse operations
        - Remove unnecessary nodes
        - Optimize for fixed-point arithmetic
        """
        import onnxoptimizer
        
        model = onnx.load(onnx_path)
        
        # Apply optimizations
        optimized_model = onnxoptimizer.optimize(model, [
            'eliminate_nop_transpose',
            'fuse_bn_into_conv',
            'fuse_matmul_add_bias_into_gemm',
            'eliminate_nop_pad',
            'eliminate_unused_initializer'
        ])
        
        onnx.save(optimized_model, output_path)
        print(f"✓ FPGA-optimized model saved to {output_path}")
        return output_path
    
    @staticmethod
    def optimize_for_arm(onnx_path: str, output_path: str):
        """
        Optimize ONNX model for ARM processors
        - Use ARM-specific operators
        - Optimize memory layout
        """
        # Use ONNX Runtime mobile optimizations
        from onnxruntime.tools import optimize_model
        
        optimized = optimize_model(
            onnx_path,
            model_type='bert',  # General optimization
            num_heads=0,
            hidden_size=0
        )
        
        optimized.save_model_to_file(output_path)
        print(f"✓ ARM-optimized model saved to {output_path}")
        return output_path
    
    @staticmethod
    def generate_verilog_wrapper(onnx_path: str, output_path: str):
        """
        Generate Verilog wrapper for FPGA implementation
        Creates interface for ONNX model inference
        """
        verilog_code = f"""
// Verilog wrapper for ONNX model: {Path(onnx_path).name}
// Auto-generated for FPGA deployment

module onnx_model_wrapper (
    input wire clk,
    input wire rst_n,
    input wire [31:0] input_data [0:INPUT_SIZE-1],
    output reg [31:0] output_data [0:OUTPUT_SIZE-1],
    output reg valid
);

    // Model parameters loaded from ONNX
    // TODO: Implement model inference logic
    // This is a template - actual implementation depends on model architecture
    
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            valid <= 1'b0;
        end else begin
            // Inference logic here
            valid <= 1'b1;
        end
    end

endmodule
"""
        
        with open(output_path, 'w') as f:
            f.write(verilog_code)
        
        print(f"✓ Verilog wrapper generated: {output_path}")


class RealTimeInference:
    """Real-time inference engine for deployed models"""
    
    def __init__(self, onnx_path: str, device: str = 'cpu'):
        """
        Initialize real-time inference session
        device: 'cpu', 'cuda', 'tensorrt', 'openvino'
        """
        self.onnx_path = onnx_path
        self.device = device
        
        # Create inference session with optimizations
        sess_options = ort.SessionOptions()
        sess_options.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
        sess_options.enable_mem_pattern = True
        sess_options.enable_cpu_mem_arena = True
        
        providers = ['CPUExecutionProvider']
        if device == 'cuda' and 'CUDAExecutionProvider' in ort.get_available_providers():
            providers = ['CUDAExecutionProvider', 'CPUExecutionProvider']
        
        self.session = ort.InferenceSession(
            onnx_path,
            sess_options=sess_options,
            providers=providers
        )
        
        self.input_name = self.session.get_inputs()[0].name
        self.output_name = self.session.get_outputs()[0].name
    
    def infer(self, input_data: np.ndarray) -> np.ndarray:
        """
        Run inference with minimal latency
        """
        outputs = self.session.run(
            [self.output_name],
            {self.input_name: input_data}
        )
        return outputs[0]
    
    def benchmark_latency(self, num_iterations: int = 1000, warmup: int = 100):
        """
        Benchmark inference latency
        """
        import time
        
        # Get input shape
        input_shape = self.session.get_inputs()[0].shape
        if input_shape[0] == 'batch' or input_shape[0] is None:
            input_shape = (1,) + tuple(input_shape[1:])
        
        dummy_input = np.random.randn(*input_shape).astype(np.float32)
        
        # Warmup
        for _ in range(warmup):
            _ = self.infer(dummy_input)
        
        # Benchmark
        latencies = []
        for _ in range(num_iterations):
            start = time.perf_counter()
            _ = self.infer(dummy_input)
            end = time.perf_counter()
            latencies.append((end - start) * 1000)  # ms
        
        avg_latency = np.mean(latencies)
        p50_latency = np.percentile(latencies, 50)
        p99_latency = np.percentile(latencies, 99)
        
        print(f"Inference Latency ({num_iterations} iterations):")
        print(f"  Average: {avg_latency:.3f} ms")
        print(f"  P50: {p50_latency:.3f} ms")
        print(f"  P99: {p99_latency:.3f} ms")
        
        return {
            'avg_ms': avg_latency,
            'p50_ms': p50_latency,
            'p99_ms': p99_latency
        }



