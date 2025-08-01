use std::{collections::HashSet, ops::Not};

use base::{Circle, Color, ContextTrait, FPos, Input, Pos, Rect, grids::Grid, shadowcasting};
use creature::CreatureSprite;
use froql::{entity_store::Entity, query, world::World};
use mapgen::{generate_map, place_enemies};
use quicksilver::reflections::reflect;
use tile_map::{DecorWithPos, TileMap};
pub mod creature;
pub mod tile_map;
pub mod tiles;
pub mod types;
use tiles::{
    Decor, DrawTile, Environment, LogicTile, TILE_SIZE, generate_draw_tile, pos_to_drawpos,
};
use types::*;
use ui::handle_ui;
pub mod mapgen;
pub mod ui;

use crate::{
    animation,
    animation::AnimationTarget,
    coroutines::{CoAccess, CoroutineStore, sleep_ticks},
    dijstra::{dijkstra, dijkstra_path},
    fleeting::FleetingState,
    persistent::PersistentState,
    rand::RandomGenerator,
};

pub const Z_TILES: i32 = 0;
pub const Z_HP_BAR: i32 = 9;
pub const Z_SPRITE: i32 = 10;
pub const Z_DEBUG: i32 = 999;
pub const Z_UI_BG: i32 = 1000;
pub const Z_UI_TEXT: i32 = 1100;

pub fn highlight_tile(c: &mut dyn ContextTrait, pos: Pos) {
    let rect =
        Rect::new(pos.x as f32 * TILE_SIZE, pos.y as f32 * TILE_SIZE, TILE_SIZE, TILE_SIZE);
    c.draw_rect(rect, Color::YELLOW, Z_DEBUG);
}

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

    // add coroutines stored in previous frame
    if !world.singleton_has::<CoroutineStore>() {
        world.singleton_add(CoroutineStore::new());
    } else {
        f.co.add_from_store(&mut world.singleton_mut::<CoroutineStore>());
    }

    if !world.singleton_has::<GameTime>() {
        world.singleton_add(GameTime(0.));
    } else {
        world.singleton_mut::<GameTime>().0 += c.delta();
    }

    // f.co.run_step(world);
    animation::handle_animations(world);

    c.draw_texture("tiles", -228., -950., 5);
    c.draw_texture("rogues", -600., -950., 5);
    c.draw_texture("monsters", -1100., -950., 5);
    c.draw_circle(Circle { pos: FPos::new(50., 60.), radius: 30. }, Color::WHITE, 15);

    handle_ui(c, world, f);
    update_systems(c, world);
    draw_systems(c, world);

    highlight_tile(c, Pos::new(1, 1));

    for (mut actor,) in query!(world, mut Actor) {
        c.inspect(&mut reflect(&mut *actor));
    }
}

fn pc_inputs(c: &mut dyn ContextTrait, world: &mut World) {
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
        let tm = world.singleton::<TileMap>();
        for (input, dir) in MOVEMENTS {
            if c.is_pressed(input) {
                let new_pos = player.pos + dir;
                if !tm.is_blocked(new_pos) {
                    animation::spawn_move_animation(world, *e, player.pos, new_pos);
                    player.pos = new_pos;
                } else {
                    if let Some(other) = tm.get_actor(new_pos) {
                        animation::spawn_bump_attack_animation(
                            world, *e, other, player.pos, new_pos, 1,
                        );
                    }
                }
                player.next_turn += 10;
            }
        }
    }
}

fn ai_turn(_c: &mut dyn ContextTrait, world: &mut World, npc: Entity) {
    // set up pathfinding dijsktra map
    let start = world.get_component::<Actor>(npc).pos;
    let tm = world.singleton::<TileMap>();
    let grid = {
        let mut grid = Grid::new(tm.tiles.width, tm.tiles.height, 0);
        let mut seeds = Vec::new();
        for (player,) in query!(world, Actor, _ Player) {
            grid[player.pos] = 25;
            seeds.push(player.pos);
        }
        let cost_function = |pos| if tm.is_blocked(pos) && pos != start { 999999 } else { 1 };
        dijkstra(&mut grid, &seeds, cost_function);
        grid
    };

    // do pathfinding
    let start = world.get_component::<Actor>(npc).pos;
    let path = dijkstra_path(&grid, start);

    if path.len() > 1
        && let Some(next) = path[1..].first()
        && !tm.is_blocked(*next)
    {
        let mut actor = world.get_component_mut::<Actor>(npc);
        animation::spawn_move_animation(world, npc, actor.pos, *next);
        actor.pos = *next;
        actor.next_turn += 10;
    } else {
        // TODO add attacking player
        let mut actor = world.get_component_mut::<Actor>(npc);
        actor.next_turn += 10;
    }
}

