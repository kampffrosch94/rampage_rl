use base::{FPos, Pos};
use froql::entity_view_deferred::EntityViewDeferred;
use froql::query;
use froql::{entity_store::Entity, world::World};
use simple_easing::{cubic_in_out, roundtrip, sine_in_out};

use crate::game::tiles::pos_to_drawpos;
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

pub fn spawn_bump_attack_animation(
    world: &World,
    e: Entity,
    target: Entity,
    start: Pos,
    end: Pos,
    damage: i32,
) {
    let start = pos_to_drawpos(start);
    let end = pos_to_drawpos(end);
    let animation_length = 0.15;

    let current_time = world.singleton::<GameTime>().0;

    let start_time = [e, target]
        .into_iter()
        .flat_map(|e| query!(world, AnimationTimer, AnimationTarget(this, *e)))
        .map(|(timer,)| timer.end)
        .fold(current_time, f32::max);

    world
        .create_deferred()
        .add(AnimationTimer { start: start_time, end: start_time + animation_length })
        .add(BumpAttackAnimation { start, end })
        .relate_to::<AnimationTarget>(e);

    //     let world = input.get();
    //     world.get_component_mut::<DrawPos>(e).0 = start;
    //     let mut tm = world.singleton_mut::<TileMap>();
    //     let mut rand = world.singleton_mut::<RandomGenerator>();
    //     let decor_pos = p_end + rand.random_direction();
    //     let decor = rand.pick_random(&[Decor::BloodRed1, Decor::BloodRed2]);
    //     tm.add_decor(decor_pos, decor);
    //     world.get_component_mut::<Actor>(target).hp.current -= damage;
    // });
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
