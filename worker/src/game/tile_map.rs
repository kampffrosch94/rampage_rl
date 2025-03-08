use std::collections::HashMap;

use base::{ContextTrait, Pos, grids::Grid};
use froql::{entity_store::Entity, query, world::World};
use nanoserde::{DeJson, SerJson};

use super::tiles::{Decor, Environment, LogicTile, TILE_DIM, TILE_SCALE, TILE_SIZE};
use crate::game::Actor;
use crate::game::tiles::generate_draw_tile;

#[derive(Debug, DeJson, SerJson)]
pub struct TileMap {
    pub tiles: Grid<LogicTile>,
    #[nserde(skip)]
    actors: HashMap<Pos, Entity>,
    decor: Vec<(Pos, Decor)>,
}

impl TileMap {
    pub fn new(w: i32, h: i32, start_tile: LogicTile) -> Self {
        Self { tiles: Grid::new(w, h, start_tile), actors: HashMap::new(), decor: Vec::new() }
    }

    /// put a wall on the outside
    pub fn enwall(&mut self) {
        let y_max = self.tiles.height - 1;
        for x in 0..self.tiles.width {
            self.tiles[(x, 0)] = LogicTile::Wall;
            self.tiles[(x, y_max)] = LogicTile::Wall;
        }

        let x_max = self.tiles.width - 1;
        for y in 0..self.tiles.height {
            self.tiles[(0, y)] = LogicTile::Wall;
            self.tiles[(x_max, y)] = LogicTile::Wall;
        }
    }

    pub fn draw(&self, c: &mut dyn ContextTrait, x_base: f32, y_base: f32) {
        let env = Environment::Catacomb;
        for (pos, lt) in self.tiles.iter_coords() {
            let mut pos_below = pos.clone();
            pos_below.y += 1;
            let below = self.tiles.get_opt(pos_below).unwrap_or(&LogicTile::Empty);
            let draw_tile = generate_draw_tile(*lt, env, *below);
            let x = x_base + pos.x as f32 * TILE_SIZE;
            let y = y_base + pos.y as f32 * TILE_SIZE;
            draw_tile.draw(c, x, y);
        }
        for (pos, decor) in &self.decor {
            let x = x_base + pos.x as f32 * TILE_SIZE;
            let y = y_base + pos.y as f32 * TILE_SIZE;
            decor.draw(c, x, y);
        }
    }

    pub fn add_decor(&mut self, pos: Pos, decor: Decor) {
        self.decor.push((pos, decor));
    }

    pub fn is_blocked(&self, pos: Pos) -> bool {
        self.actors.contains_key(&pos)
            || self.tiles.get_opt(pos).map(|tile| *tile == LogicTile::Wall).unwrap_or(false)
    }

    pub fn update_actors(world: &mut World) {
        let mut tm = world.singleton().get_mut::<TileMap>();
        tm.actors.clear();
        for (e, actor) in query!(world, &this, Actor) {
            tm.actors.insert(actor.pos, e.id);
        }
    }

    pub fn get_actor(&self, pos: Pos) -> Option<Entity> {
        self.actors.get(&pos).copied()
    }
}