fn next_turn_actor(world: &World) -> Entity {
    query!(world, &this, Actor)
        .min_by_key(|(e, a)| (a.next_turn, e.id.0))
        .map(|(e, _a)| e.entity)
        .unwrap()
}

fn should_block_input(world: &World) -> bool {
    query!(world, Player, AnimationTarget(_, this)).next().is_some()
}

fn update_systems(c: &mut dyn ContextTrait, world: &mut World) {
    TileMap::update_actors(world);
    if should_block_input(world).not() {
        let mut next = next_turn_actor(world);
        while !world.has_component::<Player>(next) {
            ai_turn(c, world, next);
            TileMap::update_actors(world);
            next = next_turn_actor(world);
        }
        pc_inputs(c, world);
        world.process();
    }

    // pc input may queue blocking animation
    if should_block_input(world).not() {
        for (e, actor, mut draw_pos) in
            query!(world, &this, Actor, mut DrawPos, !AnimationTarget(_, this))
        {
            // update draw position
            draw_pos.0 = pos_to_drawpos(actor.pos);
            // remove dead actors
            if actor.hp.current <= 0 {
                e.destroy();
            }
        }
        world.process();

        // update player fov
        for (tm, actor, mut fov) in query!(world, $ TileMap, _ Player, Actor, mut Fov) {
            shadowcasting::compute_fov(
                actor.pos,
                &mut |pos| tm.blocks_vision(pos),
                &mut |pos| {
                    fov.0.insert(pos);
                },
            );
        }
    }
}

pub fn draw_systems(c: &mut dyn ContextTrait, world: &World) {
    let (fov,) = query!(world, Fov, _ Player).next().unwrap();

    // draw tile map
    {
        let tm = world.singleton::<TileMap>();
        let x_base = 0.;
        let y_base = 0.;
        let env = Environment::Catacomb;

        for (pos, lt) in tm.tiles.iter_coords() {
            if !fov.0.contains(&pos) {
                continue;
            }

            let mut pos_below = pos.clone();
            pos_below.y += 1;
            let below = (&tm).tiles.get_opt(pos_below).unwrap_or(&LogicTile::Empty);
            let draw_tile = generate_draw_tile(*lt, env, *below);
            let x = x_base + pos.x as f32 * TILE_SIZE;
            let y = y_base + pos.y as f32 * TILE_SIZE;
            draw_tile.draw(c, x, y);
        }

        // up and down stairs
        {
            let pos = tm.up_stairs;
            let x = x_base + pos.x as f32 * TILE_SIZE;
            let y = y_base + pos.y as f32 * TILE_SIZE;
            if fov.0.contains(&pos) {
                DrawTile::UpStairs.draw(c, x, y);
            }

            let pos = tm.down_stairs;
            let x = x_base + pos.x as f32 * TILE_SIZE;
            let y = y_base + pos.y as f32 * TILE_SIZE;
            if fov.0.contains(&pos) {
                DrawTile::DownStairs.draw(c, x, y);
            }
        }

        for DecorWithPos(pos, decor) in &tm.decor {
            if !fov.0.contains(&pos) {
                continue;
            }

            let x = x_base + pos.x as f32 * TILE_SIZE;
            let y = y_base + pos.y as f32 * TILE_SIZE;
            decor.draw(c, x, y);
        }
    };

    // draw actors
    for (actor, draw_pos) in query!(world, Actor, DrawPos) {
        if !fov.0.contains(&actor.pos) {
            continue;
        }

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
}

pub fn create_world() -> World {
    let mut world = World::new();
    register_components(&mut world);

    // TODO get properly random seed
    let tm = generate_map(12345);

    let _player = world
        .create()
        .add(Player { pulse: 60., last_action: 0 })
        .add(DrawPos(FPos::new(0., 0.)))
        .add(Actor {
            name: "Player".into(),
            pos: tm.up_stairs,
            sprite: CreatureSprite::Dwarf,
            hp: HP { max: 10, current: 10 },
            next_turn: 0,
        })
        .add(Fov(HashSet::new()));

    world.singleton_add(tm);
    world.singleton_add(TurnCount { aut: 0 });
    place_enemies(&mut world, 12345);

    world.singleton_add(RandomGenerator::new(12345));

    world.process();
    world
}
