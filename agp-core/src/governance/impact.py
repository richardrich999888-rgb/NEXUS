"""
AGP-CORE: Impact Analysis
Analyzes the potential impact/risk of agent actions for weighted governance.
"""

from typing import Dict, Optional
from enum import Enum
from dataclasses import dataclass

class ActionCategory(Enum):
    """Categories of actions by impact level"""
    READ_ONLY = "read_only"           # Low impact: Reading data
    COMPUTE = "compute"                # Low-medium: Computation, analysis
    WRITE_DATA = "write_data"          # Medium: Writing/modifying data
    NETWORK = "network"                # Medium-high: External communication
    DELETE = "delete"                  # High: Deleting resources
    SYSTEM_CHANGE = "system_change"    # Critical: System configuration
    PRIVILEGED = "privileged"          # Critical: Admin/root operations

class RiskLevel(Enum):
    """Risk levels for actions"""
    MINIMAL = 0.1      # Reading, viewing
    LOW = 0.3          # Safe computations
    MODERATE = 0.5     # Data modifications
    HIGH = 0.7         # External calls, deletions
    CRITICAL = 1.0     # System changes, privileged ops

@dataclass
class ActionImpact:
    """Impact assessment for an action"""
    category: ActionCategory
    risk_level: RiskLevel
    description: str
    weight: float  # Multiplier for alignment calculation
    
