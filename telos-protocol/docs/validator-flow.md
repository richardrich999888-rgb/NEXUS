# TELOS Validator Flow Specification

## Validation Flow

```
Agent → Gateway → Validators (parallel) → Consensus → Ledger
```

## Validator Selection

- **Stake-weighted random selection**
- **Domain expertise matching** (hierarchical)
- **Minimum validators**: 5
- **Target weight**: 67%

## Validation Steps

1. **Entropy Proof** - VDF/beacon verification, amount check
2. **Authority Chain** - Status, expiration, signature, scope
3. **Constraints** - Temporal, rate-limit, approval, custom

## Consensus

```
approval_weight ≥ 67% → APPROVED
rejection_weight > 33% → REJECTED
else → PENDING
```

## Slashing Fractions

| Offense | Slash |
|---------|-------|
| Downtime | 1% |
| False attestation | 10% |
| Double attestation | 25% |
| Collusion | 50% |

## Collusion Detection

- Build voting correlation matrix over 1000 attestations
- Flag pairs with >98% agreement
- Report with confidence (high if n>200)
