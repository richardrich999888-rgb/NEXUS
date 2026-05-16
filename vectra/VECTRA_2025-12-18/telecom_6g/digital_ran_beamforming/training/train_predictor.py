"""
Train Predictor

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

from models.sparse_beam_mask_generator import SparseBeamMaskGenerator
from models.neural_csi_encoder import NeuralCSIEncoder
from utils.threegpp_channel_simulator import ThreeGPPChannelSimulator
from beamformers.baseline_svd import SVDBaseline

class PredictorTrainer:
    """Training pipeline for sparse beam predictor"""
    
    def __init__(self, config):
        self.config = config
        self.device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
        print(f"Using device: {self.device}")
        
        # Initialize components
        self.simulator = ThreeGPPChannelSimulator(
            num_antennas=config['system']['num_antennas'],
            num_users=config['system']['num_users'],
            scenario=config['system']['scenario']
        )
        
        self.baseline = SVDBaseline(
            num_antennas=config['system']['num_antennas'],
            num_users=config['system']['num_users']
        )
        
        # Load pretrained encoder
        self.encoder = NeuralCSIEncoder(
            latent_dim=int(config['system']['num_antennas'] * config['system']['num_users'] * 2 * 
                         config['neural_csi_encoder']['compression_ratio']),
            num_antennas=config['system']['num_antennas']
        ).to(self.device)
        
        if os.path.exists('best_encoder.pth'):
            checkpoint = torch.load('best_encoder.pth', map_location=self.device)
            self.encoder.load_state_dict(checkpoint['model_state_dict'])
            print("Loaded pretrained encoder")
        else:
            print("Warning: No pretrained encoder found")
        
        # Predictor model
        input_dim = int(config['system']['num_antennas'] * config['system']['num_users'] * 2 * 
                       config['neural_csi_encoder']['compression_ratio'])
        
        self.model = SparseBeamMaskGenerator(
            latent_dim=input_dim,
            num_antennas=config['system']['num_antennas'],
            hidden=256,
            topk=int(config['system']['num_antennas'] * config['sparse_beam_mask']['sparsity_ratio'])
        ).to(self.device)
        
        # Optimization
        self.criterion = nn.BCELoss()
        self.optimizer = optim.Adam(self.model.parameters(), 
                                  lr=config['training']['learning_rate'])
        
        print(f"Predictor parameters: {sum(p.numel() for p in self.model.parameters()):,}")
        
    def generate_beam_labels(self, H):
        """Generate optimal beam mask labels"""
        batch_size = H.shape[0]
        optimal_weights = self.baseline.compute_beamweights(H)
        
        # Create beam mask based on weight magnitudes
        weight_mags = torch.abs(optimal_weights)
        
        # Top-k selection
        k = int(self.config['system']['num_antennas'] * 
                self.config['sparse_beam_mask']['sparsity_ratio'])
        
        beam_masks = torch.zeros_like(weight_mags)
        topk_vals, topk_indices = torch.topk(weight_mags, k, dim=-1)
        
        for i in range(batch_size):
            beam_masks[i].scatter_(-1, topk_indices[i], 1.0)
            
        return beam_masks
    
    def generate_dataset(self, num_samples):
        """Generate training dataset"""
        print(f"Generating {num_samples} beam prediction samples...")
        
        # Generate channels
        H = self.simulator.generate_cdl_channel(num_samples)
        
        # Generate beam mask labels
        beam_masks = self.generate_beam_labels(H)
        
        # Compress channels
        with torch.no_grad():
            compressed = self.encoder.compress(H)
        
        return TensorDataset(compressed.float(), beam_masks.float())
    
    def train_epoch(self, dataloader):
        """Train for one epoch"""
        self.model.train()
        total_loss = 0
        total_accuracy = 0
        
        for batch_idx, (compressed, beam_masks) in enumerate(dataloader):
            compressed = compressed.to(self.device)
            beam_masks = beam_masks.to(self.device)
            
            self.optimizer.zero_grad()
            
            # Forward pass
            pred_masks, pred_probs = self.model(compressed, hard=False)
            
            # Compute loss
            loss = self.criterion(pred_probs, beam_masks)
            
            # Backward pass
            loss.backward()
            self.optimizer.step()
            
            total_loss += loss.item()
            
            # Calculate accuracy
            pred_hard = (pred_probs > 0.5).float()
            accuracy = (pred_hard == beam_masks).float().mean()
            total_accuracy += accuracy.item()
            
            if batch_idx % 50 == 0:
                sparsity = 1.0 - pred_hard.float().mean()
                print(f'  Batch {batch_idx}, Loss: {loss.item():.6f}, '
                      f'Accuracy: {accuracy.item():.4f}, Sparsity: {sparsity:.3f}')
                
        return total_loss / len(dataloader), total_accuracy / len(dataloader)
    
    def validate(self, dataloader):
        """Validate predictor performance"""
        self.model.eval()
        total_loss = 0
        total_accuracy = 0
        sparsity_levels = []
        
        with torch.no_grad():
            for compressed, beam_masks in dataloader:
                compressed = compressed.to(self.device)
                beam_masks = beam_masks.to(self.device)
                
                pred_masks, pred_probs = self.model(compressed, hard=True)
                loss = self.criterion(pred_probs[0], beam_masks)
                total_loss += loss.item()
                
                # Calculate accuracy
                pred_hard = (pred_probs[0] > 0.5).float()
                accuracy = (pred_hard == beam_masks).float().mean()
                total_accuracy += accuracy.item()
                
                # Calculate sparsity
                sparsity = 1.0 - pred_hard.float().mean()
                sparsity_levels.append(sparsity.item())
        
        avg_sparsity = np.mean(sparsity_levels)
        return (total_loss / len(dataloader), 
                total_accuracy / len(dataloader), 
                avg_sparsity)
    
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
        
        # Training
        best_accuracy = 0
        for epoch in range(self.config['training']['num_epochs']):
            train_loss, train_acc = self.train_epoch(train_loader)
            val_loss, val_acc, val_sparsity = self.validate(test_loader)
            
            print(f'Epoch {epoch+1}/{self.config["training"]["num_epochs"]}:')
            print(f'  Train Loss: {train_loss:.6f}, Train Acc: {train_acc:.4f}')
            print(f'  Val Loss: {val_loss:.6f}, Val Acc: {val_acc:.4f}')
            print(f'  Val Sparsity: {val_sparsity:.3f}')
            
            # Save best model
            if val_acc > best_accuracy:
                best_accuracy = val_acc
                torch.save({
                    'epoch': epoch,
                    'model_state_dict': self.model.state_dict(),
                    'optimizer_state_dict': self.optimizer.state_dict(),
                    'accuracy': best_accuracy,
                    'config': self.config
                }, 'best_predictor.pth')
                print(f'  Saved best model with accuracy: {best_accuracy:.4f}')
            
            print('-' * 50)
