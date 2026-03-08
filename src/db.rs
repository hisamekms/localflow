use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

pub fn open_db(project_root: &Path) -> Result<Connection> {
    let localflow_dir = project_root.join(".localflow");
    std::fs::create_dir_all(&localflow_dir)?;

    let db_path = localflow_dir.join("data.db");
    let conn = Connection::open(&db_path)?;

    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    create_schema(&conn)?;

    Ok(conn)
}

fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'draft',
            priority INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            completed_at TEXT
        );

        CREATE TABLE IF NOT EXISTS task_definition_of_done (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            item TEXT NOT NULL,
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS task_in_scope (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            item TEXT NOT NULL,
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS task_out_of_scope (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            item TEXT NOT NULL,
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS task_tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            tag TEXT NOT NULL,
            UNIQUE(task_id, tag),
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS task_dependencies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            depends_on_task_id INTEGER NOT NULL,
            UNIQUE(task_id, depends_on_task_id),
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
            FOREIGN KEY (depends_on_task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );
        ",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_db_and_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = open_db(tmp.path()).unwrap();
        assert!(tmp.path().join(".localflow/data.db").exists());
        drop(conn);
    }

    #[test]
    fn tables_exist() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = open_db(tmp.path()).unwrap();

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(tables.contains(&"tasks".to_string()));
        assert!(tables.contains(&"task_definition_of_done".to_string()));
        assert!(tables.contains(&"task_in_scope".to_string()));
        assert!(tables.contains(&"task_out_of_scope".to_string()));
        assert!(tables.contains(&"task_tags".to_string()));
        assert!(tables.contains(&"task_dependencies".to_string()));
    }

    #[test]
    fn wal_mode_enabled() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = open_db(tmp.path()).unwrap();

        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(mode, "wal");
    }

    #[test]
    fn foreign_keys_enabled() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = open_db(tmp.path()).unwrap();

        let fk: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk, 1);
    }

    #[test]
    fn idempotent_open() {
        let tmp = tempfile::tempdir().unwrap();
        let _conn1 = open_db(tmp.path()).unwrap();
        drop(_conn1);
        let _conn2 = open_db(tmp.path()).unwrap();
        // No error on second open
    }

    #[test]
    fn cascade_delete() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = open_db(tmp.path()).unwrap();

        conn.execute(
            "INSERT INTO tasks (title, status, priority) VALUES ('test', 'todo', 1)",
            [],
        )
        .unwrap();
        let task_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO task_tags (task_id, tag) VALUES (?1, 'rust')",
            [task_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO task_definition_of_done (task_id, item) VALUES (?1, 'done')",
            [task_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO task_dependencies (task_id, depends_on_task_id) VALUES (?1, ?1)",
            [task_id],
        )
        .unwrap();

        conn.execute("DELETE FROM tasks WHERE id = ?1", [task_id])
            .unwrap();

        let tag_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM task_tags WHERE task_id = ?1",
                [task_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tag_count, 0);

        let dod_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM task_definition_of_done WHERE task_id = ?1",
                [task_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(dod_count, 0);

        let dep_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM task_dependencies WHERE task_id = ?1",
                [task_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(dep_count, 0);
    }

    #[test]
    fn unique_constraints() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = open_db(tmp.path()).unwrap();

        conn.execute(
            "INSERT INTO tasks (title, status, priority) VALUES ('t1', 'todo', 1)",
            [],
        )
        .unwrap();
        let task_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO task_tags (task_id, tag) VALUES (?1, 'rust')",
            [task_id],
        )
        .unwrap();

        // Duplicate tag should fail
        let result = conn.execute(
            "INSERT INTO task_tags (task_id, tag) VALUES (?1, 'rust')",
            [task_id],
        );
        assert!(result.is_err());
    }
}
