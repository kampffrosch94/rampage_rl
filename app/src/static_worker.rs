use base::ContextTrait;
use worker::PersistentState;
use worker::{self, update_inner};

pub struct StaticWorker {
    persistent: PersistentState,
}

impl StaticWorker {
    pub fn new() -> Self {
        StaticWorker { persistent: PersistentState::new() }
    }

    pub fn update(&mut self, c: &mut dyn ContextTrait) {
        update_inner(c, &mut self.persistent);
    }
}
