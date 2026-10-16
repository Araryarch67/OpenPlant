//! Domain model: project, diagrams, shapes, relations, toolbox catalogue.
//! Projects save as camelCase JSON (same files the old web build wrote, so
//! existing `.json` projects still open).

mod color;
mod ids;
mod project;
mod relation;
mod shape;
pub(crate) mod toolbox;

pub(crate) use color::*;
pub(crate) use ids::*;
pub(crate) use project::*;
pub(crate) use relation::*;
pub(crate) use shape::*;

#[cfg(test)]
mod tests;
