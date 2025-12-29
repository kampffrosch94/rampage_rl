use base::{grids::Grid, zone};
use froql::{entity_store::Entity, query, world::World};

use crate::{
    dijkstra::{dijkstra, dijkstra_path},
    game::{
        game_logic::{Action, ActionKind, Actor, CreatureType, Player},
        tile_map::TileMap,
    },
};

pub struct Pathfinding {
    /// This is a dijkstra map for going towards the player and melee attacking them.
    melee_grid: Grid<i32>,
}

impl Pathfinding {
    pub fn new(world: &World) -> Self {
        zone!();

        let tm = world.singleton::<TileMap>();
        let mut melee_grid = Grid::new(tm.tiles.width, tm.tiles.height, 0);
        let mut seeds = Vec::new();
        for (player,) in query!(world, Actor, _ Player) {
            melee_grid[player.pos] = 500;
            seeds.push(player.pos);
        }
        let cost_function = |pos| {
            if tm.is_wall(pos) {
                i32::MAX
            } else if tm.get_actor(pos).is_some() {
                25
            } else {
                1
            }
        };
        dijkstra(&mut melee_grid, &seeds, cost_function);
        Self { melee_grid }
    }
}

#[must_use]
pub fn ai_turn(world: &World, pf: &Pathfinding, npc: Entity) -> Action {
    zone!();

    // check possible actions and pick the best one
    // going for a bump attack is the default if none is found (below this block)
    {
        // TODO this should use proper action source logic later

        let actor = world.get_component::<Actor>(npc);
        println!("Turn of {}", actor.name);
        if matches!(actor.creature_type, CreatureType::GoblinBrute) {
            // goblin brutes smash the player if they are in range
            for (player_a,) in query!(world, Actor, _ Player) {
                if actor.pos.distance(player_a.pos) == 1 {
                    return ActionKind::DelayedSmash { dir: player_a.pos - actor.pos }
                        .done_by(npc);
                }
            }
        }
    }

    // set up pathfinding dijsktra map
    let grid = &pf.melee_grid;
    let tm = world.singleton::<TileMap>();

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
