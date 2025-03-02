use base::{text::TextFamily, Color, ContextTrait, TextProperty};
use creature::CreatureSprite;
use tile_map::TileMap;
mod creature;
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
    let _world = &mut s.world;

    c.draw_texture("tiles", -200., -950., 5);
    c.draw_texture("rogues", -600., -950., 5);
    c.draw_texture("test", -20., 0., 8);

    c.set_text(
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
    c.draw_text(Label::ExampleText as _, 400., -530., 30);

    let mut tm = TileMap::new(12, 12);
    tm.enwall();
    tm.draw(c, 0., 0.);

    CreatureSprite::Dwarf.draw(c, 64. * 4., 64. * 2.);
}
