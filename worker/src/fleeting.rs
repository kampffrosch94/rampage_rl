use crate::coroutines::CoroutineRuntime;

/// dropped and recreated on reload
/// you can change this definition without breaking hotreloading
pub struct FleetingState {
    pub co: CoroutineRuntime,
}

impl FleetingState {
    pub fn new() -> Self {
        let co = CoroutineRuntime::new();
        Self { co }
    }
}
