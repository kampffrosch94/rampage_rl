use froql::world::World;

use crate::{
    game::ecs_types::{load_world, save_world},
    game::game_logic::create_world,
};

/// not dropped across reloads
#[repr(C)]
pub struct PersistentState {
    pub world: Option<World>,
    save: Option<String>,
    reload_buffer: Option<String>,
}

impl PersistentState {
    pub fn new() -> Self {
        Self { world: Some(create_world()), save: None, reload_buffer: None }
    }

    pub fn restart(&mut self) {
        self.world = Some(create_world());
    }

    // TODO make noop when not hotreloading
    pub fn re_register(&mut self) {
        // if re_register_components(&mut self.world).is_err() {
        //     println!("Error when re_registering. Restarting instead.");

        //     // gotta leak the old world because calling the old destructor on hotreload is
        //     // likely to crash
        //     let mut new_world = create_world();
        //     std::mem::swap(&mut new_world, &mut self.world);
        //     std::mem::forget(new_world);
        //     self.save = None;
        // }
    }

    pub fn save(&mut self) {
        self.save = Some(save_world(self.world.as_ref().unwrap()));
    }

    pub fn load(&mut self) {
        if let Some(save) = &self.save {
            self.world = Some(load_world(save));
        }
    }

    pub fn hot_save(&mut self) {
        self.reload_buffer = Some(save_world(self.world.as_ref().unwrap()));
        self.world = None;
    }

    pub fn hot_load(&mut self) {
        if let Some(save) = &self.reload_buffer {
            self.world = Some(load_world(save));
        }
    }
}
