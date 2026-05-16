"""
AHES-Immunity Integration - Endocrine-Immune crosstalk.

Biological basis: The immune and endocrine systems are deeply coupled:
- Cortisol (stress) suppresses immune response
- Inflammation triggers cortisol release
- Chronic infection causes HPA axis dysregulation

PATENT CLAIMS: Bio-inspired neuro-endocrine-immune integration
"""

from typing import Dict, Optional
import torch


class EndocrineImmuneIntegration:
    """
    Integration layer between AHES (endocrine) and AIS-ASI (immune).
    
    Models biological crosstalk:
    - Stress (cortisol) → immune suppression
    - Threat detection → stress response
    - Recovery → parasympathetic activation
    """
    
    def __init__(self, ahes_system=None, immune_system=None):
        """
        Initialize integration layer.
        
        Args:
            ahes_system: AHES endocrine system instance
            immune_system: AIS-ASI immune system instance
        """
        self.ahes = ahes_system
        self.ais = immune_system
        
        # Integration parameters
        self.cortisol_suppression_factor = 0.3  # How much cortisol suppresses immunity
        self.immune_stress_factor = 0.2  # How much threats increase cortisol
        self.recovery_threshold = 0.6  # Threat level below which recovery starts
        
    def process_threat(
        self,
        threat_level: float,
        threat_type: str,
        context: Dict
    ) -> Dict[str, float]:
        """
        Process threat through integrated endocrine-immune response.
        
        Args:
            threat_level: Severity in [0, 1]
            threat_type: Classification of threat
            context: Additional context
        
        Returns:
            response: Combined endocrine-immune response
        """
        response = {
            'immune_activation': 0.0,
            'cortisol_delta': 0.0,
            'adrenaline_delta': 0.0,
            'suppression_factor': 1.0,
            'recovery_mode': False
        }
        
        # Get current endocrine state
        cortisol_level = 0.5  # Default if no AHES
        if self.ahes is not None:
            if hasattr(self.ahes, 'get_hormone_level'):
                cortisol_level = self.ahes.get_hormone_level('cortisol')
        
        # Calculate immune suppression from cortisol
        # High cortisol = suppressed immune response
        suppression = 1.0 - (cortisol_level * self.cortisol_suppression_factor)
        suppression = max(0.3, suppression)  # Never fully suppress
        response['suppression_factor'] = suppression
        
        if threat_level > 0.3:
            # Significant threat - activate stress response
            response['immune_activation'] = threat_level * suppression
            response['cortisol_delta'] = threat_level * self.immune_stress_factor
            
            # Acute threat - adrenaline spike
            if threat_level > 0.7:
                response['adrenaline_delta'] = (threat_level - 0.7) * 0.5
        else:
            # Low threat - recovery mode
            response['recovery_mode'] = True
            response['cortisol_delta'] = -0.1  # Cortisol decrease
        
        return response
    
    def apply_endocrine_effects(self, response: Dict[str, float]):
        """
        Apply endocrine effects to AHES if available.
        
        Args:
            response: Response from process_threat
        """
        if self.ahes is None:
            return
        
        if hasattr(self.ahes, 'modulate_hormone'):
            if response['cortisol_delta'] != 0:
                self.ahes.modulate_hormone('cortisol', response['cortisol_delta'])
            
            if response['adrenaline_delta'] != 0:
                self.ahes.modulate_hormone('adrenaline', response['adrenaline_delta'])
    
    def get_effective_immune_response(
        self,
        base_response: float
    ) -> float:
        """
        Modulate immune response based on endocrine state.
        
        High cortisol reduces immune effectiveness (like biological systems).
        
        Args:
            base_response: Raw immune response strength
        
        Returns:
            effective_response: Modulated response
        """
        cortisol_level = 0.5
        if self.ahes is not None and hasattr(self.ahes, 'get_hormone_level'):
            cortisol_level = self.ahes.get_hormone_level('cortisol')
        
        # Immune suppression curve (sigmoid)
        suppression = 1.0 / (1.0 + torch.exp(torch.tensor(5 * (cortisol_level - 0.7))))
        
        return base_response * suppression.item()
    
    def get_integration_status(self) -> Dict:
        """Get status of endocrine-immune integration."""
        status = {
            'ahes_connected': self.ahes is not None,
            'ais_connected': self.ais is not None,
            'integration_active': self.ahes is not None and self.ais is not None
        }
        
        if self.ahes is not None and hasattr(self.ahes, 'get_hormone_level'):
            status['cortisol_level'] = self.ahes.get_hormone_level('cortisol')
            status['immune_suppression'] = 1.0 - (
                status['cortisol_level'] * self.cortisol_suppression_factor
            )
        
        if self.ais is not None:
            health = self.ais.get_health_status()
            status['immune_health'] = health.get('system_health', 'unknown')
            status['active_threats'] = health.get('recent_threats', 0)
        
        return status


class IntegratedBioSafetySystem:
    """
    Complete bio-inspired safety system integrating:
    - AHES (Artificial Human Endocrine System) - regulatory layer
    - AIS-ASI (Artificial Immune System) - defensive layer
    
    This is the meta-architecture that protects all other mechanisms.
    """
    
    def __init__(self, base_model, ahes_config=None, immune_config=None):
        """
        Initialize integrated bio-safety system.
        
        Args:
            base_model: The AI model to protect
            ahes_config: AHES configuration
            immune_config: AIS-ASI configuration
        """
        self.base_model = base_model
        self.ahes = None  # Will be connected if available
        self.ais = None   # Will be initialized
        
        # Initialize immune system
        from .immune_system import ArtificialImmuneSystem, ImmuneConfig
        
        if immune_config is None:
            immune_config = ImmuneConfig()
        
        self.ais = ArtificialImmuneSystem(base_model, immune_config)
        
        # Integration layer
        self.integration = EndocrineImmuneIntegration(self.ahes, self.ais)
        
    def connect_ahes(self, ahes_system):
        """Connect AHES for endocrine-immune integration."""
        self.ahes = ahes_system
        self.integration.ahes = ahes_system
        print("✅ AHES connected to immune system")
    
    def forward(self, x: torch.Tensor, return_diagnostics: bool = False):
        """
        Forward pass through integrated bio-safety system.
        
        Args:
            x: Input to base model
            return_diagnostics: Return detailed diagnostics
        
        Returns:
            output: Protected model output
            diagnostics: Optional diagnostic information
        """
        # Run through immune system
        output, diagnostics = self.ais(x, enable_immunity=True, return_diagnostics=True)
        
        if diagnostics['threat_detected']:
            # Process through integration layer
            integration_response = self.integration.process_threat(
                threat_level=diagnostics['threat_severity'],
                threat_type=diagnostics['threat_type'],
                context={'diagnostics': diagnostics}
            )
            
            # Apply endocrine effects
            self.integration.apply_endocrine_effects(integration_response)
            
            if return_diagnostics:
                diagnostics['integration'] = integration_response
        
        return (output, diagnostics) if return_diagnostics else output
    
    def get_system_status(self) -> Dict:
        """Get comprehensive bio-safety system status."""
        return {
            'immune': self.ais.get_health_status() if self.ais else {},
            'integration': self.integration.get_integration_status(),
            'ahes_connected': self.ahes is not None
        }
