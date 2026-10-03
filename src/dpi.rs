pub use ::dpi::*;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;
/// System theme.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Theme {
    /// Light theme.
    Light,
    /// Dark theme.
    Dark,
}

impl Serialize for Theme {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.to_lowercase().as_str() {
            "dark" => Self::Dark,
            _ => Self::Light,
        })
    }
}

impl Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Light => "light",
                Self::Dark => "dark",
            }
        )
    }
}
/// A rectangular region.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Rect {
    /// Rect position.
    pub position: dpi::Position,
    /// Rect size.
    pub size: dpi::Size,
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            position: Position::Logical((0, 0).into()),
            size: Size::Logical((0, 0).into()),
        }
    }
}

/// A rectangular region in physical pixels.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PhysicalRect<P: dpi::Pixel, S: dpi::Pixel> {
    /// Rect position.
    pub position: dpi::PhysicalPosition<P>,
    /// Rect size.
    pub size: dpi::PhysicalSize<S>,
}

impl<P: dpi::Pixel, S: dpi::Pixel> Default for PhysicalRect<P, S> {
    fn default() -> Self {
        Self {
            position: (0, 0).into(),
            size: (0, 0).into(),
        }
    }
}

/// A rectangular region in logical pixels.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LogicalRect<P: dpi::Pixel, S: dpi::Pixel> {
    /// Rect position.
    pub position: dpi::LogicalPosition<P>,
    /// Rect size.
    pub size: dpi::LogicalSize<S>,
}

impl<P: dpi::Pixel, S: dpi::Pixel> Default for LogicalRect<P, S> {
    fn default() -> Self {
        Self {
            position: (0, 0).into(),
            size: (0, 0).into(),
        }
    }
}
