#![allow(unused)]
use base::{FPos, Pos};

pub const GRIDSIZE: f32 = 64.;

/// translates from world coordinates to game grid
pub fn world_to_game(p: FPos) -> Pos {
    let x = (p.x / GRIDSIZE) as _;
    let y = (p.y / GRIDSIZE) as _;
    Pos { x, y }
}

/// rounds pos to align with grid
pub fn grid_world_pos(p: FPos) -> FPos {
    let x = (p.x / GRIDSIZE).floor() * GRIDSIZE;
    let y = (p.y / GRIDSIZE).floor() * GRIDSIZE;
    FPos { x, y }
}

pub fn game_to_world(p: Pos) -> FPos {
    let x = p.x as f32 * GRIDSIZE;
    let y = p.y as f32 * GRIDSIZE;
    FPos { x, y }
}
