use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use recall_cli::providers::claude::ClaudeCodeProvider;
use recall_cli::providers::copilot::CopilotProvider;

// ─── Copilot Provider ───

#[test]
fn test_copilot_loads_valid_session() {
    let dir = tempfile::tempdir().unwrap();
    let session_dir = dir.path().join("test-session-123");
    fs::create_dir_all(&session_dir).unwrap();

    fs::write(
        session_dir.join("workspace.yaml"),
        "\
id: test-session-123
cwd: /Users/test/project
summary: Test Session
created_at: 2026-03-20T10:00:00Z
updated_at: 2026-03-20T12:00:00Z
",
    )
    .unwrap();

    fs::write(
        session_dir.join("events.jsonl"),
        r#"{"type":"user.message","data":{"content":"Fix the bug in main.rs"},"id":"evt1","timestamp":"2026-03-20T10:00:00Z"}"#,
    )
    .unwrap();

    let session = CopilotProvider::load_session(&session_dir).expect("should load session");
    assert_eq!(session.id, "test-session-123");
    assert_eq!(session.summary, "Test Session");
    assert_eq!(session.cwd, "/Users/test/project");

    let expected_created = parse_dt("2026-03-20T10:00:00Z");
    let expected_updated = parse_dt("2026-03-20T12:00:00Z");
    assert_eq!(session.created_at, expected_created);
    assert_eq!(session.updated_at, expected_updated);

    assert_eq!(session.user_messages.len(), 1);
    assert_eq!(session.user_messages[0], "Fix the bug in main.rs");
}

#[test]
fn test_copilot_skips_empty_session() {
    let dir = tempfile::tempdir().unwrap();
    let session_dir = dir.path().join("empty-session");
    fs::create_dir_all(&session_dir).unwrap();

    // No summary, no events file → no user messages
    fs::write(
        session_dir.join("workspace.yaml"),
        "\
id: empty-session
",
    )
    .unwrap();

    let session = CopilotProvider::load_session(&session_dir);
    assert!(session.is_none(), "empty session should be skipped");
}

#[test]
fn test_copilot_parses_checkpoints() {
    let dir = tempfile::tempdir().unwrap();
    let session_dir = dir.path().join("cp-session");
    fs::create_dir_all(session_dir.join("checkpoints")).unwrap();

    fs::write(
        session_dir.join("workspace.yaml"),
        "\
id: cp-session
summary: Checkpoint Test
created_at: 2026-03-20T10:00:00Z
updated_at: 2026-03-20T12:00:00Z
",
    )
    .unwrap();

    fs::write(
        session_dir.join("checkpoints").join("index.md"),
        "\
| # | Title | Timestamp |
|---|-------|-----------|
| 1 | Initial setup | 2026-03-20T10:00:00Z |
| 2 | Bug fix complete | 2026-03-20T11:00:00Z |
",
    )
    .unwrap();

    let session = CopilotProvider::load_session(&session_dir).expect("should load session");
    assert_eq!(session.checkpoints.len(), 2);
    assert_eq!(session.checkpoints[0].title, "Initial setup");
    assert_eq!(session.checkpoints[1].title, "Bug fix complete");
}

#[test]
fn test_copilot_handles_malformed_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let session_dir = dir.path().join("bad-yaml");
    fs::create_dir_all(&session_dir).unwrap();

    fs::write(session_dir.join("workspace.yaml"), "not: valid: yaml: [[[").unwrap();

    let session = CopilotProvider::load_session(&session_dir);
    assert!(session.is_none(), "malformed YAML should return None");
}

#[test]
fn test_copilot_handles_malformed_events() {
    let dir = tempfile::tempdir().unwrap();
    let session_dir = dir.path().join("bad-events");
    fs::create_dir_all(&session_dir).unwrap();

    fs::write(
        session_dir.join("workspace.yaml"),
        "\
id: bad-events
summary: Malformed Events Test
created_at: 2026-03-20T10:00:00Z
updated_at: 2026-03-20T12:00:00Z
",
    )
    .unwrap();

    fs::write(
        session_dir.join("events.jsonl"),
        "not json at all\n\
         {\"type\":\"user.message\",\"data\":{\"content\":\"Good message\"},\"id\":\"evt1\",\"timestamp\":\"2026-03-20T10:00:00Z\"}\n\
         also not json\n",
    )
    .unwrap();

    let session = CopilotProvider::load_session(&session_dir).expect("should load session");
    assert_eq!(session.user_messages.len(), 1);
    assert_eq!(session.user_messages[0], "Good message");
}

