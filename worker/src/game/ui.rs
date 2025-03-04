use crate::game::Actor;
use crate::game::Player;
use base::{Color, ContextTrait, Rect, TextProperty, text::TextFamily};
use froql::{query, world::World};

use crate::FleetingState;

use super::{Z_UI_BG, Z_UI_TEXT};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
enum Label {
    Heading,
    PlayerStats,
}

pub fn handle_ui(c: &mut dyn ContextTrait, world: &mut World, _f: &mut FleetingState) {
    side_menu(c, world);
}

pub fn side_menu(c: &mut dyn ContextTrait, world: &mut World) {
    let ui_rect = c.screen_rect().take_right(300.);
    c.draw_rect(ui_rect, Color::BLACK, Z_UI_BG);
    c.draw_rect_lines(ui_rect, 5.0, Color::GRAY, Z_UI_BG);
    let mut text_rect = ui_rect.skip_all(15.).skip_top(5.);

    draw_text(
        c,
        Label::Heading,
        text_rect.cut_top(60.),
        &[(
            "Stats:",
            TextProperty::new()
                .family(TextFamily::BloodCyrillic)
                .color(Color::RED)
                .metrics(50, 40),
        )],
        Z_UI_TEXT,
    );
    text_rect.cut_left(10.);

    for (player, actor) in query!(world, Player, Actor) {
        let current = actor.hp.current;
        let max = actor.hp.max;
        let pulse = player.pulse;
        let text = format!(
            "HP: {current}/{max}
Pulse: {pulse}"
        );
        draw_text(
            c,
            Label::PlayerStats,
            text_rect.cut_top(100.),
            &[(&text, TextProperty::new())],
            Z_UI_TEXT,
        );
    }
}

// TODO think about making this the context API instead of separating creation and display
fn draw_text(
    c: &mut dyn ContextTrait,
    label: Label,
    rect: Rect,
    text: &[(&str, TextProperty)],
    z_level: i32,
) {
    c.set_text(label as _, rect.w, rect.h, text);
    c.draw_text(label as _, rect.x, rect.y, z_level);
}
