use std::ops::{Add, AddAssign, Mul, Sub};

use nanoserde::{DeJson, SerJson};

use crate::grids::Grid;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, SerJson, DeJson, PartialOrd, Ord)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct IVec {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, SerJson, DeJson)]
pub struct FPos {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct FVec {
    pub x: f32,
    pub y: f32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Neighbors in 8 directions
    pub fn neighbors<T>(&self, grid: Grid<T>) -> impl Iterator<Item = Pos> {
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

    pub fn manhattan_distance(&self, other: Pos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
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
    pub fn length(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

impl From<(f32, f32)> for FPos {
    fn from(value: (f32, f32)) -> Self {
        Self { x: value.0, y: value.1 }
    }
}
