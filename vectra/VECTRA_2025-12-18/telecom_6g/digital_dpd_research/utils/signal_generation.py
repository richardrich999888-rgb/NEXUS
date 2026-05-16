"""
Signal Generation

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import numpy as np

class SignalGenerator:
    """Generate test signals for DPD training and evaluation"""
    
    @staticmethod
    def generate_ofdm_signal(num_symbols=1000, num_subcarriers=3300, 
                            modulation='64qam', papr_reduction=True):
        """
        Generate OFDM signal for DPD training
        """
        # Generate random data
        if modulation == '64qam':
            constellation = np.array([-7, -5, -3, -1, 1, 3, 5, 7]) / np.sqrt(42)
            data = np.random.choice(constellation, 
                                   size=(num_symbols, num_subcarriers)) + \
                   1j * np.random.choice(constellation,
                                        size=(num_symbols, num_subcarriers))
        elif modulation == '16qam':
            constellation = np.array([-3, -1, 1, 3]) / np.sqrt(10)
            data = np.random.choice(constellation,
                                   size=(num_symbols, num_subcarriers)) + \
                   1j * np.random.choice(constellation,
                                        size=(num_symbols, num_subcarriers))
        else:  # QPSK
            constellation = np.array([-1, 1]) / np.sqrt(2)
            data = np.random.choice(constellation,
                                   size=(num_symbols, num_subcarriers)) + \
                   1j * np.random.choice(constellation,
                                        size=(num_symbols, num_subcarriers))
        
        # Apply IFFT
        time_signal = np.fft.ifft(data, axis=1)
        
        # Add cyclic prefix
        cp_len = num_subcarriers // 8
        time_signal_cp = np.concatenate([
            time_signal[:, -cp_len:],
            time_signal
        ], axis=1)
        
        # Flatten
        signal = time_signal_cp.flatten()
        
        # PAPR reduction (optional)
        if papr_reduction:
            signal = SignalGenerator.apply_clipping(signal, clip_ratio=3.0)
        
        return torch.tensor(signal, dtype=torch.cfloat)
    
    @staticmethod
    def apply_clipping(signal, clip_ratio=3.0):
        """Apply clipping for PAPR reduction"""
        magnitude = np.abs(signal)
        clip_level = clip_ratio * np.std(magnitude)
        
        clipped_magnitude = np.minimum(magnitude, clip_level)
        phase = np.angle(signal)
        
        return clipped_magnitude * np.exp(1j * phase)
    
    @staticmethod
    def generate_multi_tone(frequencies, amplitudes, num_samples=10000):
        """Generate multi-tone signal"""
        t = np.arange(num_samples) / num_samples
        signal = np.zeros(num_samples, dtype=complex)
        
        for freq, amp in zip(frequencies, amplitudes):
            signal += amp * np.exp(1j * 2 * np.pi * freq * t)
        
        return torch.tensor(signal, dtype=torch.cfloat)
    
    @staticmethod
    def generate_awgn(signal, snr_db):
        """Add AWGN to signal"""
        signal_power = torch.mean(torch.abs(signal)**2)
        noise_power = signal_power / (10**(snr_db/10))
        
        noise_real = torch.randn_like(signal.real) * np.sqrt(noise_power/2)
        noise_imag = torch.randn_like(signal.imag) * np.sqrt(noise_power/2)
        
        noisy_signal = torch.complex(signal.real + noise_real,
                                    signal.imag + noise_imag)
        return noisy_signal

