# SYNTRIASS Path 6 — System Architecture

**Real-Time Generative AI Preview Engine**

## Design Principles (Non-Negotiable)

1. **Never block on full generation** — Preview is non-blocking
2. **Preview must arrive < 300 ms** — Human perception threshold
3. **User input must affect ongoing generation** — Live conditioning injection
4. **No regeneration unless explicitly requested** — Continuity is key
5. **Works with existing models** — Diffusers-first, no fork required

## System Architecture

```
┌─────────────┐
│  Frontend   │  ← sliders, prompt edits, scrubber
│  (Gradio)   │
└──────┬──────┘
       │ WebSocket (bi-directional)
┌──────▼──────┐
│ Preview API │  ← control plane
└──────┬──────┘
       │
┌──────▼────────────────────────────────────────────┐
│        SYNTRIASS PREVIEW ENGINE (CORE)              │
│                                                     │
│  ┌────────────┐   ┌────────────┐   ┌────────────┐ │
│  │ Inference  │→→ │ Preview    │→→ │ Temporal   │ │
│  │ Tap        │   │ Decoder    │   │ Interpol.  │ │
│  └────────────┘   └────────────┘   └────────────┘ │
│          ↓                     ↑                  │
│   Conditioning Injection  ←────┘                  │
└─────────────────────────────────────────────────────┘
       │
┌──────▼──────┐
│ Preview Bus │  → WebSocket Stream
└─────────────┘
```

## Module Breakdown

### Module A — Inference Tap (`core/inference_tap.py`)

**Purpose:** Intercept diffusion while it runs.

**What it hooks:**
- Denoising loop
- Latent tensor at step t
- Scheduler state

**Implementation:**
- Monkey-patch Diffusers pipeline
- No fork required
- Zero restart, zero extra inference

**Key Classes:**
- `PreviewDiffusionLoop`: Extracts latent snapshots
- `DiffusionPipelineHook`: Patches pipeline automatically

### Module B — Preview Decoder (`preview/fast_decoder.py`)

**Goal:** Decode something useful in <50 ms, not perfection.

**Techniques:**
- Decode at 1/8 or 1/16 resolution
- Decode only Y channel (luma) first
- Quantize latents aggressively
- Skip refinement layers

**Key Classes:**
- `FastPreviewDecoder`: Ultra-fast decode (<50ms)
- `AdaptiveDecoder`: Quality adapts to time budget

### Module C — Temporal Interpolator (`preview/temporal.py`)

**SYNTRIASS MOAT:** Without this, preview feels jittery and fake.

**What it does:**
- Interpolates between sparse latent snapshots
- Predicts near-future frames
- Makes time continuous

**Mathematical Model:**
```
L̂(t+Δ) = L(t) + (L(t) − L(t−1)) · Δ
```

**Interpolation Methods:**
- Linear
- Polynomial (smoothstep)
- Harmonic (sine-based)

**Key Classes:**
- `TemporalInterpolator`: Smooths sparse frames

### Module D — Conditioning Injection (`core/conditioning.py`)

**INTERACTIVITY CORE:** This is why Path 6 is revolutionary.

**User changes:**
- Prompt text
- Style sliders
- Emotion / motion weights

**What you do:**
- Modify conditioning vectors
- Blend over time
- Never restart diffusion

**Key Rule:** Conditioning is a signal, not a constant.

**Key Classes:**
- `ConditioningInjector`: Live user control
- Style modifiers: `emotion_modifier`, `motion_modifier`, `detail_modifier`

### Module E — Preview Bus (`preview/stream.py`)

**Streaming Backbone:** Low-tech. Rock solid.

**Responsibilities:**
- Collect preview frames
- Throttle intelligently
- Stream to frontend

**Implementation:**
- Async queue
- WebSocket
- Drop frames if needed (never block)

**Key Classes:**
- `PreviewBus`: Async frame queue
- `FrameEncoder`: WebSocket encoding

### Module F — Preview Scheduler (`core/scheduler.py`)

**Orchestration Layer:** Makes everything work together.

**Coordinates:**
- Inference tap → Preview decoder → Temporal interpolator → Stream
- Conditioning injection (user control)

**Key Classes:**
- `PreviewScheduler`: Main orchestrator

## Data Flow (Frame-Level)

```
Latent(t)
   ↓
Inference Tap
   ↓
Fast Decode → Preview Frame → Interpolator → Stream
   ↓
Conditioning Injection (if user acts)
   ↓
Latent(t+1)
```

**No blocking. No waiting. No restart.**

## Repo Structure

```
syntriass/
├── core/
│   ├── inference_tap.py      # Module A
│   ├── conditioning.py        # Module D
│   ├── scheduler.py           # Orchestration
│   └── __init__.py
│
├── preview/
│   ├── fast_decoder.py        # Module B
│   ├── temporal.py            # Module C
│   ├── stream.py              # Module E
│   └── __init__.py
│
├── api/
│   ├── websocket.py           # WebSocket server
│   └── control.py             # REST API
│
├── front/
│   └── gradio_app.py          # Gradio UI
│
├── demos/
│   ├── image_preview.py       # Image demo
│   ├── video_preview.py       # Video demo (placeholder)
│   └── audio_preview.py       # Audio demo (placeholder)
│
├── patch/
│   └── diffusers_hook.py      # Pipeline patching
│
├── requirements.txt
├── README.md
└── ARCHITECTURE.md
```

## What Makes This Defensible

**Not the UI. Not the decoder.**

**The moat is:**
- Live conditioning injection
- Temporal latent continuity
- Preview without regeneration

**Patent Claims:**
- "Method for interactive generative inference"
- "Continuous conditioning modification during diffusion"
- "Temporal interpolation of latent states for real-time preview"

## Failure Modes (And How We Avoid Them)

| Failure | Why it happens | Fix |
|---------|---------------|-----|
| Preview lags | Decode too heavy | Aggressive downshift |
| Flicker | Sparse frames | Interpolator |
| Restart on edit | Naive UX | Conditioning blend |
| GPU stall | Blocking stream | Drop frames |
| "Cool but useless" | No control | Sliders live |

## Usage Example

```python
from diffusers import StableDiffusionPipeline
from syntriass.patch.diffusers_hook import patch_pipeline
from syntriass.core.scheduler import create_preview_scheduler

# Load pipeline
pipeline = StableDiffusionPipeline.from_pretrained("runwayml/stable-diffusion-v1-5")

# Patch for preview
preview_loop = patch_pipeline(pipeline, preview_interval=4)

# Create scheduler
scheduler = create_preview_scheduler(
    pipeline=pipeline,
    preview_interval=4,
    target_resolution=(64, 64),
)

# Start preview
await scheduler.start()

# Generate (preview streams automatically)
image = pipeline("a beautiful landscape")
```

## Integration with Other Paths

- **Path 12 (Audio)** → Same preview bus + temporal core
- **Path 7 (Temporal Engine)** → This is its first incarnation
- **Path 9 (Cinema)** → This architecture, scaled
- **Path 11 (OS)** → This becomes the kernel story post-acquisition

**Path 6 is the front door to the empire.**

---

*"The moment generative AI became interactive."*

