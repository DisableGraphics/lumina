use std::{error::Error};

use crate::{container::Container, formula::{FormulaError::{LengthError, RuntimeError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// Returns the current date and time as a UNIX timestamp
pub fn error(args: Vec<FormulaAstInner>, _container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 1 {
		return Err(Box::new(LengthError("ERROR only accepts 1 argument".to_string())));
	}

	let [msg] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Str(cell) = msg else {
		return Err(Box::new(TypeError("First argument to ERROR must be a string".to_string(),)));
	};

	Err(Box::new(RuntimeError(cell.to_owned())))
}

#[cfg(test)]
mod test {
	use crate::{Cell, container::{Container, mapcontainer::MapContainer}, formula::{exec::FormulaExecutor, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=ERROR()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn incorrect_range() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=ERROR({0,0,0})").unwrap();
		let r = FormulaExecutor::new();
		assert!(r.execute(ast, &vec![0,0,0], &m).is_err());
	}

	#[test]
	fn correct_on_string() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=ERROR(\"Hello\")").unwrap();
		let r = FormulaExecutor::new();
		let cell = Cell::default();
		m.insert(&vec![0,0,0], cell).unwrap();
		let result = r.execute(ast, &vec![0,0,0], &m);
		assert!(result.is_err());
		let msg = result.err().unwrap().to_string();
		eprintln!("{msg}");
		assert!(msg.contains("Hello"));
	}
}