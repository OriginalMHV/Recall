use std::io::BufRead;
use std::path::PathBuf;

use chrono::{DateTime, Utc};

use crate::session::{Session, truncate};

use super::Provider;

pub struct ClaudeCodeProvider;

impl ClaudeCodeProvider {
    fn projects_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".claude").join("projects"))
    }

    pub fn load_session(path: &std::path::Path) -> Option<Session> {
        let file = std::fs::File::open(path).ok()?;
        let reader = std::io::BufReader::new(file);

        let mut session_id = None;
        let mut cwd = None;
        let mut first_user_content: Option<String> = None;
        let mut user_messages = Vec::new();
        let mut first_timestamp: Option<DateTime<Utc>> = None;
        let mut last_timestamp: Option<DateTime<Utc>> = None;

        for line in reader.lines() {
            let Ok(line) = line else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
                continue;
            };

            let event_type = value.get("type").and_then(|v| v.as_str()).unwrap_or("");

            if let Some(ts_str) = value.get("timestamp").and_then(|v| v.as_str()) {
                if let Ok(ts) = ts_str.parse::<DateTime<Utc>>() {
                    if first_timestamp.is_none() {
                        first_timestamp = Some(ts);
                    }
                    last_timestamp = Some(ts);
                }
            }

            if event_type == "user" {
                if session_id.is_none() {
                    session_id = value
                        .get("sessionId")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                }
                if cwd.is_none() {
                    cwd = value.get("cwd").and_then(|v| v.as_str()).map(String::from);
                }

                let msg_content = value
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str());

                if let Some(text) = msg_content {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        if first_user_content.is_none() {
                            first_user_content = Some(trimmed.to_string());
                        }
                        user_messages.push(truncate(trimmed, 200));
                    }
                }
            }
        }

        if user_messages.is_empty() {
            return None;
        }

        let id =
            session_id.or_else(|| path.file_stem().and_then(|s| s.to_str()).map(String::from))?;

        let summary = first_user_content
            .map(|s| truncate(&s, 80))
            .unwrap_or_default();

        Some(Session {
            id,
            provider: String::new(),
            summary,
            cwd: cwd.unwrap_or_default(),
            created_at: first_timestamp.unwrap_or_default(),
            updated_at: last_timestamp.unwrap_or_default(),
            checkpoints: Vec::new(),
            user_messages,
            task_summaries: Vec::new(),
            path: path.to_path_buf(),
        })
    }

    pub fn discover_in(projects_dir: &std::path::Path) -> Vec<Session> {
        let Ok(project_entries) = std::fs::read_dir(projects_dir) else {
            return Vec::new();
        };

        let mut sessions = Vec::new();

        for project_entry in project_entries.filter_map(|e| e.ok()) {
            let project_path = project_entry.path();
            if !project_path.is_dir() {
                continue;
            }

            let Ok(session_files) = std::fs::read_dir(&project_path) else {
                continue;
            };

            for file_entry in session_files.filter_map(|e| e.ok()) {
                let file_path = file_entry.path();
                let Some(name) = file_path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };

                if !name.ends_with(".jsonl") || name.starts_with("agent-") {
                    continue;
                }

                if let Some(session) = Self::load_session(&file_path) {
                    sessions.push(session);
                }
            }
        }

        sessions
    }
}

impl Provider for ClaudeCodeProvider {
    fn name(&self) -> &str {
        "Claude Code"
    }

    fn discover_sessions(&self) -> Vec<Session> {
        let Some(projects_dir) = Self::projects_dir() else {
            return Vec::new();
        };

        Self::discover_in(&projects_dir)
    }
}
