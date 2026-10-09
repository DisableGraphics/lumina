use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, RuntimeError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// ERROR - Returns a custom runtime error with the specified message.
///
/// This function is used to intentionally trigger an error with a custom message.
/// It does not return a value - it always returns an error result.
///
/// # Arguments
/// * `args[0]` - A `Str` (string) containing the error message
///
/// # Returns
/// * Always returns an `Err(RuntimeError)` with the provided message
///
/// # Errors
/// * `LengthError` - If not exactly 1 argument provided
/// * `TypeError` - If first argument is not a string
///
/// # Example
/// ```text
/// =ERROR("Custom error message")  // Returns error: "Custom error message"
/// =IF(A1<0, ERROR("Negative not allowed"), A1)  // Conditional error
/// ```
pub fn error(args: Vec<FormulaAstInner>, _container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 1 {
		return Err(Box::new(LengthError("ERROR only accepts 1 argument".to_string())));
	}

	let [msg] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Str(cell) = msg else {
		return Err(Box::new(TypeError("ERROR requires a string as its argument".to_string())));
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