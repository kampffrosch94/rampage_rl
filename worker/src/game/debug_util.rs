use crate::game::PendingMessage;
use crate::game::game_logic::Actor;
use base::{ContextTrait, zone};
use froql::{query, world::World};
use quicksilver::{Quicksilver, reflections::reflect};

use crate::game::ecs_types::UI;

#[derive(Quicksilver)]
pub struct DebugOptions {
    pub show_debug: bool,
    pub slow_mode: bool,
    pub slowdown_factor: i32,
}

impl Default for DebugOptions {
    fn default() -> Self {
        Self { slow_mode: false, slowdown_factor: 20, show_debug: true }
    }
}

pub fn debug_ui(c: &mut dyn ContextTrait, world: &mut World) {
    let mut s = world.singleton_mut::<DebugOptions>();
    if s.show_debug {
        zone!("show debug");
        c.inspect(&mut reflect(&mut *s));

        let mut s = world.singleton_mut::<UI>();
        c.inspect(&mut reflect(&mut *s));

        for (mut msg,) in query!(world, mut PendingMessage) {
            c.inspect(&mut reflect(&mut *msg));
        }

        let mut actors =
            query!(world, &this, _ Actor).map(|(e,)| e.entity).collect::<Vec<_>>();
        actors.sort_by_key(|e| e.id.0);
        for e in actors {
            let mut actor = world.get_component_mut::<Actor>(e);
            c.inspect(&mut reflect(&mut *actor));
        }
    }
}
