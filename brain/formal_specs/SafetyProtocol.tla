---------------------- MODULE SafetyProtocol ----------------------
EXTENDS Integers, Sequences

(***************************************************************************
 * Formal Specification of the NEXUS Autonomic Safety Protocol.            *
 * This protocol manages mode transitions based on arousal levels.         *
 ***************************************************************************)

VARIABLES 
    mode,           \* Current AutonomicMode: CALM, ACT, EMERGENCY, RECOVERY
    arousal,        \* Current arousal level (scaled integer 0-100)
    duration,       \* Time (ticks) in current mode
    reflex_queue    \* Pending emergency reflexes

\* Constants for thresholds
CONSTANTS
    CALM_THRESHOLD,
    ACT_THRESHOLD,
    EMERGENCY_THRESHOLD,
    MIN_DURATION,
    MAX_AROUSAL

\* Possible Modes
MODES == {"CALM", "ACT", "EMERGENCY", "RECOVERY"}

(***************************************************************************
 * Initialization                                                          *
 ***************************************************************************)
Init ==
    / \ mode = "CALM"
    / \ arousal = 0
    / \ duration = 0
    / \ reflex_queue = << >>

(***************************************************************************
 * State Transitions                                                       *
 ***************************************************************************)

\* Simulate external stimulation or internal stress
IncreaseArousal ==
    / \ arousal < MAX_AROUSAL
    / \ arousal' = MIN(arousal + 10, MAX_AROUSAL)
    / \ UNCHANGED <<mode, duration, reflex_queue>>

DecreaseArousal ==
    / \ arousal > 0
    / \ arousal' = MAX(arousal - 5, 0)
    / \ UNCHANGED <<mode, duration, reflex_queue>>

\* Internal Tick: updates duration and checks for transitions
Tick ==
    LET 
        \* Logic to determine the target mode based on arousal
        TargetMode == 
            IF arousal >= EMERGENCY_THRESHOLD THEN "EMERGENCY"
            ELSE IF arousal >= ACT_THRESHOLD THEN 
                IF mode = "EMERGENCY" THEN "RECOVERY" ELSE "ACT"
            ELSE IF arousal <= CALM_THRESHOLD THEN "CALM"
            ELSE mode
    IN
    / \ duration' = IF TargetMode # mode THEN 0 ELSE duration + 1
    / \ mode' = IF duration >= MIN_DURATION THEN TargetMode ELSE mode
    / \ UNCHANGED <<arousal, reflex_queue>>

\* Emergency Reflex Action
TriggerReflex ==
    / \ mode = "EMERGENCY"
    / \ arousal >= EMERGENCY_THRESHOLD
    / \ reflex_queue' = Append(reflex_queue, "EmergencyBrake")
    / \ UNCHANGED <<mode, duration, arousal>>

(***************************************************************************
 * Properties to Verify                                                    *
 ***************************************************************************)

\* Type Invariant
TypeOK ==
    / \ mode \in MODES
    / \ arousal \in 0..MAX_AROUSAL
    / \ duration \in Nat
    / \ IsSafeSequence(reflex_queue)

\* Safety Property 1: Emergency Mode reached if arousal exceeds threshold
EmergencySafety ==
    (arousal >= EMERGENCY_THRESHOLD /\ duration >= MIN_DURATION) => mode = "EMERGENCY"

\* Safety Property 2: No ghost states
NoGhostStates == mode \in MODES

\* Liveness Property: System eventually returns to CALM if arousal stays low
ReturnsToCalm ==
    (arousal <= CALM_THRESHOLD) ~> (mode = "CALM")

=============================================================================
