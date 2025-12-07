use std::collections::HashSet;

use base::{FPos, Pos, zone};
use froql::{entity_store::Entity, query, world::World};

use crate::{
    animation::{self, AnimationCleanup},
    game::{
        drawing::{DrawHealth, DrawPos},
        ecs_types::{Fov, HP, Player, TurnCount, UI, register_components},
        mapgen::{generate_map, place_enemies},
        sprites::{CreatureSprite, DrawTile},
        ui::{MessageLog, log_message},
    },
    rand::RandomGenerator,
};

use super::ecs_types::Actor;

/// Anything an actor may do
#[derive(Debug)]
pub struct Action {
    pub actor: Entity,
    pub kind: ActionKind,
}

#[derive(Debug)]
pub enum ActionKind {
    Wait,
    Move { from: Pos, to: Pos },
    BumpAttack { target: Entity },
    UseAbility(Ability),
}

#[derive(Debug)]
pub enum Ability {
    RockThrow { path: Vec<Pos>, target: Entity },
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
            let a = animation::spawn_empty_animation(world, e, 0.);
            log_message(world, "Your pulse is getting low.".to_string(), a);
        }
        if player_p.pulse < 45. && before >= 45. {
            let a = animation::spawn_empty_animation(world, e, 0.);
            log_message(
                world,
                "Your pulse is getting dangerously low. Do something exiting quick!"
                    .to_string(),
                a,
            );
        }
        if player_p.pulse < 30. {
            let a = animation::spawn_empty_animation(world, e, 0.);
            log_message(world, "You die of cardiac arrest.".to_string(), a);
        }
    }
}

pub fn handle_action(world: &World, action: Action) {
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
        Action {
            actor,
            kind: ActionKind::UseAbility(Ability::RockThrow { path, target }),
        } => {
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
    };
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

    let _player = world
        .create()
        .add(Player { pulse: 60., last_pulse_action: 0 })
        .add(DrawPos(FPos::new(0., 0.)))
        .add(Actor {
            name: "Player".into(),
            pos: tm.up_stairs,
            sprite: CreatureSprite::Dwarf,
            hp: HP { max: 10, current: 10 },
            next_turn: 0,
        })
        .add(DrawHealth { ratio: 1.0 })
        .add(Fov(HashSet::new()));

    world.singleton_add(tm);

    world.singleton_add(UI::default());
    world.singleton_add(TurnCount { aut: 0 });
    world.singleton_add(MessageLog::default());
    place_enemies(&mut world, 12345);

    world.singleton_add(RandomGenerator::new(12345));

    world.process();
    world
}
