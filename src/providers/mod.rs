pub mod claude;
pub mod copilot;

use crate::session::Session;

pub trait Provider {
    fn name(&self) -> &str;
    fn discover_sessions(&self) -> Vec<Session>;
}

pub fn load_all_sessions() -> Vec<Session> {
    let providers: Vec<Box<dyn Provider>> = vec![
        Box::new(copilot::CopilotProvider),
        Box::new(claude::ClaudeCodeProvider),
    ];

    let mut sessions: Vec<Session> = providers
        .iter()
        .flat_map(|p| {
            let name = p.name().to_string();
            p.discover_sessions()
                .into_iter()
                .map(move |mut s| {
                    s.provider = name.clone();
                    s
                })
        })
        .collect();

    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    sessions
}
