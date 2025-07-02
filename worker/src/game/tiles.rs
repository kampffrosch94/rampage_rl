use crate::game::Z_TILES;
use base::{ContextTrait, FPos, Pos, Rect};
use quicksilver::Quicksilver;

pub const TILE_EXTRUSION: f32 = 1.;
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
    DownStairs,
    UpStairs,
}

impl DrawTile {
    pub fn draw(&self, c: &mut dyn ContextTrait, x: f32, y: f32) {
        let src = match self {
            DrawTile::Empty => (0, 2),
            DrawTile::SkullWallTop => (0, 5),
            DrawTile::SkullWallBot => (1, 5),
            DrawTile::GrayFloor => (0, 6),
            DrawTile::DownStairs => (7, 16),
            DrawTile::UpStairs => (8, 16),
        };

        let src = extruded_source(src);
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
        let src = match self {
            Decor::BloodRed1 => (0, 22),
            Decor::BloodRed2 => (1, 22),
        };

        let src = extruded_source(src);
        let target = Rect::new(x, y, TILE_DIM * TILE_SCALE, TILE_DIM * TILE_SCALE);
        c.draw_texture_part_scaled("tiles", src, target, Z_TILES);
    }
}

/// our tileset is extruded
/// this computes the source rect for a sprite in the tileset
fn extruded_source((sx, sy): (i32, i32)) -> Rect {
    let offset = TILE_DIM + 2. * TILE_EXTRUSION;
    Rect::new(
        sx as f32 * offset + TILE_EXTRUSION,
        sy as f32 * offset + TILE_EXTRUSION,
        TILE_DIM,
        TILE_DIM,
    )
}
