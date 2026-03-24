use std::path::PathBuf;

use chrono::{Duration, Utc};
use recall_cli::app::{App, LineStyle, Mode, ProviderFilter};
use recall_cli::session::{Checkpoint, Session};

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
    assert_eq!(app.provider_filter, ProviderFilter::Codex);

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

fn make_session_with_details(
    id: &str,
    provider: &str,
    summary: &str,
    checkpoints: Vec<Checkpoint>,
    messages: Vec<String>,
    task_summaries: Vec<String>,
) -> Session {
    Session {
        id: id.to_string(),
        provider: provider.to_string(),
        summary: summary.to_string(),
        cwd: "/test/project".to_string(),
        created_at: Utc::now() - Duration::hours(2),
        updated_at: Utc::now(),
        checkpoints,
        user_messages: messages,
        task_summaries,
        path: PathBuf::from("/test"),
    }
}

// ─── Delete confirmation flow ───

#[test]
fn test_delete_selected_first_press_sets_confirm() {
    let sessions = vec![make_session("1", "Copilot", "Session 1")];
    let mut app = App::new(sessions);
    assert!(!app.confirm_delete);

    app.delete_selected();
    assert!(
        app.confirm_delete,
        "first delete press should set confirm flag"
    );
}

#[test]
fn test_delete_selected_cancel_on_move() {
    let sessions = vec![
        make_session("1", "Copilot", "Session 1"),
        make_session("2", "Copilot", "Session 2"),
    ];
    let mut app = App::new(sessions);

    app.delete_selected(); // set confirm
    assert!(app.confirm_delete);

    app.move_down(); // should cancel confirm
    assert!(!app.confirm_delete);
}

#[test]
fn test_delete_selected_empty_list() {
    let mut app = App::new(vec![]);
    // Should not panic
    app.delete_selected();
    app.delete_selected();
}

// ─── Resume selected ───

#[test]
fn test_resume_selected_returns_correct_tuple() {
    let sessions = vec![
        make_session("sess-abc", "Copilot", "First"),
        make_session("sess-def", "Claude Code", "Second"),
    ];
    let mut app = App::new(sessions);
    app.move_down();
    app.resume_selected();

    assert!(app.should_quit);
    let (id, provider) = app.resume_session.as_ref().unwrap();
    assert_eq!(id, "sess-def");
    assert_eq!(provider, "Claude Code");
}

#[test]
fn test_resume_selected_empty_list() {
    let mut app = App::new(vec![]);
    app.resume_selected();
    assert!(!app.should_quit);
    assert!(app.resume_session.is_none());
}

// ─── Preview with various states ───

#[test]
fn test_preview_lines_with_checkpoints() {
    let sessions = vec![make_session_with_details(
        "1",
        "Copilot",
        "Checkpoint Session",
        vec![
            Checkpoint {
                title: "Initial setup".to_string(),
            },
            Checkpoint {
                title: "Added tests".to_string(),
            },
            Checkpoint {
                title: "Fixed bug".to_string(),
            },
        ],
        vec!["msg1".to_string()],
        Vec::new(),
    )];
    let app = App::new(sessions);
    let lines = app.build_preview_lines();

    let has_cp_section = lines
        .iter()
        .any(|l| l.style == LineStyle::Section && l.text.contains("Checkpoints (3)"));
    assert!(has_cp_section, "should show checkpoints section");

    let cp_lines: Vec<_> = lines
        .iter()
        .filter(|l| l.style == LineStyle::Normal && l.text.contains('.'))
        .collect();
    assert_eq!(cp_lines.len(), 3);
}

#[test]
fn test_preview_lines_with_many_messages() {
    let messages: Vec<String> = (0..15).map(|i| format!("Message {i}")).collect();
    let sessions = vec![make_session_with_details(
        "1",
        "Copilot",
        "Many Messages",
        Vec::new(),
        messages,
        Vec::new(),
    )];
    let app = App::new(sessions);
    let lines = app.build_preview_lines();

    // Should show max 10 messages plus "... and 5 more"
    let msg_lines: Vec<_> = lines
        .iter()
        .filter(|l| l.style == LineStyle::Dim && l.text.starts_with("  >"))
        .collect();
    assert_eq!(msg_lines.len(), 10);

    let more_line = lines.iter().any(|l| l.text.contains("and 5 more"));
    assert!(more_line, "should show overflow count");
}

