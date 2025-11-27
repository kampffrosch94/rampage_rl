use std::collections::HashSet;

use crate::animation::{AnimationCleanup, AnimationTarget};
use base::pos::IVec;
use base::text::Labelize;
use base::zone;
#[allow(unused)]
use base::{Circle, Color, ContextTrait, FPos, Input, Pos, Rect, grids::Grid, shadowcasting};
use base::{FVec, TextProperty};
use creature::CreatureSprite;
use froql::{entity_store::Entity, query, world::World};
use mapgen::{generate_map, place_enemies};
use quicksilver::Quicksilver;
#[allow(unused)]
use quicksilver::reflections::reflect;
use tile_map::{DecorWithPos, TileMap};
pub mod creature;
pub mod tile_map;
pub mod tiles;
pub mod types;
use tiles::{DrawTile, Environment, LogicTile, TILE_SIZE, generate_draw_tile, pos_to_drawpos};
use types::*;
use ui::{MessageLog, handle_ui, log_message, ui_inventory};
pub mod mapgen;
pub mod ui;
use crate::game::ui::PendingMessage;
use crate::{
    animation::{self},
    dijkstra::{dijkstra, dijkstra_path},
    game::DrawHealth,
    persistent::PersistentState,
    rand::RandomGenerator,
};

pub const Z_TILES: i32 = 0;
pub const Z_HP_BAR: i32 = 9;
pub const Z_SPRITE: i32 = 10;
pub const Z_PROJECTILE: i32 = 15;
pub const Z_CURSOR: i32 = 100;
#[allow(unused)]
pub const Z_DEBUG: i32 = 999;
pub const Z_AVY_LABEL: i32 = 1000;
pub const Z_MESSAGE_BG: i32 = 2000;
pub const Z_MESSAGE_TEXT: i32 = 2100;
pub const Z_SIDEBAR_BG: i32 = 3000;
pub const Z_SIDEBAR_TEXT: i32 = 3100;
pub const Z_INVENTORY_BG: i32 = 3000;
pub const Z_INVENTORY_TEXT: i32 = 3100;
pub const Z_GAME_OVER: i32 = 5000;

#[derive(Quicksilver)]
pub struct DebugOptions {
    show_debug: bool,
    slow_mode: bool,
    slowdown_factor: i32,
}

impl Default for DebugOptions {
    fn default() -> Self {
        Self { slow_mode: false, slowdown_factor: 20, show_debug: true }
    }
}

pub fn ensure_singleton<T: Default + 'static>(world: &mut World) {
    if !world.singleton_has::<T>() {
        world.singleton_add(T::default());
    }
}

pub fn ability_key_pressed(c: &mut dyn ContextTrait) -> Option<usize> {
    let inputs =
        [Input::Ability1, Input::Ability2, Input::Ability3, Input::Ability4, Input::Ability5];
    for (nr, key) in inputs.into_iter().enumerate() {
        if c.is_pressed(key) {
            return Some(nr + 1);
        }
    }
    None
}

