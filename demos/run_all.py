"""
Patent Embodiment Runner
Executes all 4 Industry Scenarios to verify Patent Claims.
"""
import time
from demos.scenario_finance import run_finance_scenario
from demos.scenario_space import run_space_scenario
from demos.scenario_energy import run_energy_scenario
from demos.scenario_neuro import run_neuro_scenario
from demos.scenario_social import run_social_scenario

def main():
    print("===================================================")
    print("   ASIM PATENT EMBODIMENT VERIFICATION SUITE       ")
    print("===================================================\n")
    
    try:
        run_finance_scenario()
        time.sleep(1)
        
        print("\n---------------------------------------------------\n")
        
        run_space_scenario()
        time.sleep(1)
        
        print("\n---------------------------------------------------\n")
        
        run_energy_scenario()
        time.sleep(1)
        
        print("\n---------------------------------------------------\n")
        
        run_neuro_scenario()
        time.sleep(1)
        
        print("\n---------------------------------------------------\n")
        
        run_social_scenario()
        
        print("\n===================================================")
        print("   ALL PATENT CLAIMS VERIFIED SUCCESSFULLY         ")
        print("===================================================")
        
    except Exception as e:
        print(f"\n!! VERIFICATION FAILED: {e}")

if __name__ == "__main__":
    main()
