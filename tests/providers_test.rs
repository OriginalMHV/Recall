use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use recall_cli::providers::claude::ClaudeCodeProvider;
use recall_cli::providers::codex::CodexProvider;
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

// ─── Codex CLI Provider ───

fn codex_meta_line(id: &str, cwd: &str, ts: &str) -> String {
    format!(
        r#"{{"timestamp":"{ts}","type":"session_meta","payload":{{"id":"{id}","cwd":"{cwd}","timestamp":"{ts}","originator":"cli","cli_version":"0.1.0","source":"cli"}}}}"#,
    )
}

fn codex_user_item(text: &str, ts: &str) -> String {
    format!(
        r#"{{"timestamp":"{ts}","type":"response_item","payload":{{"id":"item-u","role":"user","content":[{{"type":"input_text","text":"{text}"}}]}}}}"#,
    )
}

fn codex_assistant_item(text: &str, ts: &str) -> String {
    format!(
        r#"{{"timestamp":"{ts}","type":"response_item","payload":{{"id":"item-a","role":"assistant","content":[{{"type":"output_text","text":"{text}"}}]}}}}"#,
    )
}

#[test]
fn test_codex_loads_valid_session() {
    let dir = tempfile::tempdir().unwrap();
    let session_file = dir.path().join("rollout-2026-03-20T10-00-00-abc123.jsonl");

    let content = format!(
        "{}\n{}\n{}",
        codex_meta_line(
            "codex-test-123",
            "/Users/test/project",
            "2026-03-20T10:00:00Z"
        ),
        codex_user_item("Fix the bug in main.rs", "2026-03-20T10:01:00Z"),
        codex_assistant_item("I'll fix that bug.", "2026-03-20T10:02:00Z"),
    );
    fs::write(&session_file, content).unwrap();

    let session = CodexProvider::load_session(&session_file).expect("should load session");
    assert_eq!(session.id, "codex-test-123");
    assert_eq!(session.cwd, "/Users/test/project");
    assert_eq!(session.summary, "Fix the bug in main.rs");
    assert_eq!(session.user_messages.len(), 1);
    assert_eq!(session.user_messages[0], "Fix the bug in main.rs");
    assert_eq!(session.created_at, parse_dt("2026-03-20T10:00:00Z"));
    assert_eq!(session.updated_at, parse_dt("2026-03-20T10:02:00Z"));
}

#[test]
fn test_codex_skips_non_rollout_files() {
    let dir = tempfile::tempdir().unwrap();

    // Valid rollout file
    let valid = dir.path().join("rollout-2026-03-20T10-00-00-abc.jsonl");
    let content = format!(
        "{}\n{}",
        codex_meta_line("valid-1", "/test", "2026-03-20T10:00:00Z"),
        codex_user_item("Hello", "2026-03-20T10:01:00Z"),
    );
    fs::write(&valid, content).unwrap();

    // Non-rollout files that should be skipped
    fs::write(
        dir.path().join("other-file.jsonl"),
        codex_meta_line("skip-1", "/test", "2026-03-20T10:00:00Z"),
    )
    .unwrap();
    fs::write(
        dir.path().join("session_index.jsonl"),
        r#"{"thread_id":"t1","name":"test"}"#,
    )
    .unwrap();

    let sessions = CodexProvider::discover_in(dir.path());
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, "valid-1");
}

#[test]
fn test_codex_handles_malformed_jsonl() {
    let dir = tempfile::tempdir().unwrap();
    let session_file = dir.path().join("rollout-2026-03-20T10-00-00-bad.jsonl");

    let content = format!(
        "not json at all\n{}\nmore garbage\n{}\nalso broken",
        codex_meta_line("mal-1", "/test", "2026-03-20T10:00:00Z"),
        codex_user_item("Valid message", "2026-03-20T10:01:00Z"),
    );
    fs::write(&session_file, content).unwrap();

    let session = CodexProvider::load_session(&session_file).expect("should load session");
    assert_eq!(session.user_messages.len(), 1);
    assert_eq!(session.user_messages[0], "Valid message");
}