#[test]
fn test_preview_lines_with_task_summaries() {
    let sessions = vec![make_session_with_details(
        "1",
        "Copilot",
        "Task Session",
        Vec::new(),
        vec!["msg".to_string()],
        vec!["Fixed the bug".to_string(), "Updated docs".to_string()],
    )];
    let app = App::new(sessions);
    let lines = app.build_preview_lines();

    let has_task_section = lines
        .iter()
        .any(|l| l.style == LineStyle::Section && l.text.contains("Completed Tasks"));
    assert!(has_task_section);

    let task_lines: Vec<_> = lines.iter().filter(|l| l.text.contains('✓')).collect();
    assert_eq!(task_lines.len(), 2);
}

#[test]
fn test_preview_lines_empty_summary() {
    let mut session = make_session("1", "Copilot", "");
    session.summary = String::new();
    let sessions = vec![session];
    let app = App::new(sessions);
    let lines = app.build_preview_lines();

    // Header should be empty string
    assert_eq!(lines[0].style, LineStyle::Header);
    assert_eq!(lines[0].text, "");
}

// ─── Provider filter full cycle ───

#[test]
fn test_provider_filter_full_cycle_with_all_providers() {
    let sessions = vec![
        make_session("1", "Copilot", "Copilot session"),
        make_session("2", "Claude Code", "Claude session"),
        make_session("3", "Codex CLI", "Codex session"),
        make_session("4", "Copilot", "Another Copilot"),
    ];
    let mut app = App::new(sessions);
    assert_eq!(app.filtered.len(), 4);

    app.cycle_provider_filter(); // -> Copilot
    assert_eq!(app.provider_filter, ProviderFilter::Copilot);
    assert_eq!(app.filtered.len(), 2);

    app.cycle_provider_filter(); // -> Claude
    assert_eq!(app.provider_filter, ProviderFilter::Claude);
    assert_eq!(app.filtered.len(), 1);

    app.cycle_provider_filter(); // -> Codex
    assert_eq!(app.provider_filter, ProviderFilter::Codex);
    assert_eq!(app.filtered.len(), 1);
    assert_eq!(app.filtered[0], 2);

    app.cycle_provider_filter(); // -> All
    assert_eq!(app.provider_filter, ProviderFilter::All);
    assert_eq!(app.filtered.len(), 4);
}

#[test]
fn test_provider_filter_label() {
    assert_eq!(ProviderFilter::All.label(), "All");
    assert_eq!(ProviderFilter::Copilot.label(), "Copilot");
    assert_eq!(ProviderFilter::Claude.label(), "Claude Code");
    assert_eq!(ProviderFilter::Codex.label(), "Codex CLI");
}

// ─── Search + provider filter combined ───

#[test]
fn test_search_and_provider_filter_combined() {
    let sessions = vec![
        make_session("1", "Copilot", "Fix auth bug"),
        make_session("2", "Claude Code", "Fix auth design"),
        make_session("3", "Copilot", "Refactor database"),
        make_session("4", "Codex CLI", "Fix auth test"),
    ];
    let mut app = App::new(sessions);

    // Search for "auth"
    for c in "auth".chars() {
        app.search_input(c);
    }
    assert_eq!(app.filtered.len(), 3); // sessions 0, 1, 3

    // Now also filter to Copilot
    app.cycle_provider_filter();
    assert_eq!(app.provider_filter, ProviderFilter::Copilot);
    assert_eq!(app.filtered.len(), 1);
    assert_eq!(app.filtered[0], 0);

    // Switch to Codex
    app.cycle_provider_filter(); // Claude
    app.cycle_provider_filter(); // Codex
    assert_eq!(app.filtered.len(), 1);
    assert_eq!(app.filtered[0], 3);
}

#[test]
fn test_search_no_results() {
    let sessions = vec![make_session("1", "Copilot", "Hello world")];
    let mut app = App::new(sessions);

    for c in "zzzznotfound".chars() {
        app.search_input(c);
    }
    assert_eq!(app.filtered.len(), 0);
    assert_eq!(app.selected, 0);
}

