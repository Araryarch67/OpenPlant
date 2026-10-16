//! Command palette entries.

use super::*;

pub struct Command {
    pub id: &'static str,
    pub label: &'static str,
}

/// Palette list (shots/04): plain labels with the command id on the right.
pub fn all_commands() -> Vec<Command> {
    let mut out: Vec<Command> = vec![
        Command {
            id: "file.new",
            label: "New Project",
        },
        Command {
            id: "diagram.add-class",
            label: "Add Class to Canvas",
        },
        Command {
            id: "view.palette",
            label: "Command Palette...",
        },
        Command {
            id: "view.sidebar",
            label: "Toggle Sidebar",
        },
        Command {
            id: "view.navigator",
            label: "Toggle Navigator",
        },
    ];
    for m in MENUS {
        for i in m.items {
            if let Some(id) = i.cmd
                && !out.iter().any(|c| c.id == id)
            {
                out.push(Command { id, label: i.label });
            }
        }
    }
    // File commands lead, as in shots/04-palette.png.
    let lead = [
        "file.new",
        "file.open",
        "file.save",
        "file.export-png",
        "file.export-puml",
        "file.import-puml",
    ];
    out.sort_by_key(|c| lead.iter().position(|l| *l == c.id).unwrap_or(lead.len()));
    out
}

pub fn palette_matches(q: &str) -> Vec<Command> {
    all_commands()
        .into_iter()
        .filter(|c| c.label.to_lowercase().contains(q))
        .collect()
}
