use base::{ContextTrait, Rect};
use nanoserde::{DeJson, SerJson};

use super::tiles::{TILE_DIM, TILE_SCALE};

#[derive(Debug, Clone, Copy, SerJson, DeJson)]
pub enum CreatureSprite {
    Dwarf,
}

impl CreatureSprite {
    pub fn draw(&self, c: &mut dyn ContextTrait, x: f32, y: f32) {
        let (sx, sy) = match self {
            CreatureSprite::Dwarf => (0, 0),
        };

        let src = Rect::new(sx as f32 * TILE_DIM, sy as f32 * TILE_DIM, TILE_DIM, TILE_DIM);
        let target = Rect::new(x, y, TILE_DIM * TILE_SCALE, TILE_DIM * TILE_SCALE);
        c.draw_texture_part_scaled("tiles", src, target, Z_TILES);
    }
}
