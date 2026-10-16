//! Element id generation.

use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) static SEQ: AtomicU64 = AtomicU64::new(100);

/// Fresh id with a prefix; bumped past ids found in loaded projects.
pub fn nid(prefix: &str) -> String {
    format!("{prefix}{}", SEQ.fetch_add(1, Ordering::Relaxed) + 1)
}

/// Keep generated ids unique after opening a project saved elsewhere.
pub fn bump_ids(project: &Project) {
    let mut max = 0u64;
    let mut see = |id: &str| {
        let digits: String = id
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(n) = digits.chars().rev().collect::<String>().parse::<u64>() {
            max = max.max(n);
        }
    };
    for d in project.diagrams.values() {
        see(&d.id);
        d.shapes.iter().for_each(|s| see(&s.id));
        d.relations.iter().for_each(|r| see(&r.id));
    }
    SEQ.fetch_max(max + 1, Ordering::Relaxed);
}
