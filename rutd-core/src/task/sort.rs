use chrono::{DateTime, Local};
use strum::{EnumIter, EnumMessage, EnumString};

use super::Task;

/// Specifies the order of sorting (ascending or descending)
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, EnumIter, EnumMessage)]
pub enum SortOrder {
    /// Sort in ascending order
    #[strum(serialize = "+")]
    Ascending,
    /// Sort in descending order
    #[strum(serialize = "-")]
    Descending,
}

/// Criteria by which tasks can be sorted
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, EnumIter, EnumMessage)]
pub enum SortCriteria {
    /// Sort by priority (Urgent → High → Normal → Low)
    #[strum(serialize = "p")]
    Priority,
    /// Sort by scope (project name)
    #[strum(serialize = "s")]
    Scope,
    /// Sort by task type
    #[strum(serialize = "t")]
    Type,
    /// Sort by task status (Todo → Done → Aborted)
    #[strum(serialize = "S")]
    Status,
    /// Sort by creation time
    #[strum(serialize = "c")]
    CreationTime,
    /// Sort by last update time
    #[strum(serialize = "u")]
    UpdateTime,
    /// Sort by completion time
    #[strum(serialize = "C")]
    CompletionTime,
    /// Sort by time spent on task
    #[strum(serialize = "T")]
    TimeSpent,
}

/// Configuration for sorting tasks
#[derive(Debug, Clone)]
pub struct SortOptions {
    /// List of criteria to sort by, in order of precedence
    criteria: Vec<(SortCriteria, SortOrder)>,
}

impl SortOptions {
    /// Create a new empty sort options
    pub fn new() -> Self {
        Self {
            criteria: Vec::new(),
        }
    }

    /// Add a sort criterion with specified order
    pub fn add_criterion(&mut self, criterion: SortCriteria, order: SortOrder) -> &mut Self {
        self.criteria.push((criterion, order));
        self
    }

    /// Check if there are any sort criteria specified
    pub fn is_empty(&self) -> bool {
        self.criteria.is_empty()
    }

    /// Get the list of sort criteria
    pub fn criteria(&self) -> &[(SortCriteria, SortOrder)] {
        &self.criteria
    }
}

impl Default for SortOptions {
    /// Get the default sort options (status, priority, scope, creation time)
    fn default() -> Self {
        let mut options = SortOptions::new();
        options
            .add_criterion(SortCriteria::Status, SortOrder::Descending)
            .add_criterion(SortCriteria::Priority, SortOrder::Descending)
            .add_criterion(SortCriteria::Scope, SortOrder::Ascending)
            .add_criterion(SortCriteria::CreationTime, SortOrder::Descending);
        options
    }
}

/// Sort tasks based on provided sort options
pub fn sort_tasks(tasks: &mut [Task], options: &SortOptions) {
    // If no sorting criteria specified, keep the order as is
    if options.is_empty() {
        return;
    }

    tasks.sort_by(|a, b| {
        for (criterion, order) in options.criteria() {
            let comparison = compare_tasks(a, b, *criterion);
            let ordering = match order {
                SortOrder::Ascending => comparison,
                SortOrder::Descending => comparison.reverse(),
            };

            if ordering.is_ne() {
                return ordering;
            }
        }
        std::cmp::Ordering::Equal
    });
}

// Helper function to compare two tasks by a single criterion
fn compare_tasks(a: &Task, b: &Task, criterion: SortCriteria) -> std::cmp::Ordering {
    match criterion {
        SortCriteria::Priority => a.priority.cmp(&b.priority),
        SortCriteria::Scope => compare_option_string(&a.scope, &b.scope),
        SortCriteria::Type => compare_option_string(&a.task_type, &b.task_type),
        SortCriteria::Status => a.status.cmp(&b.status),
        SortCriteria::CreationTime => compare_times(&a.created_at, &b.created_at),
        SortCriteria::UpdateTime => compare_option_times(&a.updated_at, &b.updated_at),
        SortCriteria::CompletionTime => compare_option_times(&a.completed_at, &b.completed_at),
        SortCriteria::TimeSpent => compare_option_numbers(&a.time_spent, &b.time_spent),
    }
}

