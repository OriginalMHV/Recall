mod app;
mod providers;
mod session;
mod ui;

use std::io;
use std::time::Duration;

use app::{App, Mode, ProviderFilter};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{ExecutableCommand, execute};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

#[derive(Parser)]
#[command(name = "recall", about = "TUI session browser for AI CLI tools")]
struct Cli {
    /// List sessions as plain text (no TUI)
    #[arg(long)]
    list: bool,

    /// Show session count only
    #[arg(long)]
    count: bool,

    /// Filter by provider name (copilot, claude)
    #[arg(long)]
    provider: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut sessions = providers::load_all_sessions();

    if let Some(ref filter) = cli.provider {
        let filter_lower = filter.to_lowercase();
        sessions.retain(|s| s.provider.to_lowercase().contains(&filter_lower));
    }

    if cli.count {
        println!("{}", sessions.len());
        return Ok(());
    }

    if cli.list {
        for s in &sessions {
            let age = session::human_time_ago(&s.updated_at);
            let msgs = s.user_messages.len();
            let cps = s.checkpoints.len();
            let id_short = if s.id.len() >= 8 { &s.id[..8] } else { &s.id };
            println!(
                "{id_short} | {} | {} | {age} | {msgs} msgs | {cps} checkpoints",
                s.provider,
                if s.summary.is_empty() {
                    "(untitled)"
                } else {
                    &s.summary
                }
            );
        }
        return Ok(());
    }

    if sessions.is_empty() {
        println!("No AI CLI sessions found.");
        println!("Supported: ~/.copilot/session-state/ (Copilot), ~/.claude/projects/ (Claude Code)");
        return Ok(());
    }

    let provider_filter = cli.provider.as_deref().map(|p| {
        let lower = p.to_lowercase();
        if lower.contains("claude") {
            ProviderFilter::Claude
        } else if lower.contains("copilot") {
            ProviderFilter::Copilot
        } else {
            ProviderFilter::All
        }
    });

    let resume_id = run_tui(sessions, provider_filter)?;

    if let Some(session_id) = resume_id {
        println!("Resuming session: {session_id}");
        let status = std::process::Command::new("copilot")
            .args(["--resume", &session_id])
            .status()?;
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn run_tui(
    sessions: Vec<session::Session>,
    provider_filter: Option<ProviderFilter>,
) -> anyhow::Result<Option<String>> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new(sessions);

    if let Some(filter) = provider_filter {
        app.provider_filter = filter;
        app.apply_filter();
    }

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_key(&mut app, key);
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(app.resume_session)
}

fn handle_key(app: &mut App, key: KeyEvent) {
    match app.mode {
        Mode::Search => handle_search_key(app, key),
        Mode::Browse => handle_browse_key(app, key),
    }
}

fn handle_browse_key(app: &mut App, key: KeyEvent) {
    if app.confirm_delete {
        match key.code {
            KeyCode::Char('d') => app.delete_selected(),
            KeyCode::Esc => app.confirm_delete = false,
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Up | KeyCode::Char('k') => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.scroll_preview_up();
            } else {
                app.move_up();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.scroll_preview_down();
            } else {
                app.move_down();
            }
        }
        KeyCode::Enter => app.resume_selected(),
        KeyCode::Char('d') => app.delete_selected(),
        KeyCode::Char('/') => app.enter_search(),
        KeyCode::Tab => app.cycle_provider_filter(),
        KeyCode::Left => app.scroll_preview_up(),
        KeyCode::Right => app.scroll_preview_down(),
        KeyCode::Home | KeyCode::Char('g') => {
            app.selected = 0;
            app.preview_scroll = 0;
        }
        KeyCode::End | KeyCode::Char('G') => {
            if !app.filtered.is_empty() {
                app.selected = app.filtered.len() - 1;
                app.preview_scroll = 0;
            }
        }
        _ => {}
    }
}

fn handle_search_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.exit_search(),
        KeyCode::Enter => app.exit_search(),
        KeyCode::Backspace => app.search_backspace(),
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => app.clear_search(),
        KeyCode::Char(c) => app.search_input(c),
        _ => {}
    }
}
