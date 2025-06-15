use std::ops::{Add, Mul, Sub};

#[derive(Copy, Clone, Debug)]
pub struct Rational {
    pub nr: i32,
    pub denom: i32,
}

impl PartialEq for Rational {
    fn eq(&self, other: &Self) -> bool {
        let a = self.normalize();
        let b = other.normalize();
        a.nr == b.nr && a.denom == b.denom
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = self.normalize();
        let b = other.normalize();
        let x = a.nr as i128 * b.denom as i128;
        let y = b.nr as i128 * a.denom as i128;
        x.partial_cmp(&y)
    }
}

impl Rational {
    pub fn new(nr: i32, denom: i32) -> Self {
        if denom == 0 {
            panic!("Denominator cannot be zero.");
        }

        Rational { nr, denom }.normalize()
    }

    pub fn normalize(self) -> Rational {
        let Self { nr, denom } = self;
        let gcd_val = gcd(nr.abs(), denom.abs());
        let mut nr = nr / gcd_val;
        let mut denom = denom / gcd_val;

        if denom < 0 {
            nr = -nr;
            denom = -denom;
        }

        Rational { nr, denom }
    }

    pub fn as_float(self) -> f32 {
        self.nr as f32 / self.denom as f32
    }

    /// Rounds towards minus infinity.
    pub fn floor(self) -> i32 {
        if self.nr >= 0 {
            self.nr / self.denom
        } else {
            (self.nr - self.denom + 1) / self.denom
        }
    }

    /// Rounds towards plus infinity.
    pub fn ceil(self) -> i32 {
        if self.nr >= 0 {
            (self.nr + self.denom - 1) / self.denom
        } else {
            self.nr / self.denom
        }
    }
}

pub fn gcd(a: i32, b: i32) -> i32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

impl Sub for Rational {
    type Output = Rational;

    fn sub(self, rhs: Self) -> Self::Output {
        // a/b - c/d = (ad - bc) / bd
        let common_denom = self.denom * rhs.denom;
        let nr = self.nr * rhs.denom - rhs.nr * self.denom;
        Rational::new(nr, common_denom)
    }
}

impl Add for Rational {
    type Output = Rational;

    fn add(self, rhs: Self) -> Self::Output {
        // a/b + c/d = (ad + bc) / bd
        let common_denom = self.denom * rhs.denom;
        let nr = self.nr * rhs.denom + rhs.nr * self.denom;
        Rational::new(nr, common_denom)
    }
}

impl Mul for Rational {
    type Output = Rational;

    fn mul(self, rhs: Self) -> Self::Output {
        // (a/b) * (c/d) = (a*c) / (b*d)
        let nr = self.nr * rhs.nr;
        let denom = self.denom * rhs.denom;
        Rational::new(nr, denom)
    }
}