pub fn update_inner(c: &mut dyn ContextTrait, s: &mut PersistentState) {
    zone!();
    if c.is_pressed(Input::RestartGame) {
        println!("Restarting game.");
        s.restart();
        println!("Game restarted.");
    }

    if c.is_pressed(Input::Save) {
        println!("Saving game.");
        s.save();
    }
    if c.is_pressed(Input::Load) {
        println!("Loading game.");
        s.load();
    }

    let world = s.world.as_mut().unwrap();

    ensure_singleton::<GameTime>(world);
    ensure_singleton::<DebugOptions>(world);

    let state: UIState = world.singleton::<UI>().state;

    // systematic input
    match state {
        UIState::Normal | UIState::Ability | UIState::Inspect => {
            if c.mouse_wheel() > 0. {
                c.camera_zoom(1);
            }
            if c.mouse_wheel() < 0. {
                c.camera_zoom(-1);
            }

            {
                let mut ui = world.singleton_mut::<UI>();
                if c.is_pressed(Input::MouseMoveCamera)
                    && let Some(last_pos) = ui.last_mouse_pos
                {
                    let current = c.mouse_world();
                    c.camera_move_rel(last_pos - current);
                }

                ui.last_mouse_pos = Some(c.mouse_world());
            }

            if c.is_pressed(Input::DebugSlowdown) {
                let mut debug = world.singleton_mut::<DebugOptions>();
                debug.slow_mode = !debug.slow_mode;
            }
            if c.is_pressed(Input::DebugToggle) {
                let mut debug = world.singleton_mut::<DebugOptions>();
                debug.show_debug = !debug.show_debug;
            }

            {
                let debug = world.singleton::<DebugOptions>();
                let delta = if debug.slow_mode {
                    let extra_factor = 100.0; // here because of precision issues
                    (extra_factor * c.delta()) / (extra_factor * debug.slowdown_factor as f32)
                } else {
                    c.delta()
                };
                world.singleton_mut::<GameTime>().0 += delta;
            }
        }
        UIState::Inventory => {}
        UIState::GameOver => {
            ui_game_over(c, world);
            return;
        }
    }

    c.draw_texture("tiles", -228., -950., 5);
    c.draw_texture("rogues", -600., -950., 5);
    c.draw_texture("monsters", -1100., -950., 5);
    // c.draw_circle(Circle { pos: FPos::new(50., 60.), radius: 30. }, Color::WHITE, 15);

    handle_ui(c, world);
    update_systems(c, world);
    animation::handle_animations(c, world);
    draw_systems(c, world);

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

pub fn ui_game_over(c: &mut dyn ContextTrait, world: &mut World) {
    let r = c.screen_rect();
    let dim = FVec::new(1000., 300.);
    let pos = FPos::new(r.center().x - dim.x / 2.0, r.center().y - dim.y / 2.0);
    "IT'S ALL OGRE NOW"
        .labelize_prop(c, dim, TextProperty::new().color(Color::RED).size(100.))
        .draw(c, pos, Z_GAME_OVER);

    if c.is_pressed(Input::Confirm) {
        *world = create_world();
    }
}

fn input_direction(c: &mut dyn ContextTrait) -> Option<IVec> {
    zone!();
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

    for (input, (dx, dy)) in MOVEMENTS {
        if c.is_pressed(input) {
            return Some(IVec::new(dx, dy));
        }
    }
    None
}

fn direction_input(c: &mut dyn ContextTrait, world: &mut World) -> Option<Action> {
    zone!();
    if !matches!(world.singleton::<UI>().state, UIState::Normal) {
        return None;
    }

    if let Some(dir) = input_direction(c) {
        for (e, player) in query!(world, &this, _ Player, mut Actor) {
            let tm = world.singleton::<TileMap>();
            let new_pos = player.pos + dir;
            if !tm.is_blocked(new_pos) {
                return Some(Action {
                    actor: *e,
                    kind: ActionKind::Move { from: player.pos, to: new_pos },
                });
            } else {
                if let Some(other_e) = tm.get_actor(new_pos) {
                    return Some(Action {
                        actor: *e,
                        kind: ActionKind::BumpAttack { target: other_e },
                    });
                }
            }
        }
    }

    None
}

fn handle_death(world: &World, target: Entity, target_a: &Actor, animation: Entity) {
    zone!();
    if target_a.hp.current <= 0 {
        let msg = format!("{} dies.", target_a.name);
        log_message(world, msg, animation);

        if world.has_component::<Player>(target) {
            animation::spawn_game_over_animation(world, target);
        } else {
            world.view_deferred(target).relate_from::<AnimationCleanup>(animation);
        }
    }
}

fn handle_action(world: &World, action: Action) {
    zone!();
    match action {
        Action { actor, kind: ActionKind::Move { from, to } } => {
            let anim = animation::spawn_move_animation(world, actor, from, to);
            world.get_component_mut::<Actor>(actor).pos = to;
            if world.has_component::<Player>(actor) {
                animation::add_camera_move(world, anim, to);
            }
            let mut actor_a = world.get_component_mut::<Actor>(actor);
            actor_a.next_turn += 10;
        }
        Action { actor, kind: ActionKind::BumpAttack { target } } => {
            assert_ne!(actor, target);
            let mut actor_a = world.get_component_mut::<Actor>(actor);
            let mut target_a = world.get_component_mut::<Actor>(target);
            let hp_bar_change = target_a.hp.dmg(3);
            let animation = animation::spawn_bump_attack_animation(
                world,
                actor,
                target,
                actor_a.pos,
                target_a.pos,
                hp_bar_change,
            );
            let msg = format!("{} attacks {}.", actor_a.name, target_a.name);
            log_message(world, msg, animation);

            handle_death(world, target, &target_a, animation);

            actor_a.next_turn += 10;
        }
        Action {
            actor,
            kind: ActionKind::UseAbility(Ability::RockThrow { path, target }),
        } => {
            let mut target_a = world.get_component_mut::<Actor>(target);
            let hp_bar_change = target_a.hp.dmg(2);
            let animation = animation::spawn_projectile_animation(
                world,
                DrawTile::Rock,
                path,
                hp_bar_change,
                target,
            );

            let mut actor_a = world.get_component_mut::<Actor>(actor);
            let msg = format!("{} throws a huge rock at {}.", actor_a.name, target_a.name);
            log_message(world, msg, animation);
            actor_a.next_turn += 10;

            handle_death(world, target, &target_a, animation);
        }
    };
}

fn player_inputs(c: &mut dyn ContextTrait, world: &mut World) {
    zone!();
    let action = direction_input(c, world).or_else(|| ability_input(c, world));
    // TODO make general handler for inner action
    if let Some(action) = action {
        handle_action(world, action);
        world.process();
        return;
    }

    if let Some(nr) = ability_key_pressed(c) {
        println!("Ability pressed: {nr}");
        world.singleton_mut::<UI>().state = UIState::Ability;
        return;
    }

    if c.is_pressed(Input::Inventory) {
        world.singleton_mut::<UI>().state = UIState::Inventory;
        return;
    }

    if c.is_pressed(Input::Confirm)
        && let Some((player, mut p_actor)) = query!(world, &this, _ Player, mut Actor).next()
        && world.singleton::<TileMap>().down_stairs == p_actor.pos
    {
        println!("TODO: Go down the stairs.");
        let player = player.entity;

        // clean up NPCs on old level
        for (e,) in query!(world, !Player, _ Actor, &this) {
            e.destroy();
        }

        // generate new level and place player there
        let mut rand = world.singleton_mut::<RandomGenerator>();
        let seed = rand.next();
        let tm = generate_map(seed);
        p_actor.pos = tm.up_stairs;
        let start_aut = p_actor.next_turn;
        world.defer_closure(move |world| {
            world.singleton_add(tm);
            place_enemies(world, seed);
            for (mut actor,) in query!(world, !Player, mut Actor) {
                actor.next_turn = start_aut;
            }
            world.add_component(player, Fov(HashSet::new()));
        });

        // move camera to center on player
        animation::spawn_camera_move(world, player, p_actor.pos);


        return;
    }

    if c.is_pressed(Input::Inspect) {
        world.singleton_mut::<UI>().state = UIState::Inspect;
        return;
    }

    // TODO this is temporary and needs to get rolled into an ability
    if c.is_pressed(Input::Test) {
        let animation = animation::spawn_camera_shake_animation(world);

        // find enemies around player and damage them
        let (player_actor,) = query!(world, _ Player, mut Actor).next().unwrap();
        let player_pos = player_actor.pos;
        let tm = world.singleton::<TileMap>();
        for pos in player_pos.neighbors(&tm.tiles) {
            if let Some(neighbor_e) = tm.get_actor(pos) {
                let mut neighbor_actor = world.get_component_mut::<Actor>(neighbor_e);
                neighbor_actor.hp.current -= 1;
                handle_death(world, neighbor_e, &neighbor_actor, animation);
            }
        }

        return;
    }
}

// TODO return action instead of handling it directly
fn ai_turn(world: &mut World, npc: Entity) {
    zone!();
    {
        let mut actor = world.get_component_mut::<Actor>(npc);
        if actor.hp.current <= 0 {
            // dead actors wait for further handling
            // might be better to handle it with a death marker component or something
            // that could then exclude actors from the turn order
            actor.next_turn += 100;
            return;
        }
    }

    // set up pathfinding dijsktra map
    let start = world.get_component::<Actor>(npc).pos;
    let tm = world.singleton::<TileMap>();
    let grid = {
        let mut grid = Grid::new(tm.tiles.width, tm.tiles.height, 0);
        let mut seeds = Vec::new();
        for (player,) in query!(world, Actor, _ Player) {
            grid[player.pos] = 500;
            seeds.push(player.pos);
        }
        let cost_function = |pos| {
            if tm.is_wall(pos) && pos != start {
                i32::MAX
            } else if tm.get_actor(pos).is_some() {
                25
            } else {
                1
            }
        };
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
        let action = Action { actor: npc, kind: ActionKind::Move { from: start, to: *next } };
        handle_action(world, action);
    } else if path.len() > 1
        && let Some(next) = path[1..].first()
        && let Some(target) = tm.actors.get(next)
        && world.has_component::<Player>(*target)
    {
        // attack player if the player is the thing blocking movement
        let action = Action { actor: npc, kind: ActionKind::BumpAttack { target: *target } };
        handle_action(world, action);
    } else {
        // just stand in place
        // TODO add wait action
        let mut actor = world.get_component_mut::<Actor>(npc);
        actor.next_turn += 10;
    }
}

fn next_turn_actor(world: &World) -> Entity {
    zone!();
    query!(world, &this, Actor)
        .min_by_key(|(e, a)| (a.next_turn, e.id.0))
        .map(|(e, _a)| e.entity)
        .unwrap()
}

/// returns true if there is some animation targeting the player
fn player_is_animation_target(world: &World) -> bool {
    zone!();
    query!(world, Player, AnimationTarget(_, this)).next().is_some()
}

fn update_systems(c: &mut dyn ContextTrait, world: &mut World) {
    zone!();
    let state: UIState = world.singleton::<UI>().state;
    match state {
        UIState::Normal | UIState::Ability => update_systems_normal(c, world),
        UIState::Inventory => update_systems_inventory(c, world),
        UIState::Inspect => update_systems_inspect(c, world),
        state @ UIState::GameOver => {
            unreachable!("This branch should not be reached. State: {state:?}")
        }
    };
}

#[derive(Default, Quicksilver)]
pub struct AbilityUIState {
    cursor_pos: Option<Pos>,
}

fn ability_input(c: &mut dyn ContextTrait, world: &mut World) -> Option<Action> {
    zone!();
    if !matches!(world.singleton::<UI>().state, UIState::Ability) {
        return None;
    }

    fn exit_ability_state(world: &mut World) {
        world.singleton_mut::<UI>().state = UIState::Normal;
        world.singleton_remove::<AbilityUIState>();
    }

    if c.is_pressed(Input::Cancel) {
        exit_ability_state(world);
        return None;
    }

    ensure_singleton::<AbilityUIState>(world);
    let mut state = world.singleton_mut::<AbilityUIState>();

    if state.cursor_pos.is_none()
        && let Some((p_actor, fov)) = query!(world, _ Player, Actor, Fov).next()
    {
        let mut positions = Vec::new();
        for (actor, _draw_pos) in query!(world, Actor, DrawPos, !Player) {
            if fov.0.contains(&actor.pos) {
                positions.push(actor.pos);
            }
        }
        positions.sort_by_key(|pos| (p_actor.pos.distance(*pos), pos.x, pos.y));
        let tm = world.singleton::<TileMap>();
        state.cursor_pos = positions
            .into_iter()
            .filter(|pos| {
                let line = p_actor.pos.bresenham(*pos);
                let blocked =
                    line.iter().skip(1).take(line.len() - 2).any(|pos| tm.is_blocked(*pos));
                !blocked
            })
            .next();
    }

    // move cursor via normal character movement inputs
    if let Some(dir) = input_direction(c)
        && let Some(ref mut cursor) = state.cursor_pos
    {
        *cursor += dir;
    }

    // highlight whatever the cursor is on
    let tm = world.singleton::<TileMap>();
    if let Some(cursor_pos) = state.cursor_pos {
        let rect = Rect::new(
            cursor_pos.x as f32 * TILE_SIZE,
            cursor_pos.y as f32 * TILE_SIZE,
            TILE_SIZE,
            TILE_SIZE,
        );
        let color = Color::rgba(1.0, 0.2, 0.3, 0.4);
        c.draw_rect_lines(rect, 3.0, Color::YELLOW, Z_CURSOR);

        if let Some((player, fov, p_actor)) = query!(world, &this, Fov, _ Player, Actor).next()
        {
            let line: Vec<Pos> = p_actor.pos.bresenham(cursor_pos);
            let mut blocked = false;
            let mut blocker = None;
            for pos in line.iter().skip(1) {
                if !blocked {
                    c.draw_rect(pos.to_fpos(TILE_SIZE).rect(TILE_SIZE), color, Z_CURSOR);
                }
                if tm.is_blocked(*pos) {
                    // only save actor as blocker if its the first thing blocking the path
                    if !blocked {
                        blocker = tm.get_actor(*pos);
                    }
                    blocked = true;
                }
            }

            let mut positions = Vec::new();
            for (actor, _draw_pos) in query!(world, Actor, DrawPos, !Player) {
                // TODO check if draw_pos is visible
                let line = p_actor.pos.bresenham(actor.pos);
                let blocked =
                    line.iter().skip(1).take(line.len() - 2).any(|pos| tm.is_blocked(*pos));
                if fov.0.contains(&actor.pos) && !blocked {
                    positions.push(actor.pos);
                }
            }
            positions.sort_by_key(|pos| (p_actor.pos.distance(*pos), pos.x, pos.y));
            if let Some(pos) = avy_navigation(c, &positions) {
                state.cursor_pos = Some(pos);
            }

            if c.is_pressed(Input::Confirm)
                && let Some(target) = blocker
            {
                world.defer_closure(exit_ability_state);
                return Some(Action {
                    actor: *player,
                    kind: ActionKind::UseAbility(Ability::RockThrow { path: line, target }),
                });
            }
        }
    }

    None
}

/// Anything an actor may do
#[derive(Debug)]
pub struct Action {
    pub actor: Entity,
    pub kind: ActionKind,
}

#[derive(Debug)]
pub enum ActionKind {
    Move { from: Pos, to: Pos },
    BumpAttack { target: Entity },
    UseAbility(Ability),
}

#[derive(Debug)]
pub enum Ability {
    RockThrow { path: Vec<Pos>, target: Entity },
}

fn update_systems_inventory(c: &mut dyn ContextTrait, world: &mut World) {
    zone!();
    if c.is_pressed(Input::Cancel) {
        world.singleton_mut::<UI>().state = UIState::Normal;
    }

    ui_inventory(c, world);
}

#[derive(Default, Quicksilver)]
pub struct InspectUIState {
    cursor_pos: Option<Pos>,
}

fn update_systems_inspect(c: &mut dyn ContextTrait, world: &mut World) {
    zone!();
    if c.is_pressed(Input::Cancel) {
        world.singleton_mut::<UI>().state = UIState::Normal;
        world.singleton_remove::<InspectUIState>();
        return;
    }

    // init state with cursor pointing at player character if necessary
    ensure_singleton::<InspectUIState>(world);
    let mut state = world.singleton_mut::<InspectUIState>();
    if state.cursor_pos.is_none()
        && let Some((p_actor,)) = query!(world, _ Player, Actor).next()
    {
        state.cursor_pos = Some(p_actor.pos);
    }

    // move cursor via normal character movement inputs
    if let Some(dir) = input_direction(c)
        && let Some(ref mut cursor) = state.cursor_pos
    {
        *cursor += dir;
    }

    // highlight whatever the cursor is on
    if let Some(cursor_pos) = state.cursor_pos {
        let rect = Rect::new(
            cursor_pos.x as f32 * TILE_SIZE,
            cursor_pos.y as f32 * TILE_SIZE,
            TILE_SIZE,
            TILE_SIZE,
        );
        let color = Color::rgba(1., 1., 0., 0.2);
        c.draw_rect(rect, color, Z_CURSOR);
        c.draw_rect_lines(rect, 3.0, Color::YELLOW, Z_CURSOR);
    }

    // put avy label on entities in fov
    // avy jump to marked entities if the corresponding key is pressed
    if let Some((fov, player_actor)) = query!(world, Fov, _ Player, Actor).next() {
        let player_pos = player_actor.pos;
        let mut positions = Vec::new();

        for (actor, _draw_pos) in query!(world, Actor, DrawPos) {
            // TODO check if draw_pos is visible
            if fov.0.contains(&actor.pos) {
                positions.push(actor.pos);
            }
        }
        positions.sort_by_key(|pos| (player_pos.distance(*pos), pos.x, pos.y));
        if let Some(pos) = avy_navigation(c, &positions) {
            state.cursor_pos = Some(pos);
        }
    }
}

fn avy_navigation(c: &mut dyn ContextTrait, positions: &[Pos]) -> Option<Pos> {
    zone!();
    let mut r = None;
    let pressed = c.avy_is_key_pressed();
    for (choice, pos) in positions.iter().enumerate() {
        let choice = choice as u32;
        let text = c.avy_label(choice);
        let label = text.labelize_prop(
            c,
            FVec::new(50., 50.),
            TextProperty::new().color(Color::YELLOW),
        );
        let mut draw_pos = pos.to_fpos(TILE_SIZE).rect(TILE_SIZE).bl().to_screen(c);
        draw_pos.x += 3.0;
        draw_pos.y -= label.rect.h + 3.0;
        label.draw(c, draw_pos, Z_AVY_LABEL);
        if Some(choice) == pressed {
            r = Some(*pos);
        }
    }
    r
}

fn update_systems_normal(c: &mut dyn ContextTrait, world: &mut World) {
    zone!();
    TileMap::update_actors(world);
    if !player_is_animation_target(world) {
        // sanity check
        let next = next_turn_actor(world);
        assert!(
            world.has_component::<Player>(next),
            "It needs to be the players turn to enter here."
        );

        // handle player input
        TileMap::update_actors(world);
        player_inputs(c, world);
        world.process();

        // handle AI input after player
        let mut next = next_turn_actor(world);
        while !world.has_component::<Player>(next) {
            TileMap::update_actors(world);
            ai_turn(world, next);
            next = next_turn_actor(world);
        }
        world.process();
    }

    for (actor, mut draw_pos, mut draw_health) in
        query!(world, Actor, mut DrawPos, mut DrawHealth, !AnimationTarget(_, this))
    {
        // update draw position
        draw_pos.0 = pos_to_drawpos(actor.pos);
        draw_health.ratio = actor.hp.current as f32 / actor.hp.max as f32;
    }
    world.process();

    // update player fov
    for (tm, actor, mut fov) in query!(world, $ TileMap, _ Player, Actor, mut Fov) {
        shadowcasting::compute_fov(actor.pos, &mut |pos| tm.blocks_vision(pos), &mut |pos| {
            fov.0.insert(pos);
        });
    }
}

pub fn draw_systems(c: &mut dyn ContextTrait, world: &World) {
    zone!();
    let Some((fov,)) = query!(world, Fov, _ Player).next() else { return };

    // draw tile map
    {
        let tm = world.singleton::<TileMap>();
        let env = Environment::Catacomb;

        for (pos, lt) in tm.tiles.iter_coords() {
            if !fov.0.contains(&pos) {
                continue;
            }

            let mut pos_below = pos.clone();
            pos_below.y += 1;
            let below = (&tm).tiles.get_opt(pos_below).unwrap_or(&LogicTile::Empty);
            let draw_tile = generate_draw_tile(*lt, env, *below);
            draw_tile.draw(c, pos.to_fpos(TILE_SIZE), Z_TILES);
        }

        // up and down stairs
        {
            let pos = tm.up_stairs;
            if fov.0.contains(&pos) {
                DrawTile::UpStairs.draw(c, pos.to_fpos(TILE_SIZE), Z_TILES);
            }

            let pos = tm.down_stairs;
            if fov.0.contains(&pos) {
                DrawTile::DownStairs.draw(c, pos.to_fpos(TILE_SIZE), Z_TILES);
            }
        }

        for DecorWithPos(pos, decor) in &tm.decor {
            if !fov.0.contains(&pos) {
                continue;
            }

            decor.draw(c, pos.to_fpos(TILE_SIZE), Z_TILES);
        }
    };

    // draw actors
    for (draw_health, draw_pos, actor) in query!(world, DrawHealth, DrawPos, Actor) {
        if !fov.0.contains(&actor.pos) {
            // TODO: actors shouldn't just disappear when they move outside the FOV
            // so it should be also related to draw_pos :thonk:
            continue;
        }

        let (x, y) = draw_pos.0.into();
        actor.sprite.draw(c, x, y);

        if draw_health.ratio < 1.0 {
            // let hp_percent = actor.hp.current as f32 / actor.hp.max as f32;
            let rect = Rect::new(x + TILE_SIZE * 0.1, y + TILE_SIZE, TILE_SIZE * 0.8, 5.);
            c.draw_rect(rect, Color::RED, Z_HP_BAR);
            let rect = Rect::new(
                x + TILE_SIZE * 0.1,
                y + TILE_SIZE,
                TILE_SIZE * 0.8 * draw_health.ratio.max(0.),
                5.,
            );
            c.draw_rect(rect, Color::GREEN, Z_HP_BAR);
        }
    }
}

pub fn create_world() -> World {
    zone!();
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
        .add(DrawHealth { ratio: 1.0 })
        .add(Fov(HashSet::new()));

    world.singleton_add(tm);

    world.singleton_add(UI::default());
    world.singleton_add(TurnCount { aut: 0 });
    world.singleton_add(MessageLog::default());
    place_enemies(&mut world, 12345);

    world.singleton_add(RandomGenerator::new(12345));

    world.process();
    world
}

#[cfg(test)]
mod test {
    use super::*;

    /// For quickly testing stuff when reloading breaks after adding new state.
    #[test]
    fn test_persist() {
        let mut world = World::new();
        register_components(&mut world);
        world.singleton_add(UI::default());

        let s = save_world(&world);
        println!("------------");
        println!("{s}");
        let _w2 = load_world(&s);
    }
}
