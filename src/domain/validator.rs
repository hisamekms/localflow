use std::collections::{HashSet, VecDeque};

use super::error::DomainError;

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

/// Maximum serialized size for metadata JSON (64 KiB).
pub const METADATA_MAX_SIZE: usize = 65_536;

/// Maximum nesting depth for metadata JSON.
pub const METADATA_MAX_DEPTH: u32 = 10;

/// Validate metadata JSON for size and nesting depth limits.
pub fn validate_metadata(value: &serde_json::Value) -> Result<(), DomainError> {
    let size = serde_json::to_string(value)
        .map(|s| s.len())
        .unwrap_or(0);
    if size > METADATA_MAX_SIZE {
        return Err(DomainError::MetadataTooLarge {
            size,
            max: METADATA_MAX_SIZE,
        });
    }
    let depth = json_depth(value);
    if depth > METADATA_MAX_DEPTH {
        return Err(DomainError::MetadataTooDeep {
            depth,
            max: METADATA_MAX_DEPTH,
        });
    }
    Ok(())
}

fn json_depth(value: &serde_json::Value) -> u32 {
    match value {
        serde_json::Value::Array(arr) => 1 + arr.iter().map(json_depth).max().unwrap_or(0),
        serde_json::Value::Object(map) => 1 + map.values().map(json_depth).max().unwrap_or(0),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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

    // --- metadata validation tests ---

    #[test]
    fn metadata_within_limits() {
        let val = serde_json::json!({"key": "value", "num": 42});
        assert!(validate_metadata(&val).is_ok());
    }

    #[test]
    fn metadata_too_large() {
        let big = "x".repeat(70_000);
        let val = serde_json::json!({"big": big});
        let err = validate_metadata(&val).unwrap_err();
        assert!(matches!(err, DomainError::MetadataTooLarge { .. }));
    }

    #[test]
    fn metadata_depth_at_limit() {
        // 10 levels deep: {a:{a:{a:{a:{a:{a:{a:{a:{a:{a:1}}}}}}}}}}
        let mut val = serde_json::json!(1);
        for _ in 0..METADATA_MAX_DEPTH {
            val = serde_json::json!({"a": val});
        }
        assert!(validate_metadata(&val).is_ok());
    }

    #[test]
    fn metadata_too_deep() {
        // 11 levels deep
        let mut val = serde_json::json!(1);
        for _ in 0..=METADATA_MAX_DEPTH {
            val = serde_json::json!({"a": val});
        }
        let err = validate_metadata(&val).unwrap_err();
        assert!(matches!(err, DomainError::MetadataTooDeep { .. }));
    }

    #[test]
    fn metadata_empty_object() {
        let val = serde_json::json!({});
        assert!(validate_metadata(&val).is_ok());
    }

    #[test]
    fn metadata_flat_array() {
        let val = serde_json::json!([1, 2, 3]);
        assert!(validate_metadata(&val).is_ok());
    }

    #[test]
    fn json_depth_nested_arrays() {
        let val = serde_json::json!([[[1]]]);
        assert_eq!(json_depth(&val), 3);
    }

    #[test]
    fn json_depth_mixed() {
        let val = serde_json::json!({"a": [{"b": 1}]});
        // object(1) -> array(2) -> object(3) -> leaf = depth 3
        assert_eq!(json_depth(&val), 3);
    }

    #[test]
    fn json_depth_scalar() {
        assert_eq!(json_depth(&serde_json::json!(42)), 0);
        assert_eq!(json_depth(&serde_json::json!("hello")), 0);
        assert_eq!(json_depth(&serde_json::Value::Null), 0);
    }
}
