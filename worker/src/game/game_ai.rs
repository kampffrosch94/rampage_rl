use base::{grids::Grid, zone};
use froql::{entity_store::Entity, query, world::World};

use crate::{
    dijkstra::{dijkstra, dijkstra_path},
    game::{
        game_logic::{Action, ActionKind},
        game_logic::{Actor, Player},
        tile_map::TileMap,
    },
};

#[must_use]
pub fn ai_turn(world: &mut World, npc: Entity) -> Action {
    zone!();
    {
        let mut actor = world.get_component_mut::<Actor>(npc);
        if actor.hp.current <= 0 {
            // dead actors wait for further handling
            // might be better to handle it with a death marker component or something
            // that could then exclude actors from the turn order
            actor.next_turn += 100;
            return ActionKind::Wait.done_by(npc);
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
        return ActionKind::Move { from: start, to: *next }.done_by(npc);
    } else if path.len() > 1
        && let Some(next) = path[1..].first()
        && let Some(target) = tm.actors.get(next)
        && world.has_component::<Player>(*target)
    {
        // attack player if the player is the thing blocking movement
        return ActionKind::BumpAttack { target: *target }.done_by(npc);
    } else {
        // just stand in place
        return ActionKind::Wait.done_by(npc);
    }
}
