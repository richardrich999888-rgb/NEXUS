# FYNTRAX Energy Model

## Legacy RAN Energy Problem

### Power Consumption Model
```
P(L) = P_static + P_dynamic × L
```

| Component | Typical Value | Percentage |
|-----------|---------------|------------|
| P_static | 500-800 W | 50-70% |
| P_dynamic | 200-500 W | 30-50% |

### The Energy Trap

At zero load:
```
P(0) = P_static ≈ 500 W
```

This means:
- Empty cells burn massive energy
- Network densification increases absolute energy
- Energy per bit diverges to infinity as load → 0

## FYNTRAX Energy Model

### Receiver-Initiated Architecture
```
P(L) = P_WuR + P_active × L
```

| Component | Value | Reduction |
|-----------|-------|-----------|
| P_WuR | ~1 μW | 10⁹× less |
| P_active | 1000 W | (when needed) |

### Zero-Watt Idle

At zero load:
```
P_FYNTRAX(0) = P_WuR ≈ 10⁻⁶ W
```

Ratio vs legacy:
```
P_FYNTRAX(0) / P_legacy(0) = 10⁻⁶ / 500 = 2 × 10⁻⁹
```

## Energy Savings Analysis

### 24-Hour Idle Scenario

| Metric | Legacy | FYNTRAX | Savings |
|--------|--------|---------|---------|
| Energy | 12 kWh | 24 μWh | 99.9999998% |
| Power | 500 W | 1 μW | 500,000,000× |

### Network Scale (100,000 cells)

Assuming 70% average idle time:

| Metric | Legacy | FYNTRAX |
|--------|--------|---------|
| Annual Energy | 3.07 TWh | ~0 |
| Cost ($0.10/kWh) | $307M | ~$0 |
| CO2 (0.4 kg/kWh) | 1.23 Mt | ~0 |

## Power Amplifier Physics

### The PAPR Problem

5G NR uses CP-OFDM with:
```
PAPR ≈ 10-12 dB
```

To maintain linearity, PAs operate with backoff:
```
η_PA ≈ 35-55%
```

Heat dissipation:
```
P_heat = (1 - η_PA) × P_DC
```

### Implication

Even transmitting empty reference signals wastes 45-65% as heat.
The only solution is to **stop transmitting entirely** when not needed.
This is exactly what FYNTRAX does.

## Energy Per Bit

### Legacy
```
E_b = P / (L × C)
```

As L → 0: E_b → ∞

### FYNTRAX
```
E_b = (P_WuR + P_active × L) / (L × C)
```

As L → 0: E_b → P_WuR / 0, but since P_WuR ≈ 0, ratio stays finite.

At any non-zero load, FYNTRAX E_b is always finite.
