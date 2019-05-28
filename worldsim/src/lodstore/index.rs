use vek::*;
use std::ops::Sub;

/*
For our LodStructures we need a type that covers the values from 0 - 2047 in steps of 1/32.
which is 11 bits for the digits before the decimal point and 5 bits for the digits after the decimal point.
Because for accessing the decimal point makes no difference we use a u16 to represent this value.
The value needs to be shiftet to get it's "real inworld size",
e.g. 1 represents 1/32
     32 represents 1
     65535 represents 2047 + 31/32
*/

/*
Edit: now it actually implements a value from 0 - 3*2048 - 1/32, covering over 3 regions for accessing neightbor region values
*/

/*
Pos goes from -2048 to 2*2048- 1/32
*/

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct LodIndex {
    pub data: Vec3<u32>, //keep this private
}

impl Sub for LodIndex {
    type Output = LodIndex;
    fn sub(self, rhs: LodIndex) -> Self::Output {
        LodIndex::new(
            self.data.map2(rhs.data, |x,y| (x-y) as i32),
        )
    }
}
impl LodIndex {
    pub fn new(pos: Vec3<i32>) -> Self {
        Self {
            data: pos.map(|x| (x * 32 + 65535) as u32),
        }
    }

    pub fn newf(pos: Vec3<f32>) -> Self {
        Self {
            data: pos.map(|x| (x * 32.0).round() as u32 + 65535),
        }
    }

    pub fn to_pos_i(&self) -> Vec3<i32> { self.data.map(|x| (x / 32 - 2048) as i32) }

    pub fn to_pos_f(&self) -> Vec3<f32> {
        self.data.map(|x| x as f32 / 32.0 - 2048.0)
    }
}

pub fn relative_to_1d(index: LodIndex, relative_size: LodIndex) -> usize {
    (index.data[0] + index.data[1] * relative_size.data[0] + index.data[2] * relative_size.data[0] * relative_size.data[1]) as usize
}

pub const LEVEL_LENGTH_POW_MAX: i8 = 11;
pub const LEVEL_LENGTH_POW_MIN: i8 = -4;

pub const LEVEL_INDEX_POW_MAX: u8 = 15;
pub const LEVEL_INDEX_POW_MIN: u8 = 0;

pub const fn length_to_index(n: i8) -> u8 { (n+4) as u8 }

pub const fn two_pow_u(n: u8) -> u16 {
    1 << n
}

pub fn two_pow_i(n: i8) -> f32 {
    2.0_f32.powi(n as i32)
}