use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// TRIM - Removes leading and trailing whitespace from a string.
///
/// # Arguments
/// * `args[0]` - A `Str` (string) to trim
///
/// # Returns
/// * `Str` - The string with leading and trailing whitespace removed
///
/// # Errors
/// * `LengthError` - If not exactly 1 argument provided
/// * `TypeError` - If first argument is not a string
///
/// # Example
/// ```text
/// =TRIM("  Hello world  ")  // Returns "Hello world"
/// =TRIM("\t\nHello\t\n")  // Returns "Hello"
/// =TRIM("")  // Returns ""
/// ```
pub fn trim(args: Vec<FormulaAstInner>, _container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 1 {
		return Err(Box::new(LengthError("TRIM only accepts 1 argument".to_string())));
	}

	let [str] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Str(str) = str else {
		return Err(Box::new(TypeError("TRIM requires a string as its argument".to_string())));
	};

	Ok(FormulaAstInner::Str(str.trim().to_string()))
}

#[cfg(test)]
mod test {
	use crate::{container::mapcontainer::MapContainer, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=TRIM()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=TRIM(\" Hello world    \")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, "Hello world"),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_empty_str() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=TRIM(\"\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, ""),
			_ => assert!(false)
		}
	}
}