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

/// Properties that a cell may have
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct Properties {
	inner: String
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
	Str(String),
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
	/// Create empty cell (Containing the number 0 and no display content)
	pub fn new() -> Self {
		Self { 
			content: Content::Number(0.0),
			display_content: "".to_string(),
			properties: Properties { inner: Default::default() }
		}
	}
}