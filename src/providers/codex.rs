use std::io::BufRead;
use std::path::PathBuf;

use chrono::{DateTime, Utc};

use crate::session::{Session, truncate};

use super::Provider;

const USER_MESSAGE_PREFIX: &str = "## My request for Codex:";

pub struct CodexProvider;

impl CodexProvider {
    fn sessions_dirs() -> Vec<PathBuf> {
        let Some(home) = dirs::home_dir() else {
            return Vec::new();
        };
        let codex = home.join(".codex");
        vec![codex.join("sessions"), codex.join("archived_sessions")]
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

            if let Some(ts_str) = value.get("timestamp").and_then(|v| v.as_str()) {
                if let Ok(ts) = ts_str.parse::<DateTime<Utc>>() {
                    if first_timestamp.is_none() {
                        first_timestamp = Some(ts);
                    }
                    last_timestamp = Some(ts);
                }
            }

            let event_type = value.get("type").and_then(|v| v.as_str()).unwrap_or("");

            if event_type == "session_meta" {
                if let Some(payload) = value.get("payload") {
                    if session_id.is_none() {
                        session_id = payload.get("id").and_then(|v| v.as_str()).map(String::from);
                    }
                    if cwd.is_none() {
                        cwd = payload
                            .get("cwd")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                    }
                }
            }

            if event_type == "response_item" {
                if let Some(payload) = value.get("payload") {
                    let role = payload.get("role").and_then(|v| v.as_str()).unwrap_or("");
                    if role != "user" {
                        continue;
                    }

                    let Some(content_arr) = payload.get("content").and_then(|v| v.as_array())
                    else {
                        continue;
                    };

                    for item in content_arr {
                        let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        if item_type != "input_text" {
                            continue;
                        }
                        let Some(text) = item.get("text").and_then(|v| v.as_str()) else {
                            continue;
                        };

                        let cleaned = text
                            .strip_prefix(USER_MESSAGE_PREFIX)
                            .unwrap_or(text)
                            .trim();

                        if !cleaned.is_empty() {
                            if first_user_content.is_none() {
                                first_user_content = Some(cleaned.to_string());
                            }
                            user_messages.push(truncate(cleaned, 200));
                        }
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

    pub fn discover_in(base_dir: &std::path::Path) -> Vec<Session> {
        let mut sessions = Vec::new();
        Self::walk_dir(base_dir, &mut sessions);
        sessions
    }

    fn walk_dir(dir: &std::path::Path, sessions: &mut Vec<Session>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                Self::walk_dir(&path, sessions);
                continue;
            }

            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            if !name.starts_with("rollout-") || !name.ends_with(".jsonl") {
                continue;
            }

            if let Some(session) = Self::load_session(&path) {
                sessions.push(session);
            }
        }
    }
}

impl Provider for CodexProvider {
    fn name(&self) -> &str {
        "Codex CLI"
    }

    fn discover_sessions(&self) -> Vec<Session> {
        Self::sessions_dirs()
            .iter()
            .flat_map(|dir| Self::discover_in(dir))
            .collect()
    }
}
