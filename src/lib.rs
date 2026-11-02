//! Haunt's decision log as a plugin: the ⌘K `log` mode, searching the core's verified decisions.
// wit-bindgen's flattened `notify` import has 14 params.
#![allow(clippy::too_many_arguments)]

wit_bindgen::generate!({ path: "wit/haunt-plugin.wit", world: "plugin" });

use haunt::plugin::decisions::{self, Decision, Outcome};
use haunt::plugin::palette;
use haunt::plugin::types::Mark;

pub const MODE: &str = "log";
const LIMIT: u32 = 100;

struct Plugin;

fn phrase(o: &Outcome) -> String {
    match o {
        Outcome::AllowedInHaunt => "You approved".into(),
        Outcome::DeniedInHaunt => "You denied".into(),
        Outcome::AllowedByPlugin(id) => format!("{id} approved"),
        Outcome::DeniedByPlugin(id) => format!("{id} denied"),
        Outcome::AnsweredInTerminal => "Allowed in terminal:".into(),
        Outcome::Superseded => "Superseded:".into(),
    }
}

fn mark(o: &Outcome) -> Mark {
    match o {
        Outcome::AllowedInHaunt | Outcome::DeniedInHaunt | Outcome::AnsweredInTerminal => Mark::Open,
        Outcome::AllowedByPlugin(_) | Outcome::DeniedByPlugin(_) => Mark::Filled,
        Outcome::Superseded => Mark::Round,
    }
}

/// The tab it happened in, named as haunt names rows (the cwd's last component), then the agent;
/// Claude is left implicit.
fn place(d: &Decision) -> String {
    let tab = d.cwd.as_deref().and_then(|c| c.trim_end_matches('/').rsplit('/').next()).filter(|s| !s.is_empty());
    let agent = match d.agent.as_str() {
        "claude" => None,
        "codex" => Some("Codex"),
        other => Some(other),
    };
    [tab, agent].into_iter().flatten().collect::<Vec<_>>().join(" · ")
}

pub fn row(i: usize, d: &Decision) -> PaletteItem {
    let what = if d.tool == "Bash" { d.preview.clone() } else { format!("{} {}", d.tool, d.preview) };
    PaletteItem {
        id: format!("{}-{i}", d.at_ms),
        title: format!("{} {what}", phrase(&d.outcome)),
        subtitle: Some(place(d)).filter(|s| !s.is_empty()),
        at_ms: Some(d.at_ms),
        mark: Some(mark(&d.outcome)),
        surface: Some(d.surface_id.clone()),
    }
}

impl Guest for Plugin {
    fn init(_settings: String) {
        palette::mode(MODE, "Search the decision log");
    }

    fn on_settings_changed(_settings: String) {}

    fn on_event(_event: AgentEvent) -> Option<Decoration> {
        None
    }

    fn palette_items(mode: String, query: String) -> Vec<PaletteItem> {
        if mode != MODE {
            return Vec::new();
        }
        decisions::query(&query, LIMIT).iter().enumerate().map(|(i, d)| row(i, d)).collect()
    }

    fn on_action(_action_id: String, _payload: String) {}

    fn row_menu(_surface: String) -> Vec<MenuItem> {
        Vec::new()
    }

    fn on_row_moved(_surface: String, _group: String, _before: Option<String>) {}
}

export!(Plugin);

#[cfg(test)]
mod tests {
    use super::*;

    fn decision(tool: &str, preview: &str, outcome: Outcome, agent: &str) -> Decision {
        Decision {
            at_ms: 1_700_000_000_000,
            surface_id: "s1".into(),
            tool: tool.into(),
            preview: preview.into(),
            cwd: Some("/Users/me/code/vibes".into()),
            outcome,
            agent: agent.into(),
        }
    }

    #[test]
    fn rows_carry_outcome_tool_preview_agent_time_and_tab() {
        let r = row(0, &decision("Bash", "git push", Outcome::AllowedInHaunt, "claude"));
        assert_eq!(r.title, "You approved git push");
        assert_eq!(r.subtitle.as_deref(), Some("vibes"));
        assert_eq!((r.at_ms, r.mark, r.surface.as_deref()), (Some(1_700_000_000_000), Some(Mark::Open), Some("s1")));

        let r = row(1, &decision("Edit", "/w/a.rs", Outcome::DeniedByPlugin("dev.x.y".into()), "codex"));
        assert_eq!(r.title, "dev.x.y denied Edit /w/a.rs");
        assert_eq!((r.subtitle.as_deref(), r.mark), (Some("vibes · Codex"), Some(Mark::Filled)));
    }

    #[test]
    fn lapsed_and_terminal_answers_read_as_such() {
        let r = row(0, &decision("Bash", "ls", Outcome::Superseded, "aider"));
        assert_eq!((r.title.as_str(), r.mark, r.subtitle.as_deref()), ("Superseded: ls", Some(Mark::Round), Some("vibes · aider")));
        let r = row(0, &decision("Bash", "ls", Outcome::AnsweredInTerminal, "claude"));
        assert_eq!(r.title, "Allowed in terminal: ls");
    }

    #[test]
    fn a_decision_without_a_cwd_has_no_place() {
        let r = row(0, &Decision { cwd: None, ..decision("Bash", "ls", Outcome::AllowedInHaunt, "claude") });
        assert_eq!(r.subtitle, None);
    }

    #[test]
    fn manifest_asks_only_for_the_decision_log() {
        let m = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/haunt-plugin.toml")).unwrap();
        assert!(m.contains("decisions_read = true"), "{m}");
        for cap in ["events", "pty_write", "notifications", "fs_read", "fs_write", "allowlist_suggest"] {
            assert!(!m.contains(cap), "{cap}");
        }
    }
}
