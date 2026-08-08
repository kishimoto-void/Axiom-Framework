//! Golden-oriented numerical tests for DCK metrics (v2.2).

#[cfg(test)]
mod golden {
    use chrono::Utc;
    use nalgebra::{DMatrix, DVector};

    use crate::config::DCKConfig;
    use crate::metrics::{
        evaluate_difference, ConvergenceReason, ConvergenceReport, DifferenceMetrics,
        StabilityScore,
    };
    use crate::state::StateEstimate;

    fn cfg_tight() -> DCKConfig {
        DCKConfig {
            convergence_tolerance: 0.05,
            max_gap_scale: 10.0,
            cholesky_floor: 1e-9,
            gap_history_size: 64,
            ..DCKConfig::default()
        }
    }

    #[test]
    fn golden_scalar_on_target_near_zero_difference() {
        let est = StateEstimate::scalar(1.0, 0.01, 1.0, Utc::now(), "golden", 1e-9).unwrap();
        let cfg = cfg_tight();
        let m = DifferenceMetrics::from_state_and_scalar(&est, 1.0, &cfg).unwrap();
        assert!(m.difference_total < 1e-6);
        assert!(m.convergence_rate > 0.999);
        assert!(m.breakdown.position < 1e-6);
    }

    #[test]
    fn golden_breakdown_explains_residual() {
        let cfg = cfg_tight();
        let est = StateEstimate::scalar(0.0, 1.0, 0.5, Utc::now(), "g", 1e-9).unwrap();
        let m = DifferenceMetrics::from_state_and_scalar(&est, 1.0, &cfg).unwrap();
        assert!(m.breakdown.position > 0.0);
        assert!(m.breakdown.covariance > 0.0);
        assert!(m.breakdown.confidence > 0.0);
        assert!(m.breakdown.total_components() > 0.0);
    }

    #[test]
    fn golden_2d_mahalanobis_respects_covariance() {
        let cfg = cfg_tight();
        let mean = DVector::from_vec(vec![0.0, 0.0]);
        let cov = DMatrix::from_row_slice(2, 2, &[4.0, 0.0, 0.0, 0.25]);
        let est = StateEstimate::new(mean, cov, 0.9, Utc::now(), "2d", 1e-9).unwrap();
        let t_high_var = DVector::from_vec(vec![2.0, 0.0]);
        let t_low_var = DVector::from_vec(vec![0.0, 2.0]);
        let d_high = evaluate_difference(&est, &t_high_var, &cfg).unwrap();
        let d_low = evaluate_difference(&est, &t_low_var, &cfg).unwrap();
        assert!(d_low.difference_total > d_high.difference_total);
    }

    #[test]
    fn golden_history_curve_and_finish() {
        let cfg = cfg_tight();
        let mut report = ConvergenceReport::new();
        let steps = [0.0, 0.25, 0.5, 0.75, 0.92, 0.99];
        for &m in &steps {
            let est = StateEstimate::scalar(m, 0.01, 1.0, Utc::now(), "seq", 1e-9).unwrap();
            let metrics = DifferenceMetrics::from_state_and_scalar(&est, 1.0, &cfg).unwrap();
            report.push(metrics, &cfg);
        }
        report.finish(20, &cfg);
        let curve = report.difference_curve();
        assert_eq!(curve.len(), steps.len());
        assert!(curve[0] > curve[curve.len() - 1]);
        assert!(report.stability.is_some());
        let s = report.stability.as_ref().unwrap();
        assert!(s.score >= 0.0 && s.score <= 1.0);
        assert!(matches!(
            report.reason,
            ConvergenceReason::ThresholdReached | ConvergenceReason::MaxTick
        ));
    }

    #[test]
    fn golden_divergence_reason() {
        let cfg = DCKConfig {
            convergence_tolerance: 0.01,
            max_gap_scale: 10.0,
            gap_history_size: 32,
            ..DCKConfig::default()
        };
        let mut report = ConvergenceReport::new();
        let near = DifferenceMetrics::from_state_and_scalar(
            &StateEstimate::scalar(0.95, 0.01, 1.0, Utc::now(), "d", 1e-9).unwrap(),
            1.0,
            &cfg,
        )
        .unwrap();
        report.push(near, &cfg);
        let far = DifferenceMetrics::from_state_and_scalar(
            &StateEstimate::scalar(-5.0, 0.01, 1.0, Utc::now(), "d", 1e-9).unwrap(),
            1.0,
            &cfg,
        )
        .unwrap();
        report.push(far, &cfg);
        assert_eq!(report.reason, ConvergenceReason::Divergence);
    }

    #[test]
    fn golden_stability_score_bounds() {
        let cfg = cfg_tight();
        let mut hist = Vec::new();
        for m in [0.0_f64, 0.5, 0.8, 0.95] {
            let est = StateEstimate::scalar(m, 0.01, 1.0, Utc::now(), "s", 1e-9).unwrap();
            hist.push(DifferenceMetrics::from_state_and_scalar(&est, 1.0, &cfg).unwrap());
        }
        let s = StabilityScore::from_history(&hist, Some(4), 20, &cfg);
        assert!((0.0..=1.0).contains(&s.score));
        assert!((0.0..=1.0).contains(&s.speed));
        assert!((0.0..=1.0).contains(&s.smoothness));
        assert!((0.0..=1.0).contains(&s.final_accuracy));
    }
}
