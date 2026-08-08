//! Explicit difference / convergence metrics for Golden Vector locking.
//!
//! DCK v2.2 — Measurement library contract:
//! - DifferenceMetrics + DifferenceBreakdown
//! - ConvergenceReport + history + ConvergenceReason
//! - StabilityScore (0..1 composite)
//! - evaluate_difference (sync, pure)

use nalgebra::DVector;
use serde::{Deserialize, Serialize};

use crate::config::DCKConfig;
use crate::error::DCKError;
use crate::state::StateEstimate;

/// Component-wise view of residual difference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DifferenceBreakdown {
    pub position: f64,
    pub velocity: f64,
    pub covariance: f64,
    pub confidence: f64,
}

impl DifferenceBreakdown {
    pub fn total_components(&self) -> f64 {
        self.position + self.velocity + self.covariance + self.confidence
    }
}

/// Snapshot of difference between current estimate and a target.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DifferenceMetrics {
    pub difference_total: f64,
    pub per_dim: Vec<f64>,
    pub convergence_rate: f64,
    pub converged: bool,
    pub dim: usize,
    pub breakdown: DifferenceBreakdown,
}

impl DifferenceMetrics {
    pub fn from_state_and_target(
        estimate: &StateEstimate,
        target: &DVector<f64>,
        config: &DCKConfig,
    ) -> Result<Self, DCKError> {
        Self::from_state_target_velocity(estimate, target, None, config)
    }

    pub fn from_state_and_scalar(
        estimate: &StateEstimate,
        target: f64,
        config: &DCKConfig,
    ) -> Result<Self, DCKError> {
        let t = DVector::from_element(estimate.dim().max(1), target);
        Self::from_state_and_target(estimate, &t, config)
    }

