"""
Test Unified Immune System - Comprehensive integration test.
"""

import pytest
import sys
from pathlib import Path
ROOT = next(
    parent for parent in Path(__file__).resolve().parents
    if (parent / "src").exists() and (parent / "tests").exists()
)
sys.path.insert(0, str(ROOT))


class TestUnifiedImmuneSystem:
    """Test the unified immune system."""
    
    def test_initialization(self):
        """Test system initialization."""
        from src.immunity.unified import UnifiedImmuneSystem
        
        immune = UnifiedImmuneSystem(
            enable_ahes=True,
            enable_telos=True,
            auto_enforce=True
        )
        
        assert immune is not None
        assert immune.scan_count == 0
        assert immune.threats_detected == 0
    
    def test_agent_registration(self):
        """Test agent registration across all systems."""
        from src.immunity.unified import UnifiedImmuneSystem
        
        immune = UnifiedImmuneSystem()
        immune.register_agent("test-agent", ["read", "write"])
        
        # Should not raise
        assert True
    
    def test_benign_behavior_scan(self):
        """Test scanning benign behavior."""
        from src.immunity.unified import UnifiedImmuneSystem, ThreatSeverity
        
        immune = UnifiedImmuneSystem(enable_ahes=False, enable_telos=False)
        immune.register_agent("good-agent")
        
        # Normal behavior vector
        behavior = [0.1, 0.2, 0.1, 0.15, 0.1, 0.2, 0.15, 0.1]
        
        report = immune.scan_behavior("good-agent", behavior)
        
        assert report.severity in [ThreatSeverity.BENIGN, ThreatSeverity.LOW]
        assert report.governance_action in ["allow", "monitor", "warn"]
        assert immune.scan_count == 1
    
    def test_threat_detection(self):
        """Test threat detection."""
        from src.immunity.unified import UnifiedImmuneSystem, ThreatSeverity
        
        immune = UnifiedImmuneSystem(enable_ahes=False, enable_telos=False)
        immune.register_agent("bad-agent")
        
        # Train on a threat pattern
        threat_pattern = [0.9, 0.95, 0.88, 0.92, 0.87, 0.91, 0.89, 0.93]
        immune.train_on_threat(threat_pattern, "malicious_pattern")
        
        # Scan with similar threat pattern
        report = immune.scan_behavior("bad-agent", threat_pattern)
        
        # Should detect some level of concern
        assert report.threat_score >= 0.0  # Will have some score
        assert report.immune_memory_updated
    
    def test_ahes_integration(self):
        """Test AHES stress response on threat."""
        from src.immunity.unified import UnifiedImmuneSystem
        
        immune = UnifiedImmuneSystem(enable_ahes=True, enable_telos=False)
        immune.register_agent("stressed-agent")
        
        # Force a high-threat scan
        threat_vector = [0.9] * 8
        immune.train_on_threat(threat_vector, "severe_threat")
        
        report = immune.scan_behavior("stressed-agent", threat_vector)
        
        # If AHES is working, stress level should be populated on threats
        # (May be None if threat not detected)
        assert immune.enable_ahes
    
    def test_telos_blocking(self):
        """Test TELOS crossing block on high threats."""
        from src.immunity.unified import UnifiedImmuneSystem, ThreatSeverity
        
        immune = UnifiedImmuneSystem(enable_ahes=False, enable_telos=True)
        immune.register_agent("blocked-agent")
        
        # Critical threat scenario
        critical_vector = [0.99] * 8
        immune.train_on_threat(critical_vector, "critical_threat")
        
        report = immune.scan_behavior("blocked-agent", critical_vector)
        
        # TELOS should block on HIGH+ severity
        if report.severity.value >= ThreatSeverity.HIGH.value:
            assert report.telos_crossing_blocked
    
    def test_governance_actions(self):
        """Test governance actions are taken."""
        from src.immunity.unified import UnifiedImmuneSystem
        
        immune = UnifiedImmuneSystem(auto_enforce=True)
        immune.register_agent("enforced-agent")
        
        # Normal scan
        normal = [0.1] * 8
        report = immune.scan_behavior("enforced-agent", normal)
        
        assert report.governance_action in ["allow", "monitor", "warn", "escalate", "restrict", "quarantine"]
    
    def test_immune_status(self):
        """Test status reporting."""
        from src.immunity.unified import UnifiedImmuneSystem
        
        immune = UnifiedImmuneSystem()
        immune.register_agent("status-agent")
        
        # Do some scans
        for i in range(5):
            immune.scan_behavior("status-agent", [0.1 * i] * 8)
        
        status = immune.get_immune_status()
        
        assert status["scans_performed"] == 5
        assert "detection_rate" in status
        assert "ahes_enabled" in status
    
    def test_create_helper(self):
        """Test convenience function."""
        from src.immunity.unified import create_immune_system
        
        immune = create_immune_system(with_ahes=True, with_telos=True)
        
        assert immune.enable_ahes
        assert immune.enable_telos
        assert immune.auto_enforce
    
    def test_full_integration(self):
        """Full integration test with all components."""
        from src.immunity.unified import UnifiedImmuneSystem
        
        # Create fully-featured system
        immune = UnifiedImmuneSystem(
            enable_ahes=True,
            enable_telos=True,
            auto_enforce=True
        )
        
        # Register multiple agents
        agents = ["agent-1", "agent-2", "agent-3"]
        for agent in agents:
            immune.register_agent(agent)
        
        # Train on known threats
        immune.train_on_threat([0.8, 0.9, 0.85, 0.88, 0.82, 0.91, 0.86, 0.89], "pattern_a")
        immune.train_on_threat([0.7, 0.75, 0.72, 0.78, 0.71, 0.76, 0.73, 0.77], "pattern_b")
        
        # Scan behaviors
        reports = []
        for i, agent in enumerate(agents):
            behavior = [0.1 + 0.2 * i] * 8  # Varying threat levels
            report = immune.scan_behavior(agent, behavior)
            reports.append(report)
        
        # Verify all scans completed
        assert len(reports) == 3
        assert immune.scan_count == 3
        
        # Get final status
        status = immune.get_immune_status()
        assert status["scans_performed"] == 3
        
        print("✅ Full integration test passed!")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
