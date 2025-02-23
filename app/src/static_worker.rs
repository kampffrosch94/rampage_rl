use base::ContextTrait;
use worker::{self, update_inner};
use worker::{FleetingState, PersistentState};

pub struct StaticWorker {
    fleeting: FleetingState,
    persistent: PersistentState,
}

impl StaticWorker {
    pub fn new() -> Self {
        StaticWorker { fleeting: FleetingState::new(), persistent: PersistentState::new() }
    }

    pub fn update(&mut self, c: &mut dyn ContextTrait) {
        update_inner(c, &mut self.persistent, &mut self.fleeting);
    }
}
