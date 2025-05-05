use std::ffi::OsStr;

use clap_complete::CompletionCandidate;
use rutd_core::MergeStrategy;
use strum::{EnumMessage, IntoEnumIterator};

pub fn complete_merge_strategy(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    // Get the merge strategies from enum
    MergeStrategy::iter()
        .flat_map(|strategy| strategy.get_serializations())
        .filter_map(|strategy| {
            // Check if the strategy starts with the current prefix
            strategy
                .starts_with(current)
                .then_some(CompletionCandidate::new(strategy))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    use rutd_core::MergeStrategy;
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn test_complete_merge_strategy_empty() {
        // Test with empty input
        let current = OsString::from("");
        let completions = complete_merge_strategy(&current);

        // Each MergeStrategy enum has multiple serializations
        // e.g., None has "n" and "none", Local has "l" and "local", etc.
        // So we can't directly compare with MergeStrategy::iter().count()
        assert!(!completions.is_empty());
        // At least there should be the serializations for all strategies
        assert_eq!(
            completions.len(),
            MergeStrategy::iter()
                .flat_map(|strategy| strategy.get_serializations())
                .count()
        );
    }

    #[test]
    fn test_complete_merge_strategy_prefix() {
        // Test with a prefix that should match 'local' and 'l'
        let current = OsString::from("l");
        let completions = complete_merge_strategy(&current);

        // Should match at least the short (l) and long (local) forms
        assert!(!completions.is_empty());

        // Test with a prefix that should match nothing with 'e'
        let current = OsString::from("e");
        let completions = complete_merge_strategy(&current);

        // Should suggest 0 items - no strategy starts with 'e'
        assert_eq!(completions.len(), 0);

        // Test with a prefix that should match 'remote' and 'r'
        let current = OsString::from("r");
        let completions = complete_merge_strategy(&current);

        // Should match at least the short (r) and long (remote) forms
        assert!(!completions.is_empty());
    }

    #[test]
    fn test_complete_merge_strategy_full_match() {
        // Test with a full strategy name
        let current = OsString::from("local");
        let completions = complete_merge_strategy(&current);

        // Should only suggest 'local'
        assert_eq!(completions.len(), 1);
    }

    #[test]
    fn test_complete_merge_strategy_no_match() {
        // Test with a prefix that shouldn't match any strategy
        let current = OsString::from("xyz");
        let completions = complete_merge_strategy(&current);

        // Should suggest 0 items
        assert!(completions.is_empty());
    }

    #[test]
    fn test_complete_merge_strategy_invalid_input() {
        // Test with non-UTF8 input
        let current = OsString::from_vec(vec![0xff, 0xff]);
        let completions = complete_merge_strategy(&current);

        // Should return empty vector for invalid inputs
        assert!(completions.is_empty());
    }
}
