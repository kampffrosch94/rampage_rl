use base::{FPos, Pos};
use froql::query;
use froql::{entity_store::Entity, world::World};
use simple_easing::{cubic_in_out, roundtrip, sine_in_out};

use crate::game::tile_map::TileMap;
use crate::game::tiles::{Decor, pos_to_drawpos};
use crate::game::types::DrawHealth;
use crate::game::types::{Actor, GameTime};

use crate::{
    coroutines::{CoAccess, sleep_ticks},
    game::types::DrawPos,
    rand::RandomGenerator,
};

// Components and Relations

/// Relation: Animation Entity -> Affected Entity
pub enum AnimationTarget {}

#[derive(Debug)]
pub struct AnimationTimer {
    start: f32,
    end: f32,
}

impl AnimationTimer {
    fn normalize(&self, current_time: f32) -> f32 {
        let Self { start, end } = self;
        let length = end - start;
        let elapsed = current_time - start;
        let r = elapsed / length;
        r.clamp(0., 1.)
    }

    fn is_active(&self, current_time: f32) -> bool {
        current_time >= self.start
    }
}

pub struct MovementAnimation {
    start: FPos,
    end: FPos,
}

pub struct BumpAttackAnimation {
    start: FPos,
    end: FPos,
}

pub struct HPBarAnimation {
    pub start_ratio: f32,
    pub end_ratio: f32,
}

pub struct DecorSpawnAnimation {
    pub decor: Decor,
    pub pos: Pos,
}

pub fn handle_animations(world: &mut World) {
    let current_time = world.singleton::<GameTime>().0;
    // handle movement animation

    for (timer, anim, mut draw_pos) in query!(
        world,
        AnimationTimer,
        MovementAnimation,
        AnimationTarget(this, target),
        mut DrawPos(target)
    ) {
        if timer.is_active(current_time) {
            let lerpiness = sine_in_out(timer.normalize(current_time));
            draw_pos.0 = anim.start.lerp(anim.end, lerpiness);
        }
    }

    // handle bump attack
    for (timer, anim, mut draw_pos) in query!(
        world,
        AnimationTimer,
        BumpAttackAnimation,
        AnimationTarget(this, target),
        mut DrawPos(target)
    ) {
        if timer.is_active(current_time) {
            const PART_FORWARD: f32 = 0.7;
            let lerpiness =
                cubic_in_out(roundtrip(timer.normalize(current_time))) * PART_FORWARD;

            draw_pos.0 = anim.start.lerp(anim.end, lerpiness);
        }
    }

    // HP Bar animation
    for (timer, anim, mut draw_hp) in query!(
        world,
        AnimationTimer,
        HPBarAnimation,
        AnimationTarget(this, target),
        mut DrawHealth(target)
    ) {
        if timer.is_active(current_time) {
            let lerpiness = timer.normalize(current_time);
            draw_hp.ratio = lerp(anim.start_ratio, anim.end_ratio, lerpiness);
        }
    }

    // Decor spawn animation
    for (mut tm, anim_e, timer, anim) in query!(
        world,
        mut $ TileMap,
        &this,
        AnimationTimer,
        DecorSpawnAnimation,
    ) {
        if timer.start >= current_time {
            tm.add_decor(anim.pos, anim.decor);
            anim_e.destroy();
        }
    }
    world.process();

    // remove animations that are done
    for (e, timer) in query!(world, &this, AnimationTimer,) {
        if current_time >= timer.end {
            // dbg!(current_time);
            // dbg!(timer);
            e.destroy();
        }
    }
    world.process();
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

pub fn spawn_bump_attack_animation(
    world: &World,
    e: Entity,
    target: Entity,
    start_p: Pos,
    end_p: Pos,
    hp_bar_animation: HPBarAnimation,
) {
    let start = pos_to_drawpos(start_p);
    let end = pos_to_drawpos(end_p);
    let animation_length = 0.15;

    let current_time = world.singleton::<GameTime>().0;

    let start_time = [e, target]
        .into_iter()
        .flat_map(|e| query!(world, AnimationTimer, AnimationTarget(this, *e)))
        .map(|(timer,)| timer.end)
        .fold(current_time, f32::max);

    // bump animation
    world
        .create_deferred()
        .add(AnimationTimer { start: start_time, end: start_time + animation_length })
        .add(BumpAttackAnimation { start, end })
        .relate_to::<AnimationTarget>(e);

    // hp bar animation
    world
        .create_deferred()
        .add(AnimationTimer {
            start: start_time + animation_length / 2.,
            end: start_time + animation_length / 2.,
        })
        .add(hp_bar_animation)
        .relate_to::<AnimationTarget>(target);

    // spawn blood on ground
    let mut rand = world.singleton_mut::<RandomGenerator>();
    let decor_pos = end_p + rand.random_direction();
    let decor = rand.pick_random(&[Decor::BloodRed1, Decor::BloodRed2]);
    world
        .create_deferred()
        .add(AnimationTimer {
            start: start_time + animation_length / 2.,
            end: start_time + animation_length / 2.,
        })
        .add(DecorSpawnAnimation { decor, pos: decor_pos })
        .relate_to::<AnimationTarget>(target);
}

pub fn spawn_move_animation(world: &World, e: Entity, start: Pos, end: Pos) {
    assert!(world.has_component::<DrawPos>(e));

    let start = pos_to_drawpos(start);
    let end = pos_to_drawpos(end);
    let animation_length = 0.10;
    let current_time = world.singleton::<GameTime>().0;

    let start_time = query!(world, AnimationTimer, AnimationTarget(this, *e))
        .map(|(timer,)| timer.end)
        .fold(current_time, f32::max);

    world
        .create_deferred()
        .add(AnimationTimer { start: start_time, end: start_time + animation_length })
        .add(MovementAnimation { start, end })
        .relate_to::<AnimationTarget>(e);
}
