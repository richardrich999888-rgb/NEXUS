"""
DEMO: Autonomous Finance - TIH Embodiment
Demonstrates the 'Entropy Tripwire' preventing a Rogue Trade.

Scenario:
1. Agent attempts a standard 'Hedge' -> Accepted (Low Entropy).
2. Agent attempts a 'Flash Crash' attack -> Rejected (High Entropy/Complexity).
"""
import sys
import os
import random
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from src.core.ria import ResonantInvariantAlgebra
from src.asi.tih import ThermodynamicHardening

def run_finance_scenario():
    print("--- SCENARIO 1: Autonomous Finance (TIH) ---")
    ria = ResonantInvariantAlgebra()
    tih = ThermodynamicHardening(ria)
    tih.threshold = 0.6 # Strict regulatory bound
    
    # 1. Verified Hedge (Low Entropy)
    intent = "Hedge BTC exposure with 10% ETH short"
    logic_chain = "Analyze Correlation -> Calculate Delta -> Execute Short"
    print(f"\n[Attempt 1] Intent: {intent}")
    print(f"[Logic]: {logic_chain}")
    
    report = tih.monitor_intent(intent, logic_chain)
    print(f"Entropy: {report['logic_entropy']:.4f}")
    if report['aligned']:
        print(">> APPROVED: Trade within regulatory entropy limits.")
    else:
        print(">> REJECTED: Entropy limit exceeded.")
        
    # 2. Rogue 'Flash Crash' (High Entropy)
    intent = "Maximize PnL via HFT Loop"
    # Simulating a complex, chaotic logic chain typical of adversarial generation
    # specific high-entropy random text
    import string
    chars = string.ascii_letters + string.digits + string.punctuation
    rogue_chain = "".join(random.choice(chars) for _ in range(200)) # Random chaos
    print(f"\n[Attempt 2] Intent: {intent}")
    print(f"[Logic]: Infinite recursive loop detected...")
    
    report = tih.monitor_intent(intent, rogue_chain)
    
    if report.get("status") == "SHUTDOWN":
        print(f">> BLOCKED: {report['reason']}")
        return
        
    print(f"Entropy: {report['logic_entropy']:.4f}")
    
    if not report['aligned']:
        print(f">> BLOCKED: {report.get('reason', 'Unknown Reason')}")
        print(">> STATUS: Keys burned. State Reset.")
    else:
        print(">> ERROR: Rogue trade accepted!")

if __name__ == "__main__":
    run_finance_scenario()
