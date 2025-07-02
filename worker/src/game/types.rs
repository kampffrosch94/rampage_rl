#![allow(unused)]
use base::Circle;
use base::FPos;
use base::Pos;
use base::Rect;
use froql::relation::Relation;
use froql::world::World;

use crate::ecs_setup::EntityComponent;
use crate::ecs_setup::OriginTarget;
use crate::ecs_setup::SerializedState;
use crate::rand::RandomGenerator;
use froql::entity_store::EntityId;
use froql::query_helper::trivial_query_one_component;
use quicksilver::Quicksilver;
use std::any::TypeId;
use std::any::type_name;
use std::cell::RefCell;
use std::collections::HashSet;

use crate::ecs_setup::{
    ecs_types, generate_load, generate_re_register, generate_register, generate_save,
};

use super::creature::CreatureSprite;
use super::tile_map::TileMap;

/// Marker for player character
#[derive(Debug, Quicksilver)]
pub struct Player {
    pub pulse: f32,
    pub last_action: i32,
}

#[derive(Debug, Quicksilver)]
pub struct HP {
    pub max: i32,
    pub current: i32,
}

#[derive(Debug, Quicksilver)]
pub struct Actor {
    pub pos: Pos,
    pub sprite: CreatureSprite,
    pub hp: HP,
}

#[derive(Debug, Quicksilver)]
pub struct DrawPos(pub FPos);

/// How much time passed since the last frame in seconds
/// Set early in the game loop.
pub struct DeltaTime(pub f32);

#[derive(Debug, Quicksilver)]
pub struct TurnCount {
    /// 1 tenth of a turn
    pub aut: i64,
}

/// just here to check that the macro below works
pub enum ExampleRel {}

#[derive(Debug, Quicksilver)]
pub struct Fov(pub HashSet<Pos>);

ecs_types!(
    Components(
        Circle,
        Rect,
        DeltaTime,
        TurnCount[persist],
        Fov[persist],
        Pos[persist],
        Player[persist],
        Actor[persist],
        DrawPos[persist],
        TileMap[persist],
        RandomGenerator[persist]
    ),
    Relations(ExampleRel[persist])
);
