use std::{error::Error};

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, builtins::ifeval, exec::{FormulaAstInner, FormulaExecutor}}};

/// Returns the current date and time as a UNIX timestamp
pub fn or(args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 2 {
		return Err(Box::new(LengthError("OR only accepts 2 arguments".to_string())));
	}

	let [cell1, cell2] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Str(cell1) = cell1 else {
		return Err(Box::new(TypeError("First argument to OR must be a string with a parseable expression inside".to_string())));
	};

	let FormulaAstInner::Str(cell2) = cell2 else {
		return Err(Box::new(TypeError("Second argument to OR must be a string with a parseable expression inside".to_string())));
	};
	
	let exprval1 = ifeval(cell1, container, exec)?;
	let exprval2 = ifeval(cell2, container, exec)?;
	
	Ok(FormulaAstInner::Number(match exprval1 || exprval2 {
		true => 1.0,
		false => 0.0
	}))
}

#[cfg(test)]
mod test {
	use crate::{Cell, Content::Str, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=OR()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=OR(\"TONUMBER({0,0,0}) == 0\", \"TOSTRING({0,0,0}) == \\\"Sandwich\\\"\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 1.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_string() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=OR(\"TONUMBER({0,0,1}) == 0\", \"TOSTRING({0,0,0}) == \\\"Potato\\\"\")").unwrap();
		let r = FormulaExecutor::new();
		let cell = Cell {
			content: Str("Potato".to_string()),
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