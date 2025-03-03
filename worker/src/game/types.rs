use base::Circle;
use base::FPos;
use base::Pos;
use base::Rect;
use froql::world::World;
use nanoserde::{DeJson, SerJson};

use crate::ecs_setup::SerializedState;
use froql::entity_store::EntityId;
use froql::query_helper::trivial_query_one_component;
use std::any::type_name;
use std::any::TypeId;
use std::cell::RefCell;

use crate::ecs_setup::{
    ecs_types, generate_load, generate_re_register, generate_register, generate_save,
};

use super::creature::CreatureSprite;

/// Marker for player character
#[derive(Debug, DeJson, SerJson)]
pub struct Player {}

#[derive(Debug, DeJson, SerJson)]
pub struct Actor {
    pub pos: Pos,
    pub sprite: CreatureSprite,
}

#[derive(Debug, DeJson, SerJson)]
pub struct DrawPos(pub FPos);

ecs_types!(
    Components(Circle, Rect, Pos[persist], Player[persist], Actor[persist], DrawPos[persist]),
    Relations()
);