#[test]
fn test_copilot_extracts_task_summaries() {
    let dir = tempfile::tempdir().unwrap();
    let session_dir = dir.path().join("task-session");
    fs::create_dir_all(&session_dir).unwrap();

    fs::write(
        session_dir.join("workspace.yaml"),
        "\
id: task-session
summary: Task Summary Test
created_at: 2026-03-20T10:00:00Z
updated_at: 2026-03-20T12:00:00Z
",
    )
    .unwrap();

    let events = "\
{\"type\":\"user.message\",\"data\":{\"content\":\"Fix the bug\"},\"id\":\"evt1\",\"timestamp\":\"2026-03-20T10:00:00Z\"}\n\
{\"type\":\"session.task_complete\",\"data\":{\"summary\":\"Fixed the null pointer bug in main.rs\\nAlso updated tests\"},\"id\":\"evt2\",\"timestamp\":\"2026-03-20T11:00:00Z\"}";
    fs::write(session_dir.join("events.jsonl"), events).unwrap();

    let session = CopilotProvider::load_session(&session_dir).expect("should load session");
    assert_eq!(session.task_summaries.len(), 1);
    assert_eq!(
        session.task_summaries[0],
        "Fixed the null pointer bug in main.rs"
    );
    assert_eq!(session.user_messages.len(), 1);
}

// ─── Claude Code Provider ───

fn claude_user_event(session_id: &str, cwd: &str, content: &str, ts: &str) -> String {
    format!(
        r#"{{"type":"user","sessionId":"{session_id}","cwd":"{cwd}","message":{{"role":"user","content":"{content}"}},"uuid":"msg-u","timestamp":"{ts}"}}"#,
    )
}

fn claude_assistant_event(session_id: &str, cwd: &str, text: &str, ts: &str) -> String {
    format!(
        r#"{{"type":"assistant","sessionId":"{session_id}","cwd":"{cwd}","message":{{"role":"assistant","content":[{{"type":"text","text":"{text}"}}]}},"uuid":"msg-a","timestamp":"{ts}"}}"#,
    )
}

#[test]
fn test_claude_loads_valid_session() {
    let dir = tempfile::tempdir().unwrap();
    let session_file = dir.path().join("abc-123.jsonl");

    let content = format!(
        "{}\n{}",
        claude_user_event(
            "sess-123",
            "/Users/test/project",
            "Explain this code",
            "2026-03-20T10:00:00Z"
        ),
        claude_assistant_event(
            "sess-123",
            "/Users/test/project",
            "This code does...",
            "2026-03-20T10:05:00Z"
        ),
    );
    fs::write(&session_file, content).unwrap();

    let session = ClaudeCodeProvider::load_session(&session_file).expect("should load session");
    assert_eq!(session.id, "sess-123");
    assert_eq!(session.cwd, "/Users/test/project");
    assert_eq!(session.user_messages.len(), 1);
    assert_eq!(session.user_messages[0], "Explain this code");
    assert_eq!(session.summary, "Explain this code");
}

#[test]
fn test_claude_skips_agent_files() {
    let dir = tempfile::tempdir().unwrap();
    let project_dir = dir.path().join("my-project");
    fs::create_dir_all(&project_dir).unwrap();

    // Valid session file
    let valid_file = project_dir.join("abc-123.jsonl");
    fs::write(
        &valid_file,
        claude_user_event("s1", "/test", "Hello", "2026-03-20T10:00:00Z"),
    )
    .unwrap();

    // Agent file — should be ignored by discover_in
    let agent_file = project_dir.join("agent-456.jsonl");
    fs::write(
        &agent_file,
        claude_user_event("s2", "/test", "Agent msg", "2026-03-20T10:00:00Z"),
    )
    .unwrap();

    let sessions = ClaudeCodeProvider::discover_in(dir.path());
    assert_eq!(sessions.len(), 1, "agent file should be skipped");
    assert_eq!(sessions[0].id, "s1");
}

#[test]
fn test_claude_skips_empty_sessions() {
    let dir = tempfile::tempdir().unwrap();
    let session_file = dir.path().join("empty.jsonl");

    // Only an assistant event — no user messages
    fs::write(
        &session_file,
        claude_assistant_event("sess-456", "/test", "Hello", "2026-03-20T10:00:00Z"),
    )
    .unwrap();

    let session = ClaudeCodeProvider::load_session(&session_file);
    assert!(
        session.is_none(),
        "session with no user messages should be skipped"
    );
}

