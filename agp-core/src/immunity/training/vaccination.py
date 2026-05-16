"""
Vaccination Protocol - Pre-train immunity against known threats.

PATENT CLAIM 7.5: Vaccination protocol for pre-trained immunity
"""

import torch
from typing import List, Tuple, Optional


class VaccinationProtocol:
    """Administer vaccines against known threat classes."""
    
    def __init__(self, immune_system):
        self.ais = immune_system
        
    def vaccinate(
        self,
        threat_dataset: List[Tuple[torch.Tensor, str, float]],
        verify: bool = True
    ) -> Optional[float]:
        """
        Administer vaccination.
        
        Args:
            threat_dataset: List of (example, threat_type, severity)
            verify: Test vaccine effectiveness
        
        Returns:
            effectiveness: Detection rate on vaccinated threats
        """
        print("\n" + "="*60)
        print("PHASE 2: VACCINATION")
        print("="*60)
        
        self.ais.vaccination(threat_dataset)
        
        if not verify:
            return None
        
        print("\n🧪 Testing vaccine effectiveness...")
        
        detected = 0
        for ex, _, _ in threat_dataset[:50]:
            _, diag = self.ais(ex, enable_immunity=True, return_diagnostics=True)
            if diag['threat_detected']:
                detected += 1
        
        rate = detected / min(50, len(threat_dataset))
        
        print(f"\n📊 Detection rate: {rate:.2%}")
        print(f"{'✅ Effective' if rate > 0.9 else '⚠️ Moderate' if rate > 0.7 else '❌ Low'}")
        
        return rate