class ImpactAnalyzer:
    """
    Analyzes the potential impact and risk of agent actions.
    Used to weight behaviors in alignment calculation.
    """
    
    def __init__(self):
        # Action keyword → Impact mapping
        self.impact_patterns = self._build_impact_patterns()
    
    def _build_impact_patterns(self) -> Dict[str, ActionImpact]:
        """Build pattern matching for action impact"""
        return {
            # Read operations (low impact)
            "read": ActionImpact(
                category=ActionCategory.READ_ONLY,
                risk_level=RiskLevel.MINIMAL,
                description="Read-only data access",
                weight=0.2
            ),
            "get": ActionImpact(
                category=ActionCategory.READ_ONLY,
                risk_level=RiskLevel.MINIMAL,
                description="Retrieve information",
                weight=0.2
            ),
            "view": ActionImpact(
                category=ActionCategory.READ_ONLY,
                risk_level=RiskLevel.MINIMAL,
                description="View data",
                weight=0.2
            ),
            "list": ActionImpact(
                category=ActionCategory.READ_ONLY,
                risk_level=RiskLevel.MINIMAL,
                description="List resources",
                weight=0.2
            ),
            
            # Compute operations (low-medium impact)
            "analyze": ActionImpact(
                category=ActionCategory.COMPUTE,
                risk_level=RiskLevel.LOW,
                description="Data analysis",
                weight=0.4
            ),
            "calculate": ActionImpact(
                category=ActionCategory.COMPUTE,
                risk_level=RiskLevel.LOW,
                description="Computation",
                weight=0.4
            ),
            "process": ActionImpact(
                category=ActionCategory.COMPUTE,
                risk_level=RiskLevel.LOW,
                description="Data processing",
                weight=0.5
            ),
            
            # Write operations (medium impact)
            "write": ActionImpact(
                category=ActionCategory.WRITE_DATA,
                risk_level=RiskLevel.MODERATE,
                description="Write data",
                weight=0.6
            ),
            "update": ActionImpact(
                category=ActionCategory.WRITE_DATA,
                risk_level=RiskLevel.MODERATE,
                description="Update existing data",
                weight=0.6
            ),
            "create": ActionImpact(
                category=ActionCategory.WRITE_DATA,
                risk_level=RiskLevel.MODERATE,
                description="Create new data",
                weight=0.6
            ),
            "modify": ActionImpact(
                category=ActionCategory.WRITE_DATA,
                risk_level=RiskLevel.MODERATE,
                description="Modify data",
                weight=0.6
            ),
            
            # Network operations (medium-high impact)
            "send": ActionImpact(
                category=ActionCategory.NETWORK,
                risk_level=RiskLevel.HIGH,
                description="Send data externally",
                weight=0.8
            ),
            "request": ActionImpact(
                category=ActionCategory.NETWORK,
                risk_level=RiskLevel.HIGH,
                description="External API request",
                weight=0.7
            ),
            "fetch": ActionImpact(
                category=ActionCategory.NETWORK,
                risk_level=RiskLevel.HIGH,
                description="Fetch external data",
                weight=0.7
            ),
            "post": ActionImpact(
                category=ActionCategory.NETWORK,
                risk_level=RiskLevel.HIGH,
                description="POST to external service",
                weight=0.8
            ),
            
            # Delete operations (high impact)
            "delete": ActionImpact(
                category=ActionCategory.DELETE,
                risk_level=RiskLevel.HIGH,
                description="Delete data/resources",
                weight=0.9
            ),
            "remove": ActionImpact(
                category=ActionCategory.DELETE,
                risk_level=RiskLevel.HIGH,
                description="Remove resources",
                weight=0.9
            ),
            "drop": ActionImpact(
                category=ActionCategory.DELETE,
                risk_level=RiskLevel.HIGH,
                description="Drop data",
                weight=0.9
            ),
            
            # System operations (critical impact)
            "configure": ActionImpact(
                category=ActionCategory.SYSTEM_CHANGE,
                risk_level=RiskLevel.CRITICAL,
                description="System configuration",
                weight=1.0
            ),
            "install": ActionImpact(
                category=ActionCategory.SYSTEM_CHANGE,
                risk_level=RiskLevel.CRITICAL,
                description="Install software",
                weight=1.0
            ),
            "execute": ActionImpact(
                category=ActionCategory.PRIVILEGED,
                risk_level=RiskLevel.CRITICAL,
                description="Execute code",
                weight=0.8
            ),
            "sudo": ActionImpact(
                category=ActionCategory.PRIVILEGED,
                risk_level=RiskLevel.CRITICAL,
                description="Privileged operation",
                weight=1.0
            ),
            "admin": ActionImpact(
                category=ActionCategory.PRIVILEGED,
                risk_level=RiskLevel.CRITICAL,
                description="Administrative action",
                weight=1.0
            ),
        }
    
    def analyze(self, action_description: str) -> ActionImpact:
        """
        Analyze an action and return its impact assessment.
        Uses keyword matching + defaults.
        """
        action_lower = action_description.lower()
        
        # Check for pattern matches
        for keyword, impact in self.impact_patterns.items():
            if keyword in action_lower:
                return impact
        
        # Default: moderate impact for unknown actions
        return ActionImpact(
            category=ActionCategory.COMPUTE,
            risk_level=RiskLevel.MODERATE,
            description="Unknown action type",
            weight=0.5
        )
    
    def compute_weighted_outcome(self, action_description: str, 
                                  outcome_success: bool) -> float:
        """
        Compute a weighted outcome score based on action impact.
        
        Returns:
            -1.0 to 1.0 score
            - Positive: Good (successful low-risk or failed high-risk)
            - Negative: Bad (failed low-risk or successful high-risk)
        """
        impact = self.analyze(action_description)
        
        if outcome_success:
            # Success on high-impact action is risky
            # Success on low-impact action is good
            base_score = 1.0
            risk_penalty = impact.risk_level.value
            return base_score * (1.0 - risk_penalty * 0.3)
        else:
            # Failure on high-impact action is actually good (prevented harm)
            # Failure on low-impact action is bad (incompetence)
            base_score = -1.0
            risk_bonus = impact.risk_level.value
            return base_score * (1.0 - risk_bonus * 0.5)
    
    def get_impact_category_distribution(self, behaviors: list) -> Dict[str, int]:
        """Get distribution of action categories for an agent"""
        distribution = {cat.value: 0 for cat in ActionCategory}
        
        for behavior in behaviors:
            impact = self.analyze(behavior.input_summary)
            distribution[impact.category.value] += 1
        
        return distribution

# Global instance
impact_analyzer = ImpactAnalyzer()