#[test]
fn test_codex_strips_user_message_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let session_file = dir.path().join("rollout-2026-03-20T10-00-00-pfx.jsonl");

    let content = format!(
        "{}\n{}",
        codex_meta_line("pfx-1", "/test", "2026-03-20T10:00:00Z"),
        codex_user_item(
            "## My request for Codex: actual request",
            "2026-03-20T10:01:00Z"
        ),
    );
    fs::write(&session_file, content).unwrap();

    let session = CodexProvider::load_session(&session_file).expect("should load session");
    assert_eq!(session.user_messages[0], "actual request");
    assert_eq!(session.summary, "actual request");
}

#[test]
fn test_codex_derives_summary_from_first_message() {
    let dir = tempfile::tempdir().unwrap();
    let session_file = dir.path().join("rollout-2026-03-20T10-00-00-long.jsonl");

    let long_message = "B".repeat(100);
    let content = format!(
        "{}\n{}",
        codex_meta_line("long-1", "/test", "2026-03-20T10:00:00Z"),
        codex_user_item(&long_message, "2026-03-20T10:01:00Z"),
    );
    fs::write(&session_file, content).unwrap();

    let session = CodexProvider::load_session(&session_file).expect("should load session");
    // Summary = first 80 chars + ellipsis
    assert_eq!(session.summary.chars().count(), 81);
    assert!(session.summary.ends_with('\u{2026}'));
    assert!(session.summary.starts_with("BBBB"));
}

#[test]
fn test_codex_discovers_nested_date_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("2026").join("03").join("20");
    fs::create_dir_all(&nested).unwrap();

    let session_file = nested.join("rollout-2026-03-20T10-00-00-nest.jsonl");
    let content = format!(
        "{}\n{}",
        codex_meta_line("nest-1", "/test", "2026-03-20T10:00:00Z"),
        codex_user_item("Nested session", "2026-03-20T10:01:00Z"),
    );
    fs::write(&session_file, content).unwrap();

    let sessions = CodexProvider::discover_in(dir.path());
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, "nest-1");
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

// ─── Codex: compacted items ───

#[test]
fn test_codex_ignores_compacted_items() {
    let dir = tempfile::tempdir().unwrap();
    let session_file = dir.path().join("rollout-2026-03-20T10-00-00-compact.jsonl");

    let compacted_item = r#"{"timestamp":"2026-03-20T10:01:30Z","type":"response_item","payload":{"id":"item-c","role":"user","content":[{"type":"compacted","text":"old compacted data"}]}}"#;
    let content = format!(
        "{}\n{}\n{}\n{}",
        codex_meta_line("compact-1", "/test", "2026-03-20T10:00:00Z"),
        codex_user_item("Real message", "2026-03-20T10:01:00Z"),
        compacted_item,
        codex_assistant_item("Response", "2026-03-20T10:02:00Z"),
    );
    fs::write(&session_file, content).unwrap();

    let session = CodexProvider::load_session(&session_file).expect("should load session");
    // Only the real user message, not the compacted item
    assert_eq!(session.user_messages.len(), 1);
    assert_eq!(session.user_messages[0], "Real message");
}

#[test]
fn test_codex_empty_content_array() {
    let dir = tempfile::tempdir().unwrap();
    let session_file = dir.path().join("rollout-2026-03-20T10-00-00-empty.jsonl");

    let empty_content = r#"{"timestamp":"2026-03-20T10:01:30Z","type":"response_item","payload":{"id":"item-e","role":"user","content":[]}}"#;
    let content = format!(
        "{}\n{}\n{}",
        codex_meta_line("empty-c-1", "/test", "2026-03-20T10:00:00Z"),
        codex_user_item("Real message", "2026-03-20T10:01:00Z"),
        empty_content,
    );
    fs::write(&session_file, content).unwrap();

    let session = CodexProvider::load_session(&session_file).expect("should load session");
    assert_eq!(session.user_messages.len(), 1);
}

#[test]
fn test_codex_discovers_in_archived_directory() {
    let dir = tempfile::tempdir().unwrap();
    let archived = dir.path().join("archived_sessions");
    fs::create_dir_all(&archived).unwrap();

    let session_file = archived.join("rollout-2026-03-20T10-00-00-arch.jsonl");
    let content = format!(
        "{}\n{}",
        codex_meta_line("arch-1", "/test", "2026-03-20T10:00:00Z"),
        codex_user_item("Archived session", "2026-03-20T10:01:00Z"),
    );
    fs::write(&session_file, content).unwrap();

    let sessions = CodexProvider::discover_in(dir.path());
    // walk_dir recurses, so it should find the file inside archived_sessions
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, "arch-1");
}

