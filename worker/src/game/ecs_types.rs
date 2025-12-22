use super::tile_map::TileMap;
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
use crate::ecs_util::{ecs_types, generate_load, generate_register, generate_save};
use crate::game::AbilityUIState;
use crate::game::GameTime;
use crate::game::InspectUIState;
use crate::game::UI;
use crate::game::debug_util::DebugOptions;
use crate::game::drawing::DrawHealth;
use crate::game::drawing::DrawPos;
use crate::game::game_logic::Actor;
use crate::game::game_logic::DelayedAction;
use crate::game::game_logic::Fov;
use crate::game::game_logic::Player;
use crate::game::game_logic::TurnCount;
use crate::game::ui::MessageInhibitor;
use crate::game::ui::MessageLog;
use crate::game::ui::MessageOrder;
use crate::game::ui::PendingMessage;
use crate::rand::RandomGenerator;
use base::Circle;
use base::Pos;
use base::Rect;
use froql::component::CASCADING_DESTRUCT;
use froql::entity_store::EntityId;
use froql::query_helper::trivial_query_one_component;
use froql::relation::Relation;
use froql::world::World;
use std::any::TypeId;
use std::any::type_name;
use std::cell::RefCell;

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
        DelayedAction[persist],
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
