"""
Online Learning for Digital Predistortion

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Online Learning for Digital Predistortion
Adapts DPD coefficients in real-time based on feedback
"""

import torch
import torch.nn as nn
import torch.optim as optim
import numpy as np
from collections import deque
from typing import Optional, Dict, Tuple
import copy

class OnlineDPDLearner:
    """
    Online learning system for DPD adaptation
    Continuously updates DPD coefficients based on PA output feedback
    """
    
    def __init__(self, 
                 dpd_model: nn.Module,
                 learning_rate: float = 1e-4,
                 adaptation_rate: float = 0.1,
                 memory_size: int = 1000,
                 update_frequency: int = 100):
        """
        Args:
            dpd_model: Neural DPD model to adapt
            learning_rate: Learning rate for online updates
            adaptation_rate: How quickly to adapt (0-1)
            memory_size: Size of experience replay buffer
            update_frequency: Update model every N samples
        """
        self.dpd_model = dpd_model
        self.learning_rate = learning_rate
        self.adaptation_rate = adaptation_rate
        self.memory_size = memory_size
        self.update_frequency = update_frequency
        
        # Experience replay buffer
        self.memory = deque(maxlen=memory_size)
        
        # Online optimizer (lightweight)
        self.optimizer = optim.SGD(
            self.dpd_model.parameters(),
            lr=learning_rate,
            momentum=0.9
        )
        
        # Performance tracking
        self.evm_history = deque(maxlen=100)
        self.aclr_history = deque(maxlen=100)
        self.update_count = 0
        
        # Exponential moving average of model parameters
        self.ema_model = copy.deepcopy(dpd_model)
        self.ema_alpha = 0.99
    
    def add_experience(self, 
                      input_signal: torch.Tensor,
                      pa_output: torch.Tensor,
                      target_signal: torch.Tensor,
                      metrics: Dict):
        """
        Add new experience to replay buffer
        input_signal: Input to DPD
        pa_output: PA output (after DPD)
        target_signal: Desired output (original input)
        metrics: Performance metrics (EVM, ACLR, etc.)
        """
        experience = {
            'input': input_signal.detach().cpu(),
            'pa_output': pa_output.detach().cpu(),
            'target': target_signal.detach().cpu(),
            'evm': metrics.get('evm', 0.0),
            'aclr': metrics.get('aclr', 0.0)
        }
        
        self.memory.append(experience)
        self.evm_history.append(metrics.get('evm', 0.0))
        self.aclr_history.append(metrics.get('aclr', 0.0))
    
    def compute_loss(self, input_signal: torch.Tensor, 
                    pa_output: torch.Tensor,
                    target_signal: torch.Tensor) -> torch.Tensor:
        """
        Compute loss for online learning
        """
        # Signal fidelity loss
        signal_loss = nn.MSELoss()(pa_output.abs(), target_signal.abs())
        
        # Spectral regrowth penalty (simplified)
        # In practice, would compute ACLR from FFT
        spectral_loss = torch.mean(torch.abs(pa_output - target_signal) ** 2)
        
        # Total loss
        total_loss = signal_loss + 0.1 * spectral_loss
        
        return total_loss
    
    def update_model(self, batch_size: int = 32):
        """
        Update DPD model from experience replay buffer
        """
        if len(self.memory) < batch_size:
            return
        
        # Sample batch from memory
        indices = np.random.choice(len(self.memory), batch_size, replace=False)
        batch = [self.memory[i] for i in indices]
        
        # Prepare batch
        inputs = torch.stack([e['input'] for e in batch])
        pa_outputs = torch.stack([e['pa_output'] for e in batch])
        targets = torch.stack([e['target'] for e in batch])
        
        # Move to device
        device = next(self.dpd_model.parameters()).device
        inputs = inputs.to(device)
        pa_outputs = pa_outputs.to(device)
        targets = targets.to(device)
        
        # Forward pass
        self.dpd_model.train()
        self.optimizer.zero_grad()
        
        # Get DPD output
        dpd_output = self.dpd_model(inputs)
        
        # Compute loss (we don't have PA model here, use stored PA output)
        # In practice, would run through PA model
        loss = self.compute_loss(dpd_output, pa_outputs, targets)
        
        # Backward pass
        loss.backward()
        
        # Gradient clipping
        torch.nn.utils.clip_grad_norm_(self.dpd_model.parameters(), max_norm=1.0)
        
        # Update
        self.optimizer.step()
        
        # Update EMA model
        self._update_ema_model()
        
        self.update_count += 1
        
        return loss.item()
    
    def _update_ema_model(self):
        """Update exponential moving average model"""
        with torch.no_grad():
            for ema_param, param in zip(self.ema_model.parameters(), 
                                        self.dpd_model.parameters()):
                ema_param.data.mul_(self.ema_alpha).add_(
                    param.data, alpha=1 - self.ema_alpha
                )
    
    def adapt_to_channel(self, channel_quality: float):
        """
        Adapt learning rate based on channel quality
        Better channel -> slower adaptation (more stable)
        Worse channel -> faster adaptation (more responsive)
        """
        # Normalize channel quality to [0, 1]
        quality_norm = np.clip((channel_quality - 10) / 30, 0, 1)
        
        # Adjust learning rate
        new_lr = self.learning_rate * (0.5 + 0.5 * quality_norm)
        
        for param_group in self.optimizer.param_groups:
            param_group['lr'] = new_lr
    
    def get_performance_stats(self) -> Dict:
        """Get current performance statistics"""
        if len(self.evm_history) == 0:
            return {}
        
        return {
            'avg_evm': np.mean(self.evm_history),
            'avg_aclr': np.mean(self.aclr_history),
            'update_count': self.update_count,
            'memory_size': len(self.memory),
            'learning_rate': self.optimizer.param_groups[0]['lr']
        }
    
    def should_update(self) -> bool:
        """Check if model should be updated"""
        return len(self.memory) >= self.update_frequency and \
               len(self.memory) % self.update_frequency == 0


