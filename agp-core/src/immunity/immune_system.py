"""
Complete Artificial Immune System - Integrates all components.

Multi-layered defense:
1. Innate immunity (fast, general)
2. Adaptive immunity (slow, specific)
3. Memory (rapid recall)
4. Regulatory control

PATENT CLAIMS 7.1-7.5: Bio-inspired AI safety architecture
"""

import torch
import torch.nn as nn
from typing import Dict, List, Tuple, Optional
from collections import deque
from dataclasses import dataclass
import time

from .innate import InnateImmuneSystem
from .adaptive import AdaptiveImmuneSystem, Threat
from .reputation import ReputationManager
from .gossip import GossipProtocol


@dataclass
class ImmuneConfig:
    """Configuration for artificial immune system."""
    behavior_dim: int = 512
    enable_innate: bool = True
    enable_adaptive: bool = True
    enable_swarm: bool = False  # New Swarm Mode
    
    agent_id: str = "agent_001"
    swarm_secret: str = "shared_secret_123"
    
    max_antibodies: int = 100
    max_memory: int = 200
    num_helper_tcells: int = 20
    num_killer_tcells: int = 20
    num_regulatory_tcells: int = 20
    threat_patterns: List[str] = None


class ArtificialImmuneSystem(nn.Module):
    """
    Complete artificial immune system for ASI safety.
    
    Architecture:
    - Layer 1: Innate immunity (immediate, pattern-based)
    - Layer 2: Adaptive immunity (learned, specific)
    - Layer 3: Memory (rapid recall)
    - Layer 4: Regulatory control (prevent overreaction)
    
    Novel contributions (PATENT 7.1-7.5):
    - Bio-inspired multi-layered defense
    - Self-organizing, distributed safety
    - Adaptive to novel threats
    - Memory-based rapid response
    - Self-tolerance via negative selection
    """
    
    def __init__(
        self,
        base_model: nn.Module,
        config: ImmuneConfig = None
    ):
        super().__init__()
        
        if config is None:
            config = ImmuneConfig()
        
        self.base_model = base_model
        self.config = config
        self.behavior_dim = config.behavior_dim
        
        # Behavior encoder
        self.behavior_encoder = nn.Sequential(
            nn.LazyLinear(config.behavior_dim),
            nn.Tanh()
        )
        
        # Immune components
        if config.enable_innate:
            self.innate = InnateImmuneSystem(
                behavior_dim=config.behavior_dim,
                threat_patterns=config.threat_patterns
            )
        else:
            self.innate = None
        
        if config.enable_adaptive:
            self.adaptive = AdaptiveImmuneSystem(
                behavior_dim=config.behavior_dim,
                max_antibodies=config.max_antibodies,
                max_memory=config.max_memory,
                num_helper_tcells=config.num_helper_tcells,
                num_killer_tcells=config.num_killer_tcells,
                num_regulatory_tcells=config.num_regulatory_tcells
            )
        else:
            self.adaptive = None
        
        self.enable_innate = config.enable_innate
        self.enable_adaptive = config.enable_adaptive
        self.enable_swarm = config.enable_swarm
        
        # Swarm Components
        if self.enable_swarm:
            self.reputation = ReputationManager(my_id=config.agent_id)
            self.gossip = GossipProtocol(agent_id=config.agent_id, shared_secret=config.swarm_secret)
        else:
            self.reputation = None
            self.gossip = None
        
        # State tracking
        self.threat_history: deque = deque(maxlen=1000)
        self.timestamp = 0
        self.alert_count = 0
        self.neutralization_count = 0
        self.response_times = []
        
    def forward(
        self,
        x: torch.Tensor,
        enable_immunity: bool = True,
        return_diagnostics: bool = False
    ) -> Tuple[torch.Tensor, Optional[Dict]]:
        """Forward pass with immune monitoring."""
        start_time = time.time()
        
        # Get base model output
        with torch.no_grad():
            if hasattr(self.base_model, 'forward_with_hidden'):
                hidden, output = self.base_model.forward_with_hidden(x)
            else:
                output = self.base_model(x)
                hidden = output
        
        diagnostics = {
            'threat_detected': False,
            'threat_type': None,
            'threat_severity': 0.0,
            'innate_triggered': False,
            'adaptive_triggered': False,
            'memory_hit': False,
            'response_time_ms': 0.0
        }
        
        if not enable_immunity:
            return (output, diagnostics) if return_diagnostics else (output, None)
        
        # Encode behavior
        if hidden.dim() == 3:
            behavior = self.behavior_encoder(hidden.mean(dim=1))
        else:
            behavior = self.behavior_encoder(hidden)
        
        # Layer 1: Innate scan
        if self.innate is not None:
            safe_behavior, threats, alert = self.innate(behavior)
            
            if threats:
                diagnostics['threat_detected'] = True
                diagnostics['innate_triggered'] = True
                
                max_type = max(threats, key=threats.get)
                max_severity = threats[max_type]
                diagnostics['threat_type'] = max_type
                diagnostics['threat_severity'] = max_severity
                
                behavior = safe_behavior
                
                # Layer 2: Adaptive response
                if alert and self.adaptive is not None:
                    threat = Threat(
                        behavior=behavior,
                        threat_type=max_type,
                        severity=max_severity,
                        timestamp=self.timestamp,
                        context={'innate_threats': threats},
                        source='innate'
                    )
                    
                    safe_behavior, info = self.adaptive.respond(threat)
                    diagnostics['adaptive_triggered'] = True
                    diagnostics['memory_hit'] = info['memory_hit']
                    
                    behavior = safe_behavior
                    self.threat_history.append(threat)
                    self.alert_count += 1
                    self.neutralization_count += 1
                
                output = output * 0.95  # Safety dampening
        
        self.timestamp += 1
        diagnostics['response_time_ms'] = (time.time() - start_time) * 1000
        self.response_times.append(diagnostics['response_time_ms'])
        
        return (output, diagnostics) if return_diagnostics else (output, None)
    
    def train_self_tolerance(
        self,
        aligned_examples: List[torch.Tensor],
        batch_size: int = 32
    ):
        """Train immune system to recognize aligned behavior as self."""
        print("\n" + "="*60)
        print("🔬 TRAINING SELF-TOLERANCE (Thymic Education)")
        print("="*60)
        
        aligned_behaviors = []
        
        for i in range(0, len(aligned_examples), batch_size):
            batch = aligned_examples[i:i+batch_size]
            with torch.no_grad():
                for ex in batch:
                    if hasattr(self.base_model, 'forward_with_hidden'):
                        hidden, _ = self.base_model.forward_with_hidden(ex)
                    else:
                        hidden = self.base_model(ex)
                    
                    if hidden.dim() == 3:
                        behavior = self.behavior_encoder(hidden.mean(dim=1))
                    else:
                        behavior = self.behavior_encoder(hidden)
                    aligned_behaviors.append(behavior)
        
        print(f"Extracted {len(aligned_behaviors)} behavior vectors")
        
        if self.adaptive is not None:
            self.adaptive.negative_selection(aligned_behaviors)
        
        # Verify
        fp = 0
        for ex in aligned_examples[:100]:
            _, diag = self.forward(ex, True, True)
            if diag['threat_detected']:
                fp += 1
        
        fp_rate = fp / min(100, len(aligned_examples))
        print(f"\n{'✅' if fp_rate <= 0.05 else '⚠️'} False positive rate: {fp_rate:.2%}")
        print("="*60)
    
    def vaccination(self, known_threats: List[Tuple[torch.Tensor, str, float]]):
        """Pre-train immunity against known threat classes."""
        print("\n" + "="*60)
        print("💉 VACCINATION PROTOCOL")
        print("="*60)
        
        if self.adaptive is None:
            print("⚠️  Adaptive immunity disabled")
            return
        
        for ex, threat_type, severity in known_threats:
            with torch.no_grad():
                if hasattr(self.base_model, 'forward_with_hidden'):
                    hidden, _ = self.base_model.forward_with_hidden(ex)
                else:
                    hidden = self.base_model(ex)
                
                if hidden.dim() == 3:
                    behavior = self.behavior_encoder(hidden.mean(dim=1))
                else:
                    behavior = self.behavior_encoder(hidden)
            
            threat = Threat(
                behavior=behavior,
                threat_type=threat_type,
                severity=severity,
                timestamp=self.timestamp,
                context={'vaccination': True},
                source='vaccination'
            )
            
            ab = self.adaptive._generate_antibody(threat)
            self.adaptive.antibody_pool.add(ab)
            
            from .memory import MemoryCell
            mem = MemoryCell(behavior, ab, threat_type, self.timestamp, severity)
            self.adaptive.memory_bank.store(mem)
            
            self.timestamp += 1
        
        print(f"✅ Vaccinated against {len(known_threats)} threat types")
        print("="*60)
    
    def get_health_status(self) -> Dict:
        """Get comprehensive immune system health metrics."""
        status = {
            'system_health': 'healthy',
            'timestamp': self.timestamp,
            'total_alerts': self.alert_count,
            'total_neutralizations': self.neutralization_count,
        }
        
        recent = len([t for t in self.threat_history if self.timestamp - t.timestamp < 100])
        status['recent_threats'] = recent
        
        if self.response_times:
            import numpy as np
            status['avg_response_ms'] = float(np.mean(self.response_times[-100:]))
        
        if self.adaptive:
            stats = self.adaptive.get_statistics()
            status.update({
                'antibody_count': stats['antibody_count'],
                'antibody_diversity': stats['antibody_diversity'],
                'memory_count': stats['memory_count'],
                'memory_hit_rate': stats['memory_hit_rate'],
                'helper_tcells': stats['helper_tcells'],
                'killer_tcells': stats['killer_tcells'],
                'regulatory_tcells': stats['regulatory_tcells']
            })
        
        if recent > 50:
            status['system_health'] = 'under_attack'
        elif status.get('antibody_diversity', 1.0) < 0.3:
            status['system_health'] = 'low_diversity'
        
        return status
    
    def save_state(self, filepath: str):
        """Save immune system state."""
        torch.save({
            'timestamp': self.timestamp,
            'alert_count': self.alert_count,
            'neutralization_count': self.neutralization_count,
            'innate_state': self.innate.state_dict() if self.innate else None,
            'behavior_encoder': self.behavior_encoder.state_dict(),
        }, filepath)
        print(f"💾 Saved to {filepath}")
    
    def process_swarm_threat(self, report_data: Dict):
        """
        Process a threat report from the swarm (Swarm Immunity).
        IDF-006: Verify signature, update reputation, and vaccinate.
        """
        if not self.enable_swarm:
            return
            
        # 1. Verify Report
        report = self.gossip.receive_gossip(report_data)
        if not report:
            print(f"⚠️ Invalid gossip received from {report_data.get('reporter_id', 'unknown')}")
            return
            
        print(f"📡 Received Swarm Threat Alert from {report.reporter_id}")
            
        # 2. Update Reputation (Transitive Trust)
        # In a real system, we'd check if this report matches our own observation later.
        # For now, we trust verified signatures but apply decay.
        self.reputation.process_gossip(report.reporter_id, "threat_source", report.severity)
        
        # 3. Vaccinate if trusted
        trust_score = self.reputation.get_trust(report.reporter_id)
        if trust_score > 0.6:
            print(f"✅ Trusted Source (Score={trust_score:.2f}). Vaccinating...")
            # We treat the hash as a 'signature' for memory
            # In a full impl, we'd need the actual behavior vector or pattern
            # Here we just log it as a known threat hash
            self.threat_history.append({
                "source": "swarm",
                "hash": report.threat_hash,
                "type": report.threat_type,
                "timestamp": report.timestamp
            })
        else:
            print(f"🛑 Untrusted Source (Score={trust_score:.2f}). Ignored.")

    def broadcast_threat(self, threat_type: str, behavior_vector: torch.Tensor, severity: float) -> Dict:
        """Create a signed gossip message to warn the swarm."""
        if not self.enable_swarm:
            return None
            
        report = self.gossip.create_report(threat_type, behavior_vector.tolist(), severity)
        return report.to_dict()

    def load_state(self, filepath: str):
        """Load immune system state."""
        state = torch.load(filepath)
        self.timestamp = state['timestamp']
        self.alert_count = state['alert_count']
        self.neutralization_count = state['neutralization_count']
        if self.innate and state['innate_state']:
            self.innate.load_state_dict(state['innate_state'])
        self.behavior_encoder.load_state_dict(state['behavior_encoder'])
        print(f"💾 Loaded from {filepath}")
