use base::{text::TextFamily, Circle, Color, ContextTrait, FPos, Rect, TextProperty};
use froql::query;
use froql::world::World;

use crate::{
    fleeting::FleetingState,
    persistent::{Inside, PersistentState},
};

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

    c.draw_texture("ui_bg", 30., 350., 999);
    c.draw_texture_part_scaled(
        "test",
        Rect::new(0., 0., 1., 4.),
        Rect::new(00., 00., 10., 40.),
        10000,
    );
    c.draw_texture("test", 20., 360., 9999);

    for (rect,) in query!(world, Rect) {
        let color = Color::rgb(0.0, 0.5, 0.0);
        c.draw_rect_lines(*rect, 2., color, 2);
    }

    for (circle,) in query!(world, Circle, Inside(this, _)) {
        c.draw_circle(*circle, Color::YELLOW, -0);
    }

    for (circle,) in query!(world, Circle, !Inside(this, _)) {
        c.draw_circle(*circle, Color::BLUE, 2);
    }

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

    let color = Color::rgb(0.0, 0.6, 0.0);
    c.draw_rect_lines(r.move_by_pos(pos).grow_all(20.), 40., color, -5);

    let mouse = c.mouse_world();

    for (e_circle, _) in query!(world, &this, Circle).filter(|(_, c)| c.contains(mouse)) {
        for (circle,) in query!(world, Circle, Inside(this, rect), !Inside(*e_circle, rect)) {
            c.draw_circle(*circle, Color::RED, 2);
        }
    }

    let circle = Circle { pos: mouse, radius: 6. };
    c.draw_circle(circle, Color::WHITE, 10);
}
