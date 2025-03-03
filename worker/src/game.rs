use base::{text::TextFamily, Color, ContextTrait, FPos, Input, Pos, TextProperty};
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

pub fn update_inner(c: &mut dyn ContextTrait, s: &mut PersistentState, f: &mut FleetingState) {
    if c.is_pressed(Input::RestartGame) {
        println!("Restarting game.");
        s.restart();
        println!("Game restarted.");
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

    pc_inputs(c, world, f);
    update_systems(c, world, f);
    draw_systems(c, world);
}

fn pc_inputs(c: &mut dyn ContextTrait, world: &mut World, f: &mut FleetingState) {
    const MOVEMENTS: [(Input, (i32, i32)); 8] = [
        (Input::MoveW, (-1, 0)),
        (Input::MoveE, (1, 0)),
        (Input::MoveN, (0, -1)),
        (Input::MoveNE, (1, -1)),
        (Input::MoveNW, (-1, -1)),
        (Input::MoveS, (0, 1)),
        (Input::MoveSE, (1, 1)),
        (Input::MoveSW, (-1, 1)),
    ];
    for (mut player,) in query!(world, _ Player, mut Actor) {
        for (input, dir) in MOVEMENTS {
            if c.is_pressed(input) {
                player.pos += dir;
            }
        }
    }
}

fn update_systems(_c: &mut dyn ContextTrait, world: &mut World, f: &mut FleetingState) {
    if f.co.is_empty() {
        for (actor, mut draw_pos) in query!(world, Actor, mut DrawPos) {
            draw_pos.0 = actor.pos * 64.0;
        }
    }
}

pub fn draw_systems(c: &mut dyn ContextTrait, world: &World) {
    // draw actors
    for (actor, draw_pos) in query!(world, Actor, DrawPos) {
        let (x, y) = draw_pos.0.into();
        actor.sprite.draw(c, x, y);
    }
}

pub fn create_world() -> World {
    let mut world = World::new();
    register_components(&mut world);

    let _player = world
        .create_mut()
        .add(Player {})
        .add(DrawPos(FPos::new(0., 50.)))
        .add(Actor { pos: Pos::new(3, 3), sprite: CreatureSprite::Dwarf });

    world
}
