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
    pub melee_grid: Grid<i32>,
    pub ranged_grid: Grid<i32>,
}

impl Pathfinding {
    pub fn new(world: &World) -> Self {
        zone!();

        let tm = world.singleton::<TileMap>();
        let cost_function = |pos| {
            if tm.is_wall(pos) {
                i32::MAX
            } else if tm.get_actor(pos).is_some() {
                25
            } else {
                1
            }
        };

        let melee_grid = {
            let mut grid = Grid::new(tm.tiles.width, tm.tiles.height, 0);
            let mut seeds = Vec::new();
            for (player,) in query!(world, Actor, _ Player) {
                grid[player.pos] = 500;
                seeds.push(player.pos);
            }
            dijkstra(&mut grid, &seeds, cost_function);
            grid
        };

        let ranged_grid = {
            let mut grid = Grid::new(tm.tiles.width, tm.tiles.height, 0);
            let mut seeds = Vec::new();
            for (player,) in query!(world, Actor, _ Player) {
                for pos in player.pos.circle_around(5) {
                    let has_los =
                        pos.bresenham(player.pos).into_iter().all(|p| !tm.is_wall(p));
                    if has_los {
                        grid[pos] = 500;
                        seeds.push(pos);
                    }
                }
            }
            dijkstra(&mut grid, &seeds, cost_function);
            grid
        };

        Self { melee_grid, ranged_grid }
    }
}

#[must_use]
pub fn ai_turn(world: &World, pf: &Pathfinding, npc: Entity) -> Action {
    zone!();

    let actor = world.get_component::<Actor>(npc);
    println!("Turn of {}", actor.name);

    // check possible actions and pick the best one
    // going for a bump attack is the default if none is found (below this block)
    {
        // TODO this should use proper action source logic later

        if matches!(actor.creature_type, CreatureType::GoblinBrute) {
            // goblin brutes smash the player if they are in range
            for (player_a,) in query!(world, Actor, _ Player) {
                if actor.pos.distance(player_a.pos) == 1 {
                    return ActionKind::DelayedSmash { dir: player_a.pos - actor.pos }
                        .done_by(npc);
                }
            }
        }

        if matches!(actor.creature_type, CreatureType::GoblinArcher) {
            // goblin archer shoots the player if they are in range
            for (player_e, player_a) in query!(world, &this, Actor, _ Player) {
                let distance = actor.pos.distance(player_a.pos);
                if distance > 1 && distance <= 5 {
                    let tm = world.singleton::<TileMap>();
                    let path = actor.pos.bresenham(player_a.pos);
                    let blocked = path[1..(path.len() - 1)].iter().any(|p| tm.is_blocked(*p));
                    if !blocked {
                        return ActionKind::ShootArrow { path, target: *player_e }
                            .done_by(npc);
                    }
                }
            }
        }
    }

    // set up pathfinding dijsktra map
    let grid = match actor.creature_type {
        CreatureType::GoblinArcher => &pf.ranged_grid,
        _ => &pf.melee_grid,
    };
    let tm = world.singleton::<TileMap>();

    // do pathfinding
    let start = actor.pos;
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
