use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Duration, Utc};
use recall_cli::session::{Checkpoint, Session, delete_session, human_time_ago, truncate};
use recall_cli::ui::shorten_path;

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

// ─── truncate edge cases ───

#[test]
fn test_truncate_empty_string() {
    assert_eq!(truncate("", 10), "");
}

#[test]
fn test_truncate_max_zero() {
    let result = truncate("hello", 0);
    assert_eq!(result, "…");
}

#[test]
fn test_truncate_single_char_max() {
    let result = truncate("hello", 1);
    assert_eq!(result, "h…");
}

// ─── human_time_ago edge cases ───

#[test]
fn test_human_time_ago_future_timestamp() {
    let dt = Utc::now() + Duration::hours(5);
    // Future timestamps produce negative durations, so num_minutes() < 1 → "just now"
    assert_eq!(human_time_ago(&dt), "just now");
}

#[test]
fn test_human_time_ago_exactly_one_minute() {
    let dt = Utc::now() - Duration::minutes(1);
    assert_eq!(human_time_ago(&dt), "1m ago");
}

#[test]
fn test_human_time_ago_exactly_one_hour() {
    let dt = Utc::now() - Duration::hours(1);
    assert_eq!(human_time_ago(&dt), "1h ago");
}

#[test]
fn test_human_time_ago_exactly_one_day() {
    let dt = Utc::now() - Duration::days(1);
    assert_eq!(human_time_ago(&dt), "1d ago");
}

#[test]
fn test_human_time_ago_boundary_29_days() {
    let dt = Utc::now() - Duration::days(29);
    assert_eq!(human_time_ago(&dt), "29d ago");
}

#[test]
fn test_human_time_ago_boundary_30_days() {
    let dt = Utc::now() - Duration::days(30);
    // 30 days → falls into the date-format branch
    let result = human_time_ago(&dt);
    assert!(
        !result.ends_with("ago"),
        "30d should show date, not relative"
    );
}

// ─── delete_session edge cases ───

#[test]
fn test_delete_session_nonexistent_path() {
    let session = make_test_session(PathBuf::from("/nonexistent/path/to/session.jsonl"));
    let result = delete_session(&session);
    assert!(result.is_err(), "deleting nonexistent path should error");
}

// ─── shorten_path ───

#[test]
fn test_shorten_path_short() {
    let result = shorten_path("/Users/test", 50);
    assert_eq!(result, "/Users/test");
}

#[test]
fn test_shorten_path_long_gets_ellipsis() {
    let long = "/Users/test/very/deeply/nested/project/directory/structure";
    let result = shorten_path(long, 20);
    assert!(result.starts_with('…'));
    assert!(result.chars().count() <= 20);
}

#[test]
fn test_shorten_path_empty() {
    assert_eq!(shorten_path("", 10), "");
}
