use froql::world::World;

use crate::{
    game::create_world,
    game::types::{load_world, re_register_components, save_world},
};

/// not dropped across reloads
pub struct PersistentState {
    pub world: World,
    save: Option<String>,
}

impl PersistentState {
    pub fn new() -> Self {
        Self { world: create_world(), save: None }
    }

    // TODO don't leak when not hotreloading
    pub fn restart(&mut self) {
        // self.world = create_world();
        let mut new_world = create_world();
        std::mem::swap(&mut new_world, &mut self.world);
        std::mem::forget(new_world);
    }

    // TODO make noop when not hotreloading
    pub fn re_register(&mut self) {
        re_register_components(&mut self.world);
    }

    pub fn save(&mut self) {
        self.save = Some(save_world(&self.world));
    }

    // TODO don't leak when not hotreloading
    pub fn load(&mut self) {
        if let Some(save) = &self.save {
            // self.world = load_world(save);
            let mut new_world = load_world(save);
            std::mem::swap(&mut new_world, &mut self.world);
            std::mem::forget(new_world);
        }
    }
}
