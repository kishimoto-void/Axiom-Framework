use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::error::DCKError;
use crate::ids::{EventId, LeaseId};
use crate::resource::ResourceVector;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaseState {
    Reserved,
    Activated,
    Released,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct LeaseRecord {
    pub lease_id: LeaseId,
    pub event_id: EventId,
    pub reserved_resource: ResourceVector,
    pub state: LeaseState,
    pub created_at: DateTime<Utc>,
}

pub struct LeaseManager {
    current: ResourceVector,
    active_leases: HashMap<LeaseId, LeaseRecord>,
}

impl LeaseManager {
    pub fn new(initial: ResourceVector) -> Self {
        Self {
            current: initial,
            active_leases: HashMap::new(),
        }
    }

    pub fn current_resources(&self) -> &ResourceVector {
        &self.current
    }

    pub fn reserve(
        &mut self,
        lease_id: LeaseId,
        event_id: EventId,
        required: ResourceVector,
        now: DateTime<Utc>,
    ) -> Result<(), DCKError> {
        let new_rev = self.current.rev.subtract(&required.rev)?;
        self.current.rev = new_rev;
        self.active_leases.insert(
            lease_id.clone(),
            LeaseRecord {
                lease_id,
                event_id,
                reserved_resource: required,
                state: LeaseState::Reserved,
                created_at: now,
            },
        );
        Ok(())
    }

    /// Commit irreversible resources on success, or release reversible on failure.
    pub fn commit_or_release(&mut self, lease_id: &LeaseId, success: bool) -> Result<(), DCKError> {
        let Some(record) = self.active_leases.get_mut(lease_id) else {
            return Err(DCKError::ValidationError("Lease not found".into()));
        };

        if record.state != LeaseState::Reserved {
            return Err(DCKError::ValidationError(
                "Lease is not in Reserved state".into(),
            ));
        }

        if success {
            let new_irr = self.current.irr.subtract(&record.reserved_resource.irr)?;
            self.current.irr = new_irr;
            record.state = LeaseState::Activated;
        } else {
            self.current.rev = self.current.rev.add(&record.reserved_resource.rev);
            record.state = LeaseState::Released;
        }
        Ok(())
    }

    pub fn release(&mut self, lease_id: &LeaseId) -> bool {
        if let Some(record) = self.active_leases.remove(lease_id) {
            if matches!(
                record.state,
                LeaseState::Reserved | LeaseState::Activated
            ) {
                self.current.rev = self.current.rev.add(&record.reserved_resource.rev);
                return true;
            }
        }
        false
    }
}
