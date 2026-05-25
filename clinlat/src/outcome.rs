//! Operator outcome types: refined hypothesis or abstention.
//!
//! Implements DEF-MP-13 (Result type renamed to Outcome to avoid Rust collision).

/// The outcome of applying a deduction operator.
///
/// An operator either:
/// - `Refined(h)`: produces a refined hypothesis `h`.
/// - `Abstain(a)`: declines to refine, returning an abstention reason `a`.
///
/// Implements DEF-MP-13 (Result type) and OQ-MP-02 (naming decision: `Outcome` not `Result`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome<H, A> {
    /// Operator successfully refined the hypothesis to `H`.
    Refined(H),
    /// Operator abstained with reason `A`.
    Abstain(A),
}

impl<H, A> Outcome<H, A> {
    /// Maps the refined hypothesis using a function.
    ///
    /// If the outcome is `Refined(h)`, applies `f` to `h` and returns `Refined(f(h))`.
    /// If the outcome is `Abstain(a)`, returns `Abstain(a)` unchanged.
    pub fn map<B, F>(self, f: F) -> Outcome<B, A>
    where
        F: FnOnce(H) -> B,
    {
        match self {
            Outcome::Refined(h) => Outcome::Refined(f(h)),
            Outcome::Abstain(a) => Outcome::Abstain(a),
        }
    }

    /// Chains operations: applies a function that returns an `Outcome` to the refined hypothesis.
    ///
    /// If the outcome is `Refined(h)`, applies `f` to `h` and flattens the result.
    /// If the outcome is `Abstain(a)`, returns `Abstain(a)` unchanged.
    pub fn and_then<B, F>(self, f: F) -> Outcome<B, A>
    where
        F: FnOnce(H) -> Outcome<B, A>,
    {
        match self {
            Outcome::Refined(h) => f(h),
            Outcome::Abstain(a) => Outcome::Abstain(a),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_refined() {
        let outcome: Outcome<u64, &str> = Outcome::Refined(5);
        let mapped = outcome.map(|x| x * 2);
        assert_eq!(mapped, Outcome::Refined(10));
    }

    #[test]
    fn test_map_abstain() {
        let outcome: Outcome<u64, &str> = Outcome::Abstain("insufficient data");
        let mapped = outcome.map(|x| x * 2);
        assert_eq!(mapped, Outcome::Abstain("insufficient data"));
    }

    #[test]
    fn test_and_then_refined() {
        let outcome: Outcome<u64, &str> = Outcome::Refined(5);
        let chained = outcome.and_then(|x| Outcome::Refined(x * 2));
        assert_eq!(chained, Outcome::Refined(10));
    }

    #[test]
    fn test_and_then_abstain() {
        let outcome: Outcome<u64, &str> = Outcome::Abstain("insufficient data");
        let chained = outcome.and_then(|_| Outcome::Refined(10));
        assert_eq!(chained, Outcome::Abstain("insufficient data"));
    }
}
