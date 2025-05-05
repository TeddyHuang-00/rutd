use std::ffi::OsStr;

use clap::builder::StyledStr;
use clap_complete::CompletionCandidate;
use rutd_core::{SortCriteria, SortOrder};
use strum::{EnumMessage, IntoEnumIterator};

/// Get possible order options as completion candidates
pub fn complete_sort_options(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
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
    // TODO: Add tests for the completion candidates
}
