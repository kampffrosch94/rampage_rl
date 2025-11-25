use crate::game::Actor;
use crate::game::Player;
use crate::game::types::UI;
use crate::game::types::UIState;
use base::text::Labelize;
use base::{Color, ContextTrait, Rect, TextProperty, text::TextFamily};
use froql::entity_store::Entity;
use froql::{query, world::World};
use quicksilver::Quicksilver;

use super::InspectUIState;
use super::Z_INVENTORY_BG;
use super::Z_INVENTORY_TEXT;
use super::Z_MESSAGE_BG;
use super::Z_MESSAGE_TEXT;
use super::ensure_singleton;
use super::{Z_SIDEBAR_BG, Z_SIDEBAR_TEXT};

#[derive(Default, Debug, Quicksilver)]
pub struct MessageLog {
    messages: Vec<String>,
}

#[derive(Debug, Quicksilver)]
pub struct PendingMessage {
    pub msg: String,
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

const TEXT_HEADING: TextProperty =
    TextProperty::new().family(TextFamily::BloodCyrillic).color(Color::RED).metrics(50., 40.);

fn side_menu(c: &mut dyn ContextTrait, world: &mut World) {
    let side_panel_rect = c.screen_rect().take_right(400.);
    c.draw_rect(side_panel_rect, Color::BLACK, Z_SIDEBAR_BG);
    c.draw_rect_lines(side_panel_rect, 5.0, Color::GRAY, Z_SIDEBAR_BG);
    let mut ui_rect = side_panel_rect.skip_all(15.).skip_top(5.);

    let r = ui_rect.cut_top(60.);
    "STATS:".labelize_prop(c, r.dim(), TEXT_HEADING).draw(c, r.origin(), Z_SIDEBAR_TEXT);

    for (player, actor) in query!(world, Player, Actor) {
        let current = actor.hp.current;
        let max = actor.hp.max;
        let pulse = player.pulse;
        let text = format!("HP: {current}/{max}\nPulse: {pulse}");
        let r = ui_rect.cut_top(100.).skip_left(10.);
        text.labelize(c, r.dim()).draw(c, r.origin(), Z_SIDEBAR_TEXT);
    }

    let state: UIState = world.singleton::<UI>().state;
    match state {
        UIState::Normal | UIState::Ability | UIState::Inventory => {}
        UIState::Inspect => side_menu_inspect(c, world, &mut ui_rect),
        UIState::GameOver => {
            unreachable!("This branch should not be reached. State: {state:?}")
        }
    };
}

fn side_menu_inspect(c: &mut dyn ContextTrait, world: &mut World, ui_rect: &mut Rect) {
    ensure_singleton::<InspectUIState>(world);
    let state = world.singleton::<InspectUIState>();
    if let Some(cursor) = state.cursor_pos {
        ui_rect.cut_top(50.); // get some distance
        let r = ui_rect.cut_top(60.);
        "INSPECTOR:".labelize_prop(c, r.dim(), TEXT_HEADING).draw(
            c,
            r.origin(),
            Z_SIDEBAR_TEXT,
        );
        // handle actors at cursor position
        let mut something_notable = false;
        for (_actor, player) in query!(world, Actor, Player?).filter(|(a, _)| a.pos == cursor)
        {
            something_notable = true;
            if player.is_some() {
                let label = "This is you.".labelize(c, ui_rect.dim());
                let r = ui_rect.cut_top(label.rect.h).skip_left(10.);
                label.draw(c, r.origin(), Z_SIDEBAR_TEXT);
            } else {
                let label = "This is NOT you.".labelize(c, ui_rect.dim());
                let r = ui_rect.cut_top(label.rect.h).skip_left(10.);
                label.draw(c, r.origin(), Z_SIDEBAR_TEXT);
            }
        }
        if !something_notable {
            let label = "There is nothing notable here.".labelize(c, ui_rect.dim());
            let r = ui_rect.cut_top(label.rect.h).skip_left(10.);
            label.draw(c, r.origin(), Z_SIDEBAR_TEXT);
        }
    }
}

pub fn ui_inventory(c: &mut dyn ContextTrait, _world: &mut World) {
    let inv_rect = c.screen_rect().skip_all(200.).skip_left(200.).skip_right(200.);
    c.draw_rect(inv_rect, Color::BLACK, Z_INVENTORY_BG);
    c.draw_rect_lines(inv_rect, 5., Color::WHITE, Z_INVENTORY_BG);

    let mut text_rect = inv_rect.skip_all(30.);

    let pos = text_rect.cut_top(90.).origin();
    "INVENTORY".labelize_prop(c, text_rect.dim(), TEXT_HEADING).draw(c, pos, Z_INVENTORY_TEXT);
    let pos = text_rect.cut_top(150.).origin();
    "Here is where I'd put my Inventory Menu.\nIf I had one >:("
        .labelize(c, text_rect.dim())
        .draw(c, pos, Z_SIDEBAR_TEXT);
}

fn ui_message_log(c: &mut dyn ContextTrait, world: &mut World) {
    let w = c.screen_rect().w;
    let ui_rect = c.screen_rect().take_bot(300.).skip_right(w / 4.).skip_left(w / 4.);
    c.draw_rect(ui_rect, Color::BLACK, Z_MESSAGE_BG);
    c.draw_rect_lines(ui_rect, 5.0, Color::GRAY, Z_MESSAGE_BG);
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
    let start = (mlog.messages.len().saturating_sub(6)).max(0);
    let text = mlog.messages[start..].join("\n");

    text.labelize(c, text_rect.dim()).draw(c, text_rect.origin(), Z_MESSAGE_TEXT);
}

pub fn log_message(world: &World, msg: String, inhibitor: Entity) {
    let e = world.create_deferred();
    e.add(PendingMessage { msg });
    e.relate_from::<MessageInhibitor>(inhibitor);

    if let Some((pred,)) =
        query!(world, &pred, _ PendingMessage(pred), !MessageOrder(pred, _) ).next()
    {
        e.relate_from::<MessageOrder>(*pred);
    }
}
