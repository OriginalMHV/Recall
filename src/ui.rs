use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, List, ListItem, ListState, Padding, Paragraph, Wrap,
};

use crate::app::{App, LineStyle, Mode};
use crate::session::human_time_ago;

const OLIVE: Color = Color::Rgb(107, 142, 35);
const BURNT: Color = Color::Rgb(204, 85, 0);
const GOLD: Color = Color::Rgb(218, 165, 32);
const DIM: Color = Color::Rgb(120, 120, 120);
const SURFACE: Color = Color::Rgb(30, 30, 30);

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    let main_area = chunks[0];
    let status_bar = chunks[1];

    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_area);

    draw_session_list(f, app, panels[0]);
    draw_preview(f, app, panels[1]);
    draw_status_bar(f, app, status_bar);

    if app.confirm_delete {
        draw_delete_confirm(f, app);
    }
}

fn draw_session_list(f: &mut Frame, app: &App, area: Rect) {
    let title = if app.search_active || !app.search_query.is_empty() {
        format!(
            " Sessions ({}/{}) ▸ {} ",
            app.filtered.len(),
            app.sessions.len(),
            app.search_query
        )
    } else {
        format!(" Sessions ({}) ", app.filtered.len())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.mode == Mode::Search {
            GOLD
        } else {
            OLIVE
        }))
        .title(title)
        .title_style(Style::default().fg(OLIVE).add_modifier(Modifier::BOLD))
        .padding(Padding::horizontal(1));

    let items: Vec<ListItem> = app
        .filtered
        .iter()
        .enumerate()
        .map(|(display_idx, &session_idx)| {
            let session = &app.sessions[session_idx];
            let age = human_time_ago(&session.updated_at);
            let cwd = shorten_path(&session.cwd, 30);

            let summary = if session.summary.is_empty() {
                session
                    .user_messages
                    .first()
                    .map(|m| truncate_str(m, 40))
                    .unwrap_or_else(|| "(empty)".to_string())
            } else {
                truncate_str(&session.summary, 40)
            };

            let is_selected = display_idx == app.selected;

            let title_style = if is_selected {
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let meta_style = Style::default().fg(DIM);

            let checkpoints = if session.checkpoints.is_empty() {
                String::new()
            } else {
                format!(" · {}cp", session.checkpoints.len())
            };

            let messages = if session.user_messages.is_empty() {
                String::new()
            } else {
                format!(" · {}msg", session.user_messages.len())
            };

            ListItem::new(vec![
                Line::from(Span::styled(summary, title_style)),
                Line::from(Span::styled(
                    format!("{age} · {cwd}{checkpoints}{messages}"),
                    meta_style,
                )),
                Line::from(""),
            ])
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected));

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(SURFACE));

    f.render_stateful_widget(list, area, &mut state);
}

fn draw_preview(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(OLIVE))
        .title(" Preview ")
        .title_style(Style::default().fg(OLIVE).add_modifier(Modifier::BOLD))
        .padding(Padding::new(2, 2, 1, 1));

    let preview_lines = app.build_preview_lines();
    let inner = block.inner(area);

    let lines: Vec<Line> = preview_lines
        .iter()
        .skip(app.preview_scroll)
        .map(|pl| {
            let style = match pl.style {
                LineStyle::Header => Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
                LineStyle::Section => Style::default().fg(BURNT).add_modifier(Modifier::BOLD),
                LineStyle::Label => Style::default().fg(OLIVE),
                LineStyle::Normal => Style::default().fg(Color::White),
                LineStyle::Dim => Style::default().fg(DIM),
                LineStyle::Empty => Style::default(),
            };
            Line::from(Span::styled(&pl.text, style))
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);

    let total = preview_lines.len();
    if total > inner.height as usize {
        let scrollbar_text = format!(
            " {}/{} ",
            app.preview_scroll + 1,
            total.saturating_sub(inner.height as usize) + 1
        );
        let scroll_indicator = Paragraph::new(scrollbar_text)
            .style(Style::default().fg(DIM))
            .alignment(Alignment::Right);
        let indicator_area = Rect::new(area.x + area.width - 12, area.y, 12, 1);
        f.render_widget(scroll_indicator, indicator_area);
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let keys = if app.mode == Mode::Search {
        vec![("Esc", "cancel"), ("Enter", "confirm"), ("Ctrl+U", "clear")]
    } else {
        vec![
            ("↑↓", "navigate"),
            ("Enter", "resume"),
            ("d", "delete"),
            ("/", "search"),
            ("Shift+↑↓", "scroll preview"),
            ("q", "quit"),
        ]
    };

    let spans: Vec<Span> = keys
        .iter()
        .enumerate()
        .flat_map(|(i, (key, action))| {
            let mut s = vec![
                Span::styled(
                    format!(" {key} "),
                    Style::default().fg(Color::Black).bg(OLIVE),
                ),
                Span::styled(format!(" {action} "), Style::default().fg(DIM)),
            ];
            if i < keys.len() - 1 {
                s.push(Span::styled(" ", Style::default()));
            }
            s
        })
        .collect();

    let bar = Paragraph::new(Line::from(spans));
    f.render_widget(bar, area);
}

fn draw_delete_confirm(f: &mut Frame, app: &App) {
    let Some(session) = app.selected_session() else {
        return;
    };

    let area = centered_rect(50, 7, f.area());
    f.render_widget(Clear, area);

    let summary = truncate_str(&session.summary, 40);
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("Delete \"{summary}\"?"),
            Style::default().fg(BURNT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(" d ", Style::default().fg(Color::Black).bg(BURNT)),
            Span::styled(" confirm  ", Style::default().fg(DIM)),
            Span::styled(" Esc ", Style::default().fg(Color::Black).bg(OLIVE)),
            Span::styled(" cancel", Style::default().fg(DIM)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BURNT))
        .title(" Confirm ")
        .title_style(Style::default().fg(BURNT).add_modifier(Modifier::BOLD));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn shorten_path(path: &str, max: usize) -> String {
    if path.len() <= max {
        return path.to_string();
    }
    let home = dirs::home_dir()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_default();
    let shortened = path.replace(&home, "~");
    if shortened.len() <= max {
        return shortened;
    }
    format!("…{}", &shortened[shortened.len() - max + 1..])
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{truncated}…")
    }
}
