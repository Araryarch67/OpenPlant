//! Command registry (stable ids, dispatched from menus, palette and keys)
//! and the draw.io-compatible keymap.

mod actions;
mod keymap;
mod menus;
mod palette;
mod shortcuts;

pub(crate) use actions::*;
pub(crate) use keymap::*;
pub(crate) use menus::*;
pub(crate) use palette::*;
pub(crate) use shortcuts::*;
