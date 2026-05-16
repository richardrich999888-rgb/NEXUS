"""
Train Joint Dpd

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import DataLoader, TensorDataset
import numpy as np
import json
from tqdm import tqdm

class DPDTrainer:
    """Trainer for Joint Beamforming and DPD"""
    
    def __init__(self, config, model, pa_model):
        self.config = config
        self.model = model
        self.pa_model = pa_model
        self.device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
        
        # Move models to device
        self.model.to(self.device)
        self.pa_model.to(self.device)
        
        # Optimizer
        self.optimizer = optim.Adam(
            self.model.parameters(),
            lr=config['training']['learning_rate']
        )
        
        # Loss function
        self.criterion = nn.MSELoss()
        
        # Learning rate scheduler
        self.scheduler = optim.lr_scheduler.ReduceLROnPlateau(
            self.optimizer, mode='min', patience=10, factor=0.5
        )
        
        # Training history
        self.history = {
            'train_loss': [],
            'val_loss': [],
            'evm': [],
            'nmse': []
        }
    
    def generate_dataset(self, num_samples):
        """Generate training dataset"""
        from utils.signal_generation import SignalGenerator
        
        print(f"Generating {num_samples} training samples...")
        
        signals = []
        channels = []
        
        for _ in tqdm(range(num_samples)):
            # Generate OFDM signal
            signal = SignalGenerator.generate_ofdm_signal(
                num_symbols=14,
                modulation=self.config['simulation']['modulation']
            )
            signals.append(signal)
            
            # Generate random channel (simplified)
            channel = torch.randn(64, dtype=torch.cfloat) * (1/np.sqrt(2))
            channels.append(channel)
        
        # Create dataset
        signals_tensor = torch.stack(signals)
        channels_tensor = torch.stack(channels)
        
        return TensorDataset(signals_tensor, channels_tensor)
    
    def train_epoch(self, dataloader):
        """Train for one epoch"""
        self.model.train()
        total_loss = 0
        
        for batch_idx, (signals, channels) in enumerate(dataloader):
            signals = signals.to(self.device)
            channels = channels.to(self.device)
            
            # Forward pass through joint model
            outputs = self.model(channels.unsqueeze(1), signals)
            predistorted = outputs['predistorted']
            
            # Apply PA nonlinearity
            pa_output = self.pa_model(predistorted)
            
            # Calculate loss (want PA output ≈ original signal)
            loss = self.criterion(pa_output.abs(), signals.abs())
            
            # Backward pass
            self.optimizer.zero_grad()
            loss.backward()
            
            # Gradient clipping for stability
            torch.nn.utils.clip_grad_norm_(self.model.parameters(), max_norm=1.0)
            
            self.optimizer.step()
            
            total_loss += loss.item()
            
            if batch_idx % 50 == 0:
                print(f'  Batch {batch_idx}, Loss: {loss.item():.6f}')
                
        return total_loss / len(dataloader)
    
    def validate(self, dataloader):
        """Validate model performance"""
        self.model.eval()
        total_loss = 0
        evm_values = []
        nmse_values = []
        
        from utils.metrics import DPDEvaluator
        
        with torch.no_grad():
            for signals, channels in dataloader:
                signals = signals.to(self.device)
                channels = channels.to(self.device)
                
                # Forward pass
                outputs = self.model(channels.unsqueeze(1), signals)
                predistorted = outputs['predistorted']
                
                # Apply PA
                pa_output = self.pa_model(predistorted)
                
                # Calculate metrics
                loss = self.criterion(pa_output.abs(), signals.abs())
                total_loss += loss.item()
                
                # Calculate EVM and NMSE
                evm = DPDEvaluator.calculate_evm(signals, pa_output)
                nmse = DPDEvaluator.calculate_nmse(signals, pa_output)
                
                evm_values.append(evm)
                nmse_values.append(nmse)
        
        avg_evm = np.mean(evm_values)
        avg_nmse = np.mean(nmse_values)
        
        return total_loss / len(dataloader), avg_evm, avg_nmse
    
    def train(self, num_epochs=None):
        """Main training loop"""
        if num_epochs is None:
            num_epochs = self.config['training']['num_epochs']
        
        # Generate datasets
        train_dataset = self.generate_dataset(
            self.config['training']['train_samples']
        )
        test_dataset = self.generate_dataset(
            self.config['training']['test_samples']
        )
        
        train_loader = DataLoader(
            train_dataset,
            batch_size=self.config['training']['batch_size'],
            shuffle=True
        )
        test_loader = DataLoader(
            test_dataset,
            batch_size=self.config['training']['batch_size']
        )
        
        print(f"Training samples: {len(train_dataset)}")
        print(f"Test samples: {len(test_dataset)}")
        
        # Training loop
        best_loss = float('inf')
        for epoch in range(num_epochs):
            train_loss = self.train_epoch(train_loader)
            val_loss, val_evm, val_nmse = self.validate(test_loader)
            
            # Update learning rate
            self.scheduler.step(val_loss)
            
            # Save history
            self.history['train_loss'].append(train_loss)
            self.history['val_loss'].append(val_loss)
            self.history['evm'].append(val_evm)
            self.history['nmse'].append(val_nmse)
            
            print(f'Epoch {epoch+1}/{num_epochs}:')
            print(f'  Train Loss: {train_loss:.6f}')
            print(f'  Val Loss: {val_loss:.6f}')
            print(f'  EVM: {val_evm:.2f}%')
            print(f'  NMSE: {val_nmse:.2f} dB')
            print(f'  LR: {self.optimizer.param_groups[0]["lr"]:.2e}')
            
            # Save best model
            if val_loss < best_loss:
                best_loss = val_loss
                torch.save({
                    'epoch': epoch,
                    'model_state_dict': self.model.state_dict(),
                    'optimizer_state_dict': self.optimizer.state_dict(),
                    'loss': best_loss,
                    'config': self.config
                }, 'best_dpd_model.pth')
                print(f'  Saved best model with loss: {best_loss:.6f}')
            
            print('-' * 50)
        
        # Save training history
        with open('training_history.json', 'w') as f:
            json.dump(self.history, f, indent=2)
        
        return self.history

class JointOptimizationTrainer(DPDTrainer):
    """Trainer for joint beamforming + DPD optimization"""
    
    def __init__(self, config, joint_model, pa_model, beamformer):
        super().__init__(config, joint_model, pa_model)
        self.beamformer = beamformer
        self.beamformer.to(self.device)
        
        # Add beamformer parameters to optimizer
        self.optimizer = optim.Adam(
            list(self.model.parameters()) + list(self.beamformer.parameters()),
            lr=config['training']['learning_rate']
        )
    
    def train_epoch(self, dataloader):
        """Train with joint optimization"""
        self.model.train()
        self.beamformer.train()
        total_loss = 0
        
        for batch_idx, (signals, channels) in enumerate(dataloader):
            signals = signals.to(self.device)
            channels = channels.to(self.device)
            
            # Get beamforming weights
            beam_weights = self.beamformer.compute_beamweights(
                channels.unsqueeze(1)
            )
            
            # Forward pass through joint model
            outputs = self.model(channels, signals)
            predistorted = outputs['predistorted']
            
            # Apply beamforming
            beamformed = torch.einsum('bs,ba->bas', 
                                     signals.unsqueeze(1), 
                                     beam_weights.conj())
            
            # Apply PA to beamformed signal
            pa_output = self.pa_model(beamformed)
            
            # Complex loss: signal fidelity + power efficiency
            signal_loss = self.criterion(pa_output.abs(), beamformed.abs())
            
            # Power efficiency regularization
            input_power = torch.mean(torch.abs(predistorted)**2)
            output_power = torch.mean(torch.abs(pa_output)**2)
            efficiency = output_power / (input_power + 1e-8)
            efficiency_loss = -torch.log(efficiency + 1e-8)  # Maximize efficiency
            
            # Total loss
            loss = signal_loss + 0.1 * efficiency_loss
            
            # Backward pass
            self.optimizer.zero_grad()
            loss.backward()
            torch.nn.utils.clip_grad_norm_(
                list(self.model.parameters()) + list(self.beamformer.parameters()),
                max_norm=1.0
            )
            self.optimizer.step()
            
            total_loss += loss.item()
            
            if batch_idx % 50 == 0:
                print(f'  Batch {batch_idx}, Loss: {loss.item():.6f}, '
                      f'Efficiency: {efficiency.item():.3f}')
                
        return total_loss / len(dataloader)

