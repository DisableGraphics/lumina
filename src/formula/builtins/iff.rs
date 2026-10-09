use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, builtins::ifeval, exec::{FormulaAstInner, FormulaExecutor}}};

/// IF - Ternary conditional: evaluates a condition and returns one of two values.
///
/// The condition is a string expression that gets parsed and evaluated (like "A1>5" or "TONUMBER({0,0})==10").
/// If the condition evaluates to true (non-zero), the second argument is returned; otherwise the third.
///
/// # Arguments
/// * `args[0]` - A `Str` containing a parseable expression (e.g., "A1>5", "TONUMBER({0,0})==10")
/// * `args[1]` - The value to return if condition is true (any type: Number, Str, Range)
/// * `args[2]` - The value to return if condition is false (any type: Number, Str, Range)
///
/// # Returns
/// * The second argument if condition evaluates to true, otherwise the third argument
///
/// # Errors
/// * `LengthError` - If not exactly 3 arguments provided
/// * `TypeError` - If first argument is not a string, or if the expression cannot be parsed/evaluated
///
/// # Example
/// ```text
/// =IF("TONUMBER({0,0}) > 10", "Greater", "Lesser")  // String result
/// =IF("TONUMBER({0,0}) > 10", 1, 0)  // Number result
/// =IF("ISTEXT({0,0})", "Is Text", "Not Text")  // Uses other functions in condition
/// ```
pub fn iff(mut args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 3 {
		return Err(Box::new(LengthError("IF only accepts 3 arguments".to_string())));
	}

	let [cell, _, _] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Str(cell) = cell else {
		return Err(Box::new(TypeError("IF requires a string as its first argument (condition expression)".to_string())));
	};

	let exprval = ifeval(cell, container, exec)?;

	let ontrue = args.remove(1);
	let onfalse = args.remove(1);
	Ok(match exprval {
		true => ontrue,
		false => onfalse
	})
}

#[cfg(test)]
mod test {
	use crate::{Cell, Content::Str, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=IF()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=IF(\"TONUMBER({0,0,0}) == 0\", 7, 1)").unwrap();
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
		let ast = q.parse("=IF(\"TOSTRING({0,0,0}) == \\\"Potato\\\" \", 1, 0)").unwrap();
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

	#[test]
	fn correct_string_quotes() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=IF(\"TOSTRING({0,0,0}) == \"\"Ah\\\"momo\"\" \", 1, 0)").unwrap();
		let r = FormulaExecutor::new();
		let cell = Cell {
			content: Str("Ah\"momo".to_string()),
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