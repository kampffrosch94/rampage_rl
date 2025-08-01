#![allow(unused)]
use base::Circle;
use base::FPos;
use base::Pos;
use base::Rect;
use froql::relation::Relation;
use froql::world::World;

use crate::animation::AnimationTarget;
use crate::animation::AnimationTimer;
use crate::animation::BumpAttackAnimation;
use crate::animation::MovementAnimation;
use crate::coroutines::CoroutineStore;
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
    pub name: String,
    pub pos: Pos,
    pub sprite: CreatureSprite,
    pub hp: HP,
    /// when is this actors next turn in aut
    pub next_turn: i64,
}

#[derive(Debug, Quicksilver)]
pub struct DrawPos(pub FPos);
impl DrawPos {
    pub(crate) fn lerp(&self, end: DrawPos, lerpiness: f32) -> FPos {
        todo!()
    }
}

/// How much time passed since the start of the game in seconds
/// Set early in the game loop.
/// Used for animations
pub struct GameTime(pub f32);

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
        GameTime,
        CoroutineStore,
        TurnCount[persist],
        Fov[persist],
        Pos[persist],
        Player[persist],
        Actor[persist],
        DrawPos[persist],
        TileMap[persist],
        RandomGenerator[persist],
        MovementAnimation,
        BumpAttackAnimation,
        AnimationTimer
    ),
    Relations(ExampleRel[persist], AnimationTarget)
);
