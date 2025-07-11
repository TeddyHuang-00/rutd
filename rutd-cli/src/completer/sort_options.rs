use std::ffi::OsStr;

use clap::builder::StyledStr;
use clap_complete::CompletionCandidate;
use rutd_core::{SortCriteria, SortOrder};
use strum::{EnumMessage, IntoEnumIterator};

use super::utils::validate_utf8_or_empty;

/// Get possible order options as completion candidates
pub fn complete_sort_options(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = validate_utf8_or_empty(current) else {
        return vec![];
    };

    // Determine if the current position is order or criterion
    // based on the length of the current string
    if current.len() % 2 == 0 {
        // Even length: Order
        SortOrder::iter()
            .map(|order| {
                CompletionCandidate::new(current.to_owned() + order.get_serializations()[0])
                    .help(order.get_documentation().map(StyledStr::from))
            })
            .collect()
    } else {
        // Odd length: Criteria
        SortCriteria::iter()
            .map(|option| {
                CompletionCandidate::new(current.to_owned() + option.get_serializations()[0])
                    .help(option.get_documentation().map(StyledStr::from))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    use rutd_core::SortCriteria;

    use super::*;

    #[test]
    fn test_complete_sort_options_empty() {
        // Test with empty input
        let current = OsString::from("");
        let completions = complete_sort_options(&current);

        // Should suggest sort orders (+ and -)
        assert_eq!(completions.len(), 2);

        // 检查是否有两个选项，但不检查具体内容
        // 由于 CompletionCandidate 没有实现 ToString 或 Display，
        // 我们只能检查数量，无法检查具体内容
        assert_eq!(completions.len(), 2);
    }

    #[test]
    fn test_complete_sort_options_order() {
        // Test with a complete sort option, ready for a new order
        let current = OsString::from("+p");
        let completions = complete_sort_options(&current);

        // Should suggest sort orders again (+ and -)
        assert_eq!(completions.len(), 2);
    }

    #[test]
    fn test_complete_sort_options_criteria() {
        // Test with an order, ready for a criterion
        let current = OsString::from("+");
        let completions = complete_sort_options(&current);

        // Should suggest all criteria
        assert_eq!(completions.len(), SortCriteria::iter().count());
    }

    #[test]
    fn test_complete_sort_options_multiple() {
        // Test with multiple options already entered
        let current = OsString::from("+p-s");
        let completions = complete_sort_options(&current);

        // Should suggest sort orders again
        assert_eq!(completions.len(), 2);
    }

    #[test]
    fn test_complete_sort_options_invalid_input() {
        // Test with non-UTF8 input
        let current = OsString::from_vec(vec![0xff, 0xff]);
        let completions = complete_sort_options(&current);

        // Should return empty vector for invalid inputs
        assert!(completions.is_empty());
    }
}
