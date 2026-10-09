use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// ISBLANK - Checks if a single cell is empty (blank).
///
/// This function takes a single-cell range and returns 1 if the cell is empty,
/// 0 if it contains any value (including formulas, numbers, or strings).
///
/// # Arguments
/// * `args[0]` - A `Range` containing exactly one cell to check
///
/// # Returns
/// * `Number` - 1.0 if the cell is empty, 0.0 otherwise
///
/// # Errors
/// * `LengthError` - If not exactly 1 argument provided, or if the range contains more than one cell
/// * `TypeError` - If first argument is not a range
///
/// # Example
/// ```text
/// =ISBLANK({0,0})  // Returns 1.0 if cell at [0,0] is empty
/// =ISBLANK({5,10})  // Returns 0.0 if cell at [5,10] contains data
/// ```
pub fn isblank(args: Vec<FormulaAstInner>, container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 1 {
		return Err(Box::new(LengthError("ISBLANK only accepts 1 argument".to_string())));
	}

	let [cell] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Range(cell) = cell else {
		return Err(Box::new(TypeError("ISBLANK requires a range as its argument".to_string())));
	};

	if cell.len() != 1 {
		return Err(Box::new(LengthError("ISBLANK only accepts a single cell range".to_string())));
	}

	let is_blank = match container.is_empty_cell(&cell[0].1) {
		true => 1.0,
		false => 0.0
	};

	Ok(FormulaAstInner::Number(is_blank))
}

#[cfg(test)]
mod test {
	use crate::{Cell, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=ISBLANK()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=ISBLANK({0,0,0})").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 1.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_not_blank() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=ISBLANK({0,0,0})").unwrap();
		let r = FormulaExecutor::new();
		let cell = Cell::default();
		m.insert(&vec![0,0,0], cell).unwrap();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 0.0),
			_ => assert!(false)
		}
	}
}