use std::{error::Error};

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{ERROR_START, FormulaAstInner, FormulaExecutor}}};

/// Returns the current date and time as a UNIX timestamp
pub fn iserror(args: Vec<FormulaAstInner>, container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 1 {
		return Err(Box::new(LengthError("ISERROR only accepts 1 argument".to_string())));
	}

	let [cell] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Range(cell) = cell else {
		return Err(Box::new(TypeError("First argument to ISERROR must be a range".to_string(),)));
	};

	if cell.len() != 1 {
		return Err(Box::new(LengthError("ISERROR only accepts 1 cell".to_string())));
	}

	if container.is_empty_cell(&cell[0].1) {
		return Ok(FormulaAstInner::Number(0.0))
	}

	if cell[0].0.display_content.starts_with(ERROR_START) {
		return Ok(FormulaAstInner::Number(1.0))
	} else {
		return Ok(FormulaAstInner::Number(0.0))
	}
}

#[cfg(test)]
mod test {
	use crate::{Cell, container::{Container, mapcontainer::MapContainer}, formula::{exec::{ERROR_START, FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=ISERROR()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=ISERROR({0,0,0})").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 0.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_not_string() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=ISERROR({0,0,0})").unwrap();
		let r = FormulaExecutor::new();
		let cell = Cell {
			content: crate::Content::Str("yes".to_string()),
			..Default::default()
		};
		m.insert(&vec![0,0,0], cell).unwrap();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 0.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_error() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=ISERROR({0,0,0})").unwrap();
		let r = FormulaExecutor::new();
		let cell = Cell {
			content: crate::Content::Str("yes".to_string()),
			display_content: format!("{} blah", ERROR_START),
			..Default::default()
		};
		m.insert(&vec![0,0,0], cell).unwrap();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 1.0),
			_ => assert!(false)
		}
	}
}