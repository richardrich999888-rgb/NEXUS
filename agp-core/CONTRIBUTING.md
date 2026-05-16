# Contributing to AGP-CORE

Thank you for your interest in contributing to AGP-CORE!

## Getting Started

### Prerequisites
- Python 3.11+
- PostgreSQL 14+
- Redis 7+
- Node.js 18+ (for smart contracts)

### Development Setup
```bash
# Clone repository
git clone https://github.com/agp-core/agp-core.git
cd agp-core

# Create virtual environment
python -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt
pip install -r requirements-dev.txt

# Setup database
createdb agp_core_dev
psql agp_core_dev < scripts/init-db.sql

# Run tests
pytest tests/
```

---

## Code Structure

```
agp-core/
├── src/
│   ├── api/v1/         # API endpoints
│   ├── core/           # Core engine
│   ├── models/         # Pydantic models
│   ├── services/       # Business logic
│   └── compliance/     # Compliance framework
├── contracts/          # Solidity smart contracts
├── sdk/                # Python SDK
├── tests/              # Test suite
├── docs/               # Documentation
└── deploy/             # Deployment scripts
```

---

## Contribution Guidelines

### Code Style
- Python: PEP 8, Black formatting
- Solidity: OpenZeppelin style
- Type hints required for all functions
- Docstrings for public APIs

### Pull Request Process
1. Fork the repository
2. Create feature branch: `git checkout -b feature/my-feature`
3. Write tests for new functionality
4. Ensure all tests pass: `pytest`
5. Update documentation if needed
6. Submit PR with clear description

### Commit Messages
```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Example:
```
feat(swarm): add emergent pattern detection

Implements pattern recognition for high coordination
and leadership emergence in swarms.

Closes #123
```

---

## Testing

### Unit Tests
```bash
pytest tests/test_reputation_engine.py -v
```

### Integration Tests
```bash
pytest tests/integration/ -v
```

### Contract Tests
```bash
cd contracts
npx hardhat test
```

---

## Architecture Decisions

When proposing significant changes, document your decision:

1. **Context**: What problem are you solving?
2. **Decision**: What approach did you choose?
3. **Consequences**: What are the trade-offs?

---

## Areas for Contribution

### High Priority
- [ ] Additional ML models for prediction
- [ ] More blockchain network support
- [ ] Performance optimizations
- [ ] Security hardening

### Medium Priority
- [ ] Additional compliance frameworks
- [ ] New visualization tools
- [ ] Enhanced documentation
- [ ] SDK for other languages

### Good First Issues
- Documentation improvements
- Test coverage expansion
- Type hint additions
- Code cleanup

---

## Community

- **Discord**: https://discord.gg/agp-core
- **Discussions**: GitHub Discussions
- **Issues**: GitHub Issues

---

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
