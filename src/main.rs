use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};
use localflow::db;
use localflow::models::{Priority, TaskStatus, UpdateTaskArrayParams, UpdateTaskParams};
use localflow::project::resolve_project_root;

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Json,
    Text,
}

#[derive(Debug, Parser)]
#[command(name = "localflow", about = "Local task management CLI")]
struct Cli {
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    output: OutputFormat,

    /// Project root directory
    #[arg(long)]
    project_root: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Add a new task
    Add,
    /// List tasks
    List,
    /// Get task details
    Get,
    /// Show the next task to work on
    Next {
        #[arg(long)]
        session_id: Option<String>,
    },
    /// Edit a task
    Edit {
        /// Task ID
        id: i64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        background: Option<String>,
        #[arg(long)]
        clear_background: bool,
        #[arg(long)]
        details: Option<String>,
        #[arg(long)]
        clear_details: bool,
        #[arg(long, value_enum)]
        priority: Option<Priority>,
        #[arg(long, value_enum)]
        status: Option<TaskStatus>,
        // Array set
        #[arg(long, num_args = 0..)]
        set_tags: Option<Vec<String>>,
        #[arg(long, num_args = 0..)]
        set_definition_of_done: Option<Vec<String>>,
        #[arg(long, num_args = 0..)]
        set_in_scope: Option<Vec<String>>,
        #[arg(long, num_args = 0..)]
        set_out_of_scope: Option<Vec<String>>,
        // Array add
        #[arg(long)]
        add_tag: Vec<String>,
        #[arg(long)]
        add_definition_of_done: Vec<String>,
        #[arg(long)]
        add_in_scope: Vec<String>,
        #[arg(long)]
        add_out_of_scope: Vec<String>,
        // Array remove
        #[arg(long)]
        remove_tag: Vec<String>,
        #[arg(long)]
        remove_definition_of_done: Vec<String>,
        #[arg(long)]
        remove_in_scope: Vec<String>,
        #[arg(long)]
        remove_out_of_scope: Vec<String>,
    },
    /// Mark a task as complete
    Complete {
        /// Task ID
        id: i64,
    },
    /// Cancel a task
    Cancel {
        /// Task ID
        id: i64,
        /// Cancellation reason
        #[arg(long)]
        reason: Option<String>,
    },
    /// Manage task dependencies
    Deps,
    /// Install a skill
    SkillInstall {
        /// Output directory for SKILL.md
        #[arg(long)]
        output_dir: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Add => todo!("add"),
        Command::List => todo!("list"),
        Command::Get => todo!("get"),
        Command::Next { ref session_id } => cmd_next(&cli, session_id.clone()),
        Command::Edit {
            id,
            title,
            background,
            clear_background,
            details,
            clear_details,
            priority,
            status,
            set_tags,
            set_definition_of_done,
            set_in_scope,
            set_out_of_scope,
            add_tag,
            add_definition_of_done,
            add_in_scope,
            add_out_of_scope,
            remove_tag,
            remove_definition_of_done,
            remove_in_scope,
            remove_out_of_scope,
        } => {
            let project_root = resolve_project_root(cli.project_root.as_deref())?;
            let conn = db::open_db(&project_root)?;

            let scalar_params = UpdateTaskParams {
                title,
                background: if clear_background {
                    Some(None)
                } else {
                    background.map(Some)
                },
                details: if clear_details {
                    Some(None)
                } else {
                    details.map(Some)
                },
                priority,
                status,
                assignee_session_id: None,
                started_at: None,
                completed_at: None,
                canceled_at: None,
                cancel_reason: None,
            };

            let array_params = UpdateTaskArrayParams {
                set_tags,
                add_tags: add_tag,
                remove_tags: remove_tag,
                set_definition_of_done,
                add_definition_of_done,
                remove_definition_of_done,
                set_in_scope,
                add_in_scope,
                remove_in_scope,
                set_out_of_scope,
                add_out_of_scope,
                remove_out_of_scope,
            };

            db::update_task(&conn, id, &scalar_params)?;
            db::update_task_arrays(&conn, id, &array_params)?;
            let task = db::get_task(&conn, id)?;

            match cli.output {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&task)?);
                }
                OutputFormat::Text => {
                    println!("Updated task {}", task.id);
                    println!("  title: {}", task.title);
                    println!("  status: {}", task.status);
                    println!("  priority: {}", task.priority);
                    if let Some(ref bg) = task.background {
                        println!("  background: {bg}");
                    }
                    if let Some(ref det) = task.details {
                        println!("  details: {det}");
                    }
                    if !task.tags.is_empty() {
                        println!("  tags: {}", task.tags.join(", "));
                    }
                }
            }
            Ok(())
        }
        Command::Complete { id } => cmd_complete(&cli, id),
        Command::Cancel { id, ref reason } => cmd_cancel(&cli, id, reason.clone()),
        Command::Deps => todo!("deps"),
        Command::SkillInstall { output_dir } => skill_install(output_dir),
    }
}

