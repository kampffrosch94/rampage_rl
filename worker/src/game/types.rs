#![allow(unused)]
use base::Circle;
use base::FPos;
use base::FVec;
use base::Pos;
use base::Rect;
use froql::component::CASCADING_DESTRUCT;
use froql::relation::Relation;
use froql::world::World;

use crate::animation::AnimationCleanup;
use crate::animation::AnimationTarget;
use crate::animation::AnimationTimer;
use crate::animation::BumpAttackAnimation;
use crate::animation::CameraMoveAnimation;
use crate::animation::CameraShakeAnimation;
use crate::animation::DecorSpawnAnimation;
use crate::animation::GameOverAnimation;
use crate::animation::HPBarAnimation;
use crate::animation::MovementAnimation;
use crate::animation::ProjectilePathAnimation;
use crate::ecs_util::EntityComponent;
use crate::ecs_util::OriginTarget;
use crate::ecs_util::SerializedState;
use crate::game::AbilityUIState;
use crate::game::DebugOptions;
use crate::game::InspectUIState;
use crate::game::ui::MessageInhibitor;
use crate::game::ui::MessageLog;
use crate::game::ui::MessageOrder;
use crate::game::ui::PendingMessage;
use crate::rand::RandomGenerator;
use froql::entity_store::EntityId;
use froql::query_helper::trivial_query_one_component;
use quicksilver::Quicksilver;
use std::any::TypeId;
use std::any::type_name;
use std::cell::RefCell;
use std::collections::HashSet;

use crate::ecs_util::{ecs_types, generate_load, generate_register, generate_save};

use super::creature::CreatureSprite;
use super::tile_map::TileMap;

/// Marker for player character
#[derive(Debug, Quicksilver)]
pub struct Player {
    pub pulse: f32,
    /// aut of the last pulse raising action taken
    pub last_pulse_action: i64,
}

#[derive(Debug, Quicksilver)]
pub struct HP {
    pub max: i32,
    pub current: i32,
}

impl HP {
    pub fn ratio(&self) -> f32 {
        self.current as f32 / self.max as f32
    }

    pub fn dmg(&mut self, amount: i32) -> HPBarAnimation {
        let start_ratio = self.ratio();
        self.current -= amount;
        let end_ratio = self.ratio();
        HPBarAnimation { start_ratio, end_ratio }
    }
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

#[derive(Debug, Quicksilver)]
pub struct DrawHealth {
    pub ratio: f32,
}

/// How much time passed since the start of the game in seconds
/// Set early in the game loop.
/// Used for animations
#[derive(Default)]
pub struct GameTime(pub f32);

#[derive(Debug, Quicksilver)]
pub struct TurnCount {
    /// 1 tenth of a turn
    pub aut: i64,
}

#[repr(C)]
#[derive(Debug, Quicksilver, Default, Clone, Copy)]
pub enum UIState {
    #[default]
    Normal,
    PostDeath,
    GameOver,
    Inventory,
    Inspect,
    Ability,
}

#[derive(Debug, Quicksilver, Default)]
pub struct UI {
    pub state: UIState,
    pub last_mouse_pos: Option<FPos>,
}

/// Set of Positions within the sight of an actor.
#[derive(Debug, Quicksilver)]
pub struct Fov(pub HashSet<Pos>);

ecs_types!(
    Components(
        Circle,
        Rect,
        GameTime,
        TurnCount[persist],
        Fov[persist],
        Pos[persist],
        Player[persist],
        Actor[persist],
        DrawPos[persist],
        DrawHealth[persist],
        TileMap[persist],
        RandomGenerator[persist],
        // ui
        UI[persist],
        MessageLog[persist],
        PendingMessage[persist],
        DebugOptions[persist],
        InspectUIState[persist],
        AbilityUIState[persist],
        // animations
        AnimationTimer,
        BumpAttackAnimation,
        CameraShakeAnimation,
        DecorSpawnAnimation,
        HPBarAnimation,
        MovementAnimation,
        CameraMoveAnimation,
        GameOverAnimation,
        ProjectilePathAnimation
    ),
    Relations(
        AnimationTarget,
        MessageInhibitor,
        MessageOrder[persist],
        AnimationCleanup(CASCADING_DESTRUCT)
    )
);
