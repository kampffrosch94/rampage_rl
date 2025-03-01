use base::{Circle, Rect};
use froql::{query, world::World};





/// not dropped across reloads
pub struct PersistentState {
    pub world: World,
}

pub enum Inside {}

impl PersistentState {
    pub fn new() -> Self {
        let mut rand = SimpleRand::new(12345);
        let mut world = World::new();
        world.register_component::<Circle>();
        world.register_component::<Rect>();
        world.register_relation::<Inside>();

        let screen_w = 2300 / 2;
        let screen_h = 1240;

        for _ in 0..100 {
            let circle = world.create_mut();
            let x = rand.next_in_range(0, screen_w);
            let y = rand.next_in_range(0, screen_h);
            circle.add(Circle::new(x, y, 10.));
        }

        for x in [0., 0.55 * screen_w as f32] {
            for y in [0., 0.55 * screen_h as f32] {
                let rect = world.create_mut();
                rect.add(Rect { x, y, w: 0.45 * screen_w as f32, h: 0.45 * screen_h as f32 });
            }
        }

        for (e_rect, rect) in query!(world, &this, Rect) {
            for (e_circle, circle) in query!(world, &this, Circle) {
                if circle.overlaps_rect(&rect) {
                    // adding a relation here is deferred
                    // we can't mutate what we are iterating over
                    // so that we don't accidentally invalide our iterator
                    e_circle.relate_to::<Inside>(*e_rect);
                }
            }
        }
        // executes deferred operations
        world.process();

        Self { world }
    }

    pub fn re_register(&mut self) {
        unsafe {
            self.world.re_register_component::<Circle>();
            self.world.re_register_component::<Rect>();
            self.world.re_register_relation::<Inside>();
        }
    }
}

struct SimpleRand {
    seed: u32,
}

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
