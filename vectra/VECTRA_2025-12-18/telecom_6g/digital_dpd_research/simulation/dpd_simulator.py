"""
Dpd Simulator

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import numpy as np
import matplotlib.pyplot as plt
from scipy import signal as scipy_signal

class DPDSimulator:
    """Complete DPD simulation environment"""
    
    def __init__(self, config):
        self.config = config
        self.device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
        
        # Initialize models
        from models.pa_behavioral import PAArrayModel
        from models.neural_dpd import BeamAwareDPD
        from beamformers.tt_beamformer import TTBeamformer
        
        # PA model
        pa_type = config['pa_model']['type']
        pa_kwargs = {k: v for k, v in config['pa_model'].items() if k != 'type'}
        self.pa_model = PAArrayModel(
            num_antennas=config['system']['num_antennas'],
            model_type=pa_type,
            **pa_kwargs
        ).to(self.device)
        
        # DPD model
        self.dpd_model = BeamAwareDPD(
            num_clusters=config['system']['num_clusters'],
            memory_depth=config['neural_dpd']['memory_depth'],
            hidden_dims=config['neural_dpd']['hidden_layers']
        ).to(self.device)
        
        # Beamformer
        self.beamformer = TTBeamformer(
            num_ant=config['system']['num_antennas'],
            num_users=1
        ).to(self.device)
        
        # Signal generator
        from utils.signal_generation import SignalGenerator
        self.signal_gen = SignalGenerator()
        
        # Metrics
        from utils.metrics import DPDEvaluator
        self.evaluator = DPDEvaluator()
        
        # Results storage
        self.results = {}
    
    def generate_test_signals(self, num_signals=10, signal_length=10000):
        """Generate test signals for simulation"""
        signals = []
        
        for i in range(num_signals):
            # Generate OFDM signal
            sig = self.signal_gen.generate_ofdm_signal(
                num_symbols=14,
                modulation=self.config['simulation']['modulation'],
                papr_reduction=True
            )
            
            # Trim to desired length
            if len(sig) > signal_length:
                sig = sig[:signal_length]
            else:
                # Pad if necessary
                pad_len = signal_length - len(sig)
                sig = torch.cat([sig, torch.zeros(pad_len, dtype=torch.cfloat)])
            
            signals.append(sig)
        
        return torch.stack(signals)
    
    def run_simulation(self, test_signals=None):
        """Run complete DPD simulation"""
        if test_signals is None:
            test_signals = self.generate_test_signals()
        
        print("Running DPD simulation...")
        print(f"Number of test signals: {len(test_signals)}")
        print(f"Signal length: {len(test_signals[0])}")
        
        # Move to device
        test_signals = test_signals.to(self.device)
        
        # Generate random beam weights for testing
        beam_weights = torch.randn(self.config['system']['num_antennas'], 
                                  dtype=torch.cfloat, device=self.device)
        beam_weights = beam_weights / torch.norm(beam_weights)
        
        # Prepare storage for results
        evm_without_dpd = []
        evm_with_dpd = []
        aclr_without_dpd = []
        aclr_with_dpd = []
        spectra_without_dpd = []
        spectra_with_dpd = []
        
        with torch.no_grad():
            for i, sig in enumerate(test_signals):
                print(f"Processing signal {i+1}/{len(test_signals)}...")
                
                # Repeat signal for all antennas
                sig_antennas = sig.unsqueeze(0).repeat(
                    self.config['system']['num_antennas'], 1
                ).T.unsqueeze(0)  # [1, length, antennas]
                
                # Convert to I/Q format for DPD
                sig_iq = torch.stack([sig_antennas.real, sig_antennas.imag], dim=-1)
                
                # 1. Without DPD
                pa_output_no_dpd = self.pa_model(sig_iq)
                pa_output_no_dpd_complex = torch.complex(
                    pa_output_no_dpd[..., 0],
                    pa_output_no_dpd[..., 1]
                )
                
                # 2. With DPD
                # Apply DPD
                dpd_output = self.dpd_model(
                    sig_iq,
                    beam_weights=beam_weights,
                    use_cached=False
                )
                
                # Apply PA to predistorted signal
                pa_output_with_dpd = self.pa_model(dpd_output)
                pa_output_with_dpd_complex = torch.complex(
                    pa_output_with_dpd[..., 0],
                    pa_output_with_dpd[..., 1]
                )
                
                # Calculate metrics for antenna 0 (representative)
                evm_no_dpd = self.evaluator.calculate_evm(
                    sig_antennas[0, :, 0],
                    pa_output_no_dpd_complex[0, :, 0]
                )
                evm_with_dpd_val = self.evaluator.calculate_evm(
                    sig_antennas[0, :, 0],
                    pa_output_with_dpd_complex[0, :, 0]
                )
                
                evm_without_dpd.append(evm_no_dpd)
                evm_with_dpd.append(evm_with_dpd_val)
                
                # Calculate ACLR
                aclr_no_dpd = self.evaluator.calculate_aclr(
                    pa_output_no_dpd_complex[0, :, 0].cpu(),
                    self.config['system']['sample_rate'],
                    self.config['system']['bandwidth']
                )
                aclr_with_dpd_val = self.evaluator.calculate_aclr(
                    pa_output_with_dpd_complex[0, :, 0].cpu(),
                    self.config['system']['sample_rate'],
                    self.config['system']['bandwidth']
                )
                
                aclr_without_dpd.append(aclr_no_dpd)
                aclr_with_dpd.append(aclr_with_dpd_val)
                
                # Store spectra for plotting
                if i == 0:  # Just first signal for plotting
                    spectra_without_dpd = pa_output_no_dpd_complex[0, :, 0].cpu().numpy()
                    spectra_with_dpd = pa_output_with_dpd_complex[0, :, 0].cpu().numpy()
        
        # Compile results
        self.results = {
            'evm': {
                'without_dpd': np.mean(evm_without_dpd),
                'with_dpd': np.mean(evm_with_dpd),
                'improvement': np.mean(evm_without_dpd) - np.mean(evm_with_dpd)
            },
            'aclr': {
                'without_dpd': np.mean(aclr_without_dpd),
                'with_dpd': np.mean(aclr_with_dpd),
                'improvement': np.mean(aclr_without_dpd) - np.mean(aclr_with_dpd)
            },
            'spectra': {
                'without_dpd': spectra_without_dpd,
                'with_dpd': spectra_with_dpd
            },
            'model_size_kb': self.dpd_model.get_model_size(
                quantized=self.config['deployment']['quantize']
            )
        }
        
        return self.results
    
    def plot_results(self, save_path='dpd_results.png'):
        """Plot simulation results"""
        fig, axes = plt.subplots(2, 2, figsize=(12, 10))
        
        # 1. EVM comparison
        axes[0, 0].bar(['Without DPD', 'With DPD'],
                      [self.results['evm']['without_dpd'],
                       self.results['evm']['with_dpd']])
        axes[0, 0].set_ylabel('EVM (%)')
        axes[0, 0].set_title(f'EVM Improvement: {self.results["evm"]["improvement"]:.2f}%')
        axes[0, 0].grid(True, alpha=0.3)
        
        # 2. ACLR comparison
        axes[0, 1].bar(['Without DPD', 'With DPD'],
                      [self.results['aclr']['without_dpd'],
                       self.results['aclr']['with_dpd']])
        axes[0, 1].set_ylabel('ACLR (dBc)')
        axes[0, 1].set_title(f'ACLR Improvement: {self.results["aclr"]["improvement"]:.2f} dB')
        axes[0, 1].grid(True, alpha=0.3)
        
        # 3. Power spectral density
        if 'spectra' in self.results:
            f, Pxx_no_dpd = scipy_signal.welch(
                self.results['spectra']['without_dpd'],
                fs=self.config['system']['sample_rate']
            )
            f, Pxx_with_dpd = scipy_signal.welch(
                self.results['spectra']['with_dpd'],
                fs=self.config['system']['sample_rate']
            )
            
            axes[1, 0].plot(f, 10*np.log10(Pxx_no_dpd), label='Without DPD')
            axes[1, 0].plot(f, 10*np.log10(Pxx_with_dpd), label='With DPD')
            axes[1, 0].set_xlabel('Frequency (Hz)')
            axes[1, 0].set_ylabel('PSD (dB/Hz)')
            axes[1, 0].set_title('Power Spectral Density')
            axes[1, 0].legend()
            axes[1, 0].grid(True, alpha=0.3)
        
        # 4. Model size vs performance
        model_size = self.results.get('model_size_kb', 0)
        axes[1, 1].scatter(model_size, self.results['evm']['with_dpd'],
                          s=200, alpha=0.7)
        axes[1, 1].set_xlabel('Model Size (KB)')
        axes[1, 1].set_ylabel('EVM with DPD (%)')
        axes[1, 1].set_title(f'Model Size: {model_size:.1f} KB')
        axes[1, 1].grid(True, alpha=0.3)
        
        plt.tight_layout()
        plt.savefig(save_path, dpi=150, bbox_inches='tight')
        plt.show()
        
        return fig
    
    def generate_report(self, save_path='dpd_simulation_report.txt'):
        """Generate comprehensive simulation report"""
        report = f"""
        ================================================
        DPD SIMULATION REPORT
        ================================================
        
        Configuration:
        ---------------
        • Antennas: {self.config['system']['num_antennas']}
        • Clusters: {self.config['system']['num_clusters']}
        • PA Model: {self.config['pa_model']['type']}
        • DPD Memory: {self.config['neural_dpd']['memory_depth']}
        
        Performance Results:
        --------------------
        Error Vector Magnitude (EVM):
        • Without DPD: {self.results['evm']['without_dpd']:.2f}%
        • With DPD: {self.results['evm']['with_dpd']:.2f}%
        • Improvement: {self.results['evm']['improvement']:.2f}%
        
        Adjacent Channel Leakage Ratio (ACLR):
        • Without DPD: {self.results['aclr']['without_dpd']:.2f} dBc
        • With DPD: {self.results['aclr']['with_dpd']:.2f} dBc
        • Improvement: {self.results['aclr']['improvement']:.2f} dB
        
        Model Characteristics:
        ----------------------
        • Model Size: {self.results.get('model_size_kb', 'N/A'):.1f} KB
        • Quantization: {self.config['deployment']['quantize']}
        • Target Precision: {self.config['deployment']['target_precision']}
        
        Interpretation:
        ----------------
        """
        
        # Add interpretation based on results
        if self.results['evm']['improvement'] > 1.0:
            report += "• Significant EVM improvement (>1%) - enables higher order modulation\n"
        
        if self.results['aclr']['improvement'] > 3.0:
            report += "• Excellent spectral regression suppression (>3 dB improvement)\n"
        
        if self.results.get('model_size_kb', 100) < 10:
            report += f"• Model size ({self.results['model_size_kb']:.1f} KB) suitable for embedded deployment\n"
        
        report += "\n================================================\n"
        
        # Save report
        with open(save_path, 'w') as f:
            f.write(report)
        
        print(report)
        return report

