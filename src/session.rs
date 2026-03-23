use std::path::PathBuf;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub provider: String,
    pub summary: String,
    pub cwd: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub checkpoints: Vec<Checkpoint>,
    pub user_messages: Vec<String>,
    pub task_summaries: Vec<String>,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub title: String,
}

pub fn delete_session(session: &Session) -> std::io::Result<()> {
    if session.path.is_dir() {
        std::fs::remove_dir_all(&session.path)
    } else {
        std::fs::remove_file(&session.path)
    }
}

pub fn human_time_ago(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*dt);

    let minutes = duration.num_minutes();
    let hours = duration.num_hours();
    let days = duration.num_days();

    if minutes < 1 {
        "just now".to_string()
    } else if minutes < 60 {
        format!("{minutes}m ago")
    } else if hours < 24 {
        format!("{hours}h ago")
    } else if days < 30 {
        format!("{days}d ago")
    } else {
        dt.format("%b %d, %Y").to_string()
    }
}

pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{truncated}…")
    }
}
