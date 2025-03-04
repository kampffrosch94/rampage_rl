use base::{Color, ContextTrait, FPos, Input, Pos, Rect, TextProperty, text::TextFamily};
use cosync::CosyncInput;
use creature::CreatureSprite;
use froql::{entity_store::Entity, query, world::World};
use tile_map::TileMap;
mod creature;
mod tile_map;
mod tiles;
pub mod types;
use tiles::{Decor, TILE_SIZE, pos_to_drawpos};
use types::*;

use crate::{fleeting::FleetingState, persistent::PersistentState, rand::RandomGenerator};

#[repr(C)]
enum Label {
    ExampleText,
    UITest,
}

pub const Z_TILES: i32 = 0;
pub const Z_HP_BAR: i32 = 9;
pub const Z_SPRITE: i32 = 10;
pub const Z_UI_BG: i32 = 1000;
pub const Z_UI_TEXT: i32 = 1100;

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
    c.draw_texture("monsters", -1100., -950., 5);

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

    let ui_rect = c.screen_rect().take_right(300.);
    let text_rect = ui_rect.skip_all(10.);
    c.set_text(Label::UITest as _, text_rect.w, text_rect.h, &[("Test", TextProperty::new())]);
    c.draw_text(Label::UITest as _, text_rect.x, text_rect.y, Z_UI_TEXT);
    c.draw_rect(ui_rect, Color::BLACK, Z_UI_BG);
    c.draw_rect_lines(ui_rect, 5.0, Color::GRAY, Z_UI_BG);

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
                } else {
                    if let Some(other) = tm.get_actor(new_pos) {
                        spawn_bump_attack_animation(f, e.id, player.pos, new_pos, other, 1);
                    }
                }
            }
        }
    }
}

fn spawn_bump_attack_animation(
    f: &mut FleetingState,
    e: Entity,
    p_start: Pos,
    p_end: Pos,
    target: Entity,
    damage: i32,
) {
    let start = pos_to_drawpos(p_start);
    let end = pos_to_drawpos(p_end);
    let animation_time = 0.15;
    let mut elapsed = 0.0;
    const PART_FORWARD: f32 = 0.5;
    f.co.queue(move |mut input: CosyncInput<World>| async move {
        loop {
            {
                let world = input.get();
                let dt = world.singleton().get::<DeltaTime>().0;
                elapsed += dt;
                let mut draw_pos = world.get_component_mut::<DrawPos>(e);
                if elapsed < animation_time * PART_FORWARD {
                    let lerpiness = (elapsed / animation_time) * 1.5;
                    draw_pos.0 = start.lerp(end, lerpiness);
                } else {
                    // let lerpiness = (elapsed / animation_time) * 1.5;
                    let lerpiness = elapsed / animation_time;
                    draw_pos.0 = end.lerp(start, lerpiness);
                    // TODO calculate and apply damage
                    // TODO healthbar
                    // TODO death
                }
                if elapsed >= animation_time * PART_FORWARD {
                    break;
                }
            }
            cosync::sleep_ticks(1).await;
        }
        let world = input.get();
        world.get_component_mut::<DrawPos>(e).0 = start;
        let mut tm = world.singleton().get_mut::<TileMap>();
        let mut rand = world.singleton().get_mut::<RandomGenerator>();
        let decor_pos = p_end + rand.random_direction();
        let decor = rand.pick_random(&[Decor::BloodRed1, Decor::BloodRed2]);
        tm.add_decor(decor_pos, decor);
        world.get_component_mut::<Actor>(target).hp.current -= damage;
    });
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

        for (e, actor, mut draw_pos) in query!(world, &this, Actor, mut DrawPos) {
            draw_pos.0 = pos_to_drawpos(actor.pos);
            if actor.hp.current <= 0 {
                e.destroy();
            }
        }

        world.process();
    }
}

pub fn draw_systems(c: &mut dyn ContextTrait, world: &World) {
    // draw actors
    for (actor, draw_pos) in query!(world, Actor, DrawPos) {
        let (x, y) = draw_pos.0.into();
        actor.sprite.draw(c, x, y);

        if actor.hp.current < actor.hp.max {
            let hp_percent = actor.hp.current as f32 / actor.hp.max as f32;
            let rect = Rect::new(x + TILE_SIZE * 0.1, y + TILE_SIZE, TILE_SIZE * 0.8, 5.);
            c.draw_rect(rect, Color::RED, Z_HP_BAR);
            let rect = Rect::new(
                x + TILE_SIZE * 0.1,
                y + TILE_SIZE,
                TILE_SIZE * 0.8 * hp_percent,
                5.,
            );
            c.draw_rect(rect, Color::GREEN, Z_HP_BAR);
        }
    }

    let tm = world.singleton().get::<TileMap>();
    tm.draw(c, 0., 0.);
}

pub fn create_world() -> World {
    let mut world = World::new();
    register_components(&mut world);

    let _player = world
        .create_mut()
        .add(Player { pulse: 60., last_action: 0 })
        .add(DrawPos(FPos::new(0., 0.)))
        .add(Actor {
            pos: Pos::new(1, 1),
            sprite: CreatureSprite::Dwarf,
            hp: HP { max: 10, current: 10 },
        });

    let _goblin = world.create_mut().add(DrawPos(FPos::new(0., 0.))).add(Actor {
        pos: Pos::new(4, 4),
        sprite: CreatureSprite::Goblin,
        hp: HP { max: 5, current: 5 },
    });

    let mut tm = TileMap::new(12, 12);
    tm.enwall();
    // TODO get properly random seed
    world.singleton().add(tm).add(RandomGenerator::new(12345));

    world.process();
    world
}
