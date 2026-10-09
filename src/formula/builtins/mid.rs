use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// MID - Returns the middle N characters of a string (centered).
///
/// This function extracts a substring from the center of the input string.
/// If the string length is less than N, the entire string is returned.
///
/// # Arguments
/// * `args[0]` - A `Str` (string) to extract from
/// * `args[1]` - Optional `Number` specifying how many characters to extract (default: 1)
///
/// # Returns
/// * `Str` - The middle N characters of the string, centered
///
/// # Errors
/// * `LengthError` - If not 1 or 2 arguments provided
/// * `TypeError` - If first argument is not a string, or second argument is not a number
///
/// # Example
/// ```text
/// =MID("Hello")  // Returns " " (middle character)
/// =MID("Hello world", 3)  // Returns "o w" (3 middle characters)
/// =MID("Hi", 10)  // Returns "Hi" (clamped to string length)
/// ```
pub fn mid(args: Vec<FormulaAstInner>, _container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() < 1 || args.len() > 2 {
		return Err(Box::new(LengthError("MID only accepts 1 or 2 arguments".to_string())));
	}

	let text = match args.get(0).unwrap() {
		FormulaAstInner::Str(s) => s,
		_ => return Err(Box::new(TypeError("MID requires a string as its first argument".to_string())))
	};

	let num = match args.get(1) {
		Some(FormulaAstInner::Number(n)) => *n,
		None => 1.0,
		_ => return Err(Box::new(TypeError("MID requires a number as its second argument (character count)".to_string())))
	};

	let chars: Vec<char> = text.chars().collect();
	let n = (num as usize).min(chars.len());

	let start = (chars.len() - n) / 2;
	let result: String = chars[start..start + n].iter().collect();

	Ok(FormulaAstInner::Str(result))
}

#[cfg(test)]
mod test {
	use crate::{container::mapcontainer::MapContainer, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=MID()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=MID(\"Hello world\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, " "),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_multiple() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=MID(\"Hello world\", 3)").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, "o w"),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_empty_str() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=MID(\"\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, ""),
			_ => assert!(false)
		}
	}
}