#[test]
fn test_claude_handles_malformed_jsonl() {
    let dir = tempfile::tempdir().unwrap();
    let session_file = dir.path().join("malformed.jsonl");

    let content = format!(
        "not json\n{}\nalso garbage\n",
        claude_user_event("sess-789", "/test", "Valid message", "2026-03-20T10:00:00Z"),
    );
    fs::write(&session_file, content).unwrap();

    let session = ClaudeCodeProvider::load_session(&session_file).expect("should load session");
    assert_eq!(session.user_messages.len(), 1);
    assert_eq!(session.user_messages[0], "Valid message");
}

#[test]
fn test_claude_derives_summary_from_first_message() {
    let dir = tempfile::tempdir().unwrap();
    let session_file = dir.path().join("summary.jsonl");

    let long_message = "A".repeat(100);
    let content = format!(
        r#"{{"type":"user","sessionId":"sess-sum","cwd":"/test","message":{{"role":"user","content":"{long_message}"}},"uuid":"msg-1","timestamp":"2026-03-20T10:00:00Z"}}"#,
    );
    fs::write(&session_file, content).unwrap();

    let session = ClaudeCodeProvider::load_session(&session_file).expect("should load session");
    // Summary should be first message truncated to 80 chars + "…"
    assert_eq!(session.summary.chars().count(), 81);
    assert!(session.summary.ends_with('…'));
    assert!(session.summary.starts_with("AAAA"));
}

#[test]
fn test_claude_extracts_timestamps() {
    let dir = tempfile::tempdir().unwrap();
    let session_file = dir.path().join("timestamps.jsonl");

    let content = format!(
        "{}\n{}",
        claude_user_event("sess-ts", "/test", "First", "2026-03-20T10:00:00Z"),
        claude_assistant_event("sess-ts", "/test", "Response", "2026-03-20T12:00:00Z"),
    );
    fs::write(&session_file, content).unwrap();

    let session = ClaudeCodeProvider::load_session(&session_file).expect("should load session");
    assert_eq!(session.created_at, parse_dt("2026-03-20T10:00:00Z"));
    assert_eq!(session.updated_at, parse_dt("2026-03-20T12:00:00Z"));
}

// ─── Helpers ───

fn parse_dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
}

// ─── Extra edge cases ───

#[test]
fn test_copilot_filters_system_messages() {
    let dir = tempfile::tempdir().unwrap();
    let session_dir = dir.path().join("sys-msg-session");
    fs::create_dir_all(&session_dir).unwrap();

    fs::write(
        session_dir.join("workspace.yaml"),
        "\
id: sys-msg-session
summary: System Message Filter Test
created_at: 2026-03-20T10:00:00Z
updated_at: 2026-03-20T12:00:00Z
",
    )
    .unwrap();

    let events = "\
{\"type\":\"user.message\",\"data\":{\"content\":\"<current_datetime>2026-03-20</current_datetime>\"},\"id\":\"evt1\",\"timestamp\":\"2026-03-20T10:00:00Z\"}\n\
{\"type\":\"user.message\",\"data\":{\"content\":\"Real user message\"},\"id\":\"evt2\",\"timestamp\":\"2026-03-20T10:01:00Z\"}";
    fs::write(session_dir.join("events.jsonl"), events).unwrap();

    let session = CopilotProvider::load_session(&session_dir).expect("should load session");
    assert_eq!(session.user_messages.len(), 1);
    assert_eq!(session.user_messages[0], "Real user message");
}

#[test]
fn test_copilot_missing_workspace_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let session_dir = dir.path().join("no-yaml");
    fs::create_dir_all(&session_dir).unwrap();
    // No workspace.yaml at all
    let session = CopilotProvider::load_session(&session_dir);
    assert!(session.is_none());
}

#[test]
fn test_claude_file_stem_as_fallback_id() {
    let dir = tempfile::tempdir().unwrap();
    let session_file = dir.path().join("my-fallback-id.jsonl");

    // User event without sessionId
    fs::write(
        &session_file,
        r#"{"type":"user","cwd":"/test","message":{"role":"user","content":"Hello"},"uuid":"msg-1","timestamp":"2026-03-20T10:00:00Z"}"#,
    )
    .unwrap();

    let session =
        ClaudeCodeProvider::load_session(Path::new(&session_file)).expect("should load session");
    assert_eq!(session.id, "my-fallback-id");
}
