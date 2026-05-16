# SYNTRIASS Path 6 — Real-Time Generative AI Preview Engine

**The moment generative AI became interactive.**

## Design Principles (Non-Negotiable)

1. **Never block on full generation** — Preview is non-blocking
2. **Preview must arrive < 300 ms** — Human perception threshold
3. **User input must affect ongoing generation** — Live conditioning injection
4. **No regeneration unless explicitly requested** — Continuity is key
5. **Works with existing models** — Diffusers-first, no fork required

## Architecture

```
Frontend (Gradio) → WebSocket → Preview API → Preview Engine
                                              ↓
                    Inference Tap → Preview Decoder → Temporal Interpolator
                                              ↑
                                    Conditioning Injection
```

## Core Modules

- **Inference Tap**: Intercepts diffusion loop, extracts latent snapshots
- **Preview Decoder**: Fast decode (<50ms) at reduced resolution
- **Temporal Interpolator**: Smooths sparse frames into continuous preview
- **Conditioning Injection**: Live user control without restart
- **Preview Bus**: Streaming backbone (async queue + WebSocket)
- **Finalization Path**: Clean transition from preview to final output

## Quick Start

```bash
# Install dependencies
pip install -r requirements.txt

# Run demo
python demos/image_preview.py

# Run with Gradio UI
python front/gradio_app.py
```

## Patent Claims

- "Method for interactive generative inference"
- "Continuous conditioning modification during diffusion"
- "Temporal interpolation of latent states for real-time preview"

---

*This is an execution-layer system, not a model.*

