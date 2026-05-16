//! Main execution engine for NEXUS Portable Computation Units.
//!
//! The `PcuExecutor` integrates Wasmtime to provide a secure, sandboxed,
//! and resource-bounded environment for executing PCUs.

use crate::error::{ExecutorError, ExecutorResult};
use nexus_pcu::PCU;
use crate::proof::ExecutionProof;
use nexus_pcu::NodeId;
use crate::types::{ExecutionContext, ExecutionResult, ExecutionResponse};
use crate::semantic_cache::SemanticCache;
use crate::guard::{ExecutionGuard, GuardDecision};
use crate::host_functions;

use wasmtime::{Config, Engine, Instance, Linker, Module, Store};
use std::time::{Duration, Instant};
use std::sync::Arc;
use tracing::{instrument, debug};

/// Builder for PcuExecutor.
pub struct ExecutorBuilder {
    node_id: NodeId,
    signing_key: ed25519_dalek::SigningKey,
    host: Arc<dyn crate::NexusHost>,
    cache_capacity: usize,
    guard: Option<Arc<dyn ExecutionGuard>>,
}

impl ExecutorBuilder {
    /// Create new builder.
    pub fn new(node_id: NodeId, signing_key: ed25519_dalek::SigningKey, host: Arc<dyn crate::NexusHost>) -> Self {
        Self {
            node_id,
            signing_key,
            host,
            cache_capacity: 1000,
            guard: None,
        }
    }

    /// Set cache capacity.
    pub fn cache_capacity(mut self, capacity: usize) -> Self {
        self.cache_capacity = capacity;
        self
    }

    /// Set execution guard (biological / accountability gate). When set, every execute() passes through it.
    pub fn with_guard(mut self, guard: Arc<dyn ExecutionGuard>) -> Self {
        self.guard = Some(guard);
        self
    }

    /// Production build: executor with a default guard. Use for deployment; do not deploy without a guard.
    pub fn production(node_id: NodeId, signing_key: ed25519_dalek::SigningKey, host: Arc<dyn crate::NexusHost>) -> Self {
        use crate::guards::NervousSystemGuard;
        Self {
            node_id,
            signing_key,
            host,
            cache_capacity: 1000,
            guard: Some(Arc::new(NervousSystemGuard::new())),
        }
    }

    /// Build the executor.
    pub fn build(self) -> ExecutorResult<PcuExecutor> {
        PcuExecutor::new(self.node_id, self.signing_key, self.host, self.cache_capacity, self.guard)
    }
}

/// A production-grade WASM executor for NEXUS PCUs.
#[derive(Clone)]
pub struct PcuExecutor {
    engine: Engine,
    cache: Arc<SemanticCache>,
    signing_key: Arc<ed25519_dalek::SigningKey>,
    host: Arc<dyn crate::NexusHost>,
    metrics: Option<Arc<nexus_observability::NexusMetrics>>,
    guard: Option<Arc<dyn ExecutionGuard>>,
}

impl PcuExecutor {
    /// Create a new executor with default configuration.
    pub fn new(node_id: NodeId, signing_key: ed25519_dalek::SigningKey, host: Arc<dyn crate::NexusHost>, cache_capacity: usize, guard: Option<Arc<dyn ExecutionGuard>>) -> ExecutorResult<Self> {
        let mut config = Config::new();
        
        // Enable fuel metering for deterministic instruction counting
        config.consume_fuel(true);
        
        // Disable features that could lead to non-determinism or security risks
        config.wasm_threads(false);
        config.wasm_simd(true); // SIMD is generally deterministic
        config.wasm_multi_memory(false);
        
        // Optimization level
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);

        let engine = Engine::new(&config)?;
        
