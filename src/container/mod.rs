use std::error::Error;

use crate::{Cell, PositionN};
/// Map Container
pub mod mapcontainer;

/// Error that can be returned as per container operations
#[derive(Debug)]
pub enum ContainerError {
	/// Wrong coordinates
	CoordinateMismatch(String),
	/// Concurrency error
	Concurrency(String),
	/// Invalid number of axes
	InvalidAxesNumber(String),
	/// Out of bounds
	OutOfBounds(String),
}

/// An enum that determines if a whole axis is to be obtained (Any) or just a particular value of the axis (Position)
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum AnyPosition {
	/// Any position in the axis
	Any,
	/// A particular position in the axis
	Position(usize)
}

impl std::fmt::Display for ContainerError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let wr = match self {
			Self::CoordinateMismatch(str) => format!("Coordinate mismatch error: {}", str),
			Self::Concurrency(str) => format!("Concurrency error: {}", str),
			Self::InvalidAxesNumber(str) => format!("Invalid axes error: {str}"),
			Self::OutOfBounds(str) => format!("Out of bounds: {str}")
		};
		write!(f, "{}", wr)
	}
}

impl Error for ContainerError{}

type RetError = Box<dyn Error>;
/// Trait that all containers must implement. Use it instead of a MapContainer
pub trait Container : Iterator<Item = (Vec<usize>, Cell)> {
	/// Insert cell at position p
	fn insert(&self, p: &PositionN, c: Cell) -> Result<(), RetError>;
	/// Removes cell at position p
	fn remove(&self, p: &PositionN) -> Result<(), RetError>;
	/// Gets cells at axis given by p
	fn get_cells_at_axis(&self, p: &[AnyPosition]) -> Result<Vec<Cell>, RetError>;
	/// Gets cell at position p
	fn get_cell_at(&self, p: &PositionN) -> Result<Option<Cell>, RetError>;
	/// Gets cells at positions p
	fn get_cells_at(&self, p: &[PositionN]) -> Result<Vec<Cell>, RetError>;
	/// Gets whether a cell exists or not
	fn is_empty_cell(&self, p: &PositionN) -> bool;
	/// Sets cell at position p
	fn set_cell_at(&self, p: &PositionN, nc: Cell) -> Result<(), RetError>;
	/// Sets number of axes. All current data is appended the extra axes
	fn set_axes(&self, naxes: usize) -> Result<(), RetError>;
	/// Get number axes
	fn get_axes(&self) -> usize;
	/// Get number of cells
	fn get_n_cells(&self) -> usize;
	/// Remove an axis. Retains all elements in retain_coord (or coordinate 0 if None)
	fn remove_axis(&self, axispos: usize, retain_coord: Option<usize>) -> Result<(), RetError>;
}