fn cmd_next(cli: &Cli, session_id: Option<String>) -> Result<()> {
    let root = resolve_project_root(cli.project_root.as_deref())?;
    let conn = db::open_db(&root)?;

    let task = db::next_task(&conn)?.ok_or_else(|| anyhow::anyhow!("no eligible task found"))?;

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let updated = db::update_task(
        &conn,
        task.id,
        &UpdateTaskParams {
            title: None,
            background: None,
            details: None,
            priority: None,
            status: Some(TaskStatus::InProgress),
            assignee_session_id: Some(session_id),
            started_at: Some(Some(now)),
            completed_at: None,
            canceled_at: None,
            cancel_reason: None,
        },
    )?;

    match cli.output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&updated)?);
        }
        OutputFormat::Text => {
            println!("Started task #{}: {}", updated.id, updated.title);
        }
    }

    Ok(())
}

fn cmd_complete(cli: &Cli, id: i64) -> Result<()> {
    let root = resolve_project_root(cli.project_root.as_deref())?;
    let conn = db::open_db(&root)?;

    let task = db::get_task(&conn, id)?;
    task.status.transition_to(TaskStatus::Completed)?;

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let updated = db::update_task(
        &conn,
        id,
        &UpdateTaskParams {
            title: None,
            background: None,
            details: None,
            priority: None,
            status: Some(TaskStatus::Completed),
            assignee_session_id: None,
            started_at: None,
            completed_at: Some(Some(now)),
            canceled_at: None,
            cancel_reason: None,
        },
    )?;

    match cli.output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&updated)?);
        }
        OutputFormat::Text => {
            println!("Completed task #{}: {}", updated.id, updated.title);
        }
    }

    Ok(())
}

fn cmd_cancel(cli: &Cli, id: i64, reason: Option<String>) -> Result<()> {
    let root = resolve_project_root(cli.project_root.as_deref())?;
    let conn = db::open_db(&root)?;

    let task = db::get_task(&conn, id)?;
    task.status.transition_to(TaskStatus::Canceled)?;

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let updated = db::update_task(
        &conn,
        id,
        &UpdateTaskParams {
            title: None,
            background: None,
            details: None,
            priority: None,
            status: Some(TaskStatus::Canceled),
            assignee_session_id: None,
            started_at: None,
            completed_at: None,
            canceled_at: Some(Some(now)),
            cancel_reason: reason.map(Some),
        },
    )?;

    match cli.output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&updated)?);
        }
        OutputFormat::Text => {
            println!("Canceled task #{}: {}", updated.id, updated.title);
            if let Some(ref r) = updated.cancel_reason {
                println!("  reason: {r}");
            }
        }
    }

    Ok(())
}

const SKILL_MD_CONTENT: &str = include_str!("skill_md.txt");