#[test]
fn test_search_backspace() {
    let sessions = vec![
        make_session("1", "Copilot", "Fix authentication"),
        make_session("2", "Copilot", "Fix auth test"),
    ];
    let mut app = App::new(sessions);

    for c in "authentication".chars() {
        app.search_input(c);
    }
    assert_eq!(app.filtered.len(), 1);

    // Backspace enough to widen the search
    for _ in 0..8 {
        app.search_backspace();
    }
    assert_eq!(app.search_query, "authen");

    for _ in 0..2 {
        app.search_backspace();
    }
    assert_eq!(app.search_query, "auth");
    assert_eq!(app.filtered.len(), 2);
}

// ─── Navigation edge cases ───

#[test]
fn test_move_up_at_zero_stays() {
    let sessions = vec![make_session("1", "Copilot", "S1")];
    let mut app = App::new(sessions);
    assert_eq!(app.selected, 0);
    app.move_up();
    app.move_up();
    app.move_up();
    assert_eq!(app.selected, 0);
}

#[test]
fn test_move_down_at_last_stays() {
    let sessions = vec![
        make_session("1", "Copilot", "S1"),
        make_session("2", "Copilot", "S2"),
    ];
    let mut app = App::new(sessions);
    app.move_down();
    assert_eq!(app.selected, 1);
    app.move_down();
    app.move_down();
    assert_eq!(app.selected, 1);
}

#[test]
fn test_scroll_preview_down_up() {
    let sessions = vec![make_session("1", "Copilot", "S1")];
    let mut app = App::new(sessions);
    assert_eq!(app.preview_scroll, 0);

    app.scroll_preview_down();
    assert_eq!(app.preview_scroll, 3);

    app.scroll_preview_down();
    assert_eq!(app.preview_scroll, 6);

    app.scroll_preview_up();
    assert_eq!(app.preview_scroll, 3);

    app.scroll_preview_up();
    assert_eq!(app.preview_scroll, 0);

    // Can't go below 0
    app.scroll_preview_up();
    assert_eq!(app.preview_scroll, 0);
}

#[test]
fn test_navigation_resets_preview_scroll() {
    let sessions = vec![
        make_session("1", "Copilot", "S1"),
        make_session("2", "Copilot", "S2"),
    ];
    let mut app = App::new(sessions);
    app.scroll_preview_down();
    assert_eq!(app.preview_scroll, 3);

    app.move_down();
    assert_eq!(app.preview_scroll, 0, "moving should reset preview scroll");
}

// ─── Search mode ───

#[test]
fn test_enter_exit_search_mode() {
    let mut app = App::new(vec![]);
    assert_eq!(app.mode, Mode::Browse);
    assert!(!app.search_active);

    app.enter_search();
    assert_eq!(app.mode, Mode::Search);
    assert!(app.search_active);

    app.exit_search();
    assert_eq!(app.mode, Mode::Browse);
    assert!(!app.search_active);
}

#[test]
fn test_clear_search_exits_mode() {
    let sessions = vec![make_session("1", "Copilot", "S1")];
    let mut app = App::new(sessions);
    app.enter_search();
    app.search_input('a');
    assert_eq!(app.search_query, "a");

    app.clear_search();
    assert_eq!(app.search_query, "");
    assert_eq!(app.mode, Mode::Browse);
    assert!(!app.search_active);
}

// ─── Selected session ───

#[test]
fn test_selected_session_returns_correct() {
    let sessions = vec![
        make_session("1", "Copilot", "First"),
        make_session("2", "Claude Code", "Second"),
    ];
    let mut app = App::new(sessions);
    assert_eq!(app.selected_session().unwrap().id, "1");

    app.move_down();
    assert_eq!(app.selected_session().unwrap().id, "2");
}

#[test]
fn test_selected_session_empty() {
    let app = App::new(vec![]);
    assert!(app.selected_session().is_none());
}

// ─── Filter adjusts selection ───

#[test]
fn test_filter_adjusts_selected_when_out_of_bounds() {
    let sessions = vec![
        make_session("1", "Copilot", "S1"),
        make_session("2", "Claude Code", "S2"),
        make_session("3", "Copilot", "S3"),
    ];
    let mut app = App::new(sessions);

    // Select last item
    app.move_down();
    app.move_down();
    assert_eq!(app.selected, 2);

    // Filter to Claude (only 1 item) → selected should adjust
    app.cycle_provider_filter(); // Copilot (2 items)
    app.cycle_provider_filter(); // Claude (1 item)
    assert!(app.selected < app.filtered.len());
}
