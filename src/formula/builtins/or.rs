use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, builtins::ifeval, exec::{FormulaAstInner, FormulaExecutor}}};

/// OR - Returns true (1) if any condition evaluates to true, false (0) otherwise.
///
/// Each argument is a string expression that gets parsed and evaluated. At least
/// one must evaluate to true (non-zero) for the result to be true.
///
/// # Arguments
/// * `args[0..]` - Two or more `Str` values, each containing a parseable expression
///
/// # Returns
/// * `Number` - 1.0 if any condition is true, 0.0 if all are false
///
/// # Errors
/// * `LengthError` - If fewer than 2 arguments provided
/// * `TypeError` - If any argument is not a string, or if any expression cannot be parsed/evaluated
///
/// # Example
/// ```text
/// =OR("TONUMBER({0,0}) > 5", "TONUMBER({0,1}) < 10")  // Returns 1 if either true
/// =OR("ISTEXT({0,0})", "ISNUMBER({0,1})")  // Returns 1 if A1 is text OR B1 is number
/// =OR("1==2", "2==2")  // Returns 1 (second is true)
/// ```
pub fn or(args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() < 2 {
		return Err(Box::new(LengthError("OR requires at least 2 arguments".to_string())));
	}

	let mut any_true = false;

	for arg in args {
		let FormulaAstInner::Str(cell) = arg else {
			return Err(Box::new(TypeError("OR requires string arguments (condition expressions)".to_string())));
		};

		let exprval = ifeval(&cell, container, exec)?;
		if exprval {
			any_true = true;
			break;
		}
	}

	Ok(FormulaAstInner::Number(match any_true {
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

	#[test]
	fn correct_three_args() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=OR(\"1==2\", \"2==3\", \"3==3\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 1.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_false() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=OR(\"1==2\", \"2==3\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 0.0),
			_ => assert!(false)
		}
	}
}