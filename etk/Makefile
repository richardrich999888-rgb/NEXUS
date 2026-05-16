# ETK — Regulator-grade build stack. Single source of truth for builds and verification.
# Usage: make build | repro | sign | verify | sbom

.PHONY: build repro sign verify sbom clean

build:
	cargo build --release --locked

repro:
	bash build/reproducible.sh

sign:
	bash build/sign_artifacts.sh

verify:
	bash build/verify_artifacts.sh

sbom:
	bash build/sbom.sh

clean:
	cargo clean
	rm -f build/build_hash.txt build/etk.sig
