use std::collections::{HashSet, VecDeque};

use super::error::DomainError;

// --- Input size limits ---
pub const MAX_TITLE_LEN: usize = 500;
pub const MAX_LONG_TEXT_LEN: usize = 50_000;
pub const MAX_TAG_LEN: usize = 100;
pub const MAX_TAGS_COUNT: usize = 20;
pub const MAX_SHORT_TEXT_LEN: usize = 500;
pub const MAX_ITEMS_COUNT: usize = 50;
pub const MAX_SESSION_ID_LEN: usize = 100;
pub const MAX_USERNAME_LEN: usize = 100;
pub const MAX_DISPLAY_NAME_LEN: usize = 100;
pub const MAX_PROJECT_NAME_LEN: usize = 500;
pub const MAX_PROJECT_DESCRIPTION_LEN: usize = 50_000;

pub fn validate_string_length(field: &str, value: &str, max_len: usize) -> Result<(), DomainError> {
    let len = value.chars().count();
    if len > max_len {
        return Err(DomainError::ValidationError {
            field: field.to_string(),
            message: format!(
                "{field} exceeds maximum length of {max_len} characters (got {len})"
            ),
        });
    }
    Ok(())
}

pub fn validate_optional_string_length(
    field: &str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), DomainError> {
    if let Some(v) = value {
        validate_string_length(field, v, max_len)?;
    }
    Ok(())
}

pub fn validate_optional_nullable_string_length(
    field: &str,
    value: &Option<Option<String>>,
    max_len: usize,
) -> Result<(), DomainError> {
    if let Some(Some(v)) = value {
        validate_string_length(field, v, max_len)?;
    }
    Ok(())
}

pub fn validate_string_vec_items(
    field: &str,
    items: &[String],
    max_item_len: usize,
    max_count: usize,
) -> Result<(), DomainError> {
    if items.len() > max_count {
        return Err(DomainError::ValidationError {
            field: field.to_string(),
            message: format!(
                "{field} exceeds maximum of {max_count} items (got {})",
                items.len()
            ),
        });
    }
    for (i, item) in items.iter().enumerate() {
        let len = item.chars().count();
        if len > max_item_len {
            return Err(DomainError::ValidationError {
                field: format!("{field}[{i}]"),
                message: format!(
                    "{field}[{i}] exceeds maximum length of {max_item_len} characters (got {len})"
                ),
            });
        }
    }
    Ok(())
}

/// Check if adding dep_id as a dependency of task_id would create a cycle.
/// Performs BFS from dep_id following its dependencies; if task_id is reachable, it's a cycle.
///
/// `get_dependencies` returns the dependency IDs for a given task.
pub fn has_cycle<F>(task_id: i64, dep_id: i64, get_dependencies: F) -> bool
where
    F: Fn(i64) -> Vec<i64>,
{
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(dep_id);
    visited.insert(dep_id);

    while let Some(current) = queue.pop_front() {
        for d in get_dependencies(current) {
            if d == task_id {
                return true;
            }
            if visited.insert(d) {
                queue.push_back(d);
            }
        }
    }
    false
}

