use base::{Color, ContextTrait, FPos, Input, Pos, TextProperty, text::TextFamily};
use cosync::CosyncInput;
use creature::CreatureSprite;
use froql::{entity_store::Entity, query, world::World};
use tile_map::TileMap;
mod creature;
mod tile_map;
mod tiles;
pub mod types;
use tiles::pos_to_drawpos;
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

    if !world.singleton().has::<DeltaTime>() {
        world.singleton().add(DeltaTime(c.delta()));
        world.process();
    } else {
        world.singleton().get_mut::<DeltaTime>().0 = c.delta();
    }

    f.co.run_until_stall(world);

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
    for (e, mut player) in query!(world, &this, _ Player, mut Actor) {
        let tm = world.singleton().get::<TileMap>();
        for (input, dir) in MOVEMENTS {
            if c.is_pressed(input) {
                let new_pos = player.pos + dir;
                if !tm.is_blocked(new_pos) {
                    spawn_move_animation(f, e.id, player.pos, new_pos);
                    player.pos = new_pos;
                }
            }
        }
    }
}

fn spawn_move_animation(f: &mut FleetingState, e: Entity, start: Pos, end: Pos) {
    let start = pos_to_drawpos(start);
    let end = pos_to_drawpos(end);
    let animation_time = 0.08;
    let mut elapsed = 0.0;
    f.co.queue(move |mut input: CosyncInput<World>| async move {
        loop {
            {
                let world = input.get();
                let dt = world.singleton().get::<DeltaTime>().0;
                elapsed += dt;
                let mut draw_pos = world.get_component_mut::<DrawPos>(e);
                draw_pos.0 = start.lerp(end, elapsed / animation_time);
                if elapsed >= animation_time {
                    break;
                }
            }
            cosync::sleep_ticks(1).await;
        }
        let world = input.get();
        world.get_component_mut::<DrawPos>(e).0 = end;
    });
}

fn update_systems(c: &mut dyn ContextTrait, world: &mut World, f: &mut FleetingState) {
    if f.co.is_empty() {
        pc_inputs(c, world, f);
    }
    // pc input may queue animation
    if f.co.is_empty() {
        TileMap::update_actors(world);

        for (actor, mut draw_pos) in query!(world, Actor, mut DrawPos) {
            draw_pos.0 = pos_to_drawpos(actor.pos);
        }
    }
}

pub fn draw_systems(c: &mut dyn ContextTrait, world: &World) {
    // draw actors
    for (actor, draw_pos) in query!(world, Actor, DrawPos) {
        let (x, y) = draw_pos.0.into();
        actor.sprite.draw(c, x, y);
    }

    let tm = world.singleton().get::<TileMap>();
    tm.draw(c, 0., 0.);
}

pub fn create_world() -> World {
    let mut world = World::new();
    register_components(&mut world);

    let _player = world
        .create_mut()
        .add(Player {})
        .add(DrawPos(FPos::new(0., 0.)))
        .add(Actor { pos: Pos::new(1, 1), sprite: CreatureSprite::Dwarf });

    let _goblin = world
        .create_mut()
        .add(DrawPos(FPos::new(0., 0.)))
        .add(Actor { pos: Pos::new(2, 1), sprite: CreatureSprite::Goblin });

    let mut tm = TileMap::new(12, 12);
    tm.enwall();
    world.singleton().add(tm);

    world.process();
    world
}