#[test]
fn test_codex_session_without_meta() {
    let dir = tempfile::tempdir().unwrap();
    let session_file = dir.path().join("rollout-2026-03-20T10-00-00-nometa.jsonl");

    // No session_meta line — just a user item
    let content = codex_user_item("Hello with no meta", "2026-03-20T10:01:00Z");
    fs::write(&session_file, content).unwrap();

    let session = CodexProvider::load_session(&session_file).expect("should load session");
    // ID falls back to file stem
    assert_eq!(session.id, "rollout-2026-03-20T10-00-00-nometa");
    assert_eq!(session.user_messages[0], "Hello with no meta");
}

// ─── Claude: multiple project directories ───

#[test]
fn test_claude_discovers_across_multiple_projects() {
    let dir = tempfile::tempdir().unwrap();

    let project_a = dir.path().join("project-a");
    let project_b = dir.path().join("project-b");
    fs::create_dir_all(&project_a).unwrap();
    fs::create_dir_all(&project_b).unwrap();

    fs::write(
        project_a.join("sess-a1.jsonl"),
        claude_user_event("sa1", "/test/a", "Hello from A", "2026-03-20T10:00:00Z"),
    )
    .unwrap();
    fs::write(
        project_b.join("sess-b1.jsonl"),
        claude_user_event("sb1", "/test/b", "Hello from B", "2026-03-20T11:00:00Z"),
    )
    .unwrap();
    fs::write(
        project_b.join("sess-b2.jsonl"),
        claude_user_event("sb2", "/test/b", "Another B", "2026-03-20T12:00:00Z"),
    )
    .unwrap();

    let sessions = ClaudeCodeProvider::discover_in(dir.path());
    assert_eq!(sessions.len(), 3);

    let ids: Vec<&str> = sessions.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"sa1"));
    assert!(ids.contains(&"sb1"));
    assert!(ids.contains(&"sb2"));
}

#[test]
fn test_claude_multiple_user_messages() {
    let dir = tempfile::tempdir().unwrap();
    let session_file = dir.path().join("multi-msg.jsonl");

    let content = format!(
        "{}\n{}\n{}\n{}",
        claude_user_event("mm-1", "/test", "First question", "2026-03-20T10:00:00Z"),
        claude_assistant_event("mm-1", "/test", "First answer", "2026-03-20T10:01:00Z"),
        claude_user_event("mm-1", "/test", "Second question", "2026-03-20T10:02:00Z"),
        claude_assistant_event("mm-1", "/test", "Second answer", "2026-03-20T10:03:00Z"),
    );
    fs::write(&session_file, content).unwrap();

    let session = ClaudeCodeProvider::load_session(&session_file).expect("should load session");
    assert_eq!(session.user_messages.len(), 2);
    assert_eq!(session.summary, "First question"); // summary = first message
}

#[test]
fn test_claude_ignores_non_jsonl_files() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("my-project");
    fs::create_dir_all(&project).unwrap();

    // Valid jsonl
    fs::write(
        project.join("valid.jsonl"),
        claude_user_event("v1", "/test", "Valid", "2026-03-20T10:00:00Z"),
    )
    .unwrap();

    // Non-jsonl files
    fs::write(project.join("notes.txt"), "some notes").unwrap();
    fs::write(project.join("data.json"), "{}").unwrap();

    let sessions = ClaudeCodeProvider::discover_in(dir.path());
    assert_eq!(sessions.len(), 1);
}

// ─── Copilot: large events file with truncation ───

#[test]
fn test_copilot_truncates_long_messages() {
    let dir = tempfile::tempdir().unwrap();
    let session_dir = dir.path().join("long-msg-session");
    fs::create_dir_all(&session_dir).unwrap();

    fs::write(
        session_dir.join("workspace.yaml"),
        "\
id: long-msg-session
summary: Long Message Test
created_at: 2026-03-20T10:00:00Z
updated_at: 2026-03-20T12:00:00Z
",
    )
    .unwrap();

    let long_content = "X".repeat(500);
    let events = format!(
        r#"{{"type":"user.message","data":{{"content":"{long_content}"}},"id":"evt1","timestamp":"2026-03-20T10:00:00Z"}}"#,
    );
    fs::write(session_dir.join("events.jsonl"), events).unwrap();

    let session = CopilotProvider::load_session(&session_dir).expect("should load session");
    assert_eq!(session.user_messages.len(), 1);
    // truncate(msg, 200) → 200 chars + "…" = 201 chars
    assert_eq!(session.user_messages[0].chars().count(), 201);
    assert!(session.user_messages[0].ends_with('…'));
}