/// Async version of cycle detection for use in the application layer.
pub async fn has_cycle_async<F, Fut>(task_id: i64, dep_id: i64, get_dependencies: F) -> bool
where
    F: Fn(i64) -> Fut,
    Fut: std::future::Future<Output = Vec<i64>>,
{
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(dep_id);
    visited.insert(dep_id);

    while let Some(current) = queue.pop_front() {
        for d in get_dependencies(current).await {
            if d == task_id {
                return true;
            }
            if visited.insert(d) {
                queue.push_back(d);
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn string_at_limit_ok() {
        let s = "a".repeat(MAX_TITLE_LEN);
        assert!(validate_string_length("title", &s, MAX_TITLE_LEN).is_ok());
    }

    #[test]
    fn string_over_limit_err() {
        let s = "a".repeat(MAX_TITLE_LEN + 1);
        let err = validate_string_length("title", &s, MAX_TITLE_LEN).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("501"), "should contain actual length: {msg}");
        assert!(msg.contains("500"), "should contain max length: {msg}");
    }

    #[test]
    fn empty_string_ok() {
        assert!(validate_string_length("title", "", MAX_TITLE_LEN).is_ok());
    }

    #[test]
    fn multibyte_chars_counted_correctly() {
        let s = "あいうえ"; // 4 chars, 12 bytes
        assert!(validate_string_length("test", s, 4).is_ok());
        assert!(validate_string_length("test", s, 3).is_err());
    }

    #[test]
    fn optional_none_ok() {
        assert!(validate_optional_string_length("f", &None, 10).is_ok());
    }

    #[test]
    fn optional_some_over_limit() {
        let v = Some("a".repeat(11));
        assert!(validate_optional_string_length("f", &v, 10).is_err());
    }

    #[test]
    fn nullable_none_ok() {
        assert!(validate_optional_nullable_string_length("f", &None, 10).is_ok());
        assert!(validate_optional_nullable_string_length("f", &Some(None), 10).is_ok());
    }

    #[test]
    fn nullable_some_some_over_limit() {
        let v = Some(Some("a".repeat(11)));
        assert!(validate_optional_nullable_string_length("f", &v, 10).is_err());
    }

    #[test]
    fn vec_count_over_limit() {
        let items: Vec<String> = (0..MAX_TAGS_COUNT + 1).map(|i| format!("t{i}")).collect();
        let err = validate_string_vec_items("tags", &items, MAX_TAG_LEN, MAX_TAGS_COUNT).unwrap_err();
        assert!(err.to_string().contains("21"));
    }

    #[test]
    fn vec_item_too_long() {
        let items = vec!["a".repeat(MAX_TAG_LEN + 1)];
        let err = validate_string_vec_items("tags", &items, MAX_TAG_LEN, MAX_TAGS_COUNT).unwrap_err();
        assert!(err.to_string().contains("tags[0]"));
    }

    #[test]
    fn vec_empty_ok() {
        assert!(validate_string_vec_items("tags", &[], MAX_TAG_LEN, MAX_TAGS_COUNT).is_ok());
    }

    fn make_graph(edges: &[(i64, Vec<i64>)]) -> impl Fn(i64) -> Vec<i64> + '_ {
        let map: HashMap<i64, &Vec<i64>> = edges.iter().map(|(k, v)| (*k, v)).collect();
        move |id| map.get(&id).map(|v| v.to_vec()).unwrap_or_default()
    }

    #[test]
    fn no_cycle_linear_chain() {
        // 1 -> 2 -> 3, adding 4 depends on 3
        let edges = [(3, vec![2]), (2, vec![1]), (1, vec![])];
        assert!(!has_cycle(4, 3, make_graph(&edges)));
    }

    #[test]
    fn direct_cycle() {
        // 1 -> 2, adding 2 depends on 1 would create: 2 -> 1 -> 2
        let edges = [(1, vec![2]), (2, vec![])];
        assert!(has_cycle(2, 1, make_graph(&edges)));
    }

    #[test]
    fn indirect_cycle() {
        // 1 -> 2 -> 3, adding 3 depends on 1 would create: 3 -> 1 -> 2 -> 3
        let edges = [(1, vec![2]), (2, vec![3]), (3, vec![])];
        assert!(has_cycle(3, 1, make_graph(&edges)));
    }

    #[test]
    fn diamond_no_cycle() {
        // 1 -> 2, 1 -> 3, 2 -> 4, 3 -> 4, adding 5 depends on 4
        let edges = [
            (4, vec![2, 3]),
            (2, vec![1]),
            (3, vec![1]),
            (1, vec![]),
        ];
        assert!(!has_cycle(5, 4, make_graph(&edges)));
    }

    #[test]
    fn self_dependency_not_detected() {
        // has_cycle doesn't check self-dependency (task_id == dep_id);
        // that's validated separately at the call site
        let edges = [(1, vec![])];
        assert!(!has_cycle(1, 1, make_graph(&edges)));
    }

    #[test]
    fn no_dependencies_no_cycle() {
        let edges: [(i64, Vec<i64>); 0] = [];
        assert!(!has_cycle(2, 1, make_graph(&edges)));
    }
}
