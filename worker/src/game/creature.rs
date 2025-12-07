use base::{ContextTrait, Rect};
use quicksilver::Quicksilver;

use super::{
    Z_SPRITE,
    sprites::{TILE_DIM, TILE_SCALE},
};

#[derive(Debug, Clone, Copy, Quicksilver)]
#[repr(C)]
pub enum CreatureSprite {
    Dwarf,
    Goblin,
}

impl CreatureSprite {
    pub fn draw(&self, c: &mut dyn ContextTrait, x: f32, y: f32) {
        let (sheet, sx, sy) = match self {
            CreatureSprite::Dwarf => ("rogues", 0, 0),
            CreatureSprite::Goblin => ("monsters", 2, 0),
        };

        let src = Rect::new(sx as f32 * TILE_DIM, sy as f32 * TILE_DIM, TILE_DIM, TILE_DIM);
        let target = Rect::new(x, y, TILE_DIM * TILE_SCALE, TILE_DIM * TILE_SCALE);
        c.draw_texture_part_scaled(sheet, src, target, Z_SPRITE);
    }
}
