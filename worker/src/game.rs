use base::{text::TextFamily, Color, ContextTrait, FPos, Rect, TextProperty};
use tile_map::TileMap;
use tiles::DrawTile;
mod tile_map;
mod tiles;

use crate::{fleeting::FleetingState, persistent::PersistentState};

#[repr(C)]
enum Label {
    ExampleText,
}

pub const Z_TILES: i32 = 0;

pub fn update_inner(
    c: &mut dyn ContextTrait,
    s: &mut PersistentState,
    _f: &mut FleetingState,
) {
    s.re_register();
    let world = &mut s.world;

    c.draw_texture("tiles", -200., -950., 5);

    let r = c.set_text(
        Label::ExampleText as _,
        500.,
        500.,
        &[
            (
                "Placeholder of doom",
                TextProperty::new()
                    .family(TextFamily::BloodCyrillic)
                    .metrics(66, 80)
                    .color(Color::RED),
            ),
            ("\n \n", TextProperty::new().metrics(2, 20)),
            (
                r#"a) Run run run!
b) Walk walk walk!"#,
                TextProperty::new()
                    .family(TextFamily::BloodCyrillic)
                    .color(Color::PINK)
                    .metrics(44, 60),
            ),
        ],
    );
    let pos = FPos::new(400., -530.);
    c.draw_text(Label::ExampleText as _, pos.x, pos.y, 30);

    let mut tm = TileMap::new(12, 12);
    tm.enwall();
    tm.draw(c, 0., 0.);
}
