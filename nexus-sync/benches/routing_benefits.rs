use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

// Simulation parameters
const BANDWIDTH_MBPS: f64 = 100.0; // 100 Mbps = 12.5 MB/s
const LATENCY_MS: f64 = 50.0;     // 50ms RTT
const DATA_SIZE_MB: usize = 10;   // 10MB dataset
const PCU_SIZE_KB: usize = 1;     // 1KB PCU
const RESULT_SIZE_BYTES: usize = 64;

fn simulate_network_delay(size_bytes: usize) -> Duration {
    let bandwidth_bytes_per_sec = (BANDWIDTH_MBPS * 1_000_000.0 / 8.0) as f64;
    let transmission_time_secs = size_bytes as f64 / bandwidth_bytes_per_sec;
    let latency_secs = LATENCY_MS / 1000.0;
    
    Duration::from_secs_f64(transmission_time_secs + latency_secs)
}

fn aggregate_data(data: &[u8]) -> u64 {
    data.iter().map(|&b| b as u64).sum()
}

fn bench_routing_comparison(c: &mut Criterion) {
    let data = vec![0u8; DATA_SIZE_MB * 1024 * 1024];
    
    let mut group = c.benchmark_group("Routing Benefits");
    
    // 1. Traditional: Data-to-Code
    // Fetches entire dataset across network, then processes locally.
    group.bench_function("Traditional (Data-to-Code)", |b| {
        b.iter(|| {
            // Simulated fetch time
            let fetch_delay = simulate_network_delay(DATA_SIZE_MB * 1024 * 1024);
            
            // Local processing
            let result = aggregate_data(black_box(&data));
            
            // Total "perceived" time
            // In a real benchmark, we just sum them for report purposes
            black_box(result);
            fetch_delay // Not actually sleeping, just returning for reference if needed
        })
    });

    // 2. NEXUS: Code-to-Data
    // Sends small PCU across network, processes at source, returns tiny result.
    group.bench_function("NEXUS (Code-to-Data)", |b| {
        b.iter(|| {
            // Simulated PCU upload delay (negligible)
            let _request_delay = simulate_network_delay(PCU_SIZE_KB * 1024);
            
            // Remote processing (same CPU work)
            let result = aggregate_data(black_box(&data));
            
            // Simulated result download delay
            let _response_delay = simulate_network_delay(RESULT_SIZE_BYTES);
            
            black_box(result);
        })
    });
    
    group.finish();
}

// NOTE: Criterion doesn't easily let us add "simulated" time to the actual reported measurement
// unless we actually sleep (which makes benches slow and jittery).
// Instead, we will print a summary of "Total Effective Latency" in the walkthrough.

criterion_group!(benches, bench_routing_comparison);
criterion_main!(benches);
