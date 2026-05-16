//! CAUSALUX-COMPUTE: Distributed GPU Computing Extension
//! 
//! This module extends CAUSALUX with distributed GPU computation capabilities:
//! - Causal Gradient CRDTs for async gradient descent
//! - GPU Mesh Protocol for P2P compute distribution
//! - Distributed Training Runtime

pub mod tensor;
pub mod gradient_crdt;
pub mod momentum;
pub mod mesh;
pub mod shard;
pub mod trainer;

pub use tensor::{Tensor, TensorShape};
pub use gradient_crdt::{CausalGradient, GradientOp};
pub use momentum::CausalMomentum;
pub use mesh::{GPUMesh, GPUNode, MeshConfig};
pub use shard::{ModelShard, ShardRegistry};
pub use trainer::{DistributedTrainer, TrainingConfig};
