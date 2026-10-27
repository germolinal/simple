/// A simple trait required for initializing some matrices (e.g., the
/// identity matrix)
pub trait OneZero {
    /// Returns an element considered to be 0.
    fn zero() -> Self;

    /// Returns an element considered to be 1.
    fn one() -> Self;
}

impl OneZero for f64 {
    fn zero() -> Self {
        0.
    }
    fn one() -> Self {
        1.
    }
}
impl OneZero for f32 {
    fn zero() -> Self {
        0.
    }
    fn one() -> Self {
        1.
    }
}

impl OneZero for i8 {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
}
impl OneZero for i16 {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
}
impl OneZero for i32 {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
}
impl OneZero for i64 {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
}
impl OneZero for i128 {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
}

impl OneZero for u8 {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
}
impl OneZero for u16 {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
}
impl OneZero for u32 {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
}
impl OneZero for u64 {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
}
impl OneZero for u128 {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
}

impl OneZero for usize {
    fn zero() -> Self {        
        0
    }
    fn one() -> Self {
        1
    }
}

pub trait Nanish {
    fn is_it_nan(&self) -> bool;
}

impl Nanish for f64 {
    fn is_it_nan(&self) -> bool {
        self.is_nan()        
    }
}
impl Nanish for f32 {
    fn is_it_nan(&self) -> bool {
        self.is_nan()        
    }
}

impl Nanish for i8 {
    fn is_it_nan(&self) -> bool {        
        false
    }
}
impl Nanish for i16 {
    fn is_it_nan(&self) -> bool {
        false
    }
}
impl Nanish for i32 {
    fn is_it_nan(&self) -> bool {
        false
    }
}
impl Nanish for i64 {
    fn is_it_nan(&self) -> bool {
        false
    }
}
impl Nanish for i128 {
    fn is_it_nan(&self) -> bool {
        false
    }
}

impl Nanish for u8 {
    fn is_it_nan(&self) -> bool {
        false
    }
}
impl Nanish for u16 {
    fn is_it_nan(&self) -> bool {
        false
    }
}
impl Nanish for u32 {
    fn is_it_nan(&self) -> bool {
        false
    }
}
impl Nanish for u64 {
    fn is_it_nan(&self) -> bool {
        false
    }
}
impl Nanish for u128 {
    fn is_it_nan(&self) -> bool {
        false
    }
}

impl Nanish for usize {
    fn is_it_nan(&self) -> bool {
        false
    }
}

/// Define the basic algebraic requirements for T
pub trait Numberish:
    Copy
    + Nanish
    + OneZero
    + Clone
    + PartialEq
    + Sized
    + std::fmt::Display
    + std::fmt::Debug
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::AddAssign
    + std::ops::SubAssign
    + std::ops::Mul<Self, Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::MulAssign
    + std::ops::Div<Self, Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::DivAssign
    + Sync
    + Send
    + core::fmt::Debug
    + PartialOrd
    + From<f32>
    // + From<i32>
    + Into<f64>
{
}

impl<
        T: Clone
            + Nanish
            + OneZero
            + Copy
            + PartialEq
            + Sized
            + std::fmt::Display
            + std::fmt::Debug
            + std::ops::Add<Output = Self>
            + std::ops::Sub<Output = Self>
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Mul<Self, Output = Self>
            + std::ops::Mul<Output = Self>
            + std::ops::MulAssign
            + std::ops::Div<Self, Output = Self>
            + std::ops::Div<Output = Self>
            + std::ops::DivAssign
            + Sync
            + Send
            + core::fmt::Debug
            + PartialOrd
            + From<f32>
            // + From<i32>
            + Into<f64>,
    > Numberish for T
{
}
