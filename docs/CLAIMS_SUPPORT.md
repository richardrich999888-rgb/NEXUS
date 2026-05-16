# ASIM Claims Support Evidence
**Technical Reduction to Practice for AURA Sovereign Intelligence Mesh (ASIM)**

This document maps specific source code implementations to the independent claims of the ASIM Provisional Patent Application. It serves as proof of enablement.

## Patent Family 1: Thermodynamic Invariant Hardening (TIH)
**Claim**: A method for preventing rogue algorithmic execution by monitoring logical entropy.
*   **Embodiment**: `demos/scenario_finance.py`
*   **Evidence**:
    *   **Line 21**: `report = tih.monitor_intent(intent, logic_chain)` measures entropy.
    *   **Line 39**: High entropy (>0.6) logic triggers `THERMAL RESET`.
    *   **Line 93 (src/asi/tih.py)**: `self.ria.E = 0 # Invariant erasure` physically halts the system.

## Patent Family 2: Isogeny Potential Entrapment (IPE)
**Claim**: A state synchronization system using isogeny graph topology as an energetic barrier.
*   **Embodiment**: `demos/scenario_space.py`
*   **Evidence**:
    *   **Line 28**: `verify_state_transition` checks the cryptographic work path.
    *   **Line 42**: Transition fails without the correct isogeny path proof ("Trapped in Potential Well").

## Patent Family 3: Statistical Field Alignment (SFA)
**Claim**: A distributed consensus mechanism based on minimizing mutual information divergence.
*   **Embodiment**: `demos/scenario_energy.py`
*   **Evidence**:
    *   **Line 32**: `sfa.calculate_alignment_coherence(all_opinions)` computes field state.
    *   **Line 44**: Coherence < 0.95 triggers isolation of the outlier node ("Decoherence Detected").

## Patent Family 4: Neuro-Thermodynamic Provenance (NTP) / Social Truth (STP)
**Claim**: A method for entropic verification of biological signal origin.
*   **Embodiment**: `demos/scenario_neuro.py` & `demos/scenario_social.py`
*   **Evidence**:
    *   **Line 22 (Social)**: `ntp.verify_provenance(human_stream)` confirms biological entropy.
    *   **Line 40 (Social)**: Zero-entropy deepfake stream (`b'\x00' * 2048`) is rejected as synthetic.
    *   **Technical Basis**: `src/asi/alignment.py` implements the Spectral Entropy classification logic.

---
**Conclusion**:
All claims are enabled by functional Python code capable of running on standard hardware, satisfying the 35 U.S.C. § 112 enablement requirement.
