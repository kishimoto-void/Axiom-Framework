use std::collections::HashMap;
use std::sync::Arc;

use futures::future::join_all;
use nalgebra::DVector;
use tokio::sync::{Mutex, RwLock, Semaphore};

use crate::capabilities::{ExecutorCapability, ObserverCapability, PredictorCapability};
use crate::clock::Clock;
use crate::config::DCKConfig;
use crate::error::DCKError;
use crate::event::{ActionType, TransitionEvent, TransitionStage};
use crate::gap::GapHistory;
use crate::ids::{EventId, IntentId, KernelId, LeaseId};
use crate::intent::{Intent, IntentScheduler};
use crate::lease::LeaseManager;
use crate::resource::{IrreversibleResource, ResourceVector, ReversibleResource};
use crate::state::StateEstimate;

pub struct DifferenceConvergenceKernel {
    config: DCKConfig,
    turn: u64,
    kernel_id: KernelId,
    lease_manager: Arc<Mutex<LeaseManager>>,
    scheduler: Arc<RwLock<IntentScheduler>>,
    observer: Arc<dyn ObserverCapability>,
    predictor: Arc<dyn PredictorCapability>,
    executor: Arc<dyn ExecutorCapability>,
    semaphore: Arc<Semaphore>,
    gap_histories: Arc<Mutex<HashMap<String, GapHistory>>>,
    clock: Arc<dyn Clock>,
}

impl DifferenceConvergenceKernel {
    pub async fn tick(
        &mut self,
        current_turn: u64,
        raw_telemetry: HashMap<String, f64>,
    ) -> Result<Vec<TransitionEvent>, DCKError> {
        self.turn = current_turn;
        let now = self.clock.now();

        let runnable = {
            let sched = self.scheduler.read().await;
            sched.get_runnable(current_turn)
        };

        if runnable.is_empty() {
            return Ok(Vec::new());
        }

        let observed_states = self
            .observer
            .observe(&raw_telemetry, self.clock.as_ref(), self.config.cholesky_floor)
            .await?;

        let mut futures = Vec::new();

        for rec in runnable {
            for (m_name, goal) in &rec.intent.goals {
                let Some(obs_est) = observed_states.get(m_name) else {
                    continue;
                };

                let proj_est = self
                    .predictor
                    .forecast(
                        m_name,
                        obs_est,
                        rec.intent.time_horizon,
                        self.config.cholesky_floor,
                    )
                    .await?;

                let history_key = format!("{}_{}", rec.intent.intent_id, m_name);
                let smoothed_v = {
                    let mut histories = self.gap_histories.lock().await;
                    let gh = histories.entry(history_key).or_insert_with(|| {
                        GapHistory::new(
                            self.config.gap_history_size,
                            self.config.velocity_time_constant_tau,
                        )
                    });
                    let temp_gap = if proj_est.dim() == 1 {
                        (goal.target_value - proj_est.mean[0]).abs()
                    } else {
                        (goal.target_value - proj_est.mean.mean()).abs()
                    };
                    gh.push(now, temp_gap)
                };

                let gap = if proj_est.dim() == 1 {
                    proj_est.mahalanobis_distance_scalar(goal.target_value)
                } else {
                    let target = DVector::from_element(proj_est.dim(), goal.target_value);
                    proj_est
                        .mahalanobis_distance(&target)
                        .unwrap_or_else(|_| (proj_est.mean.mean() - goal.target_value).abs())
                };

                let event = TransitionEvent {
                    event_id: EventId::new(),
                    intent_id: rec.intent.intent_id.clone(),
                    metric_name: m_name.clone(),
                    expected_state: goal.target_value,
                    observed_state: obs_est.clone(),
                    projected_state: proj_est,
                    created_turn: current_turn,
                    created_at: now,
                    lease_id: None,
                    decision_action: None,
                    current_stage: TransitionStage::Projected,
                    computed_velocity: smoothed_v,
                    equivalence_gap: gap,
                };

                let sem = Arc::clone(&self.semaphore);
                let lease_mgr = Arc::clone(&self.lease_manager);
                let executor = Arc::clone(&self.executor);
                let config = self.config.clone();
                let clock = Arc::clone(&self.clock);

                futures.push(async move {
                    let _permit = match sem.acquire_owned().await {
                        Ok(p) => p,
                        Err(_) => {
                            return TransitionEvent {
                                current_stage: TransitionStage::Failed,
                                decision_action: Some(ActionType::SafetyHalt),
                                ..event
                            };
                        }
                    };
                    Self::execute_single_event(event, lease_mgr, executor, config, clock).await
                });
            }
        }

        let processed = join_all(futures).await;
        Ok(processed)
    }

