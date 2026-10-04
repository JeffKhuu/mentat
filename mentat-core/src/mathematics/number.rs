use crate::mathematics::algebra::scalar::Scalar;

/// Mentat's abstraction for numbers in the set of real numbers
#[derive(PartialEq, Clone, Copy)]
struct Real(f64);

/// Mentat's abstraction for numbers in the set of integers
#[derive(PartialEq, Clone, Copy)]
struct Integer(i32);

impl Scalar for Real {
    fn zero() -> Self {
        Real(0.0)
    }

    fn one() -> Self {
        Real(1.0)
    }
}

impl Scalar for Integer {
    fn zero() -> Self {
        Integer(0)
    }

    fn one() -> Self {
        Integer(1)
    }
}

/* Promotion and casting for these numerical types */

impl From<Integer> for i32 {
    fn from(value: Integer) -> Self {
        value.0
    }
}

impl From<Integer> for f64 {
    fn from(value: Integer) -> Self {
        value.0 as f64
    }
}

impl From<Real> for f64 {
    fn from(value: Real) -> Self {
        value.0
    }
}

impl From<Integer> for Real {
    fn from(n: Integer) -> Self {
        Real(n.into())
    }
}
