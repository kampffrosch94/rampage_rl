use base::{Circle, Color, ContextTrait, Rect};
use froql::query;
use froql::world::World;

use crate::{
    fleeting::FleetingState,
    persistent::{Inside, PersistentState},
};

const COLOR: Color = Color::rgb(0.0, 0.5, 0.5);
const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
const BLUE: Color = Color::rgb(0.3, 0.3, 1.0);
const RED: Color = Color::rgb(1.0, 0.0, 0.0);
const VIOLET: Color = Color::rgb(0.5, 0.0, 0.5);
const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);

pub fn update_inner(
    c: &mut dyn ContextTrait,
    s: &mut PersistentState,
    _f: &mut FleetingState,
) {
    s.re_register();
    let world = &mut s.world;

    for (rect,) in query!(world, Rect) {
        let color = Color::rgb(0.0, 0.5, 0.0);
        c.draw_rect_lines(*rect, 2., color, 0);
    }

    for (circle,) in query!(world, Circle, Inside(this, _)) {
        c.draw_circle(*circle, YELLOW, 2);
    }

    for (circle,) in query!(world, Circle, !Inside(this, _)) {
        c.draw_circle(*circle, BLUE, 2);
    }

    c.draw_text("Hello", 40., 200., 200., 5);

    let mouse = c.mouse_world();

    for (e_circle, _) in query!(world, &this, Circle).filter(|(_, c)| c.contains(mouse)) {
        for (circle,) in query!(world, Circle, Inside(this, rect), !Inside(*e_circle, rect)) {
            c.draw_circle(*circle, RED, 2);
        }
    }

    let circle = Circle { pos: mouse, radius: 6. };
    c.draw_circle(circle, WHITE, 10);
}
