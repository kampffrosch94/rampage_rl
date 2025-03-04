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
        self.world = create_world();
    }

    // TODO make noop when not hotreloading
    pub fn re_register(&mut self) {
        if re_register_components(&mut self.world).is_err() {
            println!("Error when re_registering. Restarting instead.");

            // gotta leak the old world because calling the old destructor on hotreload is
            // likely to crash
            let mut new_world = create_world();
            std::mem::swap(&mut new_world, &mut self.world);
            std::mem::forget(new_world);
            self.save = None;
        }
    }

    pub fn save(&mut self) {
        self.save = Some(save_world(&self.world));
    }

    // TODO don't leak when not hotreloading
    pub fn load(&mut self) {
        if let Some(save) = &self.save {
            self.world = load_world(save);
        }
    }
}
