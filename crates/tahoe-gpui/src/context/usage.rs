//! Token usage types.

/// Token usage breakdown for an AI model request.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Usage {
    /// Number of input tokens.
    pub input: u64,
    /// Number of output tokens.
    pub output: u64,
    /// Number of reasoning tokens (if applicable).
    pub reasoning: u64,
    /// Number of cached tokens (if applicable).
    pub cached: u64,
}

impl Usage {
    /// Creates a new [`Usage`] with the given token counts.
    pub fn new(input: u64, output: u64, reasoning: u64, cached: u64) -> Self {
        Self {
            input,
            output,
            reasoning,
            cached,
        }
    }

    /// Returns the total number of unique tokens used.
    ///
    /// Note: `cached` tokens are a subset of `input` tokens (served from provider
    /// cache rather than freshly processed), so they are not added separately.
    pub fn total(&self) -> u64 {
        self.input + self.output + self.reasoning
    }
}

impl std::ops::Add for Usage {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            input: self.input + other.input,
            output: self.output + other.output,
            reasoning: self.reasoning + other.reasoning,
            cached: self.cached + other.cached,
        }
    }
}

impl std::ops::AddAssign for Usage {
    fn add_assign(&mut self, other: Self) {
        self.input += other.input;
        self.output += other.output;
        self.reasoning += other.reasoning;
        self.cached += other.cached;
    }
}

#[cfg(test)]
mod tests {
    use super::Usage;
    use core::prelude::v1::test;

    #[test]
    fn new_creates_usage() {
        let u = Usage::new(100, 50, 10, 5);
        assert_eq!(u.input, 100);
        assert_eq!(u.output, 50);
        assert_eq!(u.reasoning, 10);
        assert_eq!(u.cached, 5);
    }

    #[test]
    fn total_sums_input_output_reasoning() {
        // cached is a subset of input, not added separately
        let u = Usage::new(100, 50, 10, 5);
        assert_eq!(u.total(), 160);
    }

    #[test]
    fn total_zero() {
        let u = Usage::default();
        assert_eq!(u.total(), 0);
    }

    #[test]
    fn add_combines_usages() {
        let a = Usage::new(10, 20, 5, 0);
        let b = Usage::new(30, 10, 0, 5);
        let c = a + b;
        assert_eq!(c.input, 40);
        assert_eq!(c.output, 30);
        assert_eq!(c.reasoning, 5);
        assert_eq!(c.cached, 5);
    }

    #[test]
    fn default_is_zero() {
        let u = Usage::default();
        assert_eq!(u.input, 0);
        assert_eq!(u.output, 0);
        assert_eq!(u.reasoning, 0);
        assert_eq!(u.cached, 0);
    }

    #[test]
    fn equality() {
        let a = Usage::new(1, 2, 3, 4);
        let b = Usage::new(1, 2, 3, 4);
        assert_eq!(a, b);
    }

    #[test]
    fn inequality() {
        let a = Usage::new(1, 2, 3, 4);
        let b = Usage::new(1, 2, 3, 5);
        assert_ne!(a, b);
    }

    #[test]
    fn clone_produces_equal() {
        let a = Usage::new(100, 200, 50, 10);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn add_assign_accumulates() {
        let mut a = Usage::new(10, 20, 5, 0);
        a += Usage::new(30, 10, 0, 5);
        assert_eq!(a, Usage::new(40, 30, 5, 5));
    }

    #[test]
    fn add_assign_with_zero() {
        let mut a = Usage::new(100, 200, 50, 10);
        a += Usage::default();
        assert_eq!(a, Usage::new(100, 200, 50, 10));
    }
}
