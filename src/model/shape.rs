//! Shapes: kinds, style, geometry helpers.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShapeKind {
    Class,
    Interface,
    Actor,
    Database,
    Package,
    Note,
    Component,
    Box,
    Usecase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dash {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Right,
    Left,
    Top,
    Bottom,
}

impl Side {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Right => "right",
            Self::Left => "left",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dash: Option<Dash>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
}

pub const DEFAULT_FILL: &str = "#ffffff";

pub const DEFAULT_STROKE: &str = "#777777";

pub const DEFAULT_TEXT: &str = "#111111";

pub const DEFAULT_FONT: &str = "Arial, sans-serif";

pub const DEFAULT_FONT_SIZE: f32 = 14.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shape {
    pub id: String,
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<ShapeKind>,
    /// Original toolbox label (e.g. "Enumeration", "Queue"); drives the
    /// PlantUML keyword and the exact glyph drawn.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_side: Option<Side>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_of: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ShapeStyle>,
}

impl Shape {
    pub fn kind(&self) -> ShapeKind {
        self.kind.unwrap_or(ShapeKind::Class)
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}
