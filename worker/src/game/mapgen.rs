use crate::game::DrawHealth;
use crate::rand::RandomGenerator;
use base::Color;
use base::FPos;
use base::Pos;
use base::Rect;
use base::grids::Grid;
use froql::component::SYMMETRIC;
use froql::component::TRANSITIVE;
use froql::query;
use froql::world::World;

mod astar_dig;

use super::creature::CreatureSprite;
use super::tile_map::Room;
use super::tile_map::TileMap;
use super::tiles::LogicTile;
use super::tiles::TILE_SIZE;
use super::types::Actor;
use super::types::DrawPos;
use super::types::HP;
enum Connected {}

pub fn generate_map(seed: u64) -> TileMap {
    let rand = &mut RandomGenerator::new(seed);

    let world = &mut World::new();
    world.register_component::<Room>();
    world.register_component::<Area>();
    world.register_component::<base::Color>();
    world.register_component::<ZLevel>();
    world.register_relation::<Inside>();
    world.register_relation_flags::<Connected>(SYMMETRIC | TRANSITIVE);

    let width = rand.next_in_range(15, 25) as i32;
    let height = rand.next_in_range(15, 25) as i32;
    world
        .create()
        .add(Area { x: 0, y: 0, w: width, h: height })
        .add(Color::WHITE)
        .add(ZLevel(0));

    // BSP
    let mut go_on = true;
    while go_on {
        go_on = false;
        for (container, area, z_level) in query!(world, &this, Area, ZLevel, !Inside(_, this))
        {
            if area.splitable() {
                go_on = true;
                let (a, b) = area.split(rand.next_in_range(4, 6) as i32);
                let ca = rand.random_color().alpha(0.2);
                let cb = rand.random_color().alpha(0.2);
                world
                    .create_deferred()
                    .add(a)
                    .add(ZLevel(z_level.0 + 1))
                    .add(ca)
                    .relate_to::<Inside>(*container);
                world
                    .create_deferred()
                    .add(b)
                    .add(ZLevel(z_level.0 + 1))
                    .add(cb)
                    .relate_to::<Inside>(*container);
            }
        }
        world.process();
    }

    // for (area, color, z_level) in query!(world, Area, Color, ZLevel) {
    //     c.draw_rect(area.as_rect().move_by(0., 1000.), *color, z_level.0);
    // }

    let mut tm = TileMap::new(width, height, LogicTile::Wall);

    for (e, area) in query!(world, &this, Area, !Inside(_, this)) {
        let room = area.carve(&mut tm.tiles, rand);
        e.add(room);
        tm.rooms.push(room);
    }

    // connect rooms via astar dig
    'outer: loop {
        world.process();
        for (a, room_a) in query!(world, &a, Room(a)) {
            for (b, room_b) in query!(world, &b, Room(b), !Connected(b, *a), *a != b) {
                a.relate_to::<Connected>(*b);
                let Some(path) = astar_dig::astar_orth_dig(&tm, room_a.pos(), room_b.pos())
                else {
                    panic!("failed digging")
                };
                for pos in path {
                    tm.tiles[pos] = LogicTile::Floor;
                }
                continue 'outer;
            }
        }
        world.process();
        break;
    }

    // place up and down stairs in rooms that are far apart
    for (room_a,) in query!(world, Room) {
        let room_b = query!(world, Room)
            .map(|(r,)| r)
            .max_by_key(|r| r.pos().distance_manhattan(room_a.pos()))
            .unwrap();
        tm.up_stairs = room_a.pos();
        tm.down_stairs = room_b.pos();
        break;
    }

    return tm;
}

pub fn place_enemies(world: &mut World, seed: u64) {
    let rand = &mut RandomGenerator::new(seed);
    let tm = world.singleton::<TileMap>();
    for room in &tm.rooms {
        for t in 0..room.tile_count() {
            if rand.next_in_range(0, 1000) > 20 {
                continue;
            }
            let _goblin = world
                .create_deferred()
                .add(DrawPos(FPos::new(0., 0.)))
                .add(Actor {
                    name: "Goblin".into(),
                    pos: room.tile_pos(t),
                    sprite: CreatureSprite::Goblin,
                    hp: HP { max: 5, current: 5 },
                    next_turn: 0,
                })
                .add(DrawHealth { ratio: 1.0 });
        }
    }
    drop(tm);
    world.process();
}

enum Inside {}

struct ZLevel(i32);

#[derive(Debug)]
struct Area {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Area {
    #[allow(unused)]
    fn as_rect(&self) -> Rect {
        Rect {
            x: self.x as f32 * TILE_SIZE,
            y: self.y as f32 * TILE_SIZE,
            w: self.w as f32 * TILE_SIZE,
            h: self.h as f32 * TILE_SIZE,
        }
    }

    #[allow(unused)]
    fn pos(&self) -> Pos {
        Pos { x: self.x, y: self.y }
    }

    #[allow(unused)]
    fn area(&self) -> i32 {
        self.w * self.h
    }

    fn splitable(&self) -> bool {
        self.w >= 14 || self.h >= 14
    }

    fn split(&self, ratio: i32) -> (Area, Area) {
        if self.w > self.h {
            let w_a = (self.w * ratio) / 10;
            let w_b = self.w - w_a;
            let a = Area { x: self.x, y: self.y, w: w_a, h: self.h };
            // TODO check if off by one
            let b = Area { x: self.x + w_a, y: self.y, w: w_b, h: self.h };
            (a, b)
        } else {
            let h_a = (self.h * ratio) / 10;
            let h_b = self.h - h_a;
            let a = Area { x: self.x, y: self.y, w: self.w, h: h_a };
            let b = Area { x: self.x, y: self.y + h_a, w: self.w, h: h_b };
            (a, b)
        }
    }

    #[allow(unused)]
    fn shrink(&self) -> Area {
        Area { x: self.x + 1, y: self.y + 1, w: self.w - 2, h: self.h - 2 }
    }

    fn carve(&self, grid: &mut Grid<LogicTile>, rand: &mut RandomGenerator) -> Room {
        let min_w = 3.max(self.w - 5);
        let min_h = 3.max(self.h - 5);
        let w = rand.next_in_range(min_w as u64, (self.w - 2) as _) as i32;
        let h = rand.next_in_range(min_h as u64, (self.h - 2) as _) as i32;
        if min_h > self.h {
            dbg!(self);
        }
        let max_x = self.x + self.w - w;
        let max_y = self.y + self.h - h;
        let x = rand.next_in_range(self.x as u64 + 1, max_x as _) as i32;
        let y = rand.next_in_range(self.y as u64 + 1, max_y as _) as i32;
        let from = Pos::new(x, y);
        let to = Pos::new(x + w, y + h);
        grid.fill_rect(from, to, LogicTile::Floor);
        Room { x, y, w, h }
    }
}
