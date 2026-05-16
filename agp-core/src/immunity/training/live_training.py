"""
Live Training Protocol - Adaptive learning through threat exposure.
"""

import torch
from typing import List, Tuple, Dict
import numpy as np


class LiveTrainingProtocol:
    """Train adaptive immunity through live threat exposure."""
    
    def __init__(self, immune_system):
        self.ais = immune_system
        
    def train(
        self,
        mixed_dataset: List[Tuple[torch.Tensor, bool, str]],
        num_epochs: int = 10,
        clonal_interval: int = 50
    ) -> List[Dict]:
        """
        Train through live exposure.
        
        Args:
            mixed_dataset: List of (example, is_threat, threat_type)
            num_epochs: Number of training epochs
            clonal_interval: Clonal selection frequency
        
        Returns:
            metrics: Training metrics by epoch
        """
        print("\n" + "="*60)
        print("PHASE 3: LIVE TRAINING")
        print("="*60)
        
        epoch_metrics = []
        
        for epoch in range(num_epochs):
            tp, fp, tn, fn = 0, 0, 0, 0
            times = []
            
            for i, (ex, is_threat, _) in enumerate(mixed_dataset):
                _, diag = self.ais(ex, True, True)
                detected = diag['threat_detected']
                
                if detected and is_threat:
                    tp += 1
                elif detected and not is_threat:
                    fp += 1
                elif not detected and not is_threat:
                    tn += 1
                else:
                    fn += 1
                
                times.append(diag['response_time_ms'])
                
                if (i + 1) % clonal_interval == 0 and self.ais.adaptive:
                    self.ais.adaptive.antibody_pool.clonal_selection()
            
            total = len(mixed_dataset)
            precision = tp / (tp + fp + 1e-8)
            recall = tp / (tp + fn + 1e-8)
            f1 = 2 * precision * recall / (precision + recall + 1e-8)
            
            metrics = {
                'epoch': epoch + 1,
                'accuracy': (tp + tn) / total,
                'precision': precision,
                'recall': recall,
                'f1': f1,
                'avg_response_ms': np.mean(times)
            }
            epoch_metrics.append(metrics)
            
            print(f"\nEpoch {epoch+1}: F1={f1:.3f}, Prec={precision:.3f}, Rec={recall:.3f}")
        
        if len(epoch_metrics) > 1:
            improvement = (epoch_metrics[-1]['f1'] - epoch_metrics[0]['f1']) / epoch_metrics[0]['f1'] * 100
            print(f"\n📈 F1 Improvement: {improvement:+.1f}%")
        
        return epoch_metrics
