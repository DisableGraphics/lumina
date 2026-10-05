use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::{PositionN, container::ContainerError};
/// Parser
pub mod parser;
/// Executor
pub mod exec;
/// Builtin functions
pub mod builtins;

/// Error generated while executing or parsing a formula
#[derive(Debug)]
pub enum FormulaError {
	/// Wrong type
	TypeError(String),
	/// Wrong syntax
	SyntaxError(String),
	/// Out of bounds
	BoundsError(String),
	/// Other container error
	ContainerError(ContainerError),
	/// Overlaps with another range
	OverlapError(String),
	/// Incorrect evaluation
	WrongEvaluation(String),
	/// Non-existent macro
	MacroDoesNotExist(String),
	/// Length is incorrect
	LengthError(String)
}

impl Error for FormulaError{}
impl std::fmt::Display for FormulaError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match &self {
			Self::TypeError(str) => str,
			Self::SyntaxError(str) => str,
			Self::BoundsError(str) => str,
			Self::ContainerError(str) => &str.to_string(),
			Self::OverlapError(str) => str,
			Self::WrongEvaluation(str) => str,
			Self::MacroDoesNotExist(str) => str,
			Self::LengthError(str) => str,
		};
		write!(f, "{}", s)
	}
}

impl From<ContainerError> for FormulaError {
	fn from(value: ContainerError) -> Self {
		Self::ContainerError(value)
	}
}

/// Formula Abstract Syntax Tree
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub enum FormulaAst {
	/// Range
	Range(Vec<PositionN>),
	/// Number
	Number(f64),
	/// Negation
	Neg(Box<FormulaAst>),
	/// Addition
    Add(Box<FormulaAst>, Box<FormulaAst>),
	/// Substraction
    Sub(Box<FormulaAst>, Box<FormulaAst>),
	/// Multiplication
    Mul(Box<FormulaAst>, Box<FormulaAst>),
	/// Division
    Div(Box<FormulaAst>, Box<FormulaAst>),
	/// Modulo
	Mod(Box<FormulaAst>, Box<FormulaAst>),
	/// String
	Str(String),
	/// Macro Call + arguments
	MacroCall(String, Vec<Self>),
}