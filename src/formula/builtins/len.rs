use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// LEN - Returns the length (number of characters) of a string.
///
/// # Arguments
/// * `args[0]` - A `Str` (string) to measure
///
/// # Returns
/// * `Number` - The number of characters in the string
///
/// # Errors
/// * `LengthError` - If not exactly 1 argument provided
/// * `TypeError` - If first argument is not a string
///
/// # Example
/// ```text
/// =LEN("Hello")  // Returns 5
/// =LEN("")  // Returns 0
/// ```
pub fn len(args: Vec<FormulaAstInner>, _container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 1 {
		return Err(Box::new(LengthError("LEN only accepts 1 argument".to_string())));
	}

	let [str] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Str(str) = str else {
		return Err(Box::new(TypeError("LEN requires a string as its argument".to_string())));
	};

	Ok(FormulaAstInner::Number(str.chars().count() as f64))
}

#[cfg(test)]
mod test {
	use crate::{container::mapcontainer::MapContainer, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=LEN()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct_len() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=LEN(\"Hello world\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 11.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_len_empty_str() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=LEN(\"\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 0.0),
			_ => assert!(false)
		}
	}
}