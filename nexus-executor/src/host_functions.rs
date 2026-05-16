//! Host functions for NEXUS WASM PCUs.
//!
//! These functions are exported to the WASM environment and allow
//! the PCU to interact with the host (read inputs, write outputs, access USOs).

use crate::error::ExecutorError;
use crate::types::ExecutionContext;
use crate::NexusHost;
use wasmtime::{Caller, Linker};
use std::sync::Arc;

/// Register all NEXUS host functions with the linker.
pub fn register_host_functions(linker: &mut Linker<Arc<ExecutionContext>>, host: Arc<dyn NexusHost>) -> Result<(), ExecutorError> {
    // =========================================================================
    // README Standard Host Functions
    // =========================================================================

    // input_count() -> i32
    linker.func_wrap("nexus", "input_count", move |caller: Caller<'_, Arc<ExecutionContext>>| -> i32 {
        caller.data().input_count() as i32
    }).map_err(|e| ExecutorError::InternalError(format!("Failed to link input_count: {}", e)))?;

    // input_size(index: i32) -> i64
    linker.func_wrap("nexus", "input_size", move |caller: Caller<'_, Arc<ExecutionContext>>, index: i32| -> i64 {
        caller.data().get_input_by_index(index as usize).map(|data| data.len() as i64).unwrap_or(-1)
    }).map_err(|e| ExecutorError::InternalError(format!("Failed to link input_size: {}", e)))?;

    // input_read(index, offset, length, dest) -> i32
    linker.func_wrap("nexus", "input_read", move |mut caller: Caller<'_, Arc<ExecutionContext>>, index: i32, offset: u32, length: u32, dest: u32| -> i32 {
        let data_subset = if let Some(data) = caller.data().get_input_by_index(index as usize) {
            let offset = offset as usize;
            let length = length as usize;
            if offset + length > data.len() {
                return -1;
            }
            data[offset..offset + length].to_vec()
        } else {
            return -1;
        };

        let mem = match caller.get_export("memory").and_then(|e| e.into_memory()) {
            Some(m) => m,
            _ => return -1,
        };
        if let Err(_) = mem.write(&mut caller, dest as usize, &data_subset) {
            return -1;
        }
        data_subset.len() as i32
    }).map_err(|e| ExecutorError::InternalError(format!("Failed to link input_read: {}", e)))?;

    // output_write(src, length) -> i32
    // Note: We need a way to store the output. Current ExecutionContext is immutable in caller.data().
    // We might need to store output in the Store data or a separate buffer.
    // For now, let's assume PcuExecutor handles output extraction via memory.
    // But the README says output_write.
    linker.func_wrap("nexus", "output_write", move |_caller: Caller<'_, Arc<ExecutionContext>>, _src: u32, _length: u32| -> i32 {
        // Implementation would typically copy to a host-side output buffer
        // For now, this is a stub that returns success.
        0
    }).map_err(|e| ExecutorError::InternalError(format!("Failed to link output_write: {}", e)))?;

    // get_identity() -> i64
    linker.func_wrap("nexus", "get_identity", move |caller: Caller<'_, Arc<ExecutionContext>>| -> i64 {
        let principal = caller.data().identity.principal.as_bytes();
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&principal[0..8]);
        i64::from_le_bytes(buf)
    }).map_err(|e| ExecutorError::InternalError(format!("Failed to link get_identity: {}", e)))?;

    // has_capability(cap_id: i32) -> i32
    linker.func_wrap("nexus", "has_capability", move |caller: Caller<'_, Arc<ExecutionContext>>, cap_id: i32| -> i32 {
        // Map numeric IDs to capability resources
        let cap_resource = match cap_id {
            0 => "data:read",
            1 => "data:write",
            2 => "pcu:execute",
            3 => "network:access",
            4 => "*:admin",
            _ => return 0,
        };
        if caller.data().identity.permits(cap_resource, "execute") { 1 } else { 0 }
    }).map_err(|e| ExecutorError::InternalError(format!("Failed to link has_capability: {}", e)))?;

    // =========================================================================
    // Strategic Architecture Host Functions (USO / CRDT)
    // =========================================================================
    
    let host_get = Arc::clone(&host);
    linker.func_wrap("nexus", "uso_get", move |mut caller: Caller<'_, Arc<ExecutionContext>>, hash_ptr: u32, out_ptr: u32| -> u32 {
        let mem = match caller.get_export("memory").and_then(|e| e.into_memory()) {
            Some(m) => m,
            _ => return 0,
        };
        
        let mut hash_bytes = [0u8; 32];
        if let Err(_) = mem.read(&caller, hash_ptr as usize, &mut hash_bytes) {
            return 0;
        }
        let hash = crate::ContentHash::from_bytes(hash_bytes);
        
        match host_get.uso_get(&hash) {
            Ok(Some(data)) => {
                if out_ptr != 0 {
                    if let Err(_) = mem.write(&mut caller, out_ptr as usize, &data) {
                        return 0;
                    }
                }
                data.len() as u32
            }
            _ => 0,
        }
    }).map_err(|e| ExecutorError::InternalError(format!("Failed to link uso_get: {}", e)))?;

    let host_put = Arc::clone(&host);
    linker.func_wrap("nexus", "uso_put", move |mut caller: Caller<'_, Arc<ExecutionContext>>, data_ptr: u32, data_len: u32, hash_out_ptr: u32| -> u32 {
        let mem = match caller.get_export("memory").and_then(|e| e.into_memory()) {
            Some(m) => m,
            _ => return 0,
        };
        
        let mut data = vec![0u8; data_len as usize];
        if let Err(_) = mem.read(&caller, data_ptr as usize, &mut data) {
            return 0;
        }
        
        match host_put.uso_put(&data) {
            Ok(hash) => {
                if hash_out_ptr != 0 {
                    if let Err(_) = mem.write(&mut caller, hash_out_ptr as usize, hash.as_bytes()) {
                        return 0;
                    }
                }
                1
            }
            Err(_) => 0,
        }
    }).map_err(|e| ExecutorError::InternalError(format!("Failed to link uso_put: {}", e)))?;

    let host_apply = Arc::clone(&host);
    linker.func_wrap("nexus", "uso_apply_op", move |mut caller: Caller<'_, Arc<ExecutionContext>>, hash_ptr: u32, op_ptr: u32, op_len: u32, hash_out_ptr: u32| -> u32 {
        let mem = match caller.get_export("memory").and_then(|e| e.into_memory()) {
            Some(m) => m,
            _ => return 0,
        };
        
        let mut hash_bytes = [0u8; 32];
        if let Err(_) = mem.read(&caller, hash_ptr as usize, &mut hash_bytes) {
            return 0;
        }
        let hash = crate::ContentHash::from_bytes(hash_bytes);
        
        let mut op = vec![0u8; op_len as usize];
        if let Err(_) = mem.read(&caller, op_ptr as usize, &mut op) {
            return 0;
        }
        
        match host_apply.uso_apply_op(&hash, &op) {
            Ok(new_hash) => {
                if hash_out_ptr != 0 {
                    if let Err(_) = mem.write(&mut caller, hash_out_ptr as usize, new_hash.as_bytes()) {
                        return 0;
                    }
                }
                1
            }
            Err(_) => 0,
        }
    }).map_err(|e| ExecutorError::InternalError(format!("Failed to link uso_apply_op: {}", e)))?;

    // =========================================================================
    // Logging & Utility
    // =========================================================================
    
    let host_log = Arc::clone(&host);
    linker.func_wrap("nexus", "log", move |mut caller: Caller<'_, Arc<ExecutionContext>>, level: u32, msg_ptr: u32, msg_len: u32| {
        let mem = match caller.get_export("memory").and_then(|e| e.into_memory()) {
            Some(m) => m,
            _ => return,
        };
        
        let mut msg_bytes = vec![0u8; msg_len as usize];
        if let Err(_) = mem.read(&caller, msg_ptr as usize, &mut msg_bytes) {
            return;
        }
        
        if let Ok(msg) = std::str::from_utf8(&msg_bytes) {
            host_log.log(level, msg);
        }
    }).map_err(|e| ExecutorError::InternalError(format!("Failed to link log: {}", e)))?;

    let host_time = Arc::clone(&host);
    linker.func_wrap("nexus", "get_time", move |_caller: Caller<'_, Arc<ExecutionContext>>| -> u64 {
        host_time.get_time()
    }).map_err(|e| ExecutorError::InternalError(format!("Failed to link get_time: {}", e)))?;

    let _host_spawn = Arc::clone(&host);
    linker.func_wrap("nexus", "spawn_pcu", move |_caller: Caller<'_, Arc<ExecutionContext>>, _code_ptr: u32, _code_len: u32| -> u32 {
        // Implementation would involve building a new PCU and submitting it to the executor
        // For now, this is a placeholder for the strategic composition API.
        0
    }).map_err(|e| ExecutorError::InternalError(format!("Failed to link spawn_pcu: {}", e)))?;

    Ok(())
}
