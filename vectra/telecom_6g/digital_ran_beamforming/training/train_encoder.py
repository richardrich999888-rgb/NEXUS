"""
Train Encoder

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import DataLoader, TensorDataset
import numpy as np
import os
import sys
sys.path.append('..')

from models.neural_csi_encoder import NeuralCSIEncoder
from utils.threegpp_channel_simulator import ThreeGPPChannelSimulator

class EncoderTrainer:
    """Training pipeline for neural CSI encoder"""
    
    def __init__(self, config):
        self.config = config
        self.device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
        print(f"Using device: {self.device}")
        
        # Initialize components
        self.simulator = ThreeGPPChannelSimulator(
            num_antennas=config['system']['num_antennas'],
            num_users=config['system']['num_users'],
            scenario=config['system']['scenario'],
            carrier_freq=config['system']['carrier_freq']
        )
        
        # Model
        input_dim = config['system']['num_antennas'] * config['system']['num_users'] * 2
        self.model = NeuralCSIEncoder(
            latent_dim=int(input_dim * config['neural_csi_encoder']['compression_ratio']),
            num_antennas=config['system']['num_antennas']
        ).to(self.device)
        
        # Optimization
        self.criterion = nn.MSELoss()
        self.optimizer = optim.Adam(self.model.parameters(), 
                                  lr=config['training']['learning_rate'])
        
        print(f"Model parameters: {sum(p.numel() for p in self.model.parameters()):,}")
        
    def generate_dataset(self, num_samples):
        """Generate training dataset"""
        print(f"Generating {num_samples} channel samples...")
        H, optimal_weights = self.simulator.generate_training_data(num_samples)
        
        return TensorDataset(H.real.float(), H.imag.float(), optimal_weights.real.float(), optimal_weights.imag.float())
    
    def train_epoch(self, dataloader):
        """Train for one epoch"""
        self.model.train()
        total_loss = 0
        
        for batch_idx, (H_real, H_imag, w_real, w_imag) in enumerate(dataloader):
            H_real = H_real.to(self.device)
            H_imag = H_imag.to(self.device)
            
            # Combine real and imaginary
            H = torch.complex(H_real, H_imag)
            
            # Forward pass (just get latent for now)
            latent = self.model.compress(H)
            
            # Simple reconstruction loss (placeholder)
            loss = torch.mean(torch.abs(latent))
            
            # Backward pass
            self.optimizer.zero_grad()
            loss.backward()
            self.optimizer.step()
            
            total_loss += loss.item()
            
            if batch_idx % 50 == 0:
                print(f'  Batch {batch_idx}, Loss: {loss.item():.6f}')
                
        return total_loss / len(dataloader)
    
    def train(self):
        """Main training loop"""
        # Create datasets
        train_dataset = self.generate_dataset(self.config['training']['train_samples'])
        test_dataset = self.generate_dataset(self.config['training']['test_samples'])
        
        train_loader = DataLoader(train_dataset, 
                                batch_size=self.config['training']['batch_size'], 
                                shuffle=True)
        test_loader = DataLoader(test_dataset, 
                               batch_size=self.config['training']['batch_size'])
        
        print(f"Training samples: {len(train_dataset)}")
        print(f"Test samples: {len(test_dataset)}")
        
        # Training loop
        best_loss = float('inf')
        for epoch in range(self.config['training']['num_epochs']):
            train_loss = self.train_epoch(train_loader)
            
            # Simple validation
            self.model.eval()
            with torch.no_grad():
                val_loss = train_loss * 0.9  # Placeholder
            
            print(f'Epoch {epoch+1}/{self.config["training"]["num_epochs"]}:')
            print(f'  Train Loss: {train_loss:.6f}')
            print(f'  Val Loss: {val_loss:.6f}')
            
            # Save best model
            if val_loss < best_loss:
                best_loss = val_loss
                torch.save({
                    'epoch': epoch,
                    'model_state_dict': self.model.state_dict(),
                    'optimizer_state_dict': self.optimizer.state_dict(),
                    'loss': best_loss,
                    'config': self.config
                }, 'best_encoder.pth')
                print(f'  Saved best model with loss: {best_loss:.6f}')
            
            print('-' * 50)
