use chrono::{DateTime, Local};

use super::{Priority, TaskStatus};

/// Filter options for task queries
#[derive(Clone, Debug, Default)]
pub struct Filter {
    /// Filter by priority
    pub priority: Option<Priority>,

    /// Filter by scope (project name)
    pub task_scope: Option<String>,

    /// Filter by type
    pub task_type: Option<String>,

    /// Filter by status
    pub status: Option<TaskStatus>,

    /// Filter by creation date range
    pub creation_time: Option<DateRange>,

    /// Filter by last update date range
    pub update_time: Option<DateRange>,

    /// Filter by completion date range, including cancelled tasks
    pub completion_time: Option<DateRange>,

    /// Enable fuzzy matching for description
    pub fuzzy: Option<String>,
}

/// DateRange struct for robust date parsing
#[derive(Clone, Debug, Default)]
pub struct DateRange {
    /// Start date limit (None if no lower bound)
    pub from: Option<DateTime<Local>>,
    /// End date limit (None if no upper bound)
    pub to: Option<DateTime<Local>>,
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone};

    use super::*;
    use crate::task::{Priority, Task, TaskStatus};

    // Helper function to create a date at a specific year, month, day
    fn create_date(year: i32, month: u32, day: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap()
    }

    // Helper function to create a default filter
    fn create_default_filter() -> Filter {
        Filter::default()
    }

    struct EasyTask<'a>(
        &'a str,
        &'a str,
        Priority,
        Option<&'a str>,
        Option<&'a str>,
        TaskStatus,
        &'a str,
        Option<&'a str>,
        Option<&'a str>,
    );

    // Helper function to create a test task with specific properties
    fn create_test_task(task: EasyTask) -> Task {
        let EasyTask(
            id,
            description,
            priority,
            scope,
            task_type,
            status,
            created_at,
            updated_at,
            completed_at,
        ) = task;
        Task {
            id: id.to_string(),
            description: description.to_string(),
            priority,
            scope: scope.map(|s| s.to_string()),
            task_type: task_type.map(|t| t.to_string()),
            status,
            created_at: created_at.to_string(),
            updated_at: updated_at.map(|s| s.to_string()),
            completed_at: completed_at.map(|s| s.to_string()),
            time_spent: None,
        }
    }

    // Helper function to apply a filter to a list of tasks
    fn apply_filter(tasks: &[Task], filter: &Filter) -> Vec<Task> {
        tasks
            .iter()
            .filter(|task| {
                // Filter by priority
                if let Some(priority) = filter.priority {
                    if task.priority != priority {
                        return false;
                    }
                }

                // Filter by scope
                if let Some(ref scope) = filter.task_scope {
                    match &task.scope {
                        Some(task_scope) if task_scope == scope => {}
                        _ => return false,
                    }
                }

                // Filter by type
                if let Some(ref task_type) = filter.task_type {
                    match &task.task_type {
                        Some(tt) if tt == task_type => {}
                        _ => return false,
                    }
                }

                // Filter by status
                if let Some(status) = filter.status {
                    if task.status != status {
                        return false;
                    }
                }

                // Filter by creation time
                if let Some(ref date_range) = filter.creation_time {
                    let created_at = match DateTime::parse_from_rfc3339(&task.created_at) {
                        Ok(dt) => dt.with_timezone(&Local),
                        Err(_) => return false, // Skip tasks with invalid dates
                    };

                    if let Some(from) = date_range.from {
                        if created_at < from {
                            return false;
                        }
                    }

                    if let Some(to) = date_range.to {
                        if created_at >= to {
                            return false;
                        }
                    }
                }

                // Filter by update time
                if let Some(ref date_range) = filter.update_time {
                    match &task.updated_at {
                        Some(updated_at_str) => {
                            let updated_at = match DateTime::parse_from_rfc3339(updated_at_str) {
                                Ok(dt) => dt.with_timezone(&Local),
                                Err(_) => return false, // Skip tasks with invalid dates
                            };

                            if let Some(from) = date_range.from {
                                if updated_at < from {
                                    return false;
                                }
                            }

                            if let Some(to) = date_range.to {
                                if updated_at >= to {
                                    return false;
                                }
                            }
                        }
                        None => return false, // No update time, doesn't match filter
                    }
                }

                // Filter by completion time
                if let Some(ref date_range) = filter.completion_time {
                    match &task.completed_at {
                        Some(completed_at_str) => {
                            let completed_at = match DateTime::parse_from_rfc3339(completed_at_str)
                            {
                                Ok(dt) => dt.with_timezone(&Local),
                                Err(_) => return false, // Skip tasks with invalid dates
                            };

                            if let Some(from) = date_range.from {
                                if completed_at < from {
                                    return false;
                                }
                            }

                            if let Some(to) = date_range.to {
                                if completed_at >= to {
                                    return false;
                                }
                            }
                        }
                        None => return false, // No completion time, doesn't match filter
                    }
                }

                // Filter by fuzzy search
                if let Some(ref fuzzy) = filter.fuzzy {
                    if !task
                        .description
                        .to_lowercase()
                        .contains(&fuzzy.to_lowercase())
                    {
                        return false;
                    }
                }

                // Task passed all filters
                true
            })
            .cloned()
            .collect()
    }

    #[test]
    fn test_empty_filter_options() {
        let filter = Filter::default();

        assert!(filter.priority.is_none());
        assert!(filter.task_scope.is_none());
        assert!(filter.task_type.is_none());
        assert!(filter.status.is_none());
        assert!(filter.creation_time.is_none());
        assert!(filter.update_time.is_none());
        assert!(filter.completion_time.is_none());
        assert!(filter.fuzzy.is_none());
    }

    #[test]
    fn test_filter_by_priority() {
        // Create test tasks with different priorities
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Urgent task",
                Priority::Urgent,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "High priority task",
                Priority::High,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-3",
                "Normal priority task",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-4",
                "Low priority task",
                Priority::Low,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test filter by priority
        let mut filter = create_default_filter();

        filter.priority = Some(Priority::Urgent);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        filter.priority = Some(Priority::High);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-2");

        filter.priority = Some(Priority::Normal);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-3");

        filter.priority = Some(Priority::Low);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-4");
    }

    #[test]
    fn test_filter_by_scope() {
        // Create test tasks with different scopes
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Project A task",
                Priority::Normal,
                Some("project-a"),
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "Project B task",
                Priority::Normal,
                Some("project-b"),
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-3",
                "No scope task",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test filter by scope
        let mut filter = create_default_filter();

        filter.task_scope = Some("project-a".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        filter.task_scope = Some("project-b".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-2");

        filter.task_scope = Some("nonexistent".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_by_type() {
        // Create test tasks with different types
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Feature task",
                Priority::Normal,
                None,
                Some("feat"),
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "Bug task",
                Priority::Normal,
                None,
                Some("bug"),
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-3",
                "No type task",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test filter by type
        let mut filter = create_default_filter();

        filter.task_type = Some("feat".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        filter.task_type = Some("bug".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-2");

        filter.task_type = Some("nonexistent".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_by_status() {
        // Create test tasks with different statuses
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Todo task",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "Done task",
                Priority::Normal,
                None,
                None,
                TaskStatus::Done,
                "2023-01-01T12:00:00+00:00",
                None,
                Some("2023-01-02T12:00:00+00:00"),
            )),
            create_test_task(EasyTask(
                "task-3",
                "Aborted task",
                Priority::Normal,
                None,
                None,
                TaskStatus::Aborted,
                "2023-01-01T12:00:00+00:00",
                None,
                Some("2023-01-03T12:00:00+00:00"),
            )),
        ];

        // Test filter by status
        let mut filter = create_default_filter();

        filter.status = Some(TaskStatus::Todo);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        filter.status = Some(TaskStatus::Done);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-2");

        filter.status = Some(TaskStatus::Aborted);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-3");
    }

    #[test]
    fn test_filter_by_creation_time() {
        // Create test tasks with different creation times
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Created in January",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "Created in February",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-02-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-3",
                "Created in March",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-03-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test filter by creation time
        let mut filter = create_default_filter();

        // Only January
        filter.creation_time = Some(DateRange {
            from: Some(create_date(2023, 1, 1)),
            to: Some(create_date(2023, 2, 1)),
        });
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        // January through February
        filter.creation_time = Some(DateRange {
            from: Some(create_date(2023, 1, 1)),
            to: Some(create_date(2023, 3, 1)),
        });
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, "task-1");
        assert_eq!(filtered[1].id, "task-2");

        // All of 2023
        filter.creation_time = Some(DateRange {
            from: Some(create_date(2023, 1, 1)),
            to: Some(create_date(2024, 1, 1)),
        });
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_filter_by_update_time() {
        // Create test tasks with different update times
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Updated in January",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                Some("2023-01-15T12:00:00+00:00"),
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "Updated in February",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                Some("2023-02-15T12:00:00+00:00"),
                None,
            )),
            create_test_task(EasyTask(
                "task-3",
                "Never updated",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test filter by update time
        let mut filter = create_default_filter();

        // Only January
        filter.update_time = Some(DateRange {
            from: Some(create_date(2023, 1, 1)),
            to: Some(create_date(2023, 2, 1)),
        });
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        // January through February
        filter.update_time = Some(DateRange {
            from: Some(create_date(2023, 1, 1)),
            to: Some(create_date(2023, 3, 1)),
        });
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, "task-1");
        assert_eq!(filtered[1].id, "task-2");
    }

    #[test]
    fn test_filter_by_completion_time() {
        // Create test tasks with different completion times
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Completed in January",
                Priority::Normal,
                None,
                None,
                TaskStatus::Done,
                "2023-01-01T12:00:00+00:00",
                None,
                Some("2023-01-15T12:00:00+00:00"),
            )),
            create_test_task(EasyTask(
                "task-2",
                "Completed in February",
                Priority::Normal,
                None,
                None,
                TaskStatus::Done,
                "2023-01-01T12:00:00+00:00",
                None,
                Some("2023-02-15T12:00:00+00:00"),
            )),
            create_test_task(EasyTask(
                "task-3",
                "Not completed",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test filter by completion time
        let mut filter = create_default_filter();

        // Only January
        filter.completion_time = Some(DateRange {
            from: Some(create_date(2023, 1, 1)),
            to: Some(create_date(2023, 2, 1)),
        });
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        // January through February
        filter.completion_time = Some(DateRange {
            from: Some(create_date(2023, 1, 1)),
            to: Some(create_date(2023, 3, 1)),
        });
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, "task-1");
        assert_eq!(filtered[1].id, "task-2");
    }

    #[test]
    fn test_filter_by_fuzzy_description() {
        // Create test tasks with different descriptions
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Implement feature A",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "Fix bug in module B",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
            create_test_task(EasyTask(
                "task-3",
                "Write documentation",
                Priority::Normal,
                None,
                None,
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test filter by fuzzy description
        let mut filter = create_default_filter();

        filter.fuzzy = Some("feature".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        filter.fuzzy = Some("bug".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-2");

        filter.fuzzy = Some("doc".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-3");

        // Case insensitive
        filter.fuzzy = Some("FEATURE".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        // No match
        filter.fuzzy = Some("nonexistent".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_combine_multiple_filters() {
        // Create test tasks with various properties
        let tasks = vec![
            create_test_task(EasyTask(
                "task-1",
                "Urgent feature task",
                Priority::Urgent,
                Some("project-a"),
                Some("feat"),
                TaskStatus::Todo,
                "2023-01-01T12:00:00+00:00",
                Some("2023-01-15T12:00:00+00:00"),
                None,
            )),
            create_test_task(EasyTask(
                "task-2",
                "High priority bug fix",
                Priority::High,
                Some("project-a"),
                Some("bug"),
                TaskStatus::Done,
                "2023-01-01T12:00:00+00:00",
                Some("2023-02-15T12:00:00+00:00"),
                Some("2023-02-20T12:00:00+00:00"),
            )),
            create_test_task(EasyTask(
                "task-3",
                "Normal priority documentation",
                Priority::Normal,
                Some("project-b"),
                Some("docs"),
                TaskStatus::Todo,
                "2023-02-01T12:00:00+00:00",
                None,
                None,
            )),
        ];

        // Test combining multiple filters
        let mut filter = create_default_filter();

        // Test priority + scope
        filter.priority = Some(Priority::Urgent);
        filter.task_scope = Some("project-a".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-1");

        // Test priority + scope + type
        filter.priority = Some(Priority::High);
        filter.task_scope = Some("project-a".to_string());
        filter.task_type = Some("bug".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-2");

        // Test scope + fuzzy
        filter = Filter::default();
        filter.task_scope = Some("project-b".to_string());
        filter.fuzzy = Some("document".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "task-3");

        // Test filter that should match no tasks
        filter = Filter::default();
        filter.priority = Some(Priority::Low);
        filter.task_scope = Some("project-a".to_string());
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 0);
    }
}
