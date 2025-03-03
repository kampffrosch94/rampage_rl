use froql::world::World;

/// dropped and recreated on reload
/// you can change this definition without breaking hotreloading
pub struct FleetingState {
    pub co: cosync::Cosync<World>,
}

impl FleetingState {
    pub fn new() -> Self {
        let co = cosync::Cosync::new();

        // co.queue(move |mut input: CosyncInput<PersistentState>| async move {
        //     for _ in 0..5 {
        //         cosync::sleep_ticks(30).await;
        //         let mut _s = input.get();
        //     }
        // });

        Self { co }
    }
}
