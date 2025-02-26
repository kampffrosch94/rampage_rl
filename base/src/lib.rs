use std::{ffi::c_void, ops::Sub};

pub use text::TextProperty;
pub mod circle;
pub mod grids;
pub mod ldtk;
pub mod rect;
pub mod text;
pub use rect::Rect;

pub trait ContextTrait {
    /// time since program start
    fn time(&self) -> f64;

    /// frame delta time
    fn delta(&self) -> f32;

    /// frames per second
    fn fps(&self) -> f32;

    fn draw_rect(&mut self, rect: Rect, c: Color, z_level: i32);

    fn draw_rect_lines(&mut self, rect: Rect, thickness: f32, c: Color, z_level: i32);

    fn draw_circle(&mut self, circle: Circle, c: Color, z_level: i32);

    fn draw_texture(&mut self, name: &str, x: f32, y: f32, z_level: i32);

    fn draw_texture_part(&mut self, name: &str, src: Rect, x: f32, y: f32, z_level: i32);

    fn draw_texture_part_scaled(&mut self, name: &str, src: Rect, target: Rect, z_level: i32);

    fn load_texture(&mut self, name: &str, path: &str);

    fn texture_dimensions(&mut self, name: &str) -> Rect;

    fn is_pressed(&self, button: Button) -> bool;

    fn mouse_screen(&self) -> FPos;

    fn mouse_world(&self) -> FPos;

    fn set_text(&mut self, key: u64, w: f32, h: f32, text: &[(&str, TextProperty)]) -> Rect;

    fn draw_text(&mut self, key: u64, x: f32, y: f32, z_level: i32);
}

#[derive(Debug, Clone, Copy)]
pub enum Button {
    MouseLeft,
    MouseMiddle,
    MouseRight,
}

#[derive(Debug, Clone, Copy)]
pub struct FPos {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct FVec {
    pub x: f32,
    pub y: f32,
}

impl FVec {
    pub fn length(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

impl FPos {
    pub fn lerp(self, rhs: Self, s: f32) -> Self {
        let x = self.x + ((rhs.x - self.x) * s);
        let y = self.y + ((rhs.y - self.y) * s);
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Sub<Pos> for Pos {
    type Output = (i32, i32);

    fn sub(self, rhs: Pos) -> Self::Output {
        (self.x - rhs.x, self.y - rhs.y)
    }
}

/// Wrapper for state that is persisted between reloads
#[repr(C)]
pub struct PersistWrapper {
    pub ptr: *mut c_void,
    pub size: usize,
    pub align: usize,
}

impl PersistWrapper {
    pub fn ref_mut<T>(&mut self) -> &mut T {
        // TODO add checks for size and alignment matching
        let ptr = self.ptr as *mut T;
        unsafe { &mut *ptr }
    }
}

/// rgba values from 0.0 to 1.0
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub pos: FPos,
    pub radius: f32,
}

impl Color {
    pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
    pub const BLUE: Color = Color::rgb(0.3, 0.3, 1.0);
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    pub const VIOLET: Color = Color::rgb(0.5, 0.0, 0.5);
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const PINK: Color = Color::rgb(1.0, 0.75, 0.8);

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }
}

impl FPos {
    pub fn distance(self, rhs: Self) -> f32 {
        (self - rhs).length()
    }
}

impl Sub for FPos {
    type Output = FVec;

    fn sub(self, rhs: Self) -> Self::Output {
        FVec { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}
