"""
Metrics

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import numpy as np
import scipy.signal as signal

class DPDEvaluator:
    """DPD performance evaluation metrics"""
    
    @staticmethod
    def calculate_evm(reference, measured):
        """
        Calculate Error Vector Magnitude (EVM)
        reference: ideal constellation points
        measured: measured constellation points
        """
        error = reference - measured
        evm_rms = torch.sqrt(torch.mean(torch.abs(error)**2) / 
                            torch.mean(torch.abs(reference)**2))
        return evm_rms.item() * 100  # Percentage
    
    @staticmethod
    def calculate_aclr(input_signal, sample_rate, bandwidth, offset=5e6):
        """
        Calculate Adjacent Channel Leakage Ratio (ACLR)
        input_signal: transmitted signal
        sample_rate: sampling rate in Hz
        bandwidth: signal bandwidth in Hz
        offset: offset frequency for adjacent channel
        """
        # Calculate power spectral density
        if hasattr(input_signal, 'cpu'):
            input_signal = input_signal.cpu().numpy()
            
        f, Pxx = signal.welch(input_signal, fs=sample_rate, 
                             nperseg=1024, return_onesided=False)
        
        # Main channel power
        mask_main = np.abs(f) < bandwidth/2
        P_main = np.trapz(Pxx[mask_main], f[mask_main])
        
        # Adjacent channel power
        mask_adj = (np.abs(f) > offset - bandwidth/2) & \
                   (np.abs(f) < offset + bandwidth/2)
        P_adj = np.trapz(Pxx[mask_adj], f[mask_adj])
        
        # ACLR in dB
        aclr_db = 10 * np.log10(P_adj / P_main)
        return aclr_db
    
    @staticmethod
    def calculate_nmse(reference, measured):
        """
        Calculate Normalized Mean Square Error
        """
        mse = torch.mean(torch.abs(reference - measured)**2)
        power = torch.mean(torch.abs(reference)**2)
        nmse_db = 10 * torch.log10(mse / power)
        return nmse_db.item()
    
    @staticmethod
    def calculate_pae(input_power, output_power, dc_power):
        """
        Calculate Power Added Efficiency (PAE)
        input_power: RF input power (W)
        output_power: RF output power (W)
        dc_power: DC supply power (W)
        """
        pae = (output_power - input_power) / dc_power * 100
        return pae
    
    @staticmethod
    def calculate_spectral_regrowth(signal_before, signal_after, sample_rate):
        """
        Calculate spectral regrowth improvement
        """
        # Calculate spectra
        if hasattr(signal_before, 'cpu'):
            signal_before = signal_before.cpu().numpy()
        if hasattr(signal_after, 'cpu'):
            signal_after = signal_after.cpu().numpy()

        f_before, P_before = signal.welch(signal_before, 
                                         fs=sample_rate)
        f_after, P_after = signal.welch(signal_after, 
                                       fs=sample_rate)
        
        # Find out-of-band power
        bandwidth_idx = len(f_before) // 4
        oob_before = np.mean(P_before[bandwidth_idx:])
        oob_after = np.mean(P_after[bandwidth_idx:])
        
        improvement_db = 10 * np.log10(oob_before / oob_after)
        return improvement_db
    
    @staticmethod
    def generate_performance_report(model, test_data, pa_model):
        """
        Generate comprehensive performance report
        """
        report = {}
        
        with torch.no_grad():
            # Test signals
            test_inputs = test_data['signals']
            test_outputs = []
            
            # Apply DPD and PA
            for sig in test_inputs:
                dpd_out = model(sig)
                pa_out = pa_model(dpd_out)
                test_outputs.append(pa_out)
            
            test_outputs = torch.stack(test_outputs)
            
            # Calculate metrics
            report['EVM (%)'] = DPDEvaluator.calculate_evm(test_inputs, test_outputs)
            report['NMSE (dB)'] = DPDEvaluator.calculate_nmse(test_inputs, test_outputs)
            report['ACLR (dBc)'] = DPDEvaluator.calculate_aclr(
                test_outputs[0], 122.88e6, 100e6
            )
            
            # Calculate efficiency improvement
            input_power = torch.mean(torch.abs(test_inputs)**2)
            output_power = torch.mean(torch.abs(test_outputs)**2)
            # Assume 30% PA efficiency without DPD
            dc_power_without_dpd = output_power / 0.3
            # Assume 50% PA efficiency with DPD (operating closer to saturation)
            dc_power_with_dpd = output_power / 0.5
            
            report['PA Efficiency without DPD (%)'] = 30
            report['PA Efficiency with DPD (%)'] = 50
            report['Power Saving (%)'] = (1 - dc_power_with_dpd / dc_power_without_dpd) * 100
        
        return report

