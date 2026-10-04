pub trait Scalar: Copy + Clone + PartialEq {
    /// The additive identity for this scalar
    fn zero() -> Self;
    /// The multiplicative identity for this scalar
    fn one() -> Self;
}