    async fn execute_single_event(
        mut event: TransitionEvent,
        lease_manager: Arc<Mutex<LeaseManager>>,
        executor: Arc<dyn ExecutorCapability>,
        config: DCKConfig,
        clock: Arc<dyn Clock>,
    ) -> TransitionEvent {
        let raw_gap = event.equivalence_gap;

        let action = if raw_gap > config.max_gap_scale * 1.5 {
            ActionType::SafetyHalt
        } else if raw_gap < config.convergence_tolerance * 0.001 {
            ActionType::NoAction
        } else {
            // Weighted score is computed for future multi-objective extension
            let _score = config.weight_equivalence * raw_gap
                + config.weight_velocity * event.computed_velocity.abs()
                + config.weight_risk * (raw_gap / config.max_gap_scale.max(1e-9));
            ActionType::ExecuteConvergence
        };

        event.decision_action = Some(action);

        if action != ActionType::ExecuteConvergence {
            if action == ActionType::SafetyHalt {
                event.current_stage = TransitionStage::Failed;
            }
            return event;
        }

        let lease_id = LeaseId::new();
        let required_res = ResourceVector {
            rev: ReversibleResource {
                compute_cpu: raw_gap * 0.5,
                ..Default::default()
            },
            irr: IrreversibleResource {
                capital_money: raw_gap * 1.0,
                ..Default::default()
            },
        };

        let outcome = {
            let mut lm = lease_manager.lock().await;
            match lm.reserve(
                lease_id.clone(),
                event.event_id.clone(),
                required_res.clone(),
                clock.now(),
            ) {
                Ok(()) => {
                    drop(lm);
                    let success = executor.execute(action, &required_res).await;
                    let mut lm = lease_manager.lock().await;
                    let _ = lm.commit_or_release(&lease_id, success);
                    success
                }
                Err(_) => false,
            }
        };

        event.lease_id = Some(lease_id);
        event.current_stage = if outcome {
            TransitionStage::Executed
        } else {
            TransitionStage::Failed
        };

        event
    }

    pub async fn submit_intent(&self, intent: Intent) {
        let mut sched = self.scheduler.write().await;
        sched.submit(intent);
    }

    pub fn kernel_id(&self) -> &KernelId {
        &self.kernel_id
    }

    pub fn current_turn(&self) -> u64 {
        self.turn
    }
}

// =============================================================================
// Builder
// =============================================================================
pub struct KernelBuilder {
    resources: ResourceVector,
    config: DCKConfig,
    observer: Option<Arc<dyn ObserverCapability>>,
    predictor: Option<Arc<dyn PredictorCapability>>,
    executor: Option<Arc<dyn ExecutorCapability>>,
    kernel_id: KernelId,
    clock: Arc<dyn Clock>,
}

impl KernelBuilder {
    pub fn new(initial_resources: ResourceVector) -> Self {
        Self {
            resources: initial_resources,
            config: DCKConfig::default(),
            observer: None,
            predictor: None,
            executor: None,
            kernel_id: KernelId::named("dck_k01"),
            clock: Arc::new(crate::clock::SystemClock),
        }
    }

    pub fn with_config(mut self, config: DCKConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    pub fn with_kernel_id(mut self, id: KernelId) -> Self {
        self.kernel_id = id;
        self
    }

    pub fn with_capabilities(
        mut self,
        observer: Arc<dyn ObserverCapability>,
        predictor: Arc<dyn PredictorCapability>,
        executor: Arc<dyn ExecutorCapability>,
    ) -> Self {
        self.observer = Some(observer);
        self.predictor = Some(predictor);
        self.executor = Some(executor);
        self
    }

    pub fn build(self) -> Result<DifferenceConvergenceKernel, DCKError> {
        let observer = self
            .observer
            .ok_or_else(|| DCKError::ValidationError("Observer is required".into()))?;
        let predictor = self
            .predictor
            .ok_or_else(|| DCKError::ValidationError("Predictor is required".into()))?;
        let executor = self
            .executor
            .ok_or_else(|| DCKError::ValidationError("Executor is required".into()))?;

        let max_concurrency = self.config.max_concurrency_execution;

        Ok(DifferenceConvergenceKernel {
            config: self.config.clone(),
            turn: 0,
            kernel_id: self.kernel_id,
            lease_manager: Arc::new(Mutex::new(LeaseManager::new(self.resources))),
            scheduler: Arc::new(RwLock::new(IntentScheduler::new(self.config))),
            observer,
            predictor,
            executor,
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
            gap_histories: Arc::new(Mutex::new(HashMap::new())),
            clock: self.clock,
        })
    }
}