fn skill_install(output_dir: Option<PathBuf>) -> Result<()> {
    let dir = output_dir.unwrap_or_else(|| PathBuf::from("."));
    let path = dir.join("SKILL.md");
    fs::write(&path, SKILL_MD_CONTENT)?;
    println!("SKILL.md written to {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn parse_add_subcommand() {
        let cli = Cli::parse_from(["localflow", "add"]);
        assert!(matches!(cli.command, Command::Add));
    }

    #[test]
    fn parse_list_subcommand() {
        let cli = Cli::parse_from(["localflow", "list"]);
        assert!(matches!(cli.command, Command::List));
    }

    #[test]
    fn parse_get_subcommand() {
        let cli = Cli::parse_from(["localflow", "get"]);
        assert!(matches!(cli.command, Command::Get));
    }

    #[test]
    fn parse_next_subcommand() {
        let cli = Cli::parse_from(["localflow", "next"]);
        assert!(matches!(cli.command, Command::Next { .. }));
    }

    #[test]
    fn parse_next_with_session_id() {
        let cli = Cli::parse_from(["localflow", "next", "--session-id", "abc-123"]);
        match cli.command {
            Command::Next { session_id } => {
                assert_eq!(session_id, Some("abc-123".to_string()));
            }
            _ => panic!("expected Next"),
        }
    }

    #[test]
    fn parse_edit_subcommand() {
        let cli = Cli::parse_from(["localflow", "edit", "1"]);
        assert!(matches!(cli.command, Command::Edit { id: 1, .. }));
    }

    #[test]
    fn parse_edit_with_scalar_args() {
        let cli = Cli::parse_from([
            "localflow", "edit", "5",
            "--title", "new title",
            "--priority", "p0",
            "--status", "todo",
        ]);
        match cli.command {
            Command::Edit { id, title, priority, status, .. } => {
                assert_eq!(id, 5);
                assert_eq!(title.as_deref(), Some("new title"));
                assert_eq!(priority, Some(Priority::P0));
                assert_eq!(status, Some(TaskStatus::Todo));
            }
            _ => panic!("expected Edit"),
        }
    }

    #[test]
    fn parse_edit_with_array_args() {
        let cli = Cli::parse_from([
            "localflow", "edit", "3",
            "--add-tag", "rust",
            "--add-tag", "cli",
            "--remove-tag", "old",
            "--set-in-scope", "a", "b",
        ]);
        match cli.command {
            Command::Edit { id, add_tag, remove_tag, set_in_scope, .. } => {
                assert_eq!(id, 3);
                assert_eq!(add_tag, vec!["rust", "cli"]);
                assert_eq!(remove_tag, vec!["old"]);
                assert_eq!(set_in_scope, Some(vec!["a".to_string(), "b".to_string()]));
            }
            _ => panic!("expected Edit"),
        }
    }

    #[test]
    fn parse_edit_clear_background() {
        let cli = Cli::parse_from(["localflow", "edit", "1", "--clear-background"]);
        match cli.command {
            Command::Edit { clear_background, .. } => {
                assert!(clear_background);
            }
            _ => panic!("expected Edit"),
        }
    }

    #[test]
    fn parse_complete_subcommand() {
        let cli = Cli::parse_from(["localflow", "complete", "1"]);
        assert!(matches!(cli.command, Command::Complete { id: 1 }));
    }

    #[test]
    fn parse_cancel_subcommand() {
        let cli = Cli::parse_from(["localflow", "cancel", "2"]);
        assert!(matches!(cli.command, Command::Cancel { id: 2, .. }));
    }

    #[test]
    fn parse_cancel_with_reason() {
        let cli = Cli::parse_from(["localflow", "cancel", "3", "--reason", "no longer needed"]);
        match cli.command {
            Command::Cancel { id, reason } => {
                assert_eq!(id, 3);
                assert_eq!(reason.as_deref(), Some("no longer needed"));
            }
            _ => panic!("expected Cancel"),
        }
    }

    #[test]
    fn parse_cancel_without_reason() {
        let cli = Cli::parse_from(["localflow", "cancel", "4"]);
        match cli.command {
            Command::Cancel { id, reason } => {
                assert_eq!(id, 4);
                assert!(reason.is_none());
            }
            _ => panic!("expected Cancel"),
        }
    }

    #[test]
    fn parse_deps_subcommand() {
        let cli = Cli::parse_from(["localflow", "deps"]);
        assert!(matches!(cli.command, Command::Deps));
    }

    #[test]
    fn parse_skill_install_subcommand() {
        let cli = Cli::parse_from(["localflow", "skill-install"]);
        assert!(matches!(cli.command, Command::SkillInstall { .. }));
    }

    #[test]
    fn parse_skill_install_with_output_dir() {
        let cli = Cli::parse_from(["localflow", "skill-install", "--output-dir", "/tmp/out"]);
        match cli.command {
            Command::SkillInstall { output_dir } => {
                assert_eq!(output_dir, Some(PathBuf::from("/tmp/out")));
            }
            _ => panic!("expected SkillInstall"),
        }
    }

    #[test]
    fn parse_skill_install_without_output_dir() {
        let cli = Cli::parse_from(["localflow", "skill-install"]);
        match cli.command {
            Command::SkillInstall { output_dir } => {
                assert!(output_dir.is_none());
            }
            _ => panic!("expected SkillInstall"),
        }
    }

    #[test]
    fn skill_install_creates_file() {
        let dir = std::env::temp_dir().join("localflow_test_skill_install");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        skill_install(Some(dir.clone())).unwrap();

        let content = std::fs::read_to_string(dir.join("SKILL.md")).unwrap();
        assert_eq!(content, SKILL_MD_CONTENT);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn skill_md_covers_all_commands() {
        let commands = [
            "localflow add",
            "localflow list",
            "localflow get",
            "localflow next",
            "localflow edit",
            "localflow complete",
            "localflow cancel",
            "localflow deps",
            "localflow skill-install",
        ];
        for cmd in commands {
            assert!(
                SKILL_MD_CONTENT.contains(cmd),
                "SKILL.md does not mention: {cmd}"
            );
        }
    }

    #[test]
    fn parse_output_json() {
        let cli = Cli::parse_from(["localflow", "--output", "json", "add"]);
        assert!(matches!(cli.output, OutputFormat::Json));
    }

    #[test]
    fn parse_output_text_default() {
        let cli = Cli::parse_from(["localflow", "add"]);
        assert!(matches!(cli.output, OutputFormat::Text));
    }

    #[test]
    fn parse_project_root() {
        let cli = Cli::parse_from(["localflow", "--project-root", "/tmp/test", "add"]);
        assert_eq!(cli.project_root, Some(PathBuf::from("/tmp/test")));
    }

    #[test]
    fn parse_no_project_root() {
        let cli = Cli::parse_from(["localflow", "add"]);
        assert!(cli.project_root.is_none());
    }
}
