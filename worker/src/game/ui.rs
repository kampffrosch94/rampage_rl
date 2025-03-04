use base::{Color, ContextTrait, TextProperty, text::TextFamily};
use froql::world::World;

use crate::FleetingState;

use super::{Z_UI_BG, Z_UI_TEXT};

#[repr(C)]
enum Label {
    Heading,
    PlayerStats,
}

pub fn handle_ui(c: &mut dyn ContextTrait, world: &mut World, _f: &mut FleetingState) {
    side_menu(c, world);
}

pub fn side_menu(c: &mut dyn ContextTrait, world: &mut World) {
    let ui_rect = c.screen_rect().take_right(300.);
    let text_rect = ui_rect.skip_all(10.);
    c.draw_rect(ui_rect, Color::BLACK, Z_UI_BG);
    c.draw_rect_lines(ui_rect, 5.0, Color::GRAY, Z_UI_BG);

    c.set_text(
        Label::Heading as _,
        text_rect.w,
        text_rect.h,
        &[("Stats:", TextProperty::new().family(TextFamily::BloodCyrillic).color(Color::RED))],
    );
    c.draw_text(Label::Heading as _, text_rect.x, text_rect.y, Z_UI_TEXT);
}
