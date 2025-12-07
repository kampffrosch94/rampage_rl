use std::collections::HashMap;

use base::{Pos, grids::Grid};
use froql::{entity_store::Entity, query, world::World};
use quicksilver::Quicksilver;
use quicksilver::empty::EmptyContainer;

use super::sprites::{Decor, LogicTile};
use crate::game::game_logic::Actor;

#[derive(Debug, Quicksilver)]
pub struct TileMap {
    pub tiles: Grid<LogicTile>,
    #[quicksilver(skip)]
    pub actors: HashMap<Pos, Entity>,
    pub decor: Vec<DecorWithPos>,
    pub up_stairs: Pos,
    pub down_stairs: Pos,
    pub rooms: Vec<Room>,
}

#[derive(Debug, Quicksilver)]
pub struct DecorWithPos(pub Pos, pub Decor);

impl TileMap {
    pub fn new(w: i32, h: i32, start_tile: LogicTile) -> Self {
        Self {
            tiles: Grid::new(w, h, start_tile),
            actors: HashMap::new(),
            decor: Vec::new(),
            up_stairs: Pos::new(0, 0),
            down_stairs: Pos::new(0, 0),
            rooms: Vec::new(),
        }
    }

    #[allow(unused)]
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

    pub fn add_decor(&mut self, pos: Pos, decor: Decor) {
        self.decor.push(DecorWithPos(pos, decor));
    }

    pub fn is_blocked(&self, pos: Pos) -> bool {
        self.actors.contains_key(&pos) || self.is_wall(pos)
    }

    pub fn is_wall(&self, pos: Pos) -> bool {
        self.tiles.get_opt(pos).map(|tile| *tile == LogicTile::Wall).unwrap_or(false)
    }

    pub fn get_actor(&self, pos: Pos) -> Option<Entity> {
        self.actors.get(&pos).copied()
    }

    pub fn blocks_vision(&self, pos: Pos) -> bool {
        self.is_wall(pos)
    }

    /// Updates the cache of where actors are in the tilemap.
    /// This is used for lookup of actors by position and pathfinding.
    pub fn update_actors(world: &mut World) {
        let mut tm = world.singleton_mut::<TileMap>();
        tm.actors.clear();
        for (e, actor) in query!(world, &this, Actor) {
            tm.actors.insert(actor.pos, *e);
        }
    }
}

#[derive(Debug, Quicksilver, Clone, Copy)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Room {
    pub fn pos(&self) -> Pos {
        let x = self.x + self.w / 2;
        let y = self.y + self.h / 2;
        Pos { x, y }
    }

    pub fn tile_count(&self) -> i32 {
        self.w * self.h
    }

    /// Position by number row wise
    /// Nr must be in range of `0..(self.tile_count())`
    pub fn tile_pos(&self, nr: i32) -> Pos {
        let dx = nr.rem_euclid(self.w);
        let dy = nr.div_euclid(self.w);
        Pos::new(self.x + dx, self.y + dy)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn room_test() {
        let r = Room { x: 0, y: 0, w: 5, h: 3 };
        assert_eq!(15, r.tile_count());
        assert_eq!(Pos::new(0, 0), r.tile_pos(0));
        assert_eq!(Pos::new(3, 0), r.tile_pos(3));
        assert_eq!(Pos::new(4, 2), r.tile_pos(14));
    }
}
