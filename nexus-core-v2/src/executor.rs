use crate::errors::{NexusError, Result};
use wasmtime::*;

pub struct Executor {
    engine: Engine,
}

impl Executor {
    pub fn new() -> Self {
        let mut config = Config::new();
        // Disable WASM features for deterministic execution
        config.wasm_multi_memory(false);
        config.wasm_multi_value(false);
        config.wasm_bulk_memory(true);  // Required for reference_types
        config.wasm_threads(false);
        config.wasm_reference_types(true);  // Required by wasmtime 16.x
        let engine = Engine::new(&config).unwrap();
        Executor { engine }
    }

    pub fn execute(&self, wasm_bytes: &[u8], input: &[u8]) -> Result<Vec<u8>> {
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| NexusError::ExecutionFailed(e.to_string()))?;
        
        let mut store = Store::new(&self.engine, ());
        let instance = Instance::new(&mut store, &module, &[])
            .map_err(|e| NexusError::ExecutionFailed(e.to_string()))?;

        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| NexusError::ExecutionFailed("no memory export".to_string()))?;

        let input_ptr = 0usize;
        if input.len() > 0 {
            memory.write(&mut store, input_ptr, input)
                .map_err(|e| NexusError::ExecutionFailed(e.to_string()))?;
        }

        let run = instance.get_typed_func::<(i32, i32), i32>(&mut store, "run")
            .map_err(|e| NexusError::ExecutionFailed(e.to_string()))?;
        
        let output_len = run.call(&mut store, (input_ptr as i32, input.len() as i32))
            .map_err(|e| NexusError::ExecutionFailed(e.to_string()))?;

        if output_len <= 0 {
            return Ok(Vec::new());
        }

        let mut output = vec![0u8; output_len as usize];
        memory.read(&store, input_ptr, &mut output)
            .map_err(|e| NexusError::ExecutionFailed(e.to_string()))?;

        Ok(output)
    }
}
