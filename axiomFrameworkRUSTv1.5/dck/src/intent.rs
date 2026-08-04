use std::collections::{HashMap, HashSet};

use crate::config::DCKConfig;
use crate::ids::IntentId;

#[derive(Debug, Clone)]
pub struct MetricGoal {
    pub target_value: f64,
    pub tolerance: f64,
}

#[derive(Debug, Clone)]
pub struct Intent {
    pub intent_id: IntentId,
    pub description: String,
    pub goals: HashMap<String, MetricGoal>,
    pub time_horizon: u32,
    pub created_turn: u64,
    pub base_priority: f64,
    pub deadline_turn: Option<u64>,
    pub dependencies: Vec<IntentId>,
}

impl Intent {
    pub fn effective_priority(&self, current_turn: u64, config: &DCKConfig) -> f64 {
        let age = current_turn.saturating_sub(self.created_turn) as f64;
        let aging = (1.0 + age).ln() * config.aging_factor;
        let deadline_factor = match self.deadline_turn {
            Some(dt) => {
                let remaining = dt.saturating_sub(current_turn) as f64;
                (remaining / 10.0).max(0.1)
            }
            None => 1.0,
        };
        self.base_priority + (aging / deadline_factor)
    }
}

#[derive(Debug, Clone)]
pub struct IntentRecord {
    pub intent: Intent,
    pub is_active: bool,
    pub is_completed: bool,
}

pub struct IntentScheduler {
    config: DCKConfig,
    records: HashMap<IntentId, IntentRecord>,
    completed: HashSet<IntentId>,
}

impl IntentScheduler {
    pub fn new(config: DCKConfig) -> Self {
        Self {
            config,
            records: HashMap::new(),
            completed: HashSet::new(),
        }
    }

    pub fn submit(&mut self, intent: Intent) {
        let id = intent.intent_id.clone();
        self.records.insert(
            id,
            IntentRecord {
                intent,
                is_active: false,
                is_completed: false,
            },
        );
    }

    pub fn mark_completed(&mut self, id: &IntentId) {
        if let Some(rec) = self.records.get_mut(id) {
            rec.is_completed = true;
            self.completed.insert(id.clone());
        }
    }

    pub fn get_runnable(&self, current_turn: u64) -> Vec<IntentRecord> {
        let mut runnable: Vec<_> = self
            .records
            .values()
            .filter(|r| {
                if r.is_completed {
                    return false;
                }
                if let Some(dl) = r.intent.deadline_turn {
                    if current_turn > dl {
                        return false;
                    }
                }
                r.intent
                    .dependencies
                    .iter()
                    .all(|dep| self.completed.contains(dep))
            })
            .cloned()
            .collect();

        runnable.sort_by_cached_key(|r| {
            let p = r.intent.effective_priority(current_turn, &self.config);
            ordered_float_key(-p)
        });

        runnable
    }
}

fn ordered_float_key(v: f64) -> u64 {
    let bits = v.to_bits();
    if v.is_sign_negative() {
        !bits
    } else {
        bits | (1u64 << 63)
    }
}
