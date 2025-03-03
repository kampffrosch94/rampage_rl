use base::{text::TextFamily, Color, ContextTrait, Input, Pos, TextProperty};
use creature::CreatureSprite;
use froql::{query, world::World};
use tile_map::TileMap;
mod creature;
mod tile_map;
mod tiles;
pub mod types;
use types::*;

use crate::{fleeting::FleetingState, persistent::PersistentState};

#[repr(C)]
enum Label {
    ExampleText,
}

pub const Z_TILES: i32 = 0;

pub fn update_inner(c: &mut dyn ContextTrait, s: &mut PersistentState, _f: &mut FleetingState) {
    if c.is_pressed(Input::RestartGame) {
        println!("Restarting game.");
        s.world = create_world();
    }

    s.re_register();

    if c.is_pressed(Input::Save) {
        println!("Saving game.");
        s.save();
    }
    if c.is_pressed(Input::Load) {
        println!("Loading game.");
        s.load();
    }

    let world = &mut s.world;

    c.draw_texture("tiles", -200., -950., 5);
    c.draw_texture("rogues", -600., -950., 5);

    c.set_text(
        Label::ExampleText as _,
        500.,
        500.,
        &[
            (
                "Placeholder of doom",
                TextProperty::new()
                    .family(TextFamily::BloodCyrillic)
                    .metrics(66, 80)
                    .color(Color::RED),
            ),
            ("\n \n", TextProperty::new().metrics(2, 20)),
            (
                r#"a) Run run run!
b) Walk walk walk!"#,
                TextProperty::new()
                    .family(TextFamily::BloodCyrillic)
                    .color(Color::PINK)
                    .metrics(44, 60),
            ),
        ],
    );
    c.draw_text(Label::ExampleText as _, 400., -530., 30);

    let mut tm = TileMap::new(12, 12);
    tm.enwall();
    tm.draw(c, 0., 0.);

    draw_systems(c, world);
}

pub fn draw_systems(c: &mut dyn ContextTrait, world: &World) {
    // draw actors
    // TODO use draw pos as component
    for (actor,) in query!(world, Actor) {
        let draw_pos = actor.pos * 64.0;
        actor.sprite.draw(c, draw_pos.x, draw_pos.y);
    }
}

pub fn create_world() -> World {
    let mut world = World::new();
    register_components(&mut world);

    let _player = world
        .create_mut()
        .add(Player {})
        .add(Actor { pos: Pos::new(3, 3), sprite: CreatureSprite::Dwarf });

    world
}
