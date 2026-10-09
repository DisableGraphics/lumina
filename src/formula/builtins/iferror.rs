use std::{error::Error};

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}, parser::FormulaParser}};

/// Returns the current date and time as a UNIX timestamp
pub fn iferror(mut args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 2 {
		return Err(Box::new(LengthError("IFERROR only accepts 2 arguments".to_string())));
	}

	let [cell, _] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Str(cell) = cell else {
		return Err(Box::new(TypeError("First argument to IFERROR must be a string with a parseable expression inside".to_string())));
	};

	let parser = FormulaParser::new();
	let ast = parser.parse(format!("={cell}"))?;

	let exprval = exec._eval(&ast, container);
	let onerror = args.remove(1);
	Ok(match exprval {
		Ok(ok) => ok,
		Err(_e) => onerror
	})
}

#[cfg(test)]
mod test {
	use crate::{Cell, Content::Str, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=IFERROR()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=IFERROR(\"TONUMBER({0,0,0}:{0,0,1})\", 7)").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 7.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_string() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=IFERROR(\"TOSTRING({0,0,0})\", 7)").unwrap();
		let r = FormulaExecutor::new();
		let cell = Cell {
			content: Str("Potato".to_string()),
			..Default::default()
		};
		m.insert(&vec![0,0,0], cell).unwrap();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, "Potato"),
			_ => assert!(false)
		}
	}
}