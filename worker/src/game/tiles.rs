use crate::game::Z_TILES;
use base::{ContextTrait, FPos, Pos, Rect};
use quicksilver::Quicksilver;

pub const TILE_DIM: f32 = 32.;
pub const TILE_SCALE: f32 = 2.0;
pub const TILE_SIZE: f32 = TILE_DIM * TILE_SCALE;

/// A graphical tile from the tileset.png asset in 32rogues
#[derive(Debug, Clone, Copy, Quicksilver)]
#[repr(C)]
pub enum DrawTile {
    Empty,
    SkullWallTop,
    SkullWallBot,
    GrayFloor,
}

impl DrawTile {
    pub fn draw(&self, c: &mut dyn ContextTrait, x: f32, y: f32) {
        let (sx, sy) = match self {
            DrawTile::Empty => (0, 2),
            DrawTile::SkullWallTop => (0, 5),
            DrawTile::SkullWallBot => (1, 5),
            DrawTile::GrayFloor => (0, 6),
        };

        let src = Rect::new(sx as f32 * TILE_DIM, sy as f32 * TILE_DIM, TILE_DIM, TILE_DIM);
        let target = Rect::new(x, y, TILE_DIM * TILE_SCALE, TILE_DIM * TILE_SCALE);
        c.draw_texture_part_scaled("tiles", src, target, Z_TILES);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Quicksilver)]
#[repr(C)]
pub enum LogicTile {
    Wall,
    Floor,
    Empty,
}

#[derive(Debug, Clone, Copy, Quicksilver)]
#[repr(C)]
pub enum Environment {
    Catacomb,
}

pub fn generate_draw_tile(lt: LogicTile, env: Environment, below: LogicTile) -> DrawTile {
    match env {
        Environment::Catacomb => match lt {
            LogicTile::Wall => match below {
                LogicTile::Wall => DrawTile::SkullWallTop,
                _ => DrawTile::SkullWallBot,
            },
            LogicTile::Floor => DrawTile::GrayFloor,
            LogicTile::Empty => DrawTile::Empty,
        },
    }
}

pub fn pos_to_drawpos(pos: Pos) -> FPos {
    pos * (TILE_DIM * TILE_SCALE)
}

/// Anything that can lie on top of a tile.
/// Below items
/// Below actors
#[derive(Debug, Clone, Copy, Quicksilver)]
#[repr(C)]
pub enum Decor {
    BloodRed1,
    BloodRed2,
}

impl Decor {
    pub fn draw(&self, c: &mut dyn ContextTrait, x: f32, y: f32) {
        let (sx, sy) = match self {
            Decor::BloodRed1 => (0, 22),
            Decor::BloodRed2 => (1, 22),
        };

        let src = Rect::new(sx as f32 * TILE_DIM, sy as f32 * TILE_DIM, TILE_DIM, TILE_DIM);
        let target = Rect::new(x, y, TILE_DIM * TILE_SCALE, TILE_DIM * TILE_SCALE);
        c.draw_texture_part_scaled("tiles", src, target, Z_TILES);
    }
}
