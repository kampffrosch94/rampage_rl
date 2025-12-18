pub mod debug_util;
pub mod drawing;
pub mod ecs_types;
pub mod game_ai;
pub mod game_logic;
pub mod input_handling;
pub mod mapgen;
pub mod sprites;
pub mod tile_map;
pub mod ui;
pub mod z_levels;

use crate::animation::player_is_animation_target;
use crate::ecs_util::ensure_singleton;
use crate::game::drawing::DrawPos;
use crate::game::ui::PendingMessage;
use crate::game::z_levels::*;
use crate::{animation, game::drawing::DrawHealth, persistent::PersistentState};
use animation::AnimationTarget;
use base::text::Labelize;
use base::zone;
use base::{Color, ContextTrait, FPos, Input, Pos, Rect, shadowcasting};
use base::{FVec, TextProperty};
use debug_util::{DebugOptions, debug_ui};
use drawing::draw_systems;
use ecs_types::*;
use froql::{query, world::World};
use game_ai::ai_turn;
use game_logic::{Actor, Fov, Player, create_world, handle_action, next_turn_actor};
use input_handling::{avy_navigation, input_direction, player_inputs};
use quicksilver::Quicksilver;
use sprites::{TILE_SIZE, pos_to_drawpos};
use tile_map::TileMap;
use ui::{handle_ui, ui_inventory};

/// How much time passed since the start of the game in seconds
/// Set early in the game loop.
/// Used for animations
#[derive(Default)]
pub struct GameTime(pub f32);

#[repr(C)]
#[derive(Debug, Quicksilver, Default, Clone, Copy)]
pub enum UIState {
    #[default]
    Normal,
    PostDeath,
    GameOver,
    Inventory,
    Inspect,
    Ability,
}

#[derive(Debug, Quicksilver, Default)]
pub struct UI {
    pub state: UIState,
    pub last_mouse_pos: Option<FPos>,
}

#[derive(Default, Quicksilver)]
pub struct AbilityUIState {
    cursor_pos: Option<Pos>,
}

#[derive(Default, Quicksilver)]
pub struct InspectUIState {
    cursor_pos: Option<Pos>,
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
        UIState::Normal | UIState::Ability | UIState::Inspect | UIState::PostDeath => {
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

    debug_ui(c, world);
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

fn update_systems(c: &mut dyn ContextTrait, world: &mut World) {
    zone!();
    let state: UIState = world.singleton::<UI>().state;
    match state {
        UIState::Normal | UIState::Ability => update_systems_normal(c, world),
        UIState::Inventory => update_systems_inventory(c, world),
        UIState::Inspect => update_systems_inspect(c, world),
        UIState::PostDeath => update_systems_postdeath(c, world),
        UIState::GameOver => {
            unreachable!("This branch should not be reached. State: {state:?}")
        }
    };
}

fn update_systems_inventory(c: &mut dyn ContextTrait, world: &mut World) {
    zone!();
    if c.is_pressed(Input::Cancel) {
        world.singleton_mut::<UI>().state = UIState::Normal;
    }

    ui_inventory(c, world);
}

fn update_systems_postdeath(c: &mut dyn ContextTrait, world: &mut World) {
    zone!();
    if c.is_pressed(Input::Confirm) {
        world.singleton_mut::<UI>().state = UIState::GameOver;
    }
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
            let action = ai_turn(world, next);
            handle_action(world, action);
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
