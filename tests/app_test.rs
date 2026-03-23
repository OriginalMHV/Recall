use std::path::PathBuf;

use chrono::Utc;
use recall_cli::app::{App, LineStyle, ProviderFilter};
use recall_cli::session::Session;

fn make_session(id: &str, provider: &str, summary: &str) -> Session {
    Session {
        id: id.to_string(),
        provider: provider.to_string(),
        summary: summary.to_string(),
        cwd: "/test/project".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        checkpoints: Vec::new(),
        user_messages: vec!["test message".to_string()],
        task_summaries: Vec::new(),
        path: PathBuf::from("/test"),
    }
}

// ─── Provider filter ───

#[test]
fn test_provider_filter_cycle() {
    let mut app = App::new(vec![]);
    assert_eq!(app.provider_filter, ProviderFilter::All);

    app.cycle_provider_filter();
    assert_eq!(app.provider_filter, ProviderFilter::Copilot);

    app.cycle_provider_filter();
    assert_eq!(app.provider_filter, ProviderFilter::Claude);

    app.cycle_provider_filter();
    assert_eq!(app.provider_filter, ProviderFilter::All);
}

#[test]
fn test_provider_filter_applies() {
    let sessions = vec![
        make_session("1", "Copilot", "Copilot session"),
        make_session("2", "Claude Code", "Claude session"),
        make_session("3", "Copilot", "Another Copilot session"),
    ];
    let mut app = App::new(sessions);
    assert_eq!(app.filtered.len(), 3);

    // Cycle to Copilot
    app.cycle_provider_filter();
    assert_eq!(app.provider_filter, ProviderFilter::Copilot);
    assert_eq!(app.filtered.len(), 2);
    assert_eq!(app.filtered, vec![0, 2]);
}

// ─── Search ───

#[test]
fn test_search_filter() {
    let mut sessions = vec![
        make_session("1", "Copilot", "Fix authentication bug"),
        make_session("2", "Claude Code", "Refactor database"),
    ];
    // Add a cwd-based match
    sessions[1].cwd = "/Users/test/auth-service".to_string();

    let mut app = App::new(sessions);

    for c in "auth".chars() {
        app.search_input(c);
    }

    // Should match session 0 (summary) and session 1 (cwd)
    assert_eq!(app.filtered.len(), 2);

    // Clear and search for something only in messages
    app.clear_search();
    assert_eq!(app.filtered.len(), 2); // all visible again

    for c in "test message".chars() {
        app.search_input(c);
    }
    // Both sessions have "test message" in user_messages
    assert_eq!(app.filtered.len(), 2);

    // Search for something unique
    app.clear_search();
    for c in "database".chars() {
        app.search_input(c);
    }
    assert_eq!(app.filtered.len(), 1);
    assert_eq!(app.filtered[0], 1);
}

#[test]
fn test_combined_filter() {
    let sessions = vec![
        make_session("1", "Copilot", "Fix authentication"),
        make_session("2", "Claude Code", "Fix authentication"),
        make_session("3", "Copilot", "Refactor database"),
    ];
    let mut app = App::new(sessions);

    // Set provider to Copilot
    app.cycle_provider_filter();
    assert_eq!(app.filtered.len(), 2); // sessions 0 and 2

    // Now search for "auth"
    for c in "auth".chars() {
        app.search_input(c);
    }
    assert_eq!(app.filtered.len(), 1);
    assert_eq!(app.filtered[0], 0); // only Copilot + "auth"
}

// ─── Navigation ───

#[test]
fn test_navigation() {
    let sessions = vec![
        make_session("1", "Copilot", "Session 1"),
        make_session("2", "Copilot", "Session 2"),
        make_session("3", "Copilot", "Session 3"),
    ];
    let mut app = App::new(sessions);

    assert_eq!(app.selected, 0);

    app.move_down();
    assert_eq!(app.selected, 1);

    app.move_down();
    assert_eq!(app.selected, 2);

    // Can't go past the last item
    app.move_down();
    assert_eq!(app.selected, 2);

    app.move_up();
    assert_eq!(app.selected, 1);

    app.move_up();
    assert_eq!(app.selected, 0);

    // Can't go above the first item
    app.move_up();
    assert_eq!(app.selected, 0);
}

#[test]
fn test_navigation_empty_list() {
    let mut app = App::new(vec![]);
    // Should not panic on empty list
    app.move_down();
    app.move_up();
    assert_eq!(app.selected, 0);
}

// ─── Preview ───

#[test]
fn test_preview_lines() {
    let sessions = vec![make_session("1", "Copilot", "Test Session")];
    let app = App::new(sessions);

    let lines = app.build_preview_lines();
    assert!(!lines.is_empty());

    // First line should be header with summary
    assert_eq!(lines[0].style, LineStyle::Header);
    assert_eq!(lines[0].text, "Test Session");

    // Should have label lines for Provider, Created, Updated, Directory, Session ID
    let label_lines: Vec<_> = lines
        .iter()
        .filter(|l| l.style == LineStyle::Label)
        .collect();
    assert!(label_lines.len() >= 4);

    let has_provider = label_lines.iter().any(|l| l.text.contains("Copilot"));
    assert!(has_provider, "should include provider info");

    let has_directory = label_lines.iter().any(|l| l.text.contains("/test/project"));
    assert!(has_directory, "should include directory");

    // Should have messages section
    let has_messages_section = lines
        .iter()
        .any(|l| l.style == LineStyle::Section && l.text.contains("Messages"));
    assert!(has_messages_section, "should have Messages section");
}

#[test]
fn test_preview_lines_no_session() {
    let app = App::new(vec![]);
    let lines = app.build_preview_lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].style, LineStyle::Dim);
    assert!(lines[0].text.contains("No session selected"));
}
