use base::{grids::Grid, ContextTrait};

use super::tiles::{DrawTile, Environment, LogicTile};
use crate::game::tiles::generate_draw_tile;

pub struct TileMap {
    tiles: Grid<LogicTile>,
}

impl TileMap {
    pub fn new(w: i32, h: i32) -> Self {
        Self { tiles: Grid::new(w, h, LogicTile::Floor) }
    }

    pub fn draw(&self, c: &mut dyn ContextTrait, x_base: f32, y_base: f32) {
        let env = Environment::Catacomb;
        for (pos, lt) in self.tiles.iter_coords() {
            let upper = self.tiles.get_opt(pos).unwrap_or(&LogicTile::Empty);
            let draw_tile = generate_draw_tile(*lt, env, *upper);
        }
    }
}
