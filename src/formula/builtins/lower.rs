use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// LOWER - Converts a string to lowercase.
///
/// # Arguments
/// * `args[0]` - A `Str` (string) to convert to lowercase
///
/// # Returns
/// * `Str` - The string converted to lowercase
///
/// # Errors
/// * `LengthError` - If not exactly 1 argument provided
/// * `TypeError` - If first argument is not a string
///
/// # Example
/// ```text
/// =LOWER("Hello World")  // Returns "hello world"
/// =LOWER("ALREADY LOWER")  // Returns "already lower"
/// ```
pub fn lower(args: Vec<FormulaAstInner>, _container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 1 {
		return Err(Box::new(LengthError("LOWER only accepts 1 argument".to_string())));
	}

	let [str] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Str(str) = str else {
		return Err(Box::new(TypeError("LOWER requires a string as its argument".to_string())));
	};

	Ok(FormulaAstInner::Str(str.to_lowercase()))
}

#[cfg(test)]
mod test {
	use crate::{container::mapcontainer::MapContainer, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=LOWER()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=LOWER(\"Hello world\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, "hello world"),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_empty_str() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=LOWER(\"\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, ""),
			_ => assert!(false)
		}
	}
}