class IncrementalDPD(nn.Module):
    """
    Incremental DPD that adapts coefficients in real-time
    Uses polynomial basis functions for fast adaptation
    """
    
    def __init__(self, memory_depth: int = 5, nonlinearity_order: int = 5):
        super().__init__()
        self.memory_depth = memory_depth
        self.nonlinearity_order = nonlinearity_order
        
        # Polynomial coefficients (learnable)
        num_coeffs = memory_depth * nonlinearity_order
        self.coefficients = nn.Parameter(
            torch.randn(num_coeffs, dtype=torch.cfloat) * 0.01
        )
        
        # Adaptation rate
        self.register_buffer('adaptation_rate', torch.tensor(0.1))
    
    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Apply polynomial DPD
        x: Input signal (batch, length) complex
        Returns: Predistorted signal
        """
        batch_size, length = x.shape
        
        # Create memory taps
        memory_taps = []
        for m in range(self.memory_depth):
            shifted = torch.roll(x, shifts=m, dims=1)
            if m > 0:
                shifted[:, :m] = 0
            memory_taps.append(shifted)
        
        # Apply polynomial basis
        output = torch.zeros_like(x)
        
        coeff_idx = 0
        for m in range(self.memory_depth):
            x_m = memory_taps[m]
            for n in range(1, self.nonlinearity_order + 1):
                # Basis: x[n-m] * |x[n-m]|^(2n)
                basis = x_m * (torch.abs(x_m) ** (2 * n))
                output += self.coefficients[coeff_idx] * basis
                coeff_idx += 1
        
        return output
    
    def update_coefficients(self, gradient: torch.Tensor, learning_rate: float):
        """
        Update coefficients incrementally
        gradient: Gradient w.r.t. coefficients
        learning_rate: Step size
        """
        with torch.no_grad():
            self.coefficients.data -= learning_rate * gradient
    
    def get_coefficients(self) -> torch.Tensor:
        """Get current coefficients"""
        return self.coefficients.data.clone()


class FeedbackDPD(nn.Module):
    """
    DPD with feedback loop for continuous adaptation
    Uses PA output feedback to update coefficients
    """
    
    def __init__(self, base_dpd: nn.Module, feedback_delay: int = 10):
        super().__init__()
        self.base_dpd = base_dpd
        self.feedback_delay = feedback_delay
        
        # Feedback processing network
        self.feedback_processor = nn.Sequential(
            nn.Linear(2, 32),  # I/Q input
            nn.ReLU(),
            nn.Linear(32, 16),
            nn.ReLU(),
            nn.Linear(16, 1)  # Correction signal
        )
        
        # Feedback buffer
        self.register_buffer('feedback_buffer', torch.zeros(feedback_delay, 2))
        self.register_buffer('buffer_idx', torch.tensor(0))
    
    def forward(self, x: torch.Tensor, pa_feedback: Optional[torch.Tensor] = None) -> torch.Tensor:
        """
        Forward pass with optional feedback
        x: Input signal
        pa_feedback: PA output feedback (for adaptation)
        """
        # Base DPD
        dpd_output = self.base_dpd(x)
        
        # Process feedback if available
        if pa_feedback is not None:
            # Convert to I/Q
            feedback_iq = torch.stack([pa_feedback.real, pa_feedback.imag], dim=-1)
            
            # Process feedback
            correction = self.feedback_processor(feedback_iq)
            
            # Apply correction (simplified - would need proper delay alignment)
            correction_complex = torch.complex(correction[..., 0], torch.zeros_like(correction[..., 0]))
            dpd_output = dpd_output + 0.1 * correction_complex
        
        return dpd_output
    
    def update_from_feedback(self, input_signal: torch.Tensor, 
                           pa_output: torch.Tensor,
                           target_signal: torch.Tensor):
        """
        Update DPD based on feedback
        """
        # Compute error
        error = target_signal - pa_output
        
        # Update base DPD (simplified - would use proper gradient)
        # In practice, would backprop through PA model
        pass


class RealTimeDPDAdaptation:
    """
    Real-time DPD adaptation system
    Combines online learning with incremental updates
    """
    
    def __init__(self, dpd_model: nn.Module, adaptation_mode: str = 'online'):
        """
        adaptation_mode: 'online', 'incremental', 'feedback'
        """
        self.dpd_model = dpd_model
        self.adaptation_mode = adaptation_mode
        
        if adaptation_mode == 'online':
            self.learner = OnlineDPDLearner(dpd_model)
        elif adaptation_mode == 'incremental':
            if not isinstance(dpd_model, IncrementalDPD):
                raise ValueError("Incremental mode requires IncrementalDPD model")
            self.learner = None  # Direct coefficient updates
        else:
            self.learner = None
    
    def process_sample(self, 
                      input_signal: torch.Tensor,
                      pa_output: torch.Tensor,
                      target_signal: torch.Tensor,
                      metrics: Dict) -> torch.Tensor:
        """
        Process single sample and adapt if needed
        Returns: Updated DPD output
        """
        if self.adaptation_mode == 'online':
            # Add to experience buffer
            self.learner.add_experience(input_signal, pa_output, target_signal, metrics)
            
            # Update if needed
            if self.learner.should_update():
                loss = self.learner.update_model()
                print(f"Online update: loss={loss:.6f}")
        
        # Get current DPD output
        with torch.no_grad():
            dpd_output = self.dpd_model(input_signal)
        
        return dpd_output
    
    def get_adaptation_stats(self) -> Dict:
        """Get adaptation statistics"""
        if self.adaptation_mode == 'online' and self.learner:
            return self.learner.get_performance_stats()
        return {}



