use froql::world::World;

use crate::ecs_setup::{re_register_components, register_components};

/// not dropped across reloads
pub struct PersistentState {
    pub world: World,
}

impl PersistentState {
    pub fn new() -> Self {
        let mut world = World::new();
        register_components(&mut world);

        // executes deferred operations
        world.process();

        Self { world }
    }

    pub fn re_register(&mut self) {
        re_register_components(&mut self.world);
    }
}

#[allow(unused)]
struct SimpleRand {
    seed: u32,
}

#[allow(unused)]
impl SimpleRand {
    fn new(seed: u32) -> Self {
        SimpleRand { seed }
    }

    fn next(&mut self) -> u32 {
        // Constants for the LCG algorithm
        const A: u32 = 1664525;
        const C: u32 = 1013904223;
        const M: u32 = u32::MAX;

        self.seed = (A.wrapping_mul(self.seed).wrapping_add(C)) % M;
        self.seed
    }

    fn next_in_range(&mut self, min: u32, max: u32) -> f32 {
        (min + (self.next() % (max - min + 1))) as f32
    }
}
