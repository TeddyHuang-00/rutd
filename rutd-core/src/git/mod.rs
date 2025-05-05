pub mod repo;

use std::fmt;

use strum::{AsRefStr, EnumIter, EnumMessage, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, EnumMessage, EnumIter, AsRefStr)]
pub enum MergeStrategy {
    /// Do not automatically merge
    #[strum(
        serialize = "n",
        serialize = "none",
        message = "Do not automatically merge"
    )]
    None,
    /// Prefer local version
    #[strum(serialize = "l", serialize = "local", message = "Prefer local version")]
    Local,
    /// Prefer remote version
    #[strum(
        serialize = "r",
        serialize = "remote",
        message = "Prefer remote version"
    )]
    Remote,
}

impl Default for MergeStrategy {
    fn default() -> Self {
        Self::None
    }
}

impl fmt::Display for MergeStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Local => write!(f, "Local"),
            Self::Remote => write!(f, "Remote"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_strategy_default() {
        let strategy = MergeStrategy::default();
        assert_eq!(strategy, MergeStrategy::None);
    }

    #[test]
    fn test_merge_strategy_display() {
        assert_eq!(MergeStrategy::None.to_string(), "None");
        assert_eq!(MergeStrategy::Local.to_string(), "Local");
        assert_eq!(MergeStrategy::Remote.to_string(), "Remote");
    }

    #[test]
    fn test_merge_strategy_equality() {
        assert_eq!(MergeStrategy::None, MergeStrategy::None);
        assert_eq!(MergeStrategy::Local, MergeStrategy::Local);
        assert_eq!(MergeStrategy::Remote, MergeStrategy::Remote);

        assert_ne!(MergeStrategy::None, MergeStrategy::Local);
        assert_ne!(MergeStrategy::None, MergeStrategy::Remote);
        assert_ne!(MergeStrategy::Local, MergeStrategy::Remote);
    }

    #[test]
    fn test_merge_strategy_copy() {
        let strategy1 = MergeStrategy::Local;
        let strategy2 = strategy1;

        // After copying, both variables should refer to the same value
        assert_eq!(strategy1, strategy2);

        // And modifying one shouldn't affect the other (this test is somewhat redundant
        // since Copy types are implicitly Clone and create independent values)
        let strategy3 = MergeStrategy::Remote;
        assert_ne!(strategy2, strategy3);
    }

    #[test]
    fn test_merge_strategy_clone() {
        let strategy1 = MergeStrategy::Local;
        let strategy2 = strategy1;

        // The cloned value should equal the original
        assert_eq!(strategy1, strategy2);
    }

    #[test]
    fn test_merge_strategy_debug() {
        // Test that debug formatting works
        let debug_str = format!("{:?}", MergeStrategy::Local);
        assert!(debug_str.contains("Local"));
    }
}
