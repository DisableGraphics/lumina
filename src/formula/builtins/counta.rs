use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{ERROR_START, FormulaAstInner, FormulaExecutor}}};

/// COUNTA - Counts non-empty cells in a range.
///
/// Iterates through all cells in the given range and counts those that are not empty
/// and do not contain errors. Unlike COUNT, this includes text cells.
///
/// # Arguments
/// * `args[0]` - A `Range` of cells to count
///
/// # Returns
/// * `Number` - The count of non-empty, non-error cells (as f64)
///
/// # Errors
/// * `LengthError` - If not exactly 1 argument provided
/// * `TypeError` - If argument is not a range
///
/// # Example
/// ```text
/// =COUNTA({0,0}:{2,2})  // Counts non-empty cells in 3x3 grid
/// =COUNTA({0,0,0}:{1,1,1})  // Counts non-empty in 2x2x2 cube
/// // If A1=5, B1="text", C1=empty, D1=#ERROR: returns 2 (counts 5 and "text")
/// ```
pub fn counta(args: Vec<FormulaAstInner>, container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 1 {
		return Err(Box::new(LengthError("COUNTA only accepts 1 argument".to_string())));
	}

	let [range] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Range(range) = range else {
		return Err(Box::new(TypeError("First argument to COUNTA must be a range".to_string(),)));
	};

	let mut count = 0.0;

	for (cell, p) in range {
		if !container.is_empty_cell(p) && !cell.display_content.starts_with(ERROR_START) {
			count += 1.0;
		}
	}

	Ok(FormulaAstInner::Number(count))
}

#[cfg(test)]
mod test {
	use crate::{Cell, container::{Container, mapcontainer::MapContainer}, formula::{exec::{ERROR_START, FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn no_instances() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=COUNTA({0,0,0}:{1,1,1})").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 0.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn one_instance() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=COUNTA({0,0,0}:{1,1,1})").unwrap();

		let mut def = Cell::default();
		def.content = crate::Content::Number(12.0);

		m.insert(&vec![0, 0, 1], def).unwrap();

		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 1.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn all_instances() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=COUNTA({0,0,0}:{1,1,1})").unwrap();

		for i in 0..=1 {
			for j in 0..=1 {
				for k in 0..=1 {
					let mut def = Cell::default();
					def.content = crate::Content::Number(12.0);
					m.insert(&vec![i, j, k], def).unwrap();
				}
			}
		}

		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 8.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn fails_str() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=COUNTA(\"potato\")").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn fails_num() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=COUNTA(1)").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn doesnt_count_errors() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=COUNTA({0,0,0}:{1,1,1})").unwrap();

		let mut def = Cell::default();
		def.content = crate::Content::Number(12.0);
		def.display_content = format!("{}potato", ERROR_START);

		m.insert(&vec![0, 0, 1], def).unwrap();

		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 0.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn countss_strings() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=COUNTA({0,0,0}:{1,1,1})").unwrap();

		let mut def = Cell::default();
		def.content = crate::Content::Str("Sandwich".to_string());

		m.insert(&vec![0, 0, 1], def).unwrap();

		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 1.0),
			_ => assert!(false)
		}
	}
}