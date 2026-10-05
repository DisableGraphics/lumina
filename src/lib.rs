//! Multidimensional spreadsheet engine
#![warn(missing_docs)]
use serde::{Deserialize, Serialize};

use crate::formula::FormulaAst;
/// Container
pub mod container;
/// Formula
pub mod formula;
/// IO
pub mod io;

/// Color structure
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct Color {
	/// red
	pub r: u8,
	/// green
	pub g: u8,
	/// blue
	pub b: u8
}

impl Color {
	/// Create a color from a tuple of 3 RGB values
	pub fn from_tuple(t: (u8,u8,u8)) -> Self {
		Self { r: t.0, g: t.1, b: t.2 }
	}
}

/// Properties that a cell may have
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct Properties {
	bgcolor: Color,
	fgcolor: Color,
	fontsize: usize,
	font: String
}

/// Position
pub type PositionN = Vec<usize>;

/// The content of a cell
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, PartialOrd)]
pub enum Content {
	/// Contains a compiled formula
	Formula(FormulaAst),
	/// Contains a number
	Number(f64),
	/// Contains a string
	Str(String)
}

/// A basic cell
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct Cell {
	content: Content,
	display_content: String,
	properties: Properties
}

impl Default for Cell {
	fn default() -> Self {
		Self::new()
	}
}

impl Cell {
	/// Create empty
	pub fn new() -> Self {
		Self { 
			content: Content::Number(0.0),
			display_content: "".to_string(),
			properties: Properties { bgcolor: Color::from_tuple((0,0,0)), fgcolor: Color::from_tuple((0,0,0)), fontsize: 0, font: "".to_string() }
		}
	}
}