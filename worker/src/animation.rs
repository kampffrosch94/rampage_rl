use base::{ContextTrait, FPos, FVec, Pos, Rect, zone};
use fastnoise_lite::{FastNoiseLite, NoiseType};
use froql::query;
use froql::{entity_store::Entity, world::World};
use simple_easing::{cubic_in_out, roundtrip, sine_in_out};

use crate::game::ecs_types::GameTime;
use crate::game::ecs_types::{DrawHealth, UI, UIState};
use crate::game::sprites::{Decor, DrawTile, TILE_DIM, TILE_SIZE, pos_to_drawpos};
use crate::game::tile_map::TileMap;
use crate::game::z_levels::*;

use crate::{game::ecs_types::DrawPos, rand::RandomGenerator};

// Components and Relations

/// Relation: Animation Entity -> Affected Entity
pub enum AnimationTarget {}

/// Relation: Animation Entity -> Affected Entity
///
/// If the animation is destroyed the affected entity also is destroyed
/// (this relation is registered with cascading destroy)
pub enum AnimationCleanup {}

#[derive(Debug, Clone, Copy)]
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

pub struct CameraShakeAnimation {
    pub noise: FastNoiseLite,
}

pub struct ProjectilePathAnimation {
    pub sprite: DrawTile,
    pub path: Vec<Pos>,
}

/// moves the camera center somewhere else
pub struct CameraMoveAnimation {
    /// only gets initialized once the animation starts playing
    pub from: Option<FPos>,
    pub to: FPos,
}

/// transition to GameOver UI State
pub struct GameOverAnimation {}