        Ok(Self {
            engine,
            cache: Arc::new(SemanticCache::new(node_id, cache_capacity)),
            signing_key: Arc::new(signing_key),
            host,
            metrics: None,
            guard,
        })
    }

    /// Set a custom semantic cache.
    pub fn with_cache(mut self, cache: Arc<SemanticCache>) -> Self {
        self.cache = cache;
        self
    }

    /// Set metrics for observability.
    pub fn with_metrics(mut self, metrics: Arc<nexus_observability::NexusMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Set execution guard. When set, every execute() passes through it.
    pub fn with_guard(mut self, guard: Arc<dyn ExecutionGuard>) -> Self {
        self.guard = Some(guard);
        self
    }

    /// Returns true if an execution guard is set. Production builds must have a guard.
    pub fn has_guard(&self) -> bool {
        self.guard.is_some()
    }

    /// Execute a PCU and return the result.
    #[instrument(skip(self, pcu, context), fields(pcu_id = %pcu.id))]
    pub async fn execute(
        &self,
        pcu: &PCU,
        context: ExecutionContext,
    ) -> ExecutorResult<ExecutionResponse> {
        let start = Instant::now();
        
        // Track active executions
        if let Some(ref metrics) = self.metrics {
            metrics.active_pcu_executions.inc();
            metrics.pcu_executions_total.inc();
        }

        // 0. Execution guard (biological / accountability). When set, no intelligent action without passing through.
        if let Some(guard) = &self.guard {
            match guard.check(pcu, &context) {
                GuardDecision::Allow => {}
                GuardDecision::Deny(reason) => {
                    return Err(ExecutorError::ExecutionBlocked { reason });
                }
            }
        }

        // 1. Validation - Check WASM header and size
        if !pcu.code.is_valid_header() {
            return Err(ExecutorError::InvalidPcu { reason: "Invalid WASM header".to_string() });
        }
        if pcu.code.size() > crate::MAX_MODULE_SIZE {
            return Err(ExecutorError::InvalidPcu { 
                reason: format!("Module too large: {} bytes (max: {})", pcu.code.size(), crate::MAX_MODULE_SIZE)
            });
        }
        
        // Check identity is valid
        if !pcu.identity.is_valid() {
            return Err(ExecutorError::InvalidPcu { reason: "Identity expired or invalid".to_string() });
        }

        // 2. Cache Lookup and Routing
        let inputs_hashes: Vec<_> = context.inputs.iter().map(|(h, _)| *h).collect();
        let semantic_key = crate::semantic_cache::SemanticKey::from_pcu(pcu, &inputs_hashes, &context.identity);
        
        // Check cache (no_cache field doesn't exist in ExecutionConstraints, so always check cache)
        {
            if let Some(cached_entry) = self.cache.get(&semantic_key) {
                let result = cached_entry.to_result();
                
                // Record cache hit
                if let Some(ref metrics) = self.metrics {
                    metrics.pcu_cache_hits.inc();
                    let duration = start.elapsed();
                    metrics.pcu_execution_duration.observe(duration.as_secs_f64());
                    metrics.active_pcu_executions.dec();
                }
                
                debug!("PCU cache hit: {}", pcu.id);
                // Note: The proof is already in the cached_entry, but we might want to 
                // re-sign or verify it for the current context if needed.
                // The cached proof covers the original execution.
                return Ok(ExecutionResponse::new(result, cached_entry.proof, true));
            }
        }
        
        // Record cache miss
        if let Some(ref metrics) = self.metrics {
            metrics.pcu_cache_misses.inc();
        }

        // 3. Setup Wasmtime Store
        let mut store = Store::new(&self.engine, Arc::new(context.clone()));
        
        // Set fuel limit
        store.set_fuel(context.limits.max_fuel)?;

        // 4. Compile Module
        let module = Module::new(&self.engine, &pcu.code.bytecode)?;

        // 5. Setup Linker and Host Functions
        let mut linker = Linker::new(&self.engine);
        host_functions::register_host_functions(&mut linker, Arc::clone(&self.host))?;

        // 6. Instantiate
        let instance = linker.instantiate(&mut store, &module)?;

        // 7. Execute - Find Entry Point
        let exports: Vec<_> = ["_start", "main", "execute", "run"]
            .iter()
            .map(|&s| s.to_string())
            .collect();
            
        let mut func = None;
        for entry in &exports {
            if let Ok(f) = instance.get_typed_func::<(), ()>(&mut store, entry) {
                func = Some(f);
                break;
            }
        }

        let func = func.ok_or_else(|| ExecutorError::EntryPointNotFound { tried: exports })?;

        // 8. Run with timeout
        let start_time = Instant::now();
        
        let exec_result = tokio::time::timeout(context.limits.max_time, async {
            func.call(&mut store, ())
        }).await;

        let duration = start_time.elapsed();

        // 9. Process Result
        let result = match exec_result {
            Ok(Ok(())) => {
                // Get output from memory
                let output = self.extract_output(&instance, &mut store)?;
                
                // Calculate fuel consumed
                let fuel_consumed = context.limits.max_fuel - store.get_fuel()?;
                
                // Extract memory usage (approx)
                let peak_memory = instance.get_memory(&mut store, "memory")
                    .map(|m| m.data_size(&store))
                    .unwrap_or(0);

                // 10. Generate Proof
                let proof = self.generate_proof(pcu, &context, &output, fuel_consumed, peak_memory)?;

                let result = ExecutionResult::new(
                    output,
                    fuel_consumed,
                    peak_memory,
                    duration,
                );

                // 11. Cache and Return
                self.cache.put(semantic_key, &result, proof.clone(), None);

                // Record successful execution
                if let Some(ref metrics) = self.metrics {
                    metrics.pcu_execution_duration.observe(duration.as_secs_f64());
                    metrics.active_pcu_executions.dec();
                }

                Ok(ExecutionResponse::new(result, proof, false))
            }
            Ok(Err(trap)) => {
                // Record failure
                if let Some(ref metrics) = self.metrics {
                    metrics.pcu_execution_failures.inc();
                    metrics.active_pcu_executions.dec();
                }
                Err(ExecutorError::WasmTrap { message: trap.to_string() })
            }
            Err(_) => {
                // Record timeout
                if let Some(ref metrics) = self.metrics {
                    metrics.pcu_execution_failures.inc();
                    metrics.active_pcu_executions.dec();
                }
                Err(ExecutorError::Timeout { elapsed: duration, limit: context.limits.max_time })
            }
        };
        
        result
    }

    /// Extract output data from WASM memory.
    fn extract_output(&self, instance: &Instance, store: &mut Store<Arc<ExecutionContext>>) -> ExecutorResult<Vec<u8>> {
        let output_len_fn = match instance.get_typed_func::<(), i32>(&mut *store, "__nexus_output_len") {
            Ok(f) => f,
            Err(_) => return Ok(Vec::new()), // Optional: No output if function missing
        };
        
        let output_len = output_len_fn.call(&mut *store, ())? as usize;
        
        if output_len > crate::MAX_OUTPUT_SIZE {
            return Err(ExecutorError::OutputTooLarge { size: output_len, max: crate::MAX_OUTPUT_SIZE });
        }

        let memory = instance.get_memory(&mut *store, "memory")
            .ok_or(ExecutorError::NoMemoryExport)?;
        
        let data = memory.data(&*store);
        if output_len > data.len() {
            return Err(ExecutorError::InvalidOutputLength { length: output_len as i64 });
        }

        Ok(data[..output_len].to_vec())
    }

    /// Generate a cryptographic proof of execution.
    fn generate_proof(
        &self,
        pcu: &PCU,
        context: &ExecutionContext,
        output: &[u8],
        fuel_consumed: u64,
        peak_memory: usize,
    ) -> ExecutorResult<ExecutionProof> {
        let result = ExecutionResult::new(
            output.to_vec(),
            fuel_consumed,
            peak_memory,
            Duration::ZERO, // duration not used by proof creation
        );

        let proof = ExecutionProof::create(
            pcu,
            &context.inputs,
            &result,
            &context.identity,
            &self.signing_key,
        );

        Ok(proof)
    }
}
