use crate::session::{Session, delete_session, human_time_ago};

pub struct App {
    pub sessions: Vec<Session>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub preview_scroll: usize,
    pub search_query: String,
    pub search_active: bool,
    pub mode: Mode,
    pub confirm_delete: bool,
    pub should_quit: bool,
    pub resume_session: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Search,
}

impl App {
    pub fn new(sessions: Vec<Session>) -> Self {
        let filtered: Vec<usize> = (0..sessions.len()).collect();
        Self {
            sessions,
            filtered,
            selected: 0,
            preview_scroll: 0,
            search_query: String::new(),
            search_active: false,
            mode: Mode::Browse,
            confirm_delete: false,
            should_quit: false,
            resume_session: None,
        }
    }

    pub fn selected_session(&self) -> Option<&Session> {
        self.filtered
            .get(self.selected)
            .and_then(|&idx| self.sessions.get(idx))
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.preview_scroll = 0;
            self.confirm_delete = false;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.filtered.len() {
            self.selected += 1;
            self.preview_scroll = 0;
            self.confirm_delete = false;
        }
    }

    pub fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(3);
    }

    pub fn scroll_preview_down(&mut self) {
        self.preview_scroll += 3;
    }

    pub fn resume_selected(&mut self) {
        if let Some(session) = self.selected_session() {
            self.resume_session = Some(session.id.clone());
            self.should_quit = true;
        }
    }

    pub fn delete_selected(&mut self) {
        if !self.confirm_delete {
            self.confirm_delete = true;
            return;
        }

        let Some(&idx) = self.filtered.get(self.selected) else {
            return;
        };

        if delete_session(&self.sessions[idx]).is_ok() {
            self.sessions.remove(idx);
            self.apply_filter();
            if self.selected >= self.filtered.len() && self.selected > 0 {
                self.selected -= 1;
            }
        }
        self.confirm_delete = false;
    }

    pub fn enter_search(&mut self) {
        self.mode = Mode::Search;
        self.search_active = true;
    }

    pub fn exit_search(&mut self) {
        self.mode = Mode::Browse;
        self.search_active = false;
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.apply_filter();
        self.exit_search();
    }

    pub fn search_input(&mut self, c: char) {
        self.search_query.push(c);
        self.apply_filter();
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.apply_filter();
    }

    fn apply_filter(&mut self) {
        let query = self.search_query.to_lowercase();
        self.filtered = if query.is_empty() {
            (0..self.sessions.len()).collect()
        } else {
            self.sessions
                .iter()
                .enumerate()
                .filter(|(_, s)| {
                    s.summary.to_lowercase().contains(&query)
                        || s.cwd.to_lowercase().contains(&query)
                        || s.user_messages
                            .iter()
                            .any(|m| m.to_lowercase().contains(&query))
                })
                .map(|(i, _)| i)
                .collect()
        };

        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
        self.preview_scroll = 0;
    }

    pub fn build_preview_lines(&self) -> Vec<PreviewLine> {
        let Some(session) = self.selected_session() else {
            return vec![PreviewLine::dim("No session selected".into())];
        };

        let mut lines = Vec::new();

        lines.push(PreviewLine::header(session.summary.clone()));
        lines.push(PreviewLine::empty());

        let created = session.created_at.format("%b %d, %Y %H:%M").to_string();
        let updated = human_time_ago(&session.updated_at);
        lines.push(PreviewLine::label_value("Created", &created));
        lines.push(PreviewLine::label_value("Updated", &updated));
        lines.push(PreviewLine::label_value("Directory", &session.cwd));
        lines.push(PreviewLine::label_value("Session ID", &session.id));

        if !session.checkpoints.is_empty() {
            lines.push(PreviewLine::empty());
            lines.push(PreviewLine::section(format!(
                "── Checkpoints ({}) ──",
                session.checkpoints.len()
            )));
            for (i, cp) in session.checkpoints.iter().enumerate() {
                lines.push(PreviewLine::normal(format!("  {}. {}", i + 1, cp.title)));
            }
        }

        if !session.user_messages.is_empty() {
            lines.push(PreviewLine::empty());
            lines.push(PreviewLine::section(format!(
                "── Messages ({}) ──",
                session.user_messages.len()
            )));
            for msg in session.user_messages.iter().take(10) {
                lines.push(PreviewLine::dim(format!("  > {msg}")));
            }
            if session.user_messages.len() > 10 {
                lines.push(PreviewLine::dim(format!(
                    "  ... and {} more",
                    session.user_messages.len() - 10
                )));
            }
        }

        if !session.task_summaries.is_empty() {
            lines.push(PreviewLine::empty());
            lines.push(PreviewLine::section("── Completed Tasks ──".into()));
            for summary in &session.task_summaries {
                lines.push(PreviewLine::normal(format!("  ✓ {summary}")));
            }
        }

        lines
    }
}

#[derive(Debug, Clone)]
pub struct PreviewLine {
    pub text: String,
    pub style: LineStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    Header,
    Section,
    Label,
    Normal,
    Dim,
    Empty,
}

impl PreviewLine {
    pub fn header(text: String) -> Self {
        Self {
            text,
            style: LineStyle::Header,
        }
    }

    pub fn section(text: String) -> Self {
        Self {
            text,
            style: LineStyle::Section,
        }
    }

    pub fn label_value(label: &str, value: &str) -> Self {
        Self {
            text: format!("{label}: {value}"),
            style: LineStyle::Label,
        }
    }

    pub fn normal(text: String) -> Self {
        Self {
            text,
            style: LineStyle::Normal,
        }
    }

    pub fn dim(text: String) -> Self {
        Self {
            text,
            style: LineStyle::Dim,
        }
    }

    pub fn empty() -> Self {
        Self {
            text: String::new(),
            style: LineStyle::Empty,
        }
    }
}