pub fn handle_animations(c: &mut dyn ContextTrait, world: &mut World) {
    zone!();

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

    // bump attack animation
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

    // projectile animation
    for (timer, anim) in query!(world, AnimationTimer, ProjectilePathAnimation,) {
        if timer.is_active(current_time) {
            let highest_index = anim.path.len() - 1;
            let current = timer.normalize(current_time) * highest_index as f32;
            let index = current.floor() as usize;
            let next = (index + 1).min(highest_index);
            let a = anim.path.get(index).map(|it| it.to_fpos(TILE_SIZE));
            let b = anim.path.get(next).map(|it| it.to_fpos(TILE_SIZE));
            match (a, b) {
                (Some(a), Some(b)) => {
                    let lerpiness = current.fract();
                    let dpos = a.lerp(b, lerpiness);
                    anim.sprite.draw(c, dpos, Z_PROJECTILE);
                }
                _ => {}
            }
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
        if timer.start <= current_time {
            tm.add_decor(anim.pos, anim.decor);
            anim_e.destroy();
        }
    }
    world.process();

    // Game Over
    for (timer,) in query!(world, _ GameOverAnimation, AnimationTimer) {
        if timer.is_active(current_time) {
            let mut ui = world.singleton_mut::<UI>();
            ui.state = UIState::PostDeath;
        }
    }

    // Camera shake animation
    c.camera_set_shake(FVec::ZERO); // if no animation is playing this stays at zero
    for (anim, timer) in query!(world, CameraShakeAnimation, AnimationTimer) {
        if timer.is_active(current_time) {
            let strength = 1000.;
            let range = 45.;
            let lerpiness = timer.normalize(current_time) * strength;
            let x = range * anim.noise.get_noise_2d(lerpiness, lerpiness);
            let y = range * anim.noise.get_noise_2d(lerpiness + 50., lerpiness + 50.);
            c.camera_set_shake(FVec { x, y });
        }
    }

    // camera move
    // only animate the camera with the latest start time
    let max_start = query!(world, _ CameraMoveAnimation, AnimationTimer)
        .filter(|(t,)| t.is_active(current_time))
        .map(|(t,)| t.start)
        .max_by(f32::total_cmp)
        .unwrap_or(0.);

    for (mut anim, timer) in query!(world, mut CameraMoveAnimation, AnimationTimer) {
        if timer.is_active(current_time) && timer.start == max_start {
            if anim.from.is_none() {
                anim.from = Some(c.screen_rect_world().center());
            }

            let ease = simple_easing::cubic_out;
            let lerpiness = ease(timer.normalize(current_time));
            let from = anim.from.unwrap();
            let dist = anim.to - from;
            let current_target = from + dist * lerpiness;

            let delta = current_target - c.screen_rect_world().center();
            c.camera_move_rel(delta);
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
) -> Entity {
    zone!();
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
    let animation_e = world
        .create_deferred()
        .add(AnimationTimer { start: start_time, end: start_time + animation_length })
        .add(BumpAttackAnimation { start, end })
        .relate_to::<AnimationTarget>(e)
        .entity;

    // hp bar animation
    world
        .create_deferred()
        .add(AnimationTimer {
            start: start_time + animation_length / 2.,
            end: start_time + animation_length,
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
            start: start_time + animation_length * 0.75,
            end: start_time + animation_length * 0.75,
        })
        .add(DecorSpawnAnimation { decor, pos: decor_pos })
        .relate_to::<AnimationTarget>(target);

    return animation_e;
}

pub fn spawn_move_animation(world: &World, e: Entity, start: Pos, end: Pos) -> Entity {
    zone!();
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
        .relate_to::<AnimationTarget>(e)
        .entity
}

pub fn spawn_camera_shake_animation(world: &mut World) -> Entity {
    zone!();
    let animation_length = 0.07;
    let current_time = world.singleton::<GameTime>().0;
    let start_time = current_time;

    // seed determined by fair dice roll, guaranteed to be random
    let mut noise = FastNoiseLite::with_seed(4);
    noise.set_noise_type(Some(NoiseType::Perlin));

    world
        .create()
        .add(AnimationTimer { start: start_time, end: start_time + animation_length })
        .add(CameraShakeAnimation { noise })
        .entity
}

/// Moves the camera to goal_pos at the timing of the Animation sync_anim
pub fn add_camera_move(world: &World, sync_anim: Entity, goal_pos: Pos) {
    zone!();
    let goal = {
        let p = pos_to_drawpos(goal_pos);
        Rect::new_center_wh(p, TILE_DIM, TILE_DIM).center()
    };

    world.defer_closure(move |world| {
        let mut timer = world.get_component::<AnimationTimer>(sync_anim).clone();
        const A_LENGTH: f32 = 0.5;
        timer.end = timer.start + A_LENGTH;

        world.create().add(timer).add(CameraMoveAnimation { from: None, to: goal });
    });
}

#[expect(unused)]
pub fn spawn_camera_move(world: &World, target: Entity, goal_pos: Pos) {
    zone!();
    let goal = {
        let p = pos_to_drawpos(goal_pos);
        Rect::new_center_wh(p, TILE_DIM, TILE_DIM).center()
    };

    let current_time = world.singleton::<GameTime>().0;
    let start_time = query!(world, AnimationTimer, AnimationTarget(this, *target))
        .map(|(timer,)| timer.end)
        .fold(current_time, f32::max);

    const A_LENGTH: f32 = 0.5;

    world
        .create_deferred()
        .add(AnimationTimer { start: start_time, end: start_time + A_LENGTH })
        .add(CameraMoveAnimation { from: None, to: goal });
}

pub fn spawn_projectile_animation(
    world: &World,
    projectile_sprite: DrawTile,
    path: Vec<Pos>,
    hp_bar_animation: HPBarAnimation,
    target: Entity,
) -> Entity {
    zone!();
    let animation_length = 0.02 * path.len().min(5) as f32;
    let current_time = world.singleton::<GameTime>().0;
    let start_time = query!(world, AnimationTimer, AnimationTarget(this, *target))
        .map(|(timer,)| timer.end)
        .fold(current_time, f32::max);

    world
        .create_deferred()
        .add(AnimationTimer { start: start_time, end: start_time + animation_length })
        .add(ProjectilePathAnimation { path, sprite: projectile_sprite })
        .relate_to::<AnimationTarget>(target);

    // hp bar
    let hp_anim_length = 0.07;
    let anim = world
        .create_deferred()
        .add(AnimationTimer {
            // hp bar animation starts after the hit
            start: start_time + animation_length,
            end: start_time + animation_length + hp_anim_length,
        })
        .add(hp_bar_animation)
        .relate_to::<AnimationTarget>(target)
        .entity;
    // return the animation that finishes later
    anim
}

pub fn spawn_game_over_animation(world: &World, target: Entity) -> Entity {
    zone!();
    let animation_length = 0.5;
    let current_time = world.singleton::<GameTime>().0;
    let start_time = query!(world, AnimationTimer, AnimationTarget(this, *target))
        .map(|(timer,)| timer.end)
        .fold(current_time, f32::max);

    let anim = world
        .create_deferred()
        .add(AnimationTimer { start: start_time, end: start_time + animation_length })
        .add(GameOverAnimation {})
        .relate_to::<AnimationTarget>(target)
        .entity;
    anim
}

// TODO use this as base for other animation spawn functions
pub fn spawn_empty_animation(world: &World, target: Entity, animation_length: f32) -> Entity {
    zone!();
    let current_time = world.singleton::<GameTime>().0;
    let start_time = query!(world, AnimationTimer, AnimationTarget(this, *target))
        .map(|(timer,)| timer.end)
        .fold(current_time, f32::max);

    let anim = world
        .create_deferred()
        .add(AnimationTimer { start: start_time, end: start_time + animation_length })
        .relate_to::<AnimationTarget>(target)
        .entity;
    anim
}
