use crate::error::DCKError;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReversibleResource {
    pub compute_cpu: f64,
    pub compute_gpu: f64,
    pub bandwidth: f64,
}

impl ReversibleResource {
    pub fn is_sufficient_for(&self, required: &Self) -> bool {
        self.compute_cpu >= required.compute_cpu
            && self.compute_gpu >= required.compute_gpu
            && self.bandwidth >= required.bandwidth
    }

    pub fn add(&self, other: &Self) -> Self {
        Self {
            compute_cpu: self.compute_cpu + other.compute_cpu,
            compute_gpu: self.compute_gpu + other.compute_gpu,
            bandwidth: self.bandwidth + other.bandwidth,
        }
    }

    pub fn subtract(&self, required: &Self) -> Result<Self, DCKError> {
        if !self.is_sufficient_for(required) {
            return Err(DCKError::ResourceExhausted(
                "Insufficient reversible resources".into(),
            ));
        }
        Ok(Self {
            compute_cpu: self.compute_cpu - required.compute_cpu,
            compute_gpu: self.compute_gpu - required.compute_gpu,
            bandwidth: self.bandwidth - required.bandwidth,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct IrreversibleResource {
    pub capital_money: f64,
    pub energy_power: f64,
    pub time_window: f64,
}

impl IrreversibleResource {
    pub fn is_sufficient_for(&self, required: &Self) -> bool {
        self.capital_money >= required.capital_money
            && self.energy_power >= required.energy_power
            && self.time_window >= required.time_window
    }

    pub fn subtract(&self, required: &Self) -> Result<Self, DCKError> {
        if !self.is_sufficient_for(required) {
            return Err(DCKError::ResourceExhausted(
                "Insufficient irreversible resources".into(),
            ));
        }
        Ok(Self {
            capital_money: self.capital_money - required.capital_money,
            energy_power: self.energy_power - required.energy_power,
            time_window: self.time_window - required.time_window,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResourceVector {
    pub rev: ReversibleResource,
    pub irr: IrreversibleResource,
}
