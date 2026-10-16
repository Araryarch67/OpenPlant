//! PlantUML subset: parse into shapes/relations (with grouped layout) and
//! generate back.

mod generate;
mod layout;
mod lexicon;
mod parse;

pub(crate) use generate::*;
pub(crate) use layout::*;
pub(crate) use lexicon::*;
pub(crate) use parse::*;

use crate::model::{Relation, Shape};

pub struct ParsedDiagram {
    pub shapes: Vec<Shape>,
    pub relations: Vec<Relation>,
    /// Flattened title (diagram name).
    pub title: Option<String>,
    /// Title as written, may span lines.
    pub title_text: Option<String>,
    pub preamble: Vec<String>,
    pub legend: Vec<String>,
}

#[cfg(test)]
mod tests;
