use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// LEFT - Returns the leftmost N characters of a string.
///
/// # Arguments
/// * `args[0]` - A `Str` (string) to extract from
/// * `args[1]` - Optional `Number` specifying how many characters to extract (default: 1)
///
/// # Returns
/// * `Str` - The leftmost N characters of the string
///
/// # Errors
/// * `LengthError` - If not 1 or 2 arguments provided
/// * `TypeError` - If first argument is not a string, or second argument is not a number
///
/// # Example
/// ```text
/// =LEFT("Hello")  // Returns "H"
/// =LEFT("Hello", 3)  // Returns "Hel"
/// =LEFT("Hi", 10)  // Returns "Hi" (clamped to string length)
/// ```
pub fn left(args: Vec<FormulaAstInner>, _container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() < 1 || args.len() > 2 {
		return Err(Box::new(LengthError("LEFT only accepts 1 or 2 arguments".to_string())));
	}

	let str = args.get(0).unwrap();

	let num = args.get(1).unwrap_or(&FormulaAstInner::Number(1.0));

	let FormulaAstInner::Str(str) = str else {
		return Err(Box::new(TypeError("LEFT requires a string as its first argument".to_string())));
	};

	let FormulaAstInner::Number(n) = num else {
		return Err(Box::new(TypeError("LEFT requires a number as its second argument (character count)".to_string())));
	};

	let n = (*n as usize).min(str.chars().into_iter().count());

	Ok(FormulaAstInner::Str(str.chars().take(n).collect()))
}

#[cfg(test)]
mod test {
	use crate::{container::mapcontainer::MapContainer, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=LEFT()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=LEFT(\"Hello world\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, "H"),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_multiple() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=LEFT(\"Hello world\", 3)").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, "Hel"),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_empty_str() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=LEFT(\"\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, ""),
			_ => assert!(false)
		}
	}
}