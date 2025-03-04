use nanoserde::{DeJson, SerJson};
use std::fmt::Debug;

#[derive(Debug, DeJson, SerJson)]
pub struct RandomGenerator {
    seed: u64,
}

impl RandomGenerator {
    pub fn new(seed: u64) -> Self {
        RandomGenerator { seed }
    }

    pub fn next(&mut self) -> u64 {
        // yoinked from https://github.com/ForestJ2/lcg-rand
        const MODULUS: u128 = 2u128.pow(64);
        const INCREMENT: u64 = 1442695040888963407;
        const MULTIPLIER: u64 = 6364136223846793005;
        self.seed = (((MULTIPLIER as u128) * (self.seed as u128) + INCREMENT as u128)
            % MODULUS) as u64;

        self.seed >> 5
    }

    pub fn next_in_range(&mut self, from: u64, to: u64) -> u64 {
        from + (self.next() % (to - from))
    }

    pub fn random_direction(&mut self) -> (i32, i32) {
        let index = self.next_in_range(0, DIRECTIONS.len() as u64);
        DIRECTIONS[index as usize]
    }

    pub fn pick_random<T: Copy + Debug>(&mut self, options: &[T]) -> T {
        let index = self.next() % 2;
        dbg!(&options);
        dbg!(&index);
        options[index as usize]
    }
}

const DIRECTIONS: [(i32, i32); 9] =
    [(-1, 0), (1, 0), (0, -1), (1, -1), (0, 0), (-1, -1), (0, 1), (1, 1), (-1, 1)];
