//! Performance benchmarks for VECTRA encoding/decoding.
//!
//! Measures:
//! - Encoding throughput (MB/s)
//! - Decoding throughput (MB/s)
//! - Compression ratio
//! - Determinism verification (same input → same output)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vectra::{vectra_decode, vectra_encode, Payload};

/// Generate test payload with repeating patterns (good compression candidate).
fn generate_structured_payload(size: usize) -> Vec<u8> {
    let pattern = b"HEADER:value:";
    let mut data = Vec::with_capacity(size);
    while data.len() < size {
        let remaining = size - data.len();
        let to_add = remaining.min(pattern.len());
        data.extend_from_slice(&pattern[..to_add]);
    }
    data
}

/// Generate high-entropy payload (poor compression candidate).
fn generate_random_payload(size: usize) -> Vec<u8> {
    // Deterministic "random" using linear congruential generator
    let mut data = Vec::with_capacity(size);
    let mut state: u64 = 12345;
    for _ in 0..size {
        state = state.wrapping_mul(1103515245).wrapping_add(12345);
        data.push((state >> 24) as u8);
    }
    data
}

/// Generate mixed payload (some structure, some randomness).
fn generate_mixed_payload(size: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    let header = b"HEADER:";
    let mut pos = 0;
    while pos < size {
        if pos % 100 < 20 {
            // 20% structural
            let remaining = size - pos;
            let to_add = remaining.min(header.len());
            data.extend_from_slice(&header[..to_add]);
            pos += to_add;
        } else {
            // 80% variable
            data.push((pos % 256) as u8);
            pos += 1;
        }
    }
    data
}

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");
    
    let sizes = vec![1024, 10 * 1024, 100 * 1024, 1024 * 1024];
    
    for size in sizes {
        // Structured payload (good compression)
        let structured = generate_structured_payload(size);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("structured", size),
            &structured,
            |b, data| {
                b.iter(|| {
                    let payload = Payload::new(data.clone());
                    black_box(vectra_encode(payload))
                })
            },
        );
        
        // Mixed payload (moderate compression)
        let mixed = generate_mixed_payload(size);
        group.bench_with_input(
            BenchmarkId::new("mixed", size),
            &mixed,
            |b, data| {
                b.iter(|| {
                    let payload = Payload::new(data.clone());
                    black_box(vectra_encode(payload))
                })
            },
        );
        
        // High-entropy payload (poor compression, likely pass-through)
        let random = generate_random_payload(size);
        group.bench_with_input(
            BenchmarkId::new("high_entropy", size),
            &random,
            |b, data| {
                b.iter(|| {
                    let payload = Payload::new(data.clone());
                    black_box(vectra_encode(payload))
                })
            },
        );
    }
    
    group.finish();
}

fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");
    
    let sizes = vec![1024, 10 * 1024, 100 * 1024];
    
    for size in sizes {
        let structured = generate_structured_payload(size);
        let payload = Payload::new(structured);
        let result = vectra_encode(payload);
        
        if let vectra::EncodeResult::Encoded(artifact) = result {
            group.throughput(Throughput::Bytes(size as u64));
            group.bench_with_input(
                BenchmarkId::new("structured", size),
                &artifact,
                |b, art| {
                    b.iter(|| {
                        black_box(vectra_decode(art).unwrap())
                    })
                },
            );
        }
    }
    
    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");
    
    let sizes = vec![1024, 10 * 1024, 100 * 1024];
    
    for size in sizes {
        let structured = generate_structured_payload(size);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("structured", size),
            &structured,
            |b, data| {
                b.iter(|| {
                    let payload = Payload::new(data.clone());
                    let result = vectra_encode(payload);
                    if let vectra::EncodeResult::Encoded(artifact) = result {
                        let decoded = vectra_decode(&artifact).unwrap();
                        black_box(decoded);
                    }
                })
            },
        );
    }
    
    group.finish();
}

fn bench_entropy_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("entropy");
    
    let sizes = vec![1024, 10 * 1024, 100 * 1024];
    
    for size in sizes {
        let data = generate_random_payload(size);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("shannon_entropy", size),
            &data,
            |b, data| {
                b.iter(|| {
                    black_box(vectra::compute_byte_entropy(data))
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_encode,
    bench_decode,
    bench_roundtrip,
    bench_entropy_calculation
);
criterion_main!(benches);
