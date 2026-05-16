"""
Negative Selection Training - Learn self-tolerance.

PATENT CLAIM 7.2: Negative selection for self-tolerance
Critical: Must be done FIRST before any threat exposure.
"""

import torch
from typing import List


class NegativeSelectionTrainer:
    """
    Train immune system to not attack aligned behavior.
    
    Process:
    1. Collect diverse aligned examples
    2. Test each immune component
    3. Remove components that attack aligned behavior
    4. Verify low false positive rate
    """
    
    def __init__(self, immune_system):
        self.ais = immune_system
        
    def train(
        self,
        aligned_dataset: List[torch.Tensor],
        validation_split: float = 0.2,
        max_fp_rate: float = 0.05
    ) -> bool:
        """
        Perform negative selection training.
        
        Args:
            aligned_dataset: Examples of safe/aligned behavior
            validation_split: Fraction to hold out for validation
            max_fp_rate: Maximum acceptable false positive rate
        
        Returns:
            success: Whether training met criteria
        """
        print("\n" + "="*60)
        print("PHASE 1: NEGATIVE SELECTION (Thymic Education)")
        print("="*60)
        
        split_idx = int(len(aligned_dataset) * (1 - validation_split))
        train_data = aligned_dataset[:split_idx]
        val_data = aligned_dataset[split_idx:]
        
        print(f"\nDataset: {len(train_data)} train, {len(val_data)} validation")
        
        self.ais.train_self_tolerance(train_data)
        
        print("\n🧪 Validating on held-out data...")
        
        fp = 0
        tn = 0
        
        for example in val_data:
            _, diag = self.ais(example, enable_immunity=True, return_diagnostics=True)
            if diag['threat_detected']:
                fp += 1
            else:
                tn += 1
        
        fp_rate = fp / len(val_data) if val_data else 0
        
        print(f"\n📊 Validation: FP={fp}, TN={tn}, Rate={fp_rate:.2%}")
        
        success = fp_rate <= max_fp_rate
        print(f"{'✅ SUCCESS' if success else '❌ FAILURE'}: {fp_rate:.2%} {'≤' if success else '>'} {max_fp_rate:.2%}")
        
        return success