#[test]
fn test_copilot_multiple_messages_and_task_summaries() {
    let dir = tempfile::tempdir().unwrap();
    let session_dir = dir.path().join("multi-session");
    fs::create_dir_all(&session_dir).unwrap();

    fs::write(
        session_dir.join("workspace.yaml"),
        "\
id: multi-session
summary: Multi Event Test
created_at: 2026-03-20T10:00:00Z
updated_at: 2026-03-20T12:00:00Z
",
    )
    .unwrap();

    let events = "\
{\"type\":\"user.message\",\"data\":{\"content\":\"First request\"},\"id\":\"evt1\",\"timestamp\":\"2026-03-20T10:00:00Z\"}\n\
{\"type\":\"session.task_complete\",\"data\":{\"summary\":\"Task 1 done\"},\"id\":\"evt2\",\"timestamp\":\"2026-03-20T10:30:00Z\"}\n\
{\"type\":\"user.message\",\"data\":{\"content\":\"Second request\"},\"id\":\"evt3\",\"timestamp\":\"2026-03-20T11:00:00Z\"}\n\
{\"type\":\"session.task_complete\",\"data\":{\"summary\":\"Task 2 done\"},\"id\":\"evt4\",\"timestamp\":\"2026-03-20T11:30:00Z\"}";
    fs::write(session_dir.join("events.jsonl"), events).unwrap();

    let session = CopilotProvider::load_session(&session_dir).expect("should load session");
    assert_eq!(session.user_messages.len(), 2);
    assert_eq!(session.task_summaries.len(), 2);
    assert_eq!(session.user_messages[0], "First request");
    assert_eq!(session.user_messages[1], "Second request");
    assert_eq!(session.task_summaries[0], "Task 1 done");
    assert_eq!(session.task_summaries[1], "Task 2 done");
}

#[test]
fn test_copilot_empty_checkpoints_index() {
    let dir = tempfile::tempdir().unwrap();
    let session_dir = dir.path().join("empty-cp-session");
    fs::create_dir_all(session_dir.join("checkpoints")).unwrap();

    fs::write(
        session_dir.join("workspace.yaml"),
        "\
id: empty-cp-session
summary: Empty Checkpoints
created_at: 2026-03-20T10:00:00Z
updated_at: 2026-03-20T12:00:00Z
",
    )
    .unwrap();

    // index.md with just the header, no data rows
    fs::write(
        session_dir.join("checkpoints").join("index.md"),
        "\
| # | Title | Timestamp |
|---|-------|-----------|
",
    )
    .unwrap();

    let session = CopilotProvider::load_session(&session_dir).expect("should load session");
    assert!(session.checkpoints.is_empty());
}

// ─── Cross-provider: discover_in with empty directories ───

#[test]
fn test_codex_discover_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    let sessions = CodexProvider::discover_in(dir.path());
    assert!(sessions.is_empty());
}

#[test]
fn test_claude_discover_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    let sessions = ClaudeCodeProvider::discover_in(dir.path());
    assert!(sessions.is_empty());
}

#[test]
fn test_codex_skips_empty_sessions() {
    let dir = tempfile::tempdir().unwrap();
    let session_file = dir.path().join("rollout-2026-03-20T10-00-00-nousers.jsonl");

    // Only meta + assistant item, no user messages
    let content = format!(
        "{}\n{}",
        codex_meta_line("nousers-1", "/test", "2026-03-20T10:00:00Z"),
        codex_assistant_item("Only assistant message", "2026-03-20T10:01:00Z"),
    );
    fs::write(&session_file, content).unwrap();

    let session = CodexProvider::load_session(&session_file);
    assert!(
        session.is_none(),
        "session with no user messages should be skipped"
    );
}
