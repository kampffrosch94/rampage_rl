use crate::{game::tile_map::TileMap, quicksilver_glue::EntityWrapper};
use std::collections::HashSet;

use base::{FPos, Pos, pos::IVec, zone};
use froql::{
    entity_store::Entity, entity_view_deferred::EntityViewDeferred, query, world::World,
};
use quicksilver::Quicksilver;

use crate::{
    animation::{self, AnimationCleanup, HPBarAnimation},
    game::{
        drawing::{DrawHealth, DrawPos},
        mapgen::{generate_map, place_enemies},
        sprites::{CreatureSprite, DrawTile},
        ui::{MessageLog, log_message},
        {UI, register_components},
    },
    rand::RandomGenerator,
};

#[derive(Debug, Quicksilver)]
pub struct Actor {
    pub name: String,
    pub pos: Pos,
    pub sprite: CreatureSprite,
    pub creature_type: CreatureType,
    pub hp: HP,
    /// when is this actors next turn in aut
    pub next_turn: i64,
}

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
pub struct TurnCount {
    /// 1 tenth of a turn
    pub aut: i64,
}

/// Set of Positions within the sight of an actor.
#[derive(Debug, Quicksilver)]
pub struct Fov(pub HashSet<Pos>);

/// Anything an actor may do
#[derive(Debug, Quicksilver)]
pub struct Action {
    #[quicksilver(proxy(Entity, EntityWrapper))]
    pub actor: Entity,
    pub kind: ActionKind,
}

#[derive(Debug, Quicksilver)]
pub struct DelayedAction {
    pub action: Action,
}

#[derive(Debug, Quicksilver)]
pub enum ActionKind {
    Wait,
    Move {
        from: Pos,
        to: Pos,
    },
    BumpAttack {
        #[quicksilver(proxy(Entity, EntityWrapper))]
        target: Entity,
    },
    RockThrow {
        path: Vec<Pos>,
        #[quicksilver(proxy(Entity, EntityWrapper))]
        target: Entity,
    },
    DelayedSmash {
        dir: IVec,
    },
}

impl ActionKind {
    pub fn done_by(self, actor: Entity) -> Action {
        Action { actor, kind: self }
    }
}

#[derive(Debug, Quicksilver, Copy, Clone)]
#[repr(C)]
pub enum CreatureType {
    PlayerCharacter,
    Goblin,
    GoblinBrute,
}

impl CreatureType {
    pub fn create_deferred(self, world: &World, pos: Pos) -> EntityViewDeferred<'_> {
        let e = world.create_deferred();
        e.add(DrawPos(FPos::new(0., 0.)));
        e.add(DrawHealth { ratio: 1.0 });
        match self {
            CreatureType::PlayerCharacter => {
                e.add(Actor {
                    name: "Player".into(),
                    pos,
                    creature_type: self,
                    sprite: CreatureSprite::Dwarf,
                    hp: HP { max: 10, current: 10 },
                    next_turn: 0,
                });
                e.add(Player { pulse: 60., last_pulse_action: 0 });
                e.add(Fov(HashSet::new()));
            }
            CreatureType::Goblin => {
                e.add(Actor {
                    name: "Goblin".into(),
                    pos,
                    creature_type: self,
                    sprite: CreatureSprite::Goblin,
                    hp: HP { max: 5, current: 5 },
                    next_turn: 0,
                });
            }
            CreatureType::GoblinBrute => {
                e.add(Actor {
                    name: "Goblin Brute".into(),
                    pos,
                    creature_type: self,
                    sprite: CreatureSprite::GoblinBrute,
                    hp: HP { max: 5, current: 5 },
                    next_turn: 0,
                });
            }
        }
        e
    }
}

pub fn handle_death(world: &World, target: Entity, target_a: &Actor, animation: Entity) {
    zone!();
    if target_a.hp.current <= 0 {
        let msg = format!("{} dies.", target_a.name);
        log_message(world, msg, animation);

        if world.has_component::<Player>(target) {
            animation::spawn_game_over_animation(world, target);
        } else {
            world.view_deferred(target).relate_from::<AnimationCleanup>(animation);
        }
    }
}

pub fn raise_pulse(world: &World, e: Entity, e_actor: &Actor) {
    zone!();
    if let Some(mut player) = world.get_component_mut_opt::<Player>(e) {
        player.pulse += 3.0;
        player.last_pulse_action = e_actor.next_turn;
    };
}

pub fn lower_pulse(world: &World, e: Entity, e_actor: &mut Actor) {
    zone!();
    if let Some(mut player_p) = world.get_component_mut_opt::<Player>(e)
        && (e_actor.next_turn - player_p.last_pulse_action >= 30)
    {
        let before = player_p.pulse;
        player_p.pulse -= 1.0;
        if player_p.pulse < 60. && before >= 60. {
            let a = animation::spawn_empty_animation(world, e, 0.).entity;
            log_message(world, "Your pulse is getting low.".to_string(), a);
        }
        if player_p.pulse < 45. && before >= 45. {
            let a = animation::spawn_empty_animation(world, e, 0.).entity;
            log_message(
                world,
                "Your pulse is getting dangerously low. Do something exiting quick!"
                    .to_string(),
                a,
            );
        }
        if player_p.pulse < 30. {
            let a = animation::spawn_empty_animation(world, e, 0.).entity;
            log_message(world, "You die of cardiac arrest.".to_string(), a);
        }
    }
}

