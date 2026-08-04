#[derive(Debug, Clone)]
pub struct DCKConfig {
    pub max_gap_scale: f64,
    pub target_velocity_scale: f64,
    pub velocity_time_constant_tau: f64,
    pub aging_factor: f64,
    pub convergence_tolerance: f64,
    pub max_concurrency_execution: usize,
    pub min_uncertainty: f64,
    pub gap_history_size: usize,
    pub weight_equivalence: f64,
    pub weight_velocity: f64,
    pub weight_risk: f64,
    /// Minimum diagonal value forced during Cholesky for numerical stability.
    pub cholesky_floor: f64,
}

impl Default for DCKConfig {
    fn default() -> Self {
        Self {
            max_gap_scale: 100.0,
            target_velocity_scale: 10.0,
            velocity_time_constant_tau: 2.0,
            aging_factor: 0.5,
            convergence_tolerance: 2.0,
            max_concurrency_execution: 8,
            min_uncertainty: 1e-12,
            gap_history_size: 10,
            weight_equivalence: 1.0,
            weight_velocity: 1.5,
            weight_risk: 1.0,
            cholesky_floor: 1e-9,
        }
    }
}
