#![allow(unused)]
use base::Circle;
use base::FPos;
use base::Pos;
use base::Rect;
use froql::world::World;
use nanoserde::{DeJson, SerJson};

use crate::ecs_setup::SerializedState;
use crate::rand::RandomGenerator;
use froql::entity_store::EntityId;
use froql::query_helper::trivial_query_one_component;
use std::any::TypeId;
use std::any::type_name;
use std::cell::RefCell;

use crate::ecs_setup::{
    ecs_types, generate_load, generate_re_register, generate_register, generate_save,
};

use super::creature::CreatureSprite;
use super::tile_map::TileMap;

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

/// How much time passed since the last frame in seconds
/// Set early in the game loop.
pub struct DeltaTime(pub f32);

ecs_types!(
    Components(
        Circle,
        Rect,
        DeltaTime,
        Pos[persist],
        Player[persist],
        Actor[persist],
        DrawPos[persist],
        TileMap[persist],
        RandomGenerator[persist]
    ),
    Relations()
);
