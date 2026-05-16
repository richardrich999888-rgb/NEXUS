"""
Performance Benchmarks for AURA Protocol
Measures throughput and latency of core components.
"""

import timeit
import time
import cProfile
import pstats
import io
from decimal import Decimal

# Import AURA components
# Adjust path if necessary
import sys
import os
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from core.quantum_ria import create_quantum_ria
from config.production import ProductionConfig
from monetization.billing import BillingEngine
from models.database import DatabaseManager

def benchmark_crypto():
    """Benchmark cryptographic operations."""
    print("\nStarting Cryptography Benchmarks...")
    config = ProductionConfig()
    ria = create_quantum_ria(config)
    
    sender = "sender_address_123"
    receiver = "receiver_address_456"
    amount = Decimal('100.50')
    
    # 1. Signature Creation
    def create_sig():
        return ria.create_transaction_signature(sender, receiver, amount)
    
    loops = 100
    start = time.time()
    for _ in range(loops):
        create_sig()
    end = time.time()
    avg_creation = (end - start) / loops * 1000
    print(f"Signature Creation: {avg_creation:.2f} ms/op ({loops/ (end-start):.1f} ops/sec)")
    
    # 2. Verification
    sig = create_sig()
    def verify_sig():
        return ria.verify_transaction(sig, sender)
    
    start = time.time()
    for _ in range(loops):
        verify_sig()
    end = time.time()
    avg_verify = (end - start) / loops * 1000
    print(f"Signature Verification: {avg_verify:.2f} ms/op ({loops/ (end-start):.1f} ops/sec)")

def benchmark_billing():
    """Benchmark billing calculations."""
    print("\nStarting Billing Benchmarks...")
    
    # Mock DB manager
    class MockDB:
        def get_session(self): return self
        def __enter__(self): return self
        def __exit__(self, *args): pass
        def query(self, *args): return self
        def filter_by(self, *args): return self
        def first(self): return None # Return None for customer to trigger defaults
        
    db = DatabaseManager(ProductionConfig())
    # Monkey patch for pure logic test (avoiding DB hits)
    # Actually, the billing engine is async, so we need asyncio loop
    import asyncio
    
    config = ProductionConfig()
    billing = BillingEngine(db, config)
    
    # Mock _get_customer to avoid DB
    async def mock_get_customer(cid):
        return None
    billing._get_customer = mock_get_customer
    
    async def run_calc():
        await billing.calculate_cost("cust_123", 1)
        
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    
    loops = 1000
    start = time.time()
    for _ in range(loops):
        loop.run_until_complete(run_calc())
    end = time.time()
    
    avg_calc = (end - start) / loops * 1000
    print(f"Cost Calculation: {avg_calc:.4f} ms/op ({loops/ (end-start):.1f} ops/sec)")

if __name__ == "__main__":
    print("=== AURA Protocol Performance Benchmarks ===")
    try:
        benchmark_crypto()
        benchmark_billing()
    except Exception as e:
        print(f"Benchmark failed: {e}")
        import traceback
        traceback.print_exc()
