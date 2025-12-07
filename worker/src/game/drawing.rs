use crate::game::{
    Actor, Fov, Player,
    sprites::{DrawTile, Environment, LogicTile, TILE_SIZE, generate_draw_tile},
    tile_map::{DecorWithPos, TileMap},
    z_levels::{Z_HP_BAR, Z_TILES},
};
use base::{Color, ContextTrait, FPos, Rect, zone};
use froql::{query, world::World};
use quicksilver::Quicksilver;

#[derive(Debug, Quicksilver)]
pub struct DrawPos(pub FPos);

#[derive(Debug, Quicksilver)]
pub struct DrawHealth {
    pub ratio: f32,
}

pub fn draw_systems(c: &mut dyn ContextTrait, world: &World) {
    zone!();
    let Some((fov,)) = query!(world, Fov, _ Player).next() else { return };

    // draw tile map
    {
        let tm = world.singleton::<TileMap>();
        let env = Environment::Catacomb;

        for (pos, lt) in tm.tiles.iter_coords() {
            if !fov.0.contains(&pos) {
                continue;
            }

            let mut pos_below = pos.clone();
            pos_below.y += 1;
            let below = (&tm).tiles.get_opt(pos_below).unwrap_or(&LogicTile::Empty);
            let draw_tile = generate_draw_tile(*lt, env, *below);
            draw_tile.draw(c, pos.to_fpos(TILE_SIZE), Z_TILES);
        }

        // up and down stairs
        {
            let pos = tm.up_stairs;
            if fov.0.contains(&pos) {
                DrawTile::UpStairs.draw(c, pos.to_fpos(TILE_SIZE), Z_TILES);
            }

            let pos = tm.down_stairs;
            if fov.0.contains(&pos) {
                DrawTile::DownStairs.draw(c, pos.to_fpos(TILE_SIZE), Z_TILES);
            }
        }

        for DecorWithPos(pos, decor) in &tm.decor {
            if !fov.0.contains(&pos) {
                continue;
            }

            decor.draw(c, pos.to_fpos(TILE_SIZE), Z_TILES);
        }
    };

    // draw actors
    for (draw_health, draw_pos, actor) in query!(world, DrawHealth, DrawPos, Actor) {
        if !fov.0.contains(&actor.pos) {
            // TODO: actors shouldn't just disappear when they move outside the FOV
            // so it should be also related to draw_pos :thonk:
            continue;
        }

        let (x, y) = draw_pos.0.into();
        actor.sprite.draw(c, x, y);

        if draw_health.ratio < 1.0 {
            // let hp_percent = actor.hp.current as f32 / actor.hp.max as f32;
            let rect = Rect::new(x + TILE_SIZE * 0.1, y + TILE_SIZE, TILE_SIZE * 0.8, 5.);
            c.draw_rect(rect, Color::RED, Z_HP_BAR);
            let rect = Rect::new(
                x + TILE_SIZE * 0.1,
                y + TILE_SIZE,
                TILE_SIZE * 0.8 * draw_health.ratio.max(0.),
                5.,
            );
            c.draw_rect(rect, Color::GREEN, Z_HP_BAR);
        }
    }
}
