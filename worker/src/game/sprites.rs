use base::{ContextTrait, FPos, Pos, Rect, zone};
use quicksilver::Quicksilver;

use super::z_levels::Z_SPRITE;

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
    Rock,
    Arrow,
}

impl DrawTile {
    pub fn draw(&self, c: &mut dyn ContextTrait, FPos { x, y }: FPos, z: i32) {
        zone!();
        let (asset, sx, sy) = match self {
            DrawTile::Empty => ("tiles", 0, 2),
            DrawTile::SkullWallTop => ("tiles", 0, 5),
            DrawTile::SkullWallBot => ("tiles", 1, 5),
            DrawTile::GrayFloor => ("tiles", 0, 6),
            DrawTile::DownStairs => ("tiles", 7, 16),
            DrawTile::UpStairs => ("tiles", 8, 16),
            DrawTile::Rock => ("tiles", 1, 18),
            DrawTile::Arrow => ("items", 0, 23),
        };
        let src = if asset == "tiles" {
            extruded_source((sx, sy))
        } else {
            Rect::new(sx as f32 * TILE_DIM, sy as f32 * TILE_DIM, TILE_DIM, TILE_DIM)
        };
        let target = Rect::new(x, y, TILE_DIM * TILE_SCALE, TILE_DIM * TILE_SCALE);
        c.draw_texture_part_scaled(asset, src, target, z);
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
    zone!();
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
    pos.to_fpos(TILE_SIZE)
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
    pub fn draw(&self, c: &mut dyn ContextTrait, FPos { x, y }: FPos, z: i32) {
        zone!();
        let src = match self {
            Decor::BloodRed1 => (0, 22),
            Decor::BloodRed2 => (1, 22),
        };

        let src = extruded_source(src);
        let target = Rect::new(x, y, TILE_DIM * TILE_SCALE, TILE_DIM * TILE_SCALE);
        c.draw_texture_part_scaled("tiles", src, target, z);
    }
}

/// our tileset is extruded
/// this computes the source rect for a sprite in the tileset
fn extruded_source((sx, sy): (i32, i32)) -> Rect {
    zone!();
    let offset = TILE_DIM + 2. * TILE_EXTRUSION;
    Rect::new(
        sx as f32 * offset + TILE_EXTRUSION,
        sy as f32 * offset + TILE_EXTRUSION,
        TILE_DIM,
        TILE_DIM,
    )
}

#[derive(Debug, Clone, Copy, Quicksilver)]
#[repr(C)]
pub enum CreatureSprite {
    Dwarf,
    Goblin,
    GoblinBrute,
    GoblinArcher,
}

impl CreatureSprite {
    pub fn draw(&self, c: &mut dyn ContextTrait, x: f32, y: f32) {
        zone!();
        let (sheet, sx, sy) = match self {
            CreatureSprite::Dwarf => ("rogues", 0, 0),
            CreatureSprite::Goblin => ("monsters", 2, 0),
            CreatureSprite::GoblinBrute => ("monsters", 7, 0),
            CreatureSprite::GoblinArcher => ("monsters", 5, 0),
        };

        let src = Rect::new(sx as f32 * TILE_DIM, sy as f32 * TILE_DIM, TILE_DIM, TILE_DIM);
        let target = Rect::new(x, y, TILE_DIM * TILE_SCALE, TILE_DIM * TILE_SCALE);
        c.draw_texture_part_scaled(sheet, src, target, Z_SPRITE);
    }
}
