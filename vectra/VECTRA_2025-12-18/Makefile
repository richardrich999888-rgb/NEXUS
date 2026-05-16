.PHONY: all rust cpp python test clean

all: rust cpp python

rust:
	cd vectra && cargo build --release

cpp:
	cd cpp && mkdir -p build && cd build && cmake .. && make

python:
	cd python && pip install -e .

test:
	cd vectra && cargo test
	cd python && pytest tests/
	cd cpp/build && ctest || true

clean:
	cd vectra && cargo clean
	rm -rf cpp/build
	cd python && rm -rf dist *.egg-info








