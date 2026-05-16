//! Lightweight Tensor Abstraction
//! 
//! A minimal tensor implementation for gradient storage and manipulation.
//! This is NOT a full tensor library - it provides just enough for CRDT operations.

use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Sub};

/// Shape of a tensor (dimensions)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TensorShape(pub Vec<usize>);

impl TensorShape {
    pub fn new(dims: Vec<usize>) -> Self {
        Self(dims)
    }

    pub fn size(&self) -> usize {
        self.0.iter().product()
    }

    pub fn dims(&self) -> &[usize] {
        &self.0
    }
}

/// Lightweight tensor for gradient storage
/// 
/// This is a simplified tensor that stores f32 values.
/// For production, you'd integrate with ndarray, candle, or burn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tensor {
    /// Flattened data storage
    pub data: Vec<f32>,
    /// Shape of the tensor
    pub shape: TensorShape,
    /// Content hash for deduplication
    pub content_hash: String,
}

impl Tensor {
    /// Create a new tensor with given shape, initialized to zeros
    pub fn zeros(shape: TensorShape) -> Self {
        let size = shape.size();
        let data = vec![0.0; size];
        let content_hash = Self::compute_hash(&data);
        Self { data, shape, content_hash }
    }

    /// Create a new tensor with given shape, initialized to ones
    pub fn ones(shape: TensorShape) -> Self {
        let size = shape.size();
        let data = vec![1.0; size];
        let content_hash = Self::compute_hash(&data);
        Self { data, shape, content_hash }
    }

    /// Create a tensor from raw data
    pub fn from_data(data: Vec<f32>, shape: TensorShape) -> Self {
        assert_eq!(data.len(), shape.size(), "Data size must match shape");
        let content_hash = Self::compute_hash(&data);
        Self { data, shape, content_hash }
    }

    /// Create a random tensor (for testing)
    pub fn random(shape: TensorShape) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        
        let size = shape.size();
        let mut data = Vec::with_capacity(size);
        let mut state = seed;
        
        for _ in 0..size {
            // Simple LCG random number generator
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = ((state >> 33) as f32) / (u32::MAX as f32) - 0.5;
            data.push(val);
        }
        
        let content_hash = Self::compute_hash(&data);
        Self { data, shape, content_hash }
    }

    /// Compute content hash for deduplication
    fn compute_hash(data: &[f32]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        for val in data {
            hasher.update(val.to_le_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    /// Element-wise addition (for gradient accumulation)
    pub fn add(&self, other: &Tensor) -> Tensor {
        assert_eq!(self.shape, other.shape, "Shapes must match for addition");
        let data: Vec<f32> = self.data.iter()
            .zip(other.data.iter())
            .map(|(a, b)| a + b)
            .collect();
        Tensor::from_data(data, self.shape.clone())
    }

    /// Element-wise subtraction
    pub fn sub(&self, other: &Tensor) -> Tensor {
        assert_eq!(self.shape, other.shape, "Shapes must match for subtraction");
        let data: Vec<f32> = self.data.iter()
            .zip(other.data.iter())
            .map(|(a, b)| a - b)
            .collect();
        Tensor::from_data(data, self.shape.clone())
    }

    /// Scalar multiplication (for learning rate)
    pub fn scale(&self, scalar: f32) -> Tensor {
        let data: Vec<f32> = self.data.iter()
            .map(|a| a * scalar)
            .collect();
        Tensor::from_data(data, self.shape.clone())
    }

    /// Compute L2 norm (for gradient clipping)
    pub fn norm(&self) -> f32 {
        self.data.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    /// Clip gradients to max norm
    pub fn clip_norm(&self, max_norm: f32) -> Tensor {
        let norm = self.norm();
        if norm > max_norm {
            self.scale(max_norm / norm)
        } else {
            self.clone()
        }
    }

    /// Average with another tensor (for gradient averaging)
    pub fn average(&self, other: &Tensor) -> Tensor {
        self.add(other).scale(0.5)
    }

    /// Weighted average (for momentum)
    pub fn weighted_average(&self, other: &Tensor, self_weight: f32) -> Tensor {
        let other_weight = 1.0 - self_weight;
        let scaled_self = self.scale(self_weight);
        let scaled_other = other.scale(other_weight);
        scaled_self.add(&scaled_other)
    }
}

impl Add for &Tensor {
    type Output = Tensor;
    fn add(self, other: &Tensor) -> Tensor {
        Tensor::add(self, other)
    }
}

impl Sub for &Tensor {
    type Output = Tensor;
    fn sub(self, other: &Tensor) -> Tensor {
        Tensor::sub(self, other)
    }
}

impl Mul<f32> for &Tensor {
    type Output = Tensor;
    fn mul(self, scalar: f32) -> Tensor {
        self.scale(scalar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tensor_zeros() {
        let t = Tensor::zeros(TensorShape::new(vec![2, 3]));
        assert_eq!(t.data.len(), 6);
        assert!(t.data.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_tensor_add() {
        let a = Tensor::ones(TensorShape::new(vec![2, 2]));
        let b = Tensor::ones(TensorShape::new(vec![2, 2]));
        let c = a.add(&b);
        assert!(c.data.iter().all(|&x| x == 2.0));
    }

    #[test]
    fn test_tensor_scale() {
        let a = Tensor::ones(TensorShape::new(vec![2, 2]));
        let b = a.scale(0.5);
        assert!(b.data.iter().all(|&x| x == 0.5));
    }

    #[test]
    fn test_tensor_norm() {
        let t = Tensor::from_data(vec![3.0, 4.0], TensorShape::new(vec![2]));
        assert!((t.norm() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_content_hash() {
        let a = Tensor::ones(TensorShape::new(vec![2, 2]));
        let b = Tensor::ones(TensorShape::new(vec![2, 2]));
        assert_eq!(a.content_hash, b.content_hash);
    }
}
