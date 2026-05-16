# Contributing to NEXUS

Welcome to the NEXUS community! We are excited to have you contribute to the future of distributed execution.

## Code of Conduct

By participating in this project, you agree to abide by the [SYNTRIASS Code of Conduct](https://syntriass.com/coc).

## How Can I Contribute?

### Reporting Bugs
If you find a bug, please open an issue on our GitHub repository. Include a clear description, steps to reproduce, and any relevant logs.

### Suggesting Enhancements
We welcome ideas for new features! Please open a "Feature Request" issue to discuss your proposal.

### Pull Requests
1.  **Fork the repository** and create your branch from `main`.
2.  **Ensure your code follows the project's style**. We use `cargo fmt`.
3.  **Add tests** for any new functionality.
4.  **Run the full test suite**: `cargo test --workspace`.
5.  **Submit a Pull Request** with a clear description of the changes.

## Development Environment

### Prerequisites
- Rust (latest stable)
- Python 3.10+ (for `nexus-telecom`)
- Docker (optional, for deployment testing)

### Building
```bash
cargo build
```

### Testing
```bash
cargo test --workspace
```

## Structure of the Monorepo

- `nexus-core`: Core causal algebra and cost optimizer.
- `nexus-pcu`: PCU/USO primitives and identity.
- `nexus-sync`: Causal synchronization engine.
- `vectra`: Lossless compression engine.
- `nexus-telecom`: Python SDK for 6G/Edge.

---
© 2025 SYNTRIASS Labs Pvt Ltd.
