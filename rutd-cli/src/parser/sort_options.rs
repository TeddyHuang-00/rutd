use std::str::FromStr;

use anyhow::{Context, Result};
use rutd_core::{SortCriteria, SortOptions, SortOrder};

// Parse sorting options from string for clap
pub fn parse_sort_options(sort_str: &str) -> Result<SortOptions, anyhow::Error> {
    // Every two characters represent a sort option
    let parts = sort_str
        .chars()
        .collect::<Vec<_>>()
        .chunks(2)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>();

    // Parsing empty string
    if parts.is_empty() {
        anyhow::bail!("Empty sort options string")
    }
    // Parsing invalid string
    if parts.last().unwrap().len() % 2 != 0 {
        anyhow::bail!("Invalid sort options string: {}", sort_str)
    }

    // Parsing sort options
    let mut sort_options = SortOptions::new();
    for part in parts {
        // Example: "+p", "-s", "+t"
        let (order, criterion) = part.split_at(1);
        let order =
            SortOrder::from_str(order).context(format!("Invalid sort options: {sort_str}"))?;
        let criterion = SortCriteria::from_str(criterion)
            .context(format!("Invalid sort options: {sort_str}"))?;
        sort_options.add_criterion(criterion, order);
    }
    Ok(sort_options)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_sort_options() {
        // Test valid sort options
        let result = parse_sort_options("+p-s");
        assert!(result.is_ok());
        let sort_options = result.unwrap();
        assert_eq!(sort_options.criteria().len(), 2);
        assert_eq!(sort_options.criteria()[0].0, SortCriteria::Priority);
        assert_eq!(sort_options.criteria()[1].0, SortCriteria::Scope);

        // Test invalid sort options
        let result = parse_sort_options("invalid");
        assert!(result.is_err());
    }
}