    pub fn from_state_target_velocity(
        estimate: &StateEstimate,
        target: &DVector<f64>,
        velocity: Option<f64>,
        config: &DCKConfig,
    ) -> Result<Self, DCKError> {
        if target.len() != estimate.dim() {
            return Err(DCKError::ValidationError(
                "target dimension mismatch in DifferenceMetrics".into(),
            ));
        }

        let per_dim: Vec<f64> = (0..estimate.dim())
            .map(|i| (estimate.mean[i] - target[i]).abs())
            .collect();

        let position = per_dim.iter().sum::<f64>();

        let difference_total = if estimate.dim() == 1 {
            estimate.mahalanobis_distance_scalar(target[0])
        } else {
            estimate
                .mahalanobis_distance(target)
                .unwrap_or_else(|_| per_dim.iter().map(|x| x * x).sum::<f64>().sqrt())
        };

        let vel = velocity.unwrap_or(0.0).abs();
        let cov_contrib = estimate.total_uncertainty(config.min_uncertainty);
        let conf_penalty = (1.0 - estimate.confidence.clamp(0.0, 1.0)) * config.weight_risk;

        let breakdown = DifferenceBreakdown {
            position,
            velocity: vel,
            covariance: cov_contrib,
            confidence: conf_penalty,
        };

        let scale = config.max_gap_scale.max(1e-12);
        let convergence_rate =
            (1.0 - (difference_total / scale).clamp(0.0, 1.0)).clamp(0.0, 1.0);
        let converged = difference_total <= config.convergence_tolerance;

        Ok(Self {
            difference_total,
            per_dim,
            convergence_rate,
            converged,
            dim: estimate.dim(),
            breakdown,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConvergenceReason {
    ThresholdReached,
    MaxTick,
    Divergence,
    NumericalIssue,
    InProgress,
}

impl Default for ConvergenceReason {
    fn default() -> Self {
        ConvergenceReason::InProgress
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StabilityScore {
    pub score: f64,
    pub speed: f64,
    pub smoothness: f64,
    pub final_accuracy: f64,
}

impl StabilityScore {
    pub fn from_history(
        history: &[DifferenceMetrics],
        ticks_to_threshold: Option<u64>,
        max_ticks_ref: u64,
        config: &DCKConfig,
    ) -> Self {
        if history.is_empty() {
            return Self {
                score: 0.0,
                speed: 0.0,
                smoothness: 0.0,
                final_accuracy: 0.0,
            };
        }

        let final_accuracy = history.last().map(|m| m.convergence_rate).unwrap_or(0.0);

        let speed = match ticks_to_threshold {
            Some(t) if t > 0 => {
                let ref_t = max_ticks_ref.max(1) as f64;
                (1.0 - (t as f64 / ref_t).clamp(0.0, 1.0)).clamp(0.0, 1.0)
            }
            Some(_) => 1.0,
            None => 0.0,
        };

        let mut osc = 0.0;
        if history.len() >= 2 {
            for w in history.windows(2) {
                osc += (w[1].difference_total - w[0].difference_total).abs();
            }
            osc /= (history.len() - 1) as f64;
        }
        let scale = config.max_gap_scale.max(1e-12);
        let smoothness = (1.0 - (osc / scale).clamp(0.0, 1.0)).clamp(0.0, 1.0);

        let score = (0.35 * speed + 0.30 * smoothness + 0.35 * final_accuracy).clamp(0.0, 1.0);

        Self {
            score,
            speed,
            smoothness,
            final_accuracy,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConvergenceReport {
    pub ticks: u64,
    pub history: Vec<DifferenceMetrics>,
    pub ticks_to_threshold: Option<u64>,
    pub final_difference: Option<f64>,
    pub final_convergence_rate: Option<f64>,
    pub reason: ConvergenceReason,
    pub stability: Option<StabilityScore>,
}

impl ConvergenceReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, metrics: DifferenceMetrics, config: &DCKConfig) {
        self.ticks += 1;

        if let Some(prev) = self.history.last() {
            if metrics.difference_total > prev.difference_total * 2.0
                && metrics.difference_total > config.convergence_tolerance * 10.0
            {
                self.reason = ConvergenceReason::Divergence;
            }
        }

        if self.ticks_to_threshold.is_none() && metrics.converged {
            self.ticks_to_threshold = Some(self.ticks);
            if self.reason == ConvergenceReason::InProgress {
                self.reason = ConvergenceReason::ThresholdReached;
            }
        }

        self.final_difference = Some(metrics.difference_total);
        self.final_convergence_rate = Some(metrics.convergence_rate);

        let cap = config.gap_history_size.max(8);
        if self.history.len() >= cap {
            self.history.remove(0);
        }
        self.history.push(metrics);
    }

    pub fn history(&self) -> &[DifferenceMetrics] {
        &self.history
    }

    pub fn difference_curve(&self) -> Vec<f64> {
        self.history.iter().map(|m| m.difference_total).collect()
    }

    pub fn convergence_curve(&self) -> Vec<f64> {
        self.history.iter().map(|m| m.convergence_rate).collect()
    }

    pub fn is_converged(&self) -> bool {
        matches!(self.reason, ConvergenceReason::ThresholdReached)
            || self.ticks_to_threshold.is_some()
    }

    pub fn finish(&mut self, max_ticks: u64, config: &DCKConfig) {
        if self.reason == ConvergenceReason::InProgress {
            if self.ticks_to_threshold.is_some() {
                self.reason = ConvergenceReason::ThresholdReached;
            } else if self.ticks >= max_ticks {
                self.reason = ConvergenceReason::MaxTick;
            }
        }
        self.stability = Some(StabilityScore::from_history(
            &self.history,
            self.ticks_to_threshold,
            max_ticks,
            config,
        ));
    }

    pub fn mark_numerical_issue(&mut self) {
        self.reason = ConvergenceReason::NumericalIssue;
    }
}

/// Pure, sync evaluation helper — no async, no tokio.
pub fn evaluate_difference(
    estimate: &StateEstimate,
    target: &DVector<f64>,
    config: &DCKConfig,
) -> Result<DifferenceMetrics, DCKError> {
    DifferenceMetrics::from_state_and_target(estimate, target, config)
}

pub fn evaluate_difference_with_velocity(
    estimate: &StateEstimate,
    target: &DVector<f64>,
    velocity: f64,
    config: &DCKConfig,
) -> Result<DifferenceMetrics, DCKError> {
    DifferenceMetrics::from_state_target_velocity(estimate, target, Some(velocity), config)
}
