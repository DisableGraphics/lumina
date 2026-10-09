use std::{error::Error};

use crate::{Content::Number, container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// Returns the current date and time as a UNIX timestamp
pub fn isnumber(args: Vec<FormulaAstInner>, container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 1 {
		return Err(Box::new(LengthError("ISNUMBER only accepts 1 argument".to_string())));
	}

	let [cell] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Range(cell) = cell else {
		return Err(Box::new(TypeError("First argument to ISNUMBER must be a range".to_string(),)));
	};

	if cell.len() != 1 {
		return Err(Box::new(LengthError("ISNUMBER only accepts 1 cell".to_string())));
	}

	if container.is_empty_cell(&cell[0].1) {
		return Ok(FormulaAstInner::Number(0.0))
	}

	let is_num = match &cell[0].0.content {
		Number(_) => 1.0,
		_ => 0.0
	};
	
	Ok(FormulaAstInner::Number(is_num))
}

#[cfg(test)]
mod test {
	use crate::{Cell, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=ISNUMBER()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=ISNUMBER({0,0,0})").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 0.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_not_blank() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=ISNUMBER({0,0,0})").unwrap();
		let r = FormulaExecutor::new();
		let cell = Cell::default();
		m.insert(&vec![0,0,0], cell).unwrap();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 1.0),
			_ => assert!(false)
		}
	}
}