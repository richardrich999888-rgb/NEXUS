# FYNTRAX Control Theory

## The Problem

In disaggregated RAN (O-RAN), control loops experience variable latency d(t).

If d(t) > T_c (channel coherence time), AI controllers act on stale state.
This leads to:
- Oscillation
- Spectral collapse
- Unstable resource allocation

## Lyapunov-Based Solution

### System State

Network stress state vector:
```
x = [q, T, i]ᵀ
```
Where:
- q = queue length (normalized)
- T = temperature (normalized)
- i = interference level (normalized)

### Lyapunov Function

Quadratic Lyapunov candidate:
```
V(x) = xᵀ P x
```

Where P is a positive definite matrix.

For simple case, P = I (identity):
```
V(x) = ||x||² = q² + T² + i²
```

### Stability Condition

For exponential stability:
```
V(x_{t+1}) - V(x_t) < -α · V(x_t)
```

Where α ∈ (0, 1) is the minimum decay rate.

This guarantees:
```
V(x_t) < V(x_0) · (1-α)^t → 0 as t → ∞
```

## AI Action Filtering

### Decision Process

1. AI proposes action u → predicted state x_next
2. Compute Lyapunov drift: ΔV = V(x_next) - V(x)
3. Check safety: ΔV < -α · V(x)?
4. If safe: execute action
5. If unsafe: 
   a. Try scaled action u' = β·u
   b. If still unsafe: maintain current state

### Safety Guarantee

Even if the AI is a "black box":
- All executed actions satisfy stability constraint
- System state remains bounded (BIBO stability)
- No trust required in AI internal logic

## Implementation

```python
class LyapunovController:
    def __init__(self, P, alpha=0.1):
        self.P = P
        self.alpha = alpha
    
    def V(self, x):
        return x.T @ self.P @ x
    
    def is_safe(self, x, x_next):
        drift = self.V(x_next) - self.V(x)
        return drift < -self.alpha * self.V(x)
    
    def filter_action(self, x, proposed_x_next):
        if self.is_safe(x, proposed_x_next):
            return proposed_x_next
        return x  # Maintain current state
```

## O-RAN Integration

### RIC xApp Architecture

```
xApp (ML Model) → Action → Lyapunov Filter → O1/E2 → gNB
                              ↓
                    Rejected actions logged
```

### Latency Considerations

- E2 latency: ~10 ms typical
- Channel coherence: ~1 ms at highway speeds
- Prediction horizon must exceed latency

### Fallback Controllers

When AI is repeatedly rejected:
1. Simple proportional controller
2. Conservative fixed policy
3. Last-known-good state
