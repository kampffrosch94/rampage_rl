use base::{text::TextFamily, Color, ContextTrait, FPos, Rect, TextProperty};
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

    for x in (0..10).map(|it| it as f32 * 64.) {
        let tile = DrawTile::SkullWallTop;
        tile.draw(c, x, 500.);

        let tile = DrawTile::SkullWallBot;
        tile.draw(c, x, 564.);
    }

    let pos = FPos::new(30., 30.);
    c.draw_text(Label::ExampleText as _, pos.x, pos.y, 30);
}
