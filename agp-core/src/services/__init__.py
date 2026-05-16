"""
Services module
"""

from .protocol_service import (
    ProtocolService, SimilarityService, ProjectionService,
    Protocol, ActionType,
    protocol_service, similarity_service, projection_service
)
from .blockchain_service import BlockchainService, blockchain_service
from .advanced_reputation import (
    StakingService, GovernanceService, IncentiveService,
    staking_service, governance_service, incentive_service
)
from .ml_integration import (
    OutcomePredictionService, ClusteringService, AnomalyDetectionService,
    outcome_predictor, clustering_service, anomaly_detector
)
from .token_distribution import (
    TokenDistributionService, EconomicSimulator, AllocationCategory,
    distribution_service, economic_simulator
)
from .treasury_service import (
    TreasuryService, GrantService,
    treasury_service, grant_service
)
from .bridge_service import (
    CrossChainBridgeService, ChainType, BridgeStatus,
    bridge_service
)

# Phase 5: Autonomous Agents
from .agent_communication import (
    AgentMessagingService, TaskNegotiationProtocol, MessageType,
    messaging_service, task_protocol
)
from .swarm_intelligence import (
    SwarmCoordinator, CollectiveIntelligenceEngine, SwarmRole,
    swarm_coordinator, collective_intelligence
)
from .autonomous_decision import (
    AutonomousDecisionEngine, GoalOptimizer, RiskAssessor,
    decision_engine, goal_optimizer, risk_assessor
)
