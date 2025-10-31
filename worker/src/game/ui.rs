use crate::game::Actor;
use crate::game::Player;
use base::{Color, ContextTrait, Rect, TextProperty, text::TextFamily};
use froql::entity_store::Entity;
use froql::{query, world::World};
use quicksilver::Quicksilver;

use super::{Z_UI_BG, Z_UI_TEXT};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum Label {
    SideMenuHeading,
    PlayerStats,
    InventoryHeading,
    InventoryText,
    MessageLogText,
}

#[derive(Default, Debug, Quicksilver)]
pub struct MessageLog {
    messages: Vec<String>,
}

#[derive(Debug, Quicksilver)]
pub struct PendingMessage {
    msg: String,
}

/// Relation from an inhibitor to a Message
/// MessageInhibitor(Inhibitor, Message)
/// If an an inbitor exists the message must not be displayed yet
pub enum MessageInhibitor {}

/// Relation for ordering messages
/// MessageOrder(Predecessor, Successor)
/// Only display the message once all predecessors are also displayed
pub enum MessageOrder {}

pub fn handle_ui(c: &mut dyn ContextTrait, world: &mut World) {
    side_menu(c, world);
    ui_message_log(c, world)
}

fn side_menu(c: &mut dyn ContextTrait, world: &mut World) {
    let ui_rect = c.screen_rect().take_right(300.);
    c.draw_rect(ui_rect, Color::BLACK, Z_UI_BG);
    c.draw_rect_lines(ui_rect, 5.0, Color::GRAY, Z_UI_BG);
    let mut text_rect = ui_rect.skip_all(15.).skip_top(5.);

    draw_text(
        c,
        Label::SideMenuHeading,
        text_rect.cut_top(60.),
        &[(
            "STATS:",
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
        // TODO this overrides the text in case of multiple players,
        // because there is only a single label ID
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
pub fn draw_text(
    c: &mut dyn ContextTrait,
    label: Label,
    rect: Rect,
    text: &[(&str, TextProperty)],
    z_level: i32,
) {
    c.set_text(label as _, rect.w, rect.h, text);
    c.draw_text(label as _, rect.x, rect.y, z_level);
}

pub fn ui_inventory(c: &mut dyn ContextTrait, _world: &mut World) {
    let inv_rect = c.screen_rect().skip_all(200.).skip_left(200.).skip_right(200.);
    c.draw_rect(inv_rect, Color::BLACK, Z_UI_BG);
    c.draw_rect_lines(inv_rect, 5., Color::WHITE, Z_UI_BG);

    let mut text_rect = inv_rect.skip_all(30.);
    draw_text(
        c,
        Label::InventoryHeading,
        text_rect.cut_top(90.),
        &[(
            "INVENTORY",
            TextProperty::new()
                .family(TextFamily::BloodCyrillic)
                .color(Color::RED)
                .metrics(50, 40),
        )],
        Z_UI_TEXT,
    );
    draw_text(
        c,
        Label::InventoryText,
        text_rect.cut_top(150.),
        &[("Here is where I'd put my Inventory Menu.\nIf I had one >:(", TextProperty::new())],
        Z_UI_TEXT,
    );
}

fn ui_message_log(c: &mut dyn ContextTrait, world: &mut World) {
    let w = c.screen_rect().w;
    let ui_rect = c.screen_rect().take_bot(300.).skip_right(w / 4.).skip_left(w / 4.);
    c.draw_rect(ui_rect, Color::BLACK, Z_UI_BG);
    c.draw_rect_lines(ui_rect, 5.0, Color::GRAY, Z_UI_BG);
    let text_rect = ui_rect.skip_all(15.).skip_top(5.);

    'outer: loop {
        world.process();
        let mut mlog = world.singleton_mut::<MessageLog>();
        for (e, msg) in query!(
            world,
            &this,
            PendingMessage,
            !MessageOrder(_, this),
            !MessageInhibitor(_, this)
        ) {
            mlog.messages.push(msg.msg.clone());
            e.destroy();
            continue 'outer;
        }
        break;
    }

    let mlog = world.singleton::<MessageLog>();
    let text = mlog.messages.join("\n");

    draw_text(c, Label::MessageLogText, text_rect, &[(&text, TextProperty::new())], Z_UI_TEXT);
}

pub fn log_message(world: &World, msg: String, inhibitor: Option<Entity>) {
    let e = world.create_deferred();
    e.add(PendingMessage { msg });

    if let Some(inhibitor) = inhibitor {
        e.relate_from::<MessageInhibitor>(inhibitor);
    }

    if let Some((pred,)) =
        query!(world, &pred, _ PendingMessage(pred), !MessageOrder(pred, _) ).next()
    {
        e.relate_from::<MessageOrder>(*pred);
    }
}
