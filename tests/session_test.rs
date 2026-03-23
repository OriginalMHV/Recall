use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Duration, Utc};
use recall_cli::session::{Session, delete_session, human_time_ago, truncate};

// ─── human_time_ago ───

#[test]
fn test_human_time_ago_just_now() {
    let dt = Utc::now() - Duration::seconds(30);
    assert_eq!(human_time_ago(&dt), "just now");
}

#[test]
fn test_human_time_ago_minutes() {
    let dt = Utc::now() - Duration::minutes(15);
    assert_eq!(human_time_ago(&dt), "15m ago");
}

#[test]
fn test_human_time_ago_hours() {
    let dt = Utc::now() - Duration::hours(5);
    assert_eq!(human_time_ago(&dt), "5h ago");
}

#[test]
fn test_human_time_ago_days() {
    let dt = Utc::now() - Duration::days(10);
    assert_eq!(human_time_ago(&dt), "10d ago");
}

#[test]
fn test_human_time_ago_old() {
    let dt: DateTime<Utc> = DateTime::parse_from_rfc3339("2020-06-15T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let result = human_time_ago(&dt);
    assert_eq!(result, "Jun 15, 2020");
}

// ─── truncate ───

#[test]
fn test_truncate_short_string() {
    assert_eq!(truncate("hello", 10), "hello");
}

#[test]
fn test_truncate_long_string() {
    let result = truncate("hello world, this is long", 10);
    assert_eq!(result.chars().count(), 11); // 10 + "…"
    assert!(result.ends_with('…'));
    assert!(result.starts_with("hello worl"));
}

#[test]
fn test_truncate_exact_length() {
    assert_eq!(truncate("exactly10!", 10), "exactly10!");
}

#[test]
fn test_truncate_unicode() {
    let result = truncate("日本語のテスト文字列です", 5);
    assert_eq!(result.chars().count(), 6); // 5 + "…"
    assert!(result.ends_with('…'));
    assert!(result.starts_with("日本語のテ"));
}

// ─── delete_session ───

fn make_test_session(path: PathBuf) -> Session {
    Session {
        id: "test".to_string(),
        provider: "Test".to_string(),
        summary: "Test".to_string(),
        cwd: "/test".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        checkpoints: Vec::new(),
        user_messages: Vec::new(),
        task_summaries: Vec::new(),
        path,
    }
}

#[test]
fn test_delete_session_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("session.jsonl");
    fs::write(&file, "test content").unwrap();
    assert!(file.exists());

    let session = make_test_session(file.clone());
    delete_session(&session).unwrap();
    assert!(!file.exists());
}

#[test]
fn test_delete_session_directory() {
    let dir = tempfile::tempdir().unwrap();
    let session_dir = dir.path().join("session-dir");
    fs::create_dir_all(session_dir.join("checkpoints")).unwrap();
    fs::write(session_dir.join("workspace.yaml"), "id: test").unwrap();
    fs::write(
        session_dir.join("checkpoints").join("index.md"),
        "# Checkpoints",
    )
    .unwrap();
    assert!(session_dir.exists());

    let session = make_test_session(session_dir.clone());
    delete_session(&session).unwrap();
    assert!(!session_dir.exists());
}
