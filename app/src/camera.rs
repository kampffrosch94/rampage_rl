use std::ops::{Add, Sub, SubAssign};

use base::{FPos, FVec, zone};
use macroquad::prelude::*;
use tween::{Linear, TweenValue, Tweener};

pub struct CameraWrapper {
    pub scale: f32,
    pub scale_exp: i32,
    pub offset: Vec2f,
    pub shake_offset: Vec2f,
    pub scale_tween: Tweener<f32, f32, Linear>,
    pub camera: Camera2D,
}

impl Default for CameraWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraWrapper {
    pub fn new() -> Self {
        zone!();
        let scale_exp = 1; // 2 is actually zero zoom
        let base2: f32 = 2.;
        let scale = base2.powf(scale_exp as f32);
        let scale_tween = Tweener::linear(scale, scale, 0.);

        let offset = Vec2f { x: -160., y: -40. };
        let shake_offset = Vec2f { x: 0., y: 0. };

        let camera = Self::create_camera(scale, offset.into());
        set_camera(&camera);
        CameraWrapper { scale, scale_exp, scale_tween, offset, camera, shake_offset }
    }

    pub fn create_camera(scale: f32, offset: Vec2) -> Camera2D {
        zone!();
        let scale = scale / screen_dpi_scale();
        Camera2D {
            zoom: vec2(scale / screen_width(), scale / screen_height()),
            rotation: 0.,
            offset: vec2(0., 0.),
            target: vec2(
                screen_width() / scale + offset.x,
                screen_height() / scale + offset.y,
            ),
            render_target: None,
            viewport: None,
        }
    }

    pub fn set(&self) {
        set_camera(&self.camera);
    }

    /// do tweening and stuff
    pub fn process(&mut self) {
        zone!();
        // tweening
        if !self.scale_tween.is_finished() {
            let mouse_position = Vec2f::from(mouse_position());
            let time = get_frame_time();
            let point = Vec2f::from(self.camera.screen_to_world(mouse_position.into()));
            let new_scale = self.scale_tween.move_by(time);
            let offset = self.offset + self.shake_offset;
            let new_camera = Self::create_camera(new_scale, offset.into());
            let new_point = Vec2f::from(new_camera.screen_to_world(mouse_position.into()));
            let pan_correction = new_point - point;
            self.offset -= pan_correction;
            self.scale = new_scale;
        }

        let offset = self.offset + self.shake_offset;
        self.camera = Self::create_camera(self.scale, offset.into());
    }

    pub fn zoom(&mut self, delta: i32) {
        zone!();
        self.scale_exp += delta;
        let base2: f32 = 2.;
        self.scale_exp = self.scale_exp.clamp(0, 5); // TODO set min to 1 before release?
        let target = base2.powf(self.scale_exp as f32);
        self.scale_tween = Tweener::linear(self.scale, target, 0.25);
    }

    pub fn move_camera_relativ(&mut self, FVec { x, y }: FVec) {
        zone!();
        self.offset = self.offset + Vec2f::from((x, y));
        // need to update the camera, because it may be used to save positions immediately
        let offset = self.offset + self.shake_offset;
        self.camera = Self::create_camera(self.scale, offset.into());
    }

    pub fn screen_to_world(&self, FPos { x, y }: FPos) -> FPos {
        zone!();
        let Vec2 { x, y } = self.camera.screen_to_world(Vec2 { x, y });
        FPos { x, y }
    }

    pub fn world_to_screen(&self, FPos { x, y }: FPos) -> FPos {
        zone!();
        let Vec2 { x, y } = self.camera.world_to_screen(Vec2 { x, y });
        FPos { x, y }
    }

    pub fn mouse_world(&self) -> FPos {
        zone!();
        let (x, y) = mouse_position();
        self.screen_to_world(FPos { x, y })
    }
}

pub fn screen_camera() -> Camera2D {
    zone!();
    CameraWrapper::create_camera(2.0, vec2(0., 0.))
}

// needed because orphan rules are annoying
#[derive(Default, Clone, Copy)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

impl std::fmt::Debug for Vec2f {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vec2f").field("x", &self.x).field("y", &self.y).finish()
    }
}

impl Add for Vec2f {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl Sub for Vec2f {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl SubAssign for Vec2f {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl TweenValue for Vec2f {
    fn scale(self, scale: f32) -> Self {
        Self { x: self.x * scale, y: self.y * scale }
    }
}

impl From<Vec2> for Vec2f {
    fn from(value: Vec2) -> Self {
        Vec2f { x: value.x, y: value.y }
    }
}

impl Into<Vec2> for Vec2f {
    fn into(self) -> Vec2 {
        Vec2 { x: self.x, y: self.y }
    }
}

impl From<(f32, f32)> for Vec2f {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}
