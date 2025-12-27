use crate::{
    animation::CameraMoveAnimation,
    ecs_util::ensure_singleton,
    game::{
        AbilityUIState, PlayerAbility, UI, UIState,
        drawing::DrawPos,
        game_logic::{ActionKind, Actor, Fov, Player, handle_action},
        mapgen::{generate_map, place_enemies},
        sprites::{TILE_DIM, TILE_SIZE, pos_to_drawpos},
        tile_map::TileMap,
        z_levels::{Z_AVY_LABEL, Z_CURSOR},
    },
    rand::RandomGenerator,
};
use base::{
    Color, ContextTrait, FVec, Input, Pos, Rect, TextProperty, pos::IVec, text::Labelize, zone,
};
use froql::{entity_store::Entity, query, world::World};
use std::collections::HashSet;

use super::game_logic::Action;

pub fn player_inputs(c: &mut dyn ContextTrait, world: &mut World) {
    zone!();

    if let Some(nr) = ability_key_pressed(c) {
        println!("Ability pressed: {nr}");

        world.singleton_mut::<UI>().state = UIState::Ability;
        ensure_singleton::<AbilityUIState>(world);
        let mut state = world.singleton_mut::<AbilityUIState>();
        let ability = match nr {
            1 => PlayerAbility::ThrowRock,
            2 => PlayerAbility::Kick,
            3 => PlayerAbility::GroundSlam,
            4 => PlayerAbility::JumpAttack,
            5 => PlayerAbility::Meditate,
            _ => PlayerAbility::ThrowRock,
        };
        state.ability_selected = ability;
        return;
    }

    let action = player_direction_input(c, world).or_else(|| ability_input(c, world));
    if let Some(action) = action {
        handle_action(world, action);
        world.process();
        return;
    }

    if c.is_pressed(Input::Inventory) {
        world.singleton_mut::<UI>().state = UIState::Inventory;
        return;
    }

    // go down the stairs
    if c.is_pressed(Input::Confirm)
        && let Some((player, mut p_actor)) = query!(world, &this, _ Player, mut Actor).next()
        && world.singleton::<TileMap>().down_stairs == p_actor.pos
    {
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

        // remove existing camera move animations
        for (anim,) in query!(world, &this, _ CameraMoveAnimation) {
            anim.destroy();
        }

        // move camera to center on player
        let from = c.screen_rect_world().center();
        let goal = {
            let p = pos_to_drawpos(tm.up_stairs);
            Rect::new_center_wh(p, TILE_DIM, TILE_DIM).center()
        };
        let offset = goal - from;
        dbg!(&offset);
        c.camera_move_rel(offset);

        world.defer_closure(move |world| {
            world.singleton_add(tm);
            place_enemies(world, seed);
            for (mut actor,) in query!(world, !Player, mut Actor) {
                actor.next_turn = start_aut;
            }
            world.add_component(player, Fov(HashSet::new()));
        });

        return;
    }

    if c.is_pressed(Input::Inspect) {
        world.singleton_mut::<UI>().state = UIState::Inspect;
        return;
    }
}

