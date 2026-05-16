//! Constrained quadratic programming solver.
//!
//! Solves: min Σ w_i * (x_i - s_i)² subject to L_i ≤ x_i ≤ U_i

use crate::core::bounds::HardBounds;

/// Problem definition for quadratic solver.
#[derive(Debug, Clone)]
pub struct QuadraticProblem {
    /// Number of variables.
    pub n: usize,
    /// Current values.
    pub values: Vec<f64>,
    /// Setpoints (targets).
    pub setpoints: Vec<f64>,
    /// Weights for each variable.
    pub weights: Vec<f64>,
    /// Bounds for each variable.
    pub bounds: Vec<HardBounds>,
}

/// Solution from quadratic solver.
#[derive(Debug, Clone)]
pub struct QuadraticSolution {
    /// Optimal values.
    pub values: Vec<f64>,
    /// Objective value at solution.
    pub objective: f64,
    /// Number of iterations.
    pub iterations: u32,
    /// Whether convergence was achieved.
    pub converged: bool,
    /// Indices of active bounds.
    pub active_bounds: Vec<usize>,
}

/// Simple projected gradient descent solver.
///
/// For the unconstrained problem, the solution is simply x_i = s_i.
/// With bounds, we project onto the feasible region.
pub struct ProjectedGradientSolver {
    /// Maximum iterations.
    pub max_iterations: u32,
    /// Convergence tolerance.
    pub tolerance: f64,
    /// Step size.
    pub step_size: f64,
}

impl Default for ProjectedGradientSolver {
    fn default() -> Self {
        Self {
            max_iterations: 1000,
            tolerance: 1e-8,
            step_size: 0.1,
        }
    }
}

impl ProjectedGradientSolver {
    /// Creates a new solver with given parameters.
    pub fn new(max_iterations: u32, tolerance: f64, step_size: f64) -> Self {
        Self {
            max_iterations,
            tolerance,
            step_size,
        }
    }
    
    /// Solves the quadratic problem.
    pub fn solve(&self, problem: &QuadraticProblem) -> QuadraticSolution {
        let mut values = problem.values.clone();
        
        for iter in 0..self.max_iterations {
            let obj_before = self.objective(&values, problem);
            
            // Gradient step
            for i in 0..problem.n {
                let error = values[i] - problem.setpoints[i];
                let gradient = 2.0 * problem.weights[i] * error;
                values[i] -= self.step_size * gradient;
                
                // Project onto bounds
                values[i] = problem.bounds[i].clamp(values[i]);
            }
            
            let obj_after = self.objective(&values, problem);
            
            // Check convergence
            if (obj_before - obj_after).abs() < self.tolerance {
                return QuadraticSolution {
                    values: values.clone(),
                    objective: obj_after,
                    iterations: iter + 1,
                    converged: true,
                    active_bounds: self.find_active_bounds(&values, problem),
                };
            }
        }
        
        let objective = self.objective(&values, problem);
        QuadraticSolution {
            values: values.clone(),
            objective,
            iterations: self.max_iterations,
            converged: false,
            active_bounds: self.find_active_bounds(&values, problem),
        }
    }
    
    /// Computes objective function value.
    fn objective(&self, values: &[f64], problem: &QuadraticProblem) -> f64 {
        values.iter()
            .zip(&problem.setpoints)
            .zip(&problem.weights)
            .map(|((v, s), w)| w * (v - s).powi(2))
            .sum()
    }
    
    /// Finds which bounds are active.
    fn find_active_bounds(&self, values: &[f64], problem: &QuadraticProblem) -> Vec<usize> {
        values.iter()
            .zip(&problem.bounds)
            .enumerate()
            .filter_map(|(i, (v, b))| {
                if (v - b.lower).abs() < 1e-9 || (v - b.upper).abs() < 1e-9 {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unconstrained() {
        let problem = QuadraticProblem {
            n: 2,
            values: vec![0.0, 1.0],
            setpoints: vec![0.5, 0.5],
            weights: vec![1.0, 1.0],
            bounds: vec![
                HardBounds::new(-10.0, 10.0).unwrap(),
                HardBounds::new(-10.0, 10.0).unwrap(),
            ],
        };
        
        let solver = ProjectedGradientSolver::default();
        let solution = solver.solve(&problem);
        
        assert!(solution.converged);
        assert!((solution.values[0] - 0.5).abs() < 0.01);
        assert!((solution.values[1] - 0.5).abs() < 0.01);
    }
    
    #[test]
    fn test_constrained() {
        let problem = QuadraticProblem {
            n: 1,
            values: vec![0.0],
            setpoints: vec![2.0],  // Outside bounds
            weights: vec![1.0],
            bounds: vec![HardBounds::new(0.0, 1.0).unwrap()],
        };
        
        let solver = ProjectedGradientSolver::default();
        let solution = solver.solve(&problem);
        
        // Should hit upper bound
        assert!((solution.values[0] - 1.0).abs() < 0.01);
        assert!(!solution.active_bounds.is_empty());
    }
}
