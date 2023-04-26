

use serde::Serialize;

use crate::Float;

/// A simple trait required for initializing some matrices (e.g., the
/// identity matrix)
pub trait OneZero {
    /// Returns an element considered to be 0.
    fn zero() -> Self;

    /// Returns an element considered to be 1.
    fn one() -> Self;
}

impl OneZero for Float {
    fn zero() -> Self {
        0.
    }
    fn one() -> Self {
        1.
    }
}


/// Define the basic algebraic requirements for T
pub trait Numberish:
    Copy
    + Clone
    + OneZero
    + PartialEq
    + Sized
    + std::fmt::Display
    + std::fmt::Debug
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::AddAssign
    + std::ops::SubAssign
    + std::ops::Mul<Float, Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::MulAssign
    // + std::ops::MulAssign<Float>
    + std::ops::Div<Float, Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::DivAssign
    // + std::ops::DivAssign<Float>
    + Sync
    + Send
    + Serialize
    + core::fmt::Debug
{
}
impl<
        T: OneZero
            + Copy
            + Clone
            + PartialEq
            + Sized
            + std::fmt::Display
            + std::fmt::Debug
            + std::ops::Add<Output = Self>
            + std::ops::Sub<Output = Self>
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Mul<Float, Output = Self>
            + std::ops::Mul<Output = Self>
            + std::ops::MulAssign
            // + std::ops::MulAssign<Float>
            + std::ops::Div<Float, Output = Self>
            + std::ops::Div<Output = Self>
            + std::ops::DivAssign
            // + std::ops::DivAssign<Float>
            + Sync
            + Send
            + Serialize
            + core::fmt::Debug,
    > Numberish for T
{
}