pub fn avy_navigation(c: &mut dyn ContextTrait, positions: &[Pos]) -> Option<Pos> {
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

pub fn ability_key_pressed(c: &mut dyn ContextTrait) -> Option<usize> {
    zone!();
    let inputs =
        [Input::Ability1, Input::Ability2, Input::Ability3, Input::Ability4, Input::Ability5];
    for (nr, key) in inputs.into_iter().enumerate() {
        if c.is_pressed(key) {
            return Some(nr + 1);
        }
    }
    None
}

pub fn input_direction(c: &mut dyn ContextTrait) -> Option<IVec> {
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

fn player_direction_input(c: &mut dyn ContextTrait, world: &mut World) -> Option<Action> {
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

    if c.is_pressed(Input::MoveSkip)
        && let Some((player_e,)) = query!(world, _ Player, _ Actor, &this).next()
    {
        return Some(Action { actor: *player_e, kind: ActionKind::Wait });
    }

    None
}

fn exit_ability_state(world: &mut World) {
    world.singleton_mut::<UI>().state = UIState::Normal;
    world.singleton_remove::<AbilityUIState>();
}

fn ability_input(c: &mut dyn ContextTrait, world: &mut World) -> Option<Action> {
    zone!();
    if !matches!(world.singleton::<UI>().state, UIState::Ability) {
        return None;
    }

    if c.is_pressed(Input::Cancel) {
        exit_ability_state(world);
        return None;
    }

    ensure_singleton::<AbilityUIState>(world);
    let ability = world.singleton::<AbilityUIState>().ability_selected;
    let Some(player) = query!(world, &this, _ Player, _ Actor).next().map(|(p,)| p.entity)
    else {
        return None;
    };

    return match ability {
        PlayerAbility::ThrowRock => ability_input_line(c, world, 1, 5, |path, target| {
            ActionKind::RockThrow { path, target }
        }),
        PlayerAbility::JumpAttack => ability_input_line(c, world, 2, 5, |path, target| {
            ActionKind::JumpAttack { path, target }
        }),
        PlayerAbility::Kick => ability_input_melee_single(c, world),
        PlayerAbility::Meditate => {
            exit_ability_state(world);
            Some(Action { actor: player, kind: ActionKind::Meditate })
        }
        PlayerAbility::GroundSlam => {
            exit_ability_state(world);
            Some(Action { actor: player, kind: ActionKind::GroundSlam })
        }
    };
}

fn ability_input_line(
    c: &mut dyn ContextTrait,
    world: &mut World,
    min_range: i32,
    max_range: i32,
    action_fn: impl Fn(Vec<Pos>, Entity) -> ActionKind,
) -> Option<Action> {
    zone!();

    let mut state = world.singleton_mut::<AbilityUIState>();
    let Some((player, fov, p_actor)) = query!(world, &this, Fov, _ Player, Actor).next()
    else {
        return None;
    };

    if state.cursor_pos.is_none() {
        let mut positions = Vec::new();
        for (actor, _draw_pos) in query!(world, Actor, DrawPos, !Player) {
            if fov.0.contains(&actor.pos) {
                positions.push(actor.pos);
            }
        }
        positions.sort_by_key(|pos| (p_actor.pos.distance(*pos), pos.x, pos.y));

        let tm = world.singleton::<TileMap>();
        let new_cursor = positions
            .into_iter()
            .filter(|pos| {
                let dist = p_actor.pos.distance(*pos);
                dist <= max_range && dist >= min_range
            })
            .filter(|pos| {
                let line = p_actor.pos.bresenham(*pos);
                let blocked =
                    line.iter().skip(1).take(line.len() - 2).any(|pos| tm.is_blocked(*pos));
                !blocked
            })
            .next()
            .unwrap_or(p_actor.pos);
        state.cursor_pos = Some(new_cursor);
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

        let line: Vec<Pos> = p_actor.pos.bresenham(cursor_pos);
        let mut blocked = false;
        let mut blocker = None;
        let mut blocker_pos = None;
        for pos in line.iter().skip(1) {
            if !blocked {
                c.draw_rect(pos.to_fpos(TILE_SIZE).rect(TILE_SIZE), color, Z_CURSOR);
            }
            if tm.is_blocked(*pos) {
                // only save actor as blocker if its the first thing blocking the path
                if !blocked {
                    blocker = tm.get_actor(*pos);
                    blocker_pos = Some(*pos);
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
            && let Some(dist) = blocker_pos.map(|p| p.distance(p_actor.pos))
            && dist >= min_range
            && dist <= max_range
        {
            world.defer_closure(exit_ability_state);
            return Some(Action { actor: *player, kind: action_fn(line, target) });
        }
    }

    None
}

fn ability_input_melee_single(c: &mut dyn ContextTrait, world: &mut World) -> Option<Action> {
    zone!();

    let mut state = world.singleton_mut::<AbilityUIState>();

    let Some((player, p_actor, fov)) = query!(world, &this, _ Player, Actor, Fov).next()
    else {
        return None;
    };
    let player = player.entity;

    // initial cursor position
    if state.cursor_pos.is_none() {
        let mut positions = Vec::new();
        for (actor, _draw_pos) in query!(world, Actor, DrawPos, !Player) {
            if fov.0.contains(&actor.pos) {
                positions.push(actor.pos);
            }
        }
        positions.sort_by_key(|pos| (p_actor.pos.distance(*pos), pos.x, pos.y));

        let new_cursor = positions
            .into_iter()
            .filter(|pos| p_actor.pos.distance(*pos) <= 1) // melee range
            .next()
            .unwrap_or(p_actor.pos);
        state.cursor_pos = Some(new_cursor);
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
        let rect = cursor_pos.to_fpos(TILE_SIZE).rect(TILE_SIZE);
        c.draw_rect_lines(rect, 3.0, Color::YELLOW, Z_CURSOR);
        let color = Color::rgba(1.0, 0.2, 0.3, 0.4);
        c.draw_rect(rect, color, Z_CURSOR);

        let mut positions = Vec::new();
        for (actor, _draw_pos) in query!(world, Actor, DrawPos, !Player) {
            if fov.0.contains(&actor.pos) {
                positions.push(actor.pos);
            }
        }
        positions.sort_by_key(|pos| (p_actor.pos.distance(*pos), pos.x, pos.y));
        positions.retain(|pos| pos.distance(p_actor.pos) <= 1);
        if let Some(pos) = avy_navigation(c, &positions) {
            state.cursor_pos = Some(pos);
        }

        if c.is_pressed(Input::Confirm)
            && let Some(pos) = state.cursor_pos
            && let Some(target) = tm.get_actor(pos)
        {
            world.defer_closure(exit_ability_state);
            return Some(Action { actor: player, kind: ActionKind::Kick { target } });
        }
    }

    None
}
