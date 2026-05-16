# Contributing to VECTRA

Thank you for your interest in contributing to VECTRA!

## ⚠️ Important Legal Requirements

VECTRA is **proprietary software** protected by patents and trade secrets. Before contributing, you **MUST**:

1. **Sign the Contributor License Agreement (CLA)** - See below
2. **Agree to patent assignment** - Any inventions in contributions become property of SYNTRIASS
3. **Agree to confidentiality** - Do not disclose VECTRA algorithms or trade secrets

## Contributor License Agreement (CLA)

**ALL contributors MUST sign this CLA before any contributions can be accepted.**

### CLA Terms

By submitting any contribution (code, documentation, feedback, ideas) to VECTRA, you agree to the following terms:

#### 1. Definitions
- **"You"** means the individual or entity submitting the contribution
- **"Contribution"** means any code, documentation, feedback, ideas, or other materials you submit
- **"SYNTRIASS"** means SYNTRIASS Labs Private Limited

#### 2. Grant of Rights

You hereby grant to SYNTRIASS:

**a) Copyright License**
- A perpetual, worldwide, royalty-free, irrevocable, exclusive license to use, reproduce, modify, create derivative works from, publicly display, publicly perform, sublicense, and distribute your Contributions

**b) Patent License**
- A perpetual, worldwide, royalty-free, irrevocable patent license to make, use, sell, offer for sale, import, and otherwise transfer your Contributions
- Rights to file patent applications covering inventions in your Contributions
- Full ownership of any patents granted on inventions in your Contributions

**c) Moral Rights Waiver**
- You waive all moral rights in your Contributions to the maximum extent permitted by law

#### 3. Ownership Transfer

You agree that:
- All Contributions become the sole and exclusive property of SYNTRIASS
- SYNTRIASS owns all intellectual property rights in Contributions
- You retain no rights to use Contributions independently of VECTRA

#### 4. Representations

You represent and warrant that:
- You have the legal authority to enter into this CLA
- Your Contributions are your original work
- Your Contributions do not infringe any third-party rights
- You have not granted conflicting rights to anyone else
- Your employer (if applicable) has waived all rights to your Contributions

#### 5. Employer Rights

If your employer has rights to intellectual property you create, you represent that:
- You have received permission to make Contributions on behalf of your employer, OR
- Your employer has waived such rights for your Contributions to VECTRA

#### 6. No Compensation

You understand and agree that:
- Contributions are voluntary
- You will not receive any compensation for Contributions
- SYNTRIASS has no obligation to use your Contributions

#### 7. Confidentiality

You agree to:
- Keep all VECTRA trade secrets and confidential information confidential
- Not reverse engineer, decompile, or disassemble VECTRA
- Not disclose VECTRA algorithms or implementation details
- Not use VECTRA confidential information for any purpose other than contributing

## How to Sign the CLA

### Option 1: Electronic Signature (Recommended)

1. Go to: https://vectra.syntriass.com/cla
2. Fill out the CLA form
3. Electronically sign the agreement
4. You'll receive a confirmation email

### Option 2: Manual Signature

1. Download the CLA: [CLA.pdf](https://vectra.syntriass.com/CLA.pdf)
2. Print, sign, and scan the document
3. Email signed CLA to: legal@syntriass.com
4. Wait for confirmation before submitting contributions

## Contribution Guidelines

### What We Accept

✅ **Bug Fixes**: Fixes for confirmed bugs (must not change algorithms)
✅ **Documentation**: Improvements to docs, examples, tutorials
✅ **Test Cases**: Additional test cases for existing functionality
✅ **Performance Optimizations**: Improvements that don't change behavior
✅ **Platform Support**: Ports to new platforms (with SYNTRIASS approval)

### What We Don't Accept

❌ **Algorithm Changes**: Core algorithm modifications (patent-protected)
❌ **New Features**: Feature additions without prior SYNTRIASS approval
❌ **Forks**: We do not accept contributions to forked versions
❌ **License Changes**: Any changes to licensing terms
❌ **Watermark Removal**: Removal of copyright or patent notices

### Contribution Process

1. **Sign CLA** (required before first contribution)

2. **Discuss First**
   - Open an issue describing your proposed contribution
   - Wait for SYNTRIASS approval before starting work
   - Major contributions require design review

3. **Development**
   - Fork the repository (for your own use only, not for distribution)
   - Create a feature branch
   - Follow existing code style and conventions
   - Add tests for your changes
   - Ensure all tests pass

4. **Submit Pull Request**
   - Provide clear description of changes
   - Reference related issues
   - Include CLA signature confirmation
   - Wait for SYNTRIASS review

5. **Code Review**
   - SYNTRIASS will review your contribution
   - Address any feedback
   - Once approved, SYNTRIASS will merge (you cannot merge)

6. **Recognition**
   - Contributors will be listed in CONTRIBUTORS.md
   - Significant contributions may receive public acknowledgment

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- Git
- Signed CLA on file

### Building

```bash
git clone https://github.com/syntriass/vectra.git
cd vectra/vectra
cargo build --release
cargo test
```

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

### Code Style

- Follow Rust standard style (use `cargo fmt`)
- Run clippy: `cargo clippy -- -D warnings`
- Document public APIs with `///` doc comments
- Write tests for new functionality

## Patent-Protected Areas

The following areas are **heavily patented** and contributions MAY NOT modify these algorithms without explicit written permission from SYNTRIASS:

🔒 **EBTA (Entropy-Bounded Tensor Algebra)**
- Files: `ebta.rs`
- Patent: US Provisional 63/XXX,XXX

🔒 **EBTA-X (Adaptive Multi-Dimensional Entropy)**  
- Files: `ebta.rs` (adaptive features)
- Patent: US Provisional 63/XXX,XXX

🔒 **Deterministic Compression Pipeline**
- Files: `encode.rs`, `decode.rs`
- Patent: US Provisional 63/XXX,XXX

🔒 **FEE (Fractal Entropy Encoding)**
- Files: `fee.rs`
- Trade Secret + Patent Pending

🔒 **SPE (Symbolic Predictor Engine)**  
- Files: `spe.rs`
- Trade Secret + Patent Pending

## Trade Secrets

The following are **trade secrets** of SYNTRIASS:

🔐 **Entropy Threshold Calculations**: Proprietary formulas for H_MAX
🔐 **Pattern Detection Algorithms**: Exact decomposition logic
🔐 **Predictor Parameters**: Tuning constants and weights
🔐 **Performance Optimizations**: Specific implementation techniques

**DO NOT**:
- Reverse engineer these algorithms
- Publish analysis of trade secret implementations
- Share knowledge of trade secret details
- Attempt to recreate trade secrets independently

## Enforcement

SYNTRIASS actively monitors for:
- Unauthorized use of patented algorithms
- Disclosure of trade secrets
- Violation of license terms  
- Unauthorized forks or derivatives

Violations may result in:
- Immediate termination of rights to contribute
- Removal of your contributions
- Legal action, including patent infringement litigation
- Damages and injunctive relief

## Questions?

- **General Questions**: community@syntriass.com
- **Legal/CLA Questions**: legal@syntriass.com
- **Patent Licensing**: patents@syntriass.com
- **Security Issues**: security@syntriass.com (DO NOT open public issues)

## License

By contributing, you agree that your contributions will be licensed under the [VECTRA Proprietary License](LICENSE).

---

**Remember**: You MUST sign the CLA before your first contribution!

Sign here: https://vectra.syntriass.com/cla



