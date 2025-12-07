use std::cmp::Ordering;
use std::collections::BinaryHeap;

use base::Pos;
use base::grids::Grid;

use crate::game::tile_map::TileMap;
use crate::game::sprites::LogicTile;

#[derive(Copy, Clone, Eq, PartialEq)]
struct PosAndCost {
    cost: i32,
    position: Pos,
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for PosAndCost {
    fn cmp(&self, other: &Self) -> Ordering {
        // Notice that we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other.cost.cmp(&self.cost).then_with(|| self.position.cmp(&other.position))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for PosAndCost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn astar_orth_dig(tm: &TileMap, start: Pos, goal: Pos) -> Option<Vec<Pos>> {
    let mut cost = Grid::new(tm.tiles.width, tm.tiles.height, i32::MAX);
    let mut frontier = BinaryHeap::new();
    let mut came_from = Grid::new(tm.tiles.width, tm.tiles.height, None::<Pos>);

    frontier.push(PosAndCost { cost: 0, position: start });
    cost[start] = 0;

    while !frontier.is_empty() {
        let current = frontier.pop().unwrap().position;

        if current == goal {
            break;
        }

        for next in current.neighbors_orth(&tm.tiles) {
            let step_cost = match tm.tiles[current] {
                LogicTile::Empty => continue,
                LogicTile::Wall => 10,
                LogicTile::Floor => 1,
            };
            let new_cost = cost[current] + step_cost;
            if new_cost < cost[next] {
                cost[next] = new_cost;
                came_from[next] = Some(current);
                let heuristic_cost = new_cost + next.distance_manhattan(goal);
                frontier.push(PosAndCost { cost: heuristic_cost, position: next });
            }
        }
    }

    let mut pos = came_from[goal]?; // return None if no path reaches goal
    let mut r = Vec::with_capacity(start.distance_manhattan(goal) as usize);
    r.push(goal);

    while let Some(next) = came_from[pos] {
        r.push(pos);
        pos = next;
    }

    r.reverse();
    return Some(r);
}
