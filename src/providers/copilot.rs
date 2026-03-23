use std::path::PathBuf;

use chrono::DateTime;
use serde::Deserialize;

use crate::session::{Checkpoint, Session, truncate};

use super::Provider;

#[derive(Debug, Deserialize)]
struct WorkspaceYaml {
    id: String,
    cwd: Option<String>,
    summary: Option<String>,
    created_at: Option<DateTime<chrono::Utc>>,
    updated_at: Option<DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
struct Event {
    #[serde(rename = "type")]
    event_type: String,
    data: Option<serde_json::Value>,
}

pub struct CopilotProvider;

impl CopilotProvider {
    fn session_state_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".copilot").join("session-state"))
    }

    pub fn load_session(path: &std::path::Path) -> Option<Session> {
        let workspace_path = path.join("workspace.yaml");
        let content = std::fs::read_to_string(&workspace_path).ok()?;
        let ws: WorkspaceYaml = serde_yaml::from_str(&content).ok()?;

        let checkpoints = Self::load_checkpoints(path);
        let (user_messages, task_summaries) = Self::load_events(path);

        let summary = ws.summary.unwrap_or_default();
        if summary.is_empty() && user_messages.is_empty() {
            return None;
        }

        Some(Session {
            id: ws.id,
            provider: String::new(),
            summary,
            cwd: ws.cwd.unwrap_or_default(),
            created_at: ws.created_at.unwrap_or_default(),
            updated_at: ws.updated_at.unwrap_or_default(),
            checkpoints,
            user_messages,
            task_summaries,
            path: path.to_path_buf(),
        })
    }

    fn load_checkpoints(session_path: &std::path::Path) -> Vec<Checkpoint> {
        let index_path = session_path.join("checkpoints").join("index.md");
        let Ok(content) = std::fs::read_to_string(&index_path) else {
            return Vec::new();
        };

        content
            .lines()
            .filter(|line| {
                line.starts_with("| ") && !line.contains("---") && !line.contains("Title")
            })
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 3 {
                    let title = parts[2].trim().to_string();
                    if !title.is_empty() {
                        return Some(Checkpoint { title });
                    }
                }
                None
            })
            .collect()
    }

    fn load_events(session_path: &std::path::Path) -> (Vec<String>, Vec<String>) {
        let events_path = session_path.join("events.jsonl");
        let Ok(content) = std::fs::read_to_string(&events_path) else {
            return (Vec::new(), Vec::new());
        };

        let mut user_messages = Vec::new();
        let mut task_summaries = Vec::new();

        for line in content.lines() {
            let Ok(event) = serde_json::from_str::<Event>(line) else {
                continue;
            };

            match event.event_type.as_str() {
                "user.message" => {
                    if let Some(data) = &event.data {
                        if let Some(content) = data.get("content").and_then(|v| v.as_str()) {
                            let trimmed = content.trim();
                            if !trimmed.is_empty() && !trimmed.starts_with('<') {
                                user_messages.push(truncate(trimmed, 200));
                            }
                        }
                    }
                }
                "session.task_complete" => {
                    if let Some(data) = &event.data {
                        if let Some(summary) = data.get("summary").and_then(|v| v.as_str()) {
                            let first_line = summary.lines().next().unwrap_or("").trim();
                            if !first_line.is_empty() {
                                task_summaries.push(truncate(first_line, 200));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        (user_messages, task_summaries)
    }
}

impl Provider for CopilotProvider {
    fn name(&self) -> &str {
        "Copilot"
    }

    fn discover_sessions(&self) -> Vec<Session> {
        let Some(state_dir) = Self::session_state_dir() else {
            return Vec::new();
        };

        let Ok(entries) = std::fs::read_dir(&state_dir) else {
            return Vec::new();
        };

        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| Self::load_session(&e.path()))
            .collect()
    }
}
