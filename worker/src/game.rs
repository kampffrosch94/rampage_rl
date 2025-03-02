#![allow(unused)]
use base::{text::TextFamily, Circle, Color, ContextTrait, FPos, Rect, TextProperty};
use froql::query;
use froql::world::World;

use crate::{fleeting::FleetingState, persistent::PersistentState};

#[repr(C)]
enum Label {
    ExampleText,
}

pub fn update_inner(
    c: &mut dyn ContextTrait,
    s: &mut PersistentState,
    _f: &mut FleetingState,
) {
    s.re_register();
    let world = &mut s.world;

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

    let pos = FPos::new(30., 30.);
    c.draw_text(Label::ExampleText as _, pos.x, pos.y, 30);
}
