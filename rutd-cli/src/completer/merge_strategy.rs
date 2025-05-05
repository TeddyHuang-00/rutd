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
