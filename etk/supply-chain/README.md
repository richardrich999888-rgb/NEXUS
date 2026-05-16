# Supply-chain artifacts

- **provenance.json** — Build lineage: source commit, compiler, binary hash, profile. Updated by `make repro` when `jq` is available.
- **SBOM.json** — CycloneDX placeholder; fill via `cargo cyclonedx` or CI.
- **deps.txt** — Dependency list (from `make sbom` when `cargo metadata`/`cargo tree` available).

Do not commit private keys or signing secrets. Commit `public.pem` for auditors; keep `private.pem` offline.