pub fn handle_action(world: &mut World, action: Action) {
    zone!();
    match action {
        Action { actor, kind: ActionKind::Wait } => {
            let mut actor_a = world.get_component_mut::<Actor>(actor);
            lower_pulse(world, actor, &mut actor_a);
            actor_a.next_turn += 10;
        }
        Action { actor, kind: ActionKind::Move { from, to } } => {
            let anim = animation::spawn_move_animation(world, actor, from, to);
            world.get_component_mut::<Actor>(actor).pos = to;
            if world.has_component::<Player>(actor) {
                animation::add_camera_move(world, anim, to);
            }
            let mut actor_a = world.get_component_mut::<Actor>(actor);
            lower_pulse(world, actor, &mut actor_a);
            actor_a.next_turn += 10;
        }
        Action { actor, kind: ActionKind::BumpAttack { target } } => {
            assert_ne!(actor, target);
            let mut actor_a = world.get_component_mut::<Actor>(actor);
            let mut target_a = world.get_component_mut::<Actor>(target);
            let hp_bar_change = target_a.hp.dmg(3);
            let animation = animation::spawn_bump_attack_animation(
                world,
                actor,
                target,
                actor_a.pos,
                target_a.pos,
                hp_bar_change,
            );
            let msg = format!("{} attacks {}.", actor_a.name, target_a.name);
            log_message(world, msg, animation);

            raise_pulse(world, actor, &actor_a);
            raise_pulse(world, target, &target_a);
            handle_death(world, target, &target_a, animation);
            actor_a.next_turn += 10;
        }
        Action { actor, kind: ActionKind::RockThrow { path, target } } => {
            let mut target_a = world.get_component_mut::<Actor>(target);
            let hp_bar_change = target_a.hp.dmg(2);
            let animation = animation::spawn_projectile_animation(
                world,
                DrawTile::Rock,
                path,
                hp_bar_change,
                target,
            );

            let mut actor_a = world.get_component_mut::<Actor>(actor);
            let msg = format!("{} throws a huge rock at {}.", actor_a.name, target_a.name);
            log_message(world, msg, animation);
            raise_pulse(world, target, &target_a);
            actor_a.next_turn += 10;

            handle_death(world, target, &target_a, animation);
        }
        Action { actor, kind: ActionKind::DelayedSmash { .. } } => {
            world.add_component(actor, DelayedAction { action });
            let mut actor_a = world.get_component_mut::<Actor>(actor);
            let animation = animation::spawn_empty_animation(world, actor, 0.);
            let msg = format!("{} prepares to smash.", actor_a.name);
            log_message(world, msg, *animation);
            actor_a.next_turn += 10;
        }
    };
}

pub fn handle_delayed_action(world: &World, action: Action) {
    zone!();

    match action {
        Action { actor, kind: ActionKind::DelayedSmash { dir } } => {
            let tm = world.singleton::<TileMap>();
            let mut actor_a = world.get_component_mut::<Actor>(actor);
            let target_pos = actor_a.pos + dir;

            if let Some(target) = tm.get_actor(target_pos) {
                assert_ne!(actor, target);
                let mut target_a = world.get_component_mut::<Actor>(target);
                let hp_bar_change = target_a.hp.dmg(3);
                let animation = animation::spawn_bump_attack_animation(
                    world,
                    actor,
                    target,
                    actor_a.pos,
                    target_a.pos,
                    hp_bar_change,
                );
                let msg = format!("{} smashes {}.", actor_a.name, target_a.name);
                log_message(world, msg, animation);

                raise_pulse(world, actor, &actor_a);
                raise_pulse(world, target, &target_a);
                handle_death(world, target, &target_a, animation);
            } else {
                let animation = animation::spawn_empty_animation(world, actor, 0.);
                let msg = format!("{} smashes nothing.", actor_a.name);
                log_message(world, msg, *animation);
            }
            actor_a.next_turn += 10;
        }
        other => panic!("Unhandled delayed action: {other:?}"),
    }
}

pub fn next_turn_actor(world: &World) -> Entity {
    zone!();
    query!(world, &this, Actor)
        .min_by_key(|(e, a)| (a.next_turn, e.id.0))
        .map(|(e, _a)| e.entity)
        .unwrap()
}

pub fn create_world() -> World {
    zone!();
    let mut world = World::new();
    register_components(&mut world);

    // TODO get properly random seed
    let tm = generate_map(12345);

    CreatureType::PlayerCharacter.create_deferred(&world, tm.up_stairs);
    world.process();

    world.singleton_add(tm);

    world.singleton_add(UI::default());
    world.singleton_add(TurnCount { aut: 0 });
    world.singleton_add(MessageLog::default());
    place_enemies(&mut world, 12345);

    world.singleton_add(RandomGenerator::new(12345));

    world.process();
    world
}
