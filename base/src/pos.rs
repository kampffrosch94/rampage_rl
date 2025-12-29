use std::ops::{Add, AddAssign, Mul, Sub};

use quicksilver::Quicksilver;

use crate::{ContextTrait, Rect, grids::Grid};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord, Quicksilver)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Hash, Eq, Quicksilver)]
pub struct IVec {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Quicksilver)]
pub struct FPos {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Quicksilver)]
pub struct FVec {
    pub x: f32,
    pub y: f32,
}

impl FVec {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Neighbors in 8 directions
    pub fn neighbors<T>(&self, grid: &Grid<T>) -> impl Iterator<Item = Pos> {
        const DIRECTIONS: [(i32, i32); 8] =
            [(-1, 0), (1, 0), (0, -1), (1, -1), (-1, -1), (0, 1), (1, 1), (-1, 1)];

        let h = grid.height;
        let w = grid.width;
        DIRECTIONS
            .iter()
            .copied()
            .map(move |dir| *self + dir)
            .filter(move |pos| 0 <= pos.x && pos.x < w && 0 <= pos.y && pos.y < h)
    }

    /// Neighbors in 4 directions
    pub fn neighbors_orth<T>(&self, grid: &Grid<T>) -> impl Iterator<Item = Pos> {
        const DIRECTIONS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

        let h = grid.height;
        let w = grid.width;
        DIRECTIONS
            .iter()
            .map(move |dir| *self + *dir)
            .filter(move |pos| 0 <= pos.x && pos.x < w && 0 <= pos.y && pos.y < h)
    }

    pub fn distance_manhattan(&self, other: Pos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    pub fn distance(&self, other: Pos) -> i32 {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        i32::max(dx, dy)
    }

    pub fn to_fpos(&self, factor: f32) -> FPos {
        FPos { x: self.x as f32 * factor, y: self.y as f32 * factor }
    }

    /// Uses the algorithm from: https://www.redblobgames.com/grids/line-drawing/
    pub fn line(&self, target: Pos) -> Vec<Pos> {
        let mut result = Vec::new();
        let n = self.distance(target);
        for step in 0..=n {
            let t = if n == 0 { 0. } else { step as f32 / n as f32 };
            let FPos { x: mut fx, y: mut fy } = self.to_fpos(1.0).lerp(target.to_fpos(1.0), t);
            fx += f32::EPSILON;
            fy += f32::EPSILON;
            result.push(Pos { x: fx.round() as i32, y: fy.round() as i32 });
        }
        result
    }

    /// From: https://www.roguebasin.com/index.php/Bresenham%27s_Line_Algorithm#Rust
    pub fn bresenham(&self, b: Pos) -> Vec<Pos> {
        let mut points = Vec::new();
        let mut x1 = self.x;
        let mut y1 = self.y;
        let mut x2 = b.x;
        let mut y2 = b.y;
        let is_steep = (y2 - y1).abs() > (x2 - x1).abs();
        if is_steep {
            std::mem::swap(&mut x1, &mut y1);
            std::mem::swap(&mut x2, &mut y2);
        }
        let mut reversed = false;
        if x1 > x2 {
            std::mem::swap(&mut x1, &mut x2);
            std::mem::swap(&mut y1, &mut y2);
            reversed = true;
        }
        let dx = x2 - x1;
        let dy = (y2 - y1).abs();
        let mut err = dx / 2;
        let mut y = y1;
        let ystep: i32;
        if y1 < y2 {
            ystep = 1;
        } else {
            ystep = -1;
        }
        for x in x1..(x2 + 1) {
            if is_steep {
                points.push(Pos { x: y, y: x });
            } else {
                points.push(Pos { x: x, y: y });
            }
            err -= dy;
            if err < 0 {
                y += ystep;
                err += dx;
            }
        }

        if reversed {
            for i in 0..(points.len() / 2) {
                let end = points.len() - 1;
                points.swap(i, end - i);
            }
        }
        points
    }

    /// A circle looks suspiciouly like a rect if diagonals have distance 1
    pub fn circle_around(self, radius: i32) -> Vec<Pos> {
        let mut r = Vec::new();
        for x in (self.x - radius)..=(self.x + radius) {
            r.push(Self { x, y: self.y - radius });
            r.push(Self { x, y: self.y + radius });
        }
        for y in (self.y - radius + 1)..=(self.y + radius - 1) {
            r.push(Self { x: self.x - radius, y });
            r.push(Self { x: self.x + radius, y });
        }
        r
    }
}

impl From<(i32, i32)> for Pos {
    fn from(value: (i32, i32)) -> Self {
        Self { x: value.0, y: value.1 }
    }
}

impl Sub<Pos> for Pos {
    type Output = IVec;

    fn sub(self, rhs: Pos) -> Self::Output {
        IVec { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl AddAssign<(i32, i32)> for Pos {
    fn add_assign(&mut self, rhs: (i32, i32)) {
        self.x += rhs.0;
        self.y += rhs.1;
    }
}

impl AddAssign<IVec> for Pos {
    fn add_assign(&mut self, rhs: IVec) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Add<(i32, i32)> for Pos {
    type Output = Pos;

    fn add(self, rhs: (i32, i32)) -> Self::Output {
        Pos { x: self.x + rhs.0, y: self.y + rhs.1 }
    }
}

impl Mul<f32> for Pos {
    type Output = FPos;

    fn mul(self, rhs: f32) -> Self::Output {
        FPos { x: self.x as f32 * rhs, y: self.y as f32 * rhs }
    }
}

impl FPos {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn lerp(self, rhs: Self, s: f32) -> Self {
        let x = self.x + ((rhs.x - self.x) * s);
        let y = self.y + ((rhs.y - self.y) * s);
        Self { x, y }
    }

    pub fn distance(self, rhs: Self) -> f32 {
        (self - rhs).length()
    }

    pub fn rect(self, size: f32) -> Rect {
        Rect::new(self.x, self.y, size, size)
    }

    /// the same position but converted from world to screen coordinates
    pub fn to_screen(self, c: &mut dyn ContextTrait) -> FPos {
        c.camera_world_to_screen(self)
    }
}

impl Sub for FPos {
    type Output = FVec;

    fn sub(self, rhs: Self) -> Self::Output {
        FVec { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl Into<(f32, f32)> for FPos {
    fn into(self) -> (f32, f32) {
        (self.x, self.y)
    }
}

impl FVec {
    pub const ZERO: FVec = FVec { x: 0., y: 0. };
    pub fn length(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

impl From<(f32, f32)> for FPos {
    fn from(value: (f32, f32)) -> Self {
        Self { x: value.0, y: value.1 }
    }
}

impl From<(i32, i32)> for FVec {
    fn from(value: (i32, i32)) -> Self {
        Self { x: value.0 as f32, y: value.1 as f32 }
    }
}

impl Mul<f32> for FVec {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self { x: self.x * rhs, y: self.y * rhs }
    }
}

impl Add<FVec> for FPos {
    type Output = Self;

    fn add(self, rhs: FVec) -> Self::Output {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl IVec {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Add<IVec> for Pos {
    type Output = Self;

    fn add(self, rhs: IVec) -> Self::Output {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}
