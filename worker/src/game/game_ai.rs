use base::{grids::Grid, zone};
use froql::{entity_store::Entity, query, world::World};

use crate::{
    dijkstra::{dijkstra, dijkstra_path},
    game::{
        Action, ActionKind,
        ecs_types::{Actor, Player},
        handle_action,
        tile_map::TileMap,
    },
};

// TODO return action instead of handling it directly
pub fn ai_turn(world: &mut World, npc: Entity) {
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