// Helper function to compare optional strings
fn compare_option_string(a: &Option<String>, b: &Option<String>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(a_val), Some(b_val)) => a_val.cmp(b_val),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

// Helper function to compare times in RFC3339 format
fn compare_times(a: &str, b: &str) -> std::cmp::Ordering {
    let a_time = parse_time(a);
    let b_time = parse_time(b);
    match (a_time, b_time) {
        (Ok(a_dt), Ok(b_dt)) => a_dt.cmp(&b_dt),
        _ => std::cmp::Ordering::Equal, // Fall back to equality for parse errors
    }
}

// Helper function to compare optional times
fn compare_option_times(a: &Option<String>, b: &Option<String>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(a_val), Some(b_val)) => compare_times(a_val, b_val),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

// Helper function to compare optional numbers
fn compare_option_numbers<T: Ord>(a: &Option<T>, b: &Option<T>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(a_val), Some(b_val)) => a_val.cmp(b_val),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

// Helper function to parse time from RFC3339 format
fn parse_time(time_str: &str) -> Result<DateTime<Local>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(time_str).map(|dt| dt.with_timezone(&Local))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Priority, TaskStatus};

    struct EasyTask<'a>(
        &'a str,
        Priority,
        Option<&'a str>,
        Option<&'a str>,
        TaskStatus,
        &'a str,
        Option<&'a str>,
        Option<&'a str>,
        Option<u64>,
    );

    // Helper function to create a test task with specific properties
    fn create_test_task(task: EasyTask) -> Task {
        let EasyTask(
            id,
            priority,
            scope,
            task_type,
            status,
            created_at,
            updated_at,
            completed_at,
            time_spent,
        ) = task;
        Task {
            id: id.to_string(),
            description: format!("Task {}", id),
            priority,
            scope: scope.map(|s| s.to_string()),
            task_type: task_type.map(|t| t.to_string()),
            status,
            created_at: created_at.to_string(),
            updated_at: updated_at.map(|s| s.to_string()),
            completed_at: completed_at.map(|s| s.to_string()),
            time_spent,
        }
    }

    // Generate a standard set of diverse test tasks for all tests
    fn create_test_tasks() -> Vec<Task> {
        // Create a diverse set of tasks with identifiable values for all attributes
        vec![
            // Task 1: Normal priority, project-b, feature, Todo, medium creation time, medium
            // update, no completion, medium time
            create_test_task(EasyTask(
                "1",
                Priority::Normal,
                Some("project-b"),
                Some("feature"),
                TaskStatus::Todo,
                "2023-02-15T12:00:00+00:00",
                Some("2023-02-20T12:00:00+00:00"),
                None,
                Some(60),
            )),
            // Task 2: High priority, project-a, bug, Todo, oldest creation time, oldest update, no
            // completion, low time
            create_test_task(EasyTask(
                "2",
                Priority::High,
                Some("project-a"),
                Some("bug"),
                TaskStatus::Todo,
                "2023-01-10T12:00:00+00:00",
                Some("2023-01-15T12:00:00+00:00"),
                None,
                Some(30),
            )),
            // Task 3: Urgent priority, no scope, no type, Aborted, newest creation time, no
            // update, oldest completion, no time
            create_test_task(EasyTask(
                "3",
                Priority::Urgent,
                None,
                None,
                TaskStatus::Aborted,
                "2023-03-25T12:00:00+00:00",
                None,
                Some("2023-04-01T12:00:00+00:00"),
                None,
            )),
            // Task 4: Low priority, project-c, document, Done, second newest creation, newest
            // update, newest completion, highest time
            create_test_task(EasyTask(
                "4",
                Priority::Low,
                Some("project-c"),
                Some("document"),
                TaskStatus::Done,
                "2023-03-10T12:00:00+00:00",
                Some("2023-03-30T12:00:00+00:00"),
                Some("2023-04-05T12:00:00+00:00"),
                Some(120),
            )),
        ]
    }

    #[test]
    fn test_sort_by_priority() {
        // Use the shared test task set
        let mut tasks = create_test_tasks();

        // Create sort options for priority descending (Urgent first)
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::Priority, SortOrder::Descending);

        // Sort the tasks
        sort_tasks(&mut tasks, &options);

        // Verify the order: Urgent (3) -> High (2) -> Normal (1) -> Low (4)
        assert_eq!(tasks[0].id, "3");
        assert_eq!(tasks[1].id, "2");
        assert_eq!(tasks[2].id, "1");
        assert_eq!(tasks[3].id, "4");

        // Test ascending order
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::Priority, SortOrder::Ascending);
        sort_tasks(&mut tasks, &options);

        // Verify the order: Low (4) -> Normal (1) -> High (2) -> Urgent (3)
        assert_eq!(tasks[0].id, "4");
        assert_eq!(tasks[1].id, "1");
        assert_eq!(tasks[2].id, "2");
        assert_eq!(tasks[3].id, "3");
    }

    #[test]
    fn test_sort_by_scope() {
        // Use the shared test task set
        let mut tasks = create_test_tasks();

        // Create sort options for scope ascending (alphabetical)
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::Scope, SortOrder::Ascending);

        // Sort the tasks
        sort_tasks(&mut tasks, &options);

        // Verify the order: project-a (2) -> project-b (1) -> project-c (4) -> None (3)
        assert_eq!(tasks[0].id, "2");
        assert_eq!(tasks[1].id, "1");
        assert_eq!(tasks[2].id, "4");
        assert_eq!(tasks[3].id, "3");

        // Test descending order
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::Scope, SortOrder::Descending);
        sort_tasks(&mut tasks, &options);

        // Verify the order: None (3) -> project-c (4) -> project-b (1) -> project-a (2)
        assert_eq!(tasks[0].id, "3");
        assert_eq!(tasks[1].id, "4");
        assert_eq!(tasks[2].id, "1");
        assert_eq!(tasks[3].id, "2");
    }

    #[test]
    fn test_sort_by_creation_time() {
        // Use the shared test task set
        let mut tasks = create_test_tasks();

        // Create sort options for creation time ascending (oldest first)
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::CreationTime, SortOrder::Ascending);

        // Sort the tasks
        sort_tasks(&mut tasks, &options);

        // Verify the order: Jan (2) -> Feb (1) -> March (4) -> Late March (3)
        assert_eq!(tasks[0].id, "2");
        assert_eq!(tasks[1].id, "1");
        assert_eq!(tasks[2].id, "4");
        assert_eq!(tasks[3].id, "3");

        // Test descending order (newest first)
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::CreationTime, SortOrder::Descending);
        sort_tasks(&mut tasks, &options);

        // Verify the order: Late March (3) -> March (4) -> Feb (1) -> Jan (2)
        assert_eq!(tasks[0].id, "3");
        assert_eq!(tasks[1].id, "4");
        assert_eq!(tasks[2].id, "1");
        assert_eq!(tasks[3].id, "2");
    }

    #[test]
    fn test_multi_criteria_sort() {
        // Use the shared test task set
        let mut tasks = create_test_tasks();

        // Sort by priority (desc), then scope (asc)
        let mut options = SortOptions::new();
        options
            .add_criterion(SortCriteria::Priority, SortOrder::Descending)
            .add_criterion(SortCriteria::Scope, SortOrder::Ascending);

        // Sort the tasks
        sort_tasks(&mut tasks, &options);

        // Expected order:
        // 1. Urgent, no scope (3)
        // 2. High, project-a (2)
        // 3. Normal, project-b (1)
        // 4. Low, project-c (4)
        assert_eq!(tasks[0].id, "3");
        assert_eq!(tasks[1].id, "2");
        assert_eq!(tasks[2].id, "1");
        assert_eq!(tasks[3].id, "4");

        // Sort by status (desc), then priority (desc)
        let mut options = SortOptions::new();
        options
            .add_criterion(SortCriteria::Status, SortOrder::Descending)
            .add_criterion(SortCriteria::Priority, SortOrder::Descending);

        // Sort the tasks
        sort_tasks(&mut tasks, &options);

        // Expected order:
        // 1. Todo, High (2)
        // 2. Todo, Normal (1)
        // 3. Done, Low (4)
        // 4. Aborted, Urgent (3)
        assert_eq!(tasks[0].id, "2");
        assert_eq!(tasks[1].id, "1");
        assert_eq!(tasks[2].id, "4");
        assert_eq!(tasks[3].id, "3");
    }

    #[test]
    fn test_default_sort_options() {
        let default_options = SortOptions::default();
        let criteria = default_options.criteria();

        assert_eq!(criteria.len(), 4);
        assert_eq!(criteria[0].0, SortCriteria::Status);
        assert_eq!(criteria[0].1, SortOrder::Descending);
        assert_eq!(criteria[1].0, SortCriteria::Priority);
        assert_eq!(criteria[1].1, SortOrder::Descending);
        assert_eq!(criteria[2].0, SortCriteria::Scope);
        assert_eq!(criteria[2].1, SortOrder::Ascending);
        assert_eq!(criteria[3].0, SortCriteria::CreationTime);
        assert_eq!(criteria[3].1, SortOrder::Descending);
    }

    #[test]
    fn test_default_sort_options_uses_default() {
        // Use the shared test task set
        let mut tasks = create_test_tasks();

        // Create empty sort options
        let options = SortOptions::default();

        // Sort the tasks
        sort_tasks(&mut tasks, &options);

        // With default sort (Status desc, Priority desc, Scope asc, CreationTime desc):
        // 1. Todo, High, project-a (2)
        // 2. Todo, Normal, project-b (1)
        // 3. Done, Low, project-c (4)
        // 4. Aborted, Urgent, no scope (3)
        assert_eq!(tasks[0].id, "2");
        assert_eq!(tasks[1].id, "1");
        assert_eq!(tasks[2].id, "4");
        assert_eq!(tasks[3].id, "3");
    }

    #[test]
    fn test_sort_by_type() {
        // Use the shared test task set
        let mut tasks = create_test_tasks();

        // Create sort options for type ascending (alphabetical)
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::Type, SortOrder::Ascending);

        // Sort the tasks
        sort_tasks(&mut tasks, &options);

        // Verify the order: bug (2) -> document (4) -> feature (1) -> None (3)
        assert_eq!(tasks[0].id, "2");
        assert_eq!(tasks[1].id, "4");
        assert_eq!(tasks[2].id, "1");
        assert_eq!(tasks[3].id, "3");

        // Test descending order
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::Type, SortOrder::Descending);
        sort_tasks(&mut tasks, &options);

        // Verify the order: None (3) -> feature (1) -> document (4) -> bug (2)
        assert_eq!(tasks[0].id, "3");
        assert_eq!(tasks[1].id, "1");
        assert_eq!(tasks[2].id, "4");
        assert_eq!(tasks[3].id, "2");
    }

    #[test]
    fn test_sort_by_status() {
        // Use the shared test task set
        let mut tasks = create_test_tasks();

        // Create sort options for status descending (Todo -> Done -> Aborted)
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::Status, SortOrder::Descending);

        // Sort the tasks
        sort_tasks(&mut tasks, &options);

        // Verify the order: Todo (1, 2) -> Done (4) -> Aborted (3)
        // For equal status, original order is preserved (1 before 2)
        assert_eq!(tasks[0].id, "1");
        assert_eq!(tasks[1].id, "2");
        assert_eq!(tasks[2].id, "4");
        assert_eq!(tasks[3].id, "3");

        // Test ascending order
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::Status, SortOrder::Ascending);
        sort_tasks(&mut tasks, &options);

        // Verify the order: Aborted (3) -> Done (4) -> Todo (1, 2)
        assert_eq!(tasks[0].id, "3");
        assert_eq!(tasks[1].id, "4");
        assert_eq!(tasks[2].id, "1");
        assert_eq!(tasks[3].id, "2");
    }

    #[test]
    fn test_sort_by_update_time() {
        // Use the shared test task set
        let mut tasks = create_test_tasks();

        // Create sort options for update time ascending (oldest first)
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::UpdateTime, SortOrder::Ascending);

        // Sort the tasks
        sort_tasks(&mut tasks, &options);

        // Verify the order: Jan (2) -> Feb (1) -> Mar (4) -> None (3)
        assert_eq!(tasks[0].id, "2");
        assert_eq!(tasks[1].id, "1");
        assert_eq!(tasks[2].id, "4");
        assert_eq!(tasks[3].id, "3");

        // Test descending order (newest first)
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::UpdateTime, SortOrder::Descending);
        sort_tasks(&mut tasks, &options);

        // Verify the order: None (3) -> Mar (4) -> Feb (1) -> Jan (2)
        assert_eq!(tasks[0].id, "3");
        assert_eq!(tasks[1].id, "4");
        assert_eq!(tasks[2].id, "1");
        assert_eq!(tasks[3].id, "2");
    }

    #[test]
    fn test_sort_by_completion_time() {
        // Use the shared test task set
        let mut tasks = create_test_tasks();

        // Create sort options for completion time ascending (oldest first)
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::CompletionTime, SortOrder::Ascending);

        // Sort the tasks
        sort_tasks(&mut tasks, &options);

        // Verify the order: April 1 (3) -> April 5 (4) -> No completion (1, 2)
        assert_eq!(tasks[0].id, "3");
        assert_eq!(tasks[1].id, "4");
        assert_eq!(tasks[2].id, "1");
        assert_eq!(tasks[3].id, "2");

        // Test descending order (newest first)
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::CompletionTime, SortOrder::Descending);
        sort_tasks(&mut tasks, &options);

        // Verify the order: No completion (1, 2) -> April 5 (4) -> April 1 (3)
        assert_eq!(tasks[0].id, "1");
        assert_eq!(tasks[1].id, "2");
        assert_eq!(tasks[2].id, "4");
        assert_eq!(tasks[3].id, "3");
    }

    #[test]
    fn test_sort_by_time_spent() {
        // Use the shared test task set
        let mut tasks = create_test_tasks();

        // Create sort options for time spent ascending (shortest first)
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::TimeSpent, SortOrder::Ascending);

        // Sort the tasks
        sort_tasks(&mut tasks, &options);

        // Verify the order: 30 seconds (2) -> 60 seconds (1) -> 120 seconds (4) -> None
        // (3)
        assert_eq!(tasks[0].id, "2");
        assert_eq!(tasks[1].id, "1");
        assert_eq!(tasks[2].id, "4");
        assert_eq!(tasks[3].id, "3");

        // Test descending order (longest first)
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::TimeSpent, SortOrder::Descending);
        sort_tasks(&mut tasks, &options);

        // Verify the order: None (3) -> 120 seconds (4) -> 60 seconds (1) -> 30 seconds
        // (2)
        assert_eq!(tasks[0].id, "3");
        assert_eq!(tasks[1].id, "4");
        assert_eq!(tasks[2].id, "1");
        assert_eq!(tasks[3].id, "2");
    }

    #[test]
    fn test_empty_sort_options() {
        // Use the shared test task set
        let mut tasks = create_test_tasks();

        // Create empty sort options
        let options = SortOptions::new();
        assert!(options.is_empty());

        // Make a copy of the original order
        let original_order = tasks.iter().map(|t| t.id.clone()).collect::<Vec<_>>();

        // Sort the tasks
        sort_tasks(&mut tasks, &options);

        // Verify that order was not changed
        let sorted_order = tasks.iter().map(|t| t.id.clone()).collect::<Vec<_>>();
        assert_eq!(original_order, sorted_order);

        // Confirm the original order is maintained
        assert_eq!(original_order, vec!["1", "2", "3", "4"]);
    }

    #[test]
    fn test_sort_options_methods() {
        // Test empty sort options
        let options = SortOptions::new();
        assert!(options.is_empty());
        assert!(options.criteria().is_empty());

        // Test adding criteria
        let mut options = SortOptions::new();
        options.add_criterion(SortCriteria::Priority, SortOrder::Descending);
        assert!(!options.is_empty());
        assert_eq!(options.criteria().len(), 1);
        assert_eq!(options.criteria()[0].0, SortCriteria::Priority);
        assert_eq!(options.criteria()[0].1, SortOrder::Descending);

        // Test method chaining
        let mut options = SortOptions::new();
        options
            .add_criterion(SortCriteria::Priority, SortOrder::Descending)
            .add_criterion(SortCriteria::Scope, SortOrder::Ascending);
        assert_eq!(options.criteria().len(), 2);
    }